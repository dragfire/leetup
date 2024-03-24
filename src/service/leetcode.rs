use std::cmp::Ord;
use std::collections::HashMap;
use std::env;
use std::fs::{self, File};
use std::io::{prelude::*, stdin};
use std::ops::Deref;
use std::path::{Path, PathBuf};

use anyhow::anyhow;
use async_trait::async_trait;
use colci::Color;
use html2text::from_read;
use leetup_cache::kvstore::KvStore;
use log::{debug, info};
use reqwest::header::{self, HeaderMap, HeaderValue};
use serde_json::{json, Value};

use crate::model::{
    CodeDefinition, Problem, ProblemInfo, ProblemInfoSeq, StatStatusPair, SubmissionResponse,
    TopicTagQuestion,
};
use crate::printer::SubmitExecutionResult;
use crate::template::parse_code;
use crate::{
    client::RemoteClient,
    cmd::{self, List, OrderBy, Query, User},
    printer::{Printer, TestExecutionResult},
    service::{self, auth, CacheKey, Comment, CommentStyle, LangInfo, ServiceProvider, Session},
    template::{InjectPosition, Pattern},
    Config, Either, LeetUpError, Result,
};

/// Leetcode holds all attributes required to implement ServiceProvider trait.
pub struct Leetcode<'a> {
    /// Store user session
    ///
    /// If session is empty, user should be able to view problems.
    session: Option<&'a Session>,

    /// Get config from config.json
    config: &'a Config,

    /// Provides caching mechanism for OJ(Online Judge).
    cache: KvStore,

    /// Service provider name
    name: &'a str,

    remote_client: RemoteClient<'a>,
}

