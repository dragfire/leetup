use crate::model::{
    CodeDefinition, Problem, ProblemInfo, ProblemInfoSeq, StatStatusPair, SubmissionResult,
    TopicTagQuestion,
};
use crate::{
    client::RemoteClient,
    cmd::{self, List, OrderBy, Query, User},
    icon::Icon,
    service::{self, auth, CacheKey, Comment, CommentStyle, LangInfo, ServiceProvider, Session},
    template::{parse_code, InjectPosition, Pattern},
    Config, Either, LeetUpError, Result,
};
use anyhow::anyhow;
use async_trait::async_trait;
use colci::Color;
use html2text::from_read;
use leetup_cache::kvstore::KvStore;
use log::{debug, info};
use reqwest::header::{self, HeaderMap, HeaderValue};
use serde_json::{json, Value};
use std::cmp::Ord;
use std::collections::HashMap;
use std::env;
use std::fs::{self, File};
use std::io::prelude::*;
use std::ops::Deref;
use std::path::{Path, PathBuf};
use std::str::FromStr;

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

    async fn run_code(
        &self,
        url: &str,
        problem: &Problem,
        body: serde_json::Value,
    ) -> Result<serde_json::Value> {
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

    async fn verify_run_code(&self, url: &str) -> Result<serde_json::Value> {
        loop {
            let response = self
                .remote_client
                .get(url, None, self.session())
                .await?
                .json::<serde_json::Value>()
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

    fn print_judge_result(
        &self,
        test_data: Option<String>,
        result: SubmissionResult,
    ) -> Result<()> {
        debug!("judge result: {:?}", result);
        match result.status_code {
            10 => {
                // Accepted

                // Test result
                if result.expected_status_code.is_some() {
                    println!(
                        "{}",
                        Color::Green(&format!(
                            r#"
 {} {}
Input:
{}

Output:
{}

Expected:
{}
                    "#,
                            Icon::Yes.to_string(),
                            result.status_msg,
                            test_data.ok_or(LeetUpError::OptNone)?,
                            result
                                .code_answer
                                .expect("Code answer required!")
                                .to_string(),
                            result
                                .expected_code_answer
                                .expect("Expected code answer required!")
                                .to_string(),
                        ))
                        .make()
                    );
                } else {
                    println!(
                        "{}",
                        Color::Green(&format!(
                            r#"
 {} {}
 {}/{} cases passed ({})
 Your runtime beats {}% of {} submissions
 Your memory usage beats {}% of {} submissions ({})
                    "#,
                            Icon::Yes.to_string(),
                            result.status_msg,
                            result.total_correct.unwrap_or(0),
                            result.total_testcases.unwrap_or(0),
                            result.status_runtime,
                            result.runtime_percentile.unwrap_or(0.0),
                            result.lang,
                            result.memory_percentile.unwrap_or(0.0),
                            result.lang,
                            result.status_memory
                        ))
                        .make()
                    );
                }
            }
            15 => {
                // Runtime error
                println!(
                    "{}",
                    Color::Red(&format!(
                        r#"
 {} {}
Input:
{}

Output:
{}

Expected:
{}
                    "#,
                        Icon::_No.to_string(),
                        result.status_msg,
                        test_data.unwrap_or_default(),
                        result
                            .code_output
                            .unwrap_or_else(|| Either::String("".to_string()))
                            .to_string(),
                        result
                            .code_answer
                            .unwrap_or(Either::String("".to_string()))
                            .to_string()
                    ))
                    .make()
                );
            }
            20 => {
                // Compile Error
                println!(
                    "{}",
                    Color::Red(&format!(
                        "\n {} {}\n {}",
                        Icon::_No.to_string(),
                        result.status_msg,
                        result
                            .full_compile_error
                            .ok_or_else(|| "Failed to get compilation error!")
                            .map_err(anyhow::Error::msg)?
                    ))
                    .make()
                );
            }
            _ => {
                // Wrong Answer | TimeLimitExceeded
                println!(
                    "{}",
                    Color::Red(&format!(
                        r#"
 {} {}
 {}/{} cases passed ({})
 Failed Test: {:#?}
                    "#,
                        Icon::_No.to_string(),
                        result.status_msg,
                        result.total_correct.unwrap_or(0),
                        result.total_testcases.unwrap_or(0),
                        result.status_runtime,
                        result
                            .code_output
                            .unwrap_or_else(|| Either::String("[Empty]".to_string()))
                    ))
                    .make()
                );
            }
        }
        Ok(())
    }

    async fn get_problems_with_topic_tag(&self, tag: &str) -> Result<serde_json::Value> {
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
        let body: serde_json::Value = json!({
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
        let mut single_comment = "";

        // TODO should have Single and Multiline comment available?
        let comment_style: &CommentStyle = match &lang.comment {
            Comment::C(single, multi) => multi.as_ref().unwrap_or(single),
            Comment::Python3(single, _) => single,
            Comment::MySQL(single, _) => single,
        };

        match comment_style {
            CommentStyle::Single(s) => {
                line_comment = s;
                single_comment = s;
            }
            CommentStyle::Multiline {
                start,
                between,
                end,
            } => {
                start_comment = start;
                line_comment = between;
                end_comment = end;
            }
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
            let code_defs: Vec<(String, CodeDefinition)> =
                serde_json::from_str::<Vec<CodeDefinition>>(code_defs)?
                    .into_iter()
                    .map(|def| (def.value.to_owned(), def))
                    .collect();
            let code_defs: HashMap<_, _> = code_defs.into_iter().collect();
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

            self.pick_hook(&buf, &problem, &lang)?;
        }

        Ok(())
    }
}

#[async_trait]
impl<'a> ServiceProvider<'a> for Leetcode<'a> {
    fn session(&self) -> Option<&Session> {
        self.session
    }

    fn config(&self) -> Result<&Config> {
        Ok(&self.config)
    }

    /// Fetch all problems
    ///
    /// Use cache wherever necessary
    async fn fetch_all_problems(&mut self) -> Result<serde_json::value::Value> {
        let problems_res: serde_json::value::Value;
        if let Some(ref val) = self.cache.get(CacheKey::Problems.into())? {
            debug!("Fetching problems from cache...");
            problems_res = serde_json::from_str::<serde_json::value::Value>(val)?;
        } else {
            let url = &self.config.urls.problems_all;
            let session = self.session();
            problems_res = self
                .remote_client
                .get(url, None, session)
                .await?
                .json::<serde_json::value::Value>()
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
        let lang = pick.lang.info();

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
        let body: serde_json::value::Value = json!({
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
        let test_data = test.test_data.replace("\\n", "\n");
        let body = json!({
                "lang":        problem.lang.to_owned(),
                "question_id": problem.id,
                "typed_code":  parse_code(problem.typed_code.as_ref().expect("Expected typed_code")),
                "data_input":  test_data,
                "judge_type":  "large"
        });
        let url = &self.config()?.urls.test;
        debug!("problem_test url: {}, {:?}", url, body);
        let response = self.run_code(url, &problem, body).await?;
        debug!("problem_test response: {:?}", response);
        let url = self.config.urls.verify.replace(
            "$id",
            response["interpret_id"]
                .as_str()
                .ok_or_else(|| LeetUpError::Any(anyhow!("Unable to replace `interpret_id`")))?,
        );
        let result: SubmissionResult = serde_json::from_value(self.verify_run_code(&url).await?)?;
        self.print_judge_result(Some(test_data), result)
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
        let result: SubmissionResult = serde_json::from_value(self.verify_run_code(&url).await?)?;
        self.print_judge_result(None, result)
    }

    async fn process_auth(&mut self, user: User) -> Result<()> {
        // cookie login
        if let Some(val) = user.cookie {
            let cookie = val.unwrap_or_else(|| {
                let mut cookie_value = String::new();
                println!("Enter Cookie:");
                std::io::stdin()
                    .read_line(&mut cookie_value)
                    .expect("Failed to read cookie from input");
                cookie_value.trim_end().to_string()
            });

            // filter out all unnecessary cookies
            let session = Session::from_str(&cookie)
                .map_err(|_| LeetUpError::Any(anyhow!("Unable to parse cookie string")))?;
            println!("\n{}", Color::Green("User logged in!").make());
            self.cache_session(session)?;
        }

        // github login
        if user.github.is_some() {
            match auth::github_login(self).await {
                Ok(session) => {
                    println!("\n{}", Color::Green("User logged in!").make());
                    self.cache_session(session)?;
                }
                Err(_) => {
                    println!("\n{}", Color::Red("Github login failed!").make());
                }
            }
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