#[async_trait]
impl<'a> ServiceProvider<'a> for Leetcode<'a> {
    fn session(&self) -> Option<&Session> {
        self.session
    }

    fn config(&self) -> Result<&Config> {
        Ok(self.config)
    }

    /// Fetch all problems
    ///
    /// Use cache wherever necessary
    async fn fetch_all_problems(&mut self) -> Result<Value> {
        let problems_res: Value;
        if let Some(ref val) = self.cache.get(CacheKey::Problems.into())? {
            debug!("Fetching problems from cache...");
            problems_res = serde_json::from_str::<Value>(val)?;
        } else {
            let url = &self.config.urls.problems_all;
            let session = self.session();
            problems_res = self
                .remote_client
                .get(url, None, session)
                .await?
                .json::<Value>()
                .await
                .map_err(LeetUpError::Reqwest)?;
            let res_serialized = serde_json::to_string(&problems_res)?;
            self.cache.set(CacheKey::Problems.into(), res_serialized)?;
        }

        Ok(problems_res)
    }

    async fn list_problems(&mut self, list: List) -> Result<()> {
        let problems_res = self.fetch_all_problems().await?;
        let mut probs: ProblemInfoSeq = vec![];

        if let Some(ref tag) = list.tag {
            let tag_questions = self.get_problems_with_topic_tag(tag).await?["data"]["topicTag"]
                ["questions"]
                .clone();
            let problems: Vec<TopicTagQuestion> = serde_json::from_value(tag_questions)?;
            for prob in problems {
                probs.push(Box::new(prob));
            }
        } else {
            let problems: Vec<StatStatusPair> =
                serde_json::from_value(problems_res["stat_status_pairs"].clone())?;

            for prob in problems {
                probs.push(Box::new(prob));
            }
        }

        if let Some(ref order) = list.order {
            let orders = OrderBy::from_str(order);
            probs.sort_by(|a, b| Leetcode::with_ordering(orders.as_slice(), a, b));
        } else {
            probs.sort_by(Ord::cmp);
        }

        if list.query.is_some() || list.keyword.is_some() {
            let filter_predicate = |o: &Box<dyn ProblemInfo + Send>| {
                let default_keyword = String::from("");
                let keyword = list
                    .keyword
                    .as_ref()
                    .unwrap_or(&default_keyword)
                    .to_ascii_lowercase();
                let has_keyword = o.question_title().to_lowercase().contains(&keyword);

                return list
                    .query
                    .as_ref()
                    .map(|query| Query::from_str(query))
                    .map(|queries| Leetcode::apply_queries(&queries, o))
                    .map(|result| has_keyword && result)
                    .unwrap_or(has_keyword);
            };

            Leetcode::pretty_list(
                &probs
                    .into_iter()
                    .filter(filter_predicate)
                    .collect::<Vec<Box<dyn ProblemInfo + Send + 'static>>>(),
            );
        } else {
            Leetcode::pretty_list(probs.iter());
        }

        Ok(())
    }

    async fn pick_problem(&mut self, pick: cmd::Pick) -> Result<()> {
        let probs = self.fetch_problems().await?;
        let urls = &self.config.urls;
        let lang = pick
            .lang
            .as_ref()
            .map(|l| l.info())
            .unwrap_or(self.config.lang.info());

        let problem: Problem = probs
            .iter()
            .find(|item| {
                item.stat.frontend_question_id == pick.id.expect("Expected frontend_question_id")
            })
            .map(|item| Problem {
                id: item.stat.frontend_question_id,
                link: format!("{}{}/", urls.problems, item.stat.question_title_slug),
                slug: item.stat.question_title_slug.to_string(),
                lang: lang.name.to_owned(),
                typed_code: None,
            })
            .expect("Problem with given ID not found");

        let problem_id = problem.id;
        let slug = problem.slug.to_owned();
        let query = r#"
            query getQuestionDetail($titleSlug: String!) {
               question(titleSlug: $titleSlug) {
                 content
                 stats
                 likes
                 dislikes
                 codeDefinition
                 sampleTestCase
                 enableRunCode
                 metaData
                 translatedContent
               }
            }
        "#;
        let body: Value = json!({
            "query": query,
            "variables": json!({
                "titleSlug": slug.to_owned(),
            }),
            "operationName": "getQuestionDetail"
        });

        let response = self
            .remote_client
            .post(&urls.graphql, &body, || None)
            .await?;
        debug!("Response: {}", response);

        self.generate_problem_stub(&lang, &problem, problem_id, slug, &response)?;

        Ok(())
    }

    async fn problem_test(&self, test: cmd::Test) -> Result<()> {
        let problem = service::extract_problem(test.filename)?;

        let test_data = self.get_test_data(test.test_data);
        debug!("Test data: {:?}", test_data);
        let typed_code = parse_code(problem.typed_code.as_ref().expect("Expected typed_code"));
        let body = json!({
                "lang":        problem.lang.to_owned(),
                "question_id": problem.id,
                "typed_code":  typed_code,
                "data_input":  test_data,
                "judge_type":  "large"
        });
        let url = &self.config()?.urls.test;
        debug!("problem_test url: {}, {:?}", url, body);
        let response = self.run_code(url, &problem, body).await;
        debug!("problem_test response: {:?}", response);

        match response {
            Err(e) => {
                println!("\n\n{}", Color::Red(e.to_string().as_str()).make());
                println!(
                    "\n{}",
                    Color::Yellow("Note: If error status is 4XX, make sure you are logged in!")
                        .make()
                );
            }
            Ok(json) => {
                let url = self.config.urls.verify.replace(
                    "$id",
                    json["interpret_id"].as_str().ok_or_else(|| {
                        LeetUpError::Any(anyhow!("Unable to replace `interpret_id`"))
                    })?,
                );
                let result: SubmissionResponse =
                    serde_json::from_value(self.verify_run_code(&url).await?)?;
                let execution_result = TestExecutionResult::new(test_data.into(), result);
                execution_result.print();
            }
        }

        Ok(())
    }

    async fn problem_submit(&self, submit: cmd::Submit) -> Result<()> {
        let problem = service::extract_problem(submit.filename)?;
        let body = json!({
            "lang":        problem.lang.to_owned(),
            "question_id": problem.id,
            "test_mode":   false,
            "typed_code":  parse_code(problem.typed_code.as_ref().expect("Expected typed_code")),
            "judge_type": "large",
        });
        let url = &self.config()?.urls.submit;
        let response = self.run_code(url, &problem, body).await?;
        let url = self
            .config
            .urls
            .verify
            .replace("$id", &response["submission_id"].to_string());
        let result: SubmissionResponse = serde_json::from_value(self.verify_run_code(&url).await?)?;
        let execution_result = SubmitExecutionResult::new(result);
        execution_result.print();
        Ok(())
    }

    async fn process_auth(&mut self, user: User) -> Result<()> {
        // cookie login
        if user.cookie.is_some() {
            let session = auth::cookie_login(self).await?;
            self.cache_session(session)?;
        }

        if user.logout.is_some() {
            self.logout()?;
            println!("User logged out!");
        }

        Ok(())
    }

    fn cache(&mut self) -> Result<&KvStore> {
        Ok(&self.cache)
    }

    fn name(&self) -> &'a str {
        self.name
    }
}

impl<'a> Leetcode<'a> {
    pub fn new(session: Option<&'a Session>, config: &'a Config, cache: KvStore) -> Result<Self> {
        let name = "leetcode";

        Ok(Leetcode {
            session,
            config,
            cache,
            name,
            remote_client: RemoteClient::new(config, session),
        })
    }

    fn cache_session(&mut self, session: Session) -> Result<()> {
        let session_str = serde_json::to_string(&session)?;
        self.cache.set(CacheKey::Session.into(), session_str)?;
        // remove key `problems`, rebuild problems cache.
        //
        // NOTE: cache.remove throws "Key not found" error
        // so ignore that error if it is thrown.
        if self.cache.remove(CacheKey::Problems.into()).is_err() {}
        Ok(())
    }

    pub async fn fetch_problems(&mut self) -> Result<Vec<StatStatusPair>> {
        let problems = self.fetch_all_problems().await?;
        let problems: Vec<StatStatusPair> =
            serde_json::from_value(problems["stat_status_pairs"].clone())?;

        Ok(problems)
    }

    async fn run_code(&self, url: &str, problem: &Problem, body: Value) -> Result<Value> {
        let url = url.replace("$slug", &problem.slug);
        self.remote_client
            .post(&url, &body, || {
                let mut headers = HeaderMap::new();
                headers.insert(
                    header::REFERER,
                    HeaderValue::from_str(&problem.link).expect("Link is required!"),
                );
                Some(headers)
            })
            .await
    }

    async fn verify_run_code(&self, url: &str) -> Result<Value> {
        loop {
            let response = self
                .remote_client
                .get(url, None, self.session())
                .await?
                .json::<Value>()
                .await?;
            if response["state"] == "SUCCESS" {
                return Ok(response);
            }
            std::thread::sleep(std::time::Duration::from_millis(200));
        }
    }

    fn write_code_fragment(
        &self,
        buf: &mut String,
        comment: &str,
        code_fragment: Option<&Either>,
        pos: InjectPosition,
    ) -> Result<()> {
        if let Some(either) = code_fragment {
            let inject_code_pos_pattern = format!(
                "\n{} {}\n",
                comment,
                Pattern::InjectCodePosition(pos).to_string()
            );
            buf.push_str(&inject_code_pos_pattern);
            let code_fragment = either.to_string();
            buf.push_str(&code_fragment);
            buf.push_str(&inject_code_pos_pattern);
        }
        Ok(())
    }

    fn logout(&mut self) -> Result<()> {
        if self.cache.remove(CacheKey::Session.into()).is_err() {
            println!("User not logged in!");
            return Ok(());
        }
        if self.cache.remove(CacheKey::Problems.into()).is_err() {}
        Ok(())
    }

    fn execute_script(&self, cmd: &str, problem: &Problem, dir: &Path) -> Result<()> {
        let dir_str = dir.to_str().expect("Expected a valid directory");
        let cmd = cmd.replace(&Pattern::WorkingDir.to_string(), dir_str);
        let cmd = cmd.replace(&Pattern::Problem.to_string(), &problem.slug);
        std::process::Command::new("sh")
            .args(["-c", &cmd])
            .spawn()?
            .wait()?;
        Ok(())
    }

    fn pick_hook(&self, content: &str, problem: &Problem, lang: &LangInfo) -> Result<()> {
        let mut curr_dir = env::current_dir()?;
        let mut filename = curr_dir.clone();
        let cfg = self.config()?;
        if let Some(ref cfg) = cfg.pick_hook {
            if let Some(hook_cfg) = cfg.get(&lang.name) {
                if let Some(dir) = hook_cfg.working_dir() {
                    let dir = shellexpand::tilde(dir);
                    curr_dir = PathBuf::from(dir.deref());
                    fs::create_dir_all(&curr_dir)?;
                    filename = curr_dir.clone();
                }
                if let Some(pre) = hook_cfg.script_pre_generation() {
                    println!(
                        "{}",
                        Color::Cyan("Executing pre-generation script...").make()
                    );
                    let cmd = pre.to_string();
                    self.execute_script(&cmd, problem, &curr_dir)?;
                }
                self.write_content(&mut filename, problem, lang, content.as_bytes())?;

                if let Some(post) = hook_cfg.script_post_generation() {
                    println!(
                        "{}",
                        Color::Cyan("Executing post-generation script...").make()
                    );
                    let cmd = post.to_string();
                    self.execute_script(&cmd, problem, &curr_dir)?;
                }

                // File path can be wrong if you used: `mkdir`, `cd`, `mv` to move
                // around the generated file. Find the right path used in your script!
                println!(
                    "Generated: {}\n{}",
                    Color::Magenta(filename.to_str().ok_or(LeetUpError::OptNone)?).make(),
                    Color::Yellow("Note: File path can be wrong if you used: `mkdir`, `cd`, `mv` to move around the generated file. Find the right path used in your script!").make()
                );
                return Ok(());
            }
        }
        self.write_content(&mut filename, problem, lang, content.as_bytes())?;
        println!(
            "Generated: {}",
            Color::Magenta(filename.to_str().ok_or(LeetUpError::OptNone)?).make()
        );

        Ok(())
    }

    fn write_content(
        &self,
        filename: &mut PathBuf,
        problem: &Problem,
        lang: &LangInfo,
        content: &[u8],
    ) -> Result<()> {
        filename.push(&problem.slug);
        filename.set_extension(&lang.extension);

        let mut file = File::create(&filename)?;
        file.write_all(content)?;
        Ok(())
    }

    async fn get_problems_with_topic_tag(&self, tag: &str) -> Result<Value> {
        let query = r#"
            query getTopicTag($slug: String!) {
                 topicTag(slug: $slug) {
                   name
                   slug
                   questions {
                     difficulty
                     isPaidOnly
                     title
                     titleSlug
                     questionFrontendId
                     status
                   }
                 }
             }
        "#;
        let body: Value = json!({
            "operationName": "getTopicTag",
            "variables": {
                "slug": tag,
            },
            "query": query
        });

        self.remote_client
            .post(&self.config.urls.graphql, &body, || None)
            .await
    }

    fn generate_problem_stub(
        &mut self,
        lang: &LangInfo,
        problem: &Problem,
        problem_id: usize,
        slug: String,
        response: &Value,
    ) -> Result<()> {
        let mut definition = None;
        let mut start_comment = "";
        let line_comment;
        let mut end_comment = "";
        let single_comment;

        match &lang.comment {
            Comment::C(CommentStyle::Single(s), multi) => {
                single_comment = s;
                if let Some(CommentStyle::Multiline {
                    start,
                    between,
                    end,
                }) = multi
                {
                    start_comment = start.as_str();
                    line_comment = between.as_str();
                    end_comment = end.as_str();
                } else {
                    line_comment = single_comment;
                }
            }
            Comment::Python3(CommentStyle::Single(s), _)
            | Comment::MySQL(CommentStyle::Single(s), _) => {
                line_comment = s;
                single_comment = s;
            }
            _ => unreachable!(),
        };

        if let Some(content) = &response["data"]["question"]["content"].as_str() {
            let content = from_read(content.as_bytes(), 80);
            let content = content.replace("**", "");
            let content = content
                .split('\n')
                .map(|s| format!("{} {}", line_comment, s))
                .collect::<Vec<String>>()
                .join("\n");
            info!("Single Comment: {}", single_comment);

            let pattern_custom = format!("{} {}", single_comment, Pattern::CustomCode.to_string());
            let pattern_leetup_info =
                format!("{} {}", single_comment, Pattern::LeetUpInfo.to_string());
            let content = format!(
                "{}\n{} id={} lang={} slug={}\n\n{}\n{}\n{}\n{}",
                pattern_custom,
                pattern_leetup_info,
                problem_id,
                lang.name,
                slug,
                start_comment,
                content,
                end_comment,
                pattern_custom
            );
            debug!("Content: {}", content);
            definition = Some(content);
        }

        let mut filename = env::current_dir()?;
        filename.push(slug);
        filename.set_extension(&lang.extension);

        if let Some(code_defs) = &response["data"]["question"]["codeDefinition"].as_str() {
            let mut buf = String::new();
            let code_defs: HashMap<_, _> = serde_json::from_str::<Vec<CodeDefinition>>(code_defs)?
                .into_iter()
                .map(|def| (def.value.to_owned(), def))
                .into_iter()
                .collect();
            if let Some(ref definition) = definition {
                buf.push_str(definition)
            }
            let pattern_code = format!("\n{} {}\n", single_comment, Pattern::Code.to_string());
            let code = &code_defs
                .get(&lang.name)
                .ok_or(LeetUpError::OptNone)?
                .default_code;
            debug!("Code: {}", code);
            let inject_code = self
                .config()?
                .inject_code
                .as_ref()
                .and_then(|c| c.get(&problem.lang));
            debug!("InjectCode: {:#?}", inject_code);
            if let Some(inject_code) = inject_code {
                self.write_code_fragment(
                    &mut buf,
                    single_comment,
                    inject_code.before_code_exclude.as_ref(),
                    InjectPosition::BeforeCodeExclude,
                )?;
            }
            buf.push_str(&pattern_code);
            if let Some(inject_code) = inject_code {
                self.write_code_fragment(
                    &mut buf,
                    single_comment,
                    inject_code.before_code.as_ref(),
                    InjectPosition::BeforeCode,
                )?;
            }
            buf.push('\n');
            buf.push_str(code);
            buf.push_str(&pattern_code);
            if let Some(inject_code) = inject_code {
                self.write_code_fragment(
                    &mut buf,
                    single_comment,
                    inject_code.after_code.as_ref(),
                    InjectPosition::AfterCode,
                )?;
            }

            self.pick_hook(&buf, problem, lang)?;
        }

        Ok(())
    }

    /*
     * Parse Option<Option<String>> from structopt
     *
     * Get string from command line if provided, otherwise try to get string from stdin
     *
     * We can provide test data as multiline input using stdin.
     *
     * # Example:
     * ```bash
     * leetup test 3sum.java -t << END
     * [1,-1,0]
     * [0, 1, 1, 1, 2, -3, -1]
     * [1,2,3]
     * END
     * ```
     */
    fn get_test_data(&self, test_data: Option<Option<String>>) -> String {
        test_data.unwrap().unwrap_or_else(|| {
            let mut buf = String::new();
            stdin()
                .lock()
                .read_to_string(&mut buf)
                .expect("test input expected from stdin");
            buf
        })
    }
}
