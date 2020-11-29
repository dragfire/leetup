use crate::{
    client,
    cmd::{self, List, OrderBy, Query, User},
    icon::Icon,
    service::{
        self, auth, CacheKey, Comment, CommentStyle, Difficulty, DifficultyType, LangInfo, Problem,
        ProblemInfo, ServiceProvider, Session,
    },
    template::{parse_code, InjectPosition, Pattern},
    Config, Either, LeetUpError, Result,
};
use anyhow::anyhow;
use colci::Color;
use html2text::from_read;
use leetup_cache::kvstore::KvStore;
use log::{debug, info};
use reqwest::header::{self, HeaderMap, HeaderValue};
use serde::Deserialize;
use serde_json::json;
use std::cmp::{Ord, Ordering};
use std::collections::HashMap;
use std::env;
use std::fs::{self, File};
use std::io::prelude::*;
use std::ops::Deref;
use std::path::PathBuf;
use std::str::FromStr;

/// Leetcode holds all attributes required to implement ServiceProvider trait.
pub struct Leetcode<'a> {
    /// Store user session
    ///
    /// If session is empty, user should be able to view problems.
    session: Option<Session>,

    /// Get config from config.json
    config: Config,

    /// Provides caching mechanism for OJ(Online Judge).
    cache: KvStore,

    /// Service provider name
    name: &'a str,
}

impl<'a> Leetcode<'a> {
    pub fn new() -> Self {
        let name = "leetcode";

        // create .leetup directory: ~/.leetup/*.log
        let mut data_dir = PathBuf::new();
        data_dir.push(dirs::home_dir().expect("Home directory not available!"));
        data_dir.push(".leetup");

        let mut cache = KvStore::open(&data_dir).unwrap();
        let mut session: Option<Session> = None;
        let session_val = cache.get(CacheKey::Session.into()).unwrap();

        // Set session if the user is logged in
        if let Some(ref val) = session_val {
            session =
                Some(serde_json::from_str::<Session>(val).expect("Session format not correct"));
        }
        data_dir.push("config.json");
        let config = Config::get(data_dir);

        Leetcode {
            session,
            config,
            cache,
            name,
        }
    }

    fn cache_session(&mut self, session: Session) -> Result<()> {
        let session_str = serde_json::to_string(&session)?;
        self.cache.set(CacheKey::Session.into(), session_str)?;
        self.session = Some(session);
        // remove key `problems`, rebuild problems cache.
        //
        // NOTE: cache.remove throws "Key not found" error
        // so ignore that error if it is thrown.
        if let Err(_) = self.cache.remove(CacheKey::Problems.into()) {}
        Ok(())
    }

    pub fn fetch_problems(&mut self) -> Result<Vec<StatStatusPair>> {
        let problems = self.fetch_all_problems()?;
        let problems: Vec<StatStatusPair> =
            serde_json::from_value(problems["stat_status_pairs"].clone())?;

        Ok(problems)
    }

    fn run_code(
        &self,
        url: &str,
        problem: &Problem,
        body: serde_json::Value,
    ) -> Result<serde_json::Value> {
        let url = url.replace("$slug", &problem.slug);
        client::post(self, &url, &body, || {
            let mut headers = HeaderMap::new();
            headers.insert(
                header::REFERER,
                HeaderValue::from_str(&problem.link).unwrap(),
            );
            Some(headers)
        })
    }

    fn verify_run_code(&self, url: &str) -> Result<serde_json::Value> {
        loop {
            let response = client::get(url, None, self.session())?.json::<serde_json::Value>()?;
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
        if let Err(_) = self.cache.remove(CacheKey::Session.into()) {
            println!("User not logged in!");
            return Ok(());
        }
        if let Err(_) = self.cache.remove(CacheKey::Problems.into()) {}
        Ok(())
    }

    fn execute_script(&self, cmd: &str, problem: &Problem, dir: &PathBuf) -> Result<()> {
        let dir_str = dir.to_str().unwrap();
        let cmd = cmd.replace(&Pattern::WorkingDir.to_string(), dir_str);
        let cmd = cmd.replace(&Pattern::Problem.to_string(), &problem.slug);
        std::process::Command::new("sh")
            .args(&["-c", &cmd])
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
                Color::Magenta(filename.to_str().unwrap()).make(),
                Color::Yellow("Note: File path can be wrong if you used: `mkdir`, `cd`, `mv` to move around the generated file. Find the right path used in your script!").make()
            );
                return Ok(());
            }
        }
        self.write_content(&mut filename, problem, lang, content.as_bytes())?;
        println!(
            "Generated: {}",
            Color::Magenta(filename.to_str().unwrap()).make()
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

    fn print_judge_result(&self, test_data: Option<String>, result: SubmissionResult) {
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
                            test_data.unwrap(),
                            result.code_answer.unwrap().to_string(),
                            result.expected_code_answer.unwrap().to_string(),
                        ))
                        .make()
                    );
                    return;
                }
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
                        result.total_correct.unwrap(),
                        result.total_testcases.unwrap(),
                        result.status_runtime,
                        result.runtime_percentile.unwrap(),
                        result.lang,
                        result.memory_percentile.unwrap(),
                        result.lang,
                        result.status_memory
                    ))
                    .make()
                );
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
                        test_data.unwrap(),
                        result.code_output.unwrap().to_string(),
                        result.code_answer.unwrap().to_string(),
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
                        result.full_compile_error.unwrap(),
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
                        result.total_correct.unwrap(),
                        result.total_testcases.unwrap(),
                        result.status_runtime,
                        result.code_output.unwrap(),
                    ))
                    .make()
                );
            }
        }
    }

    fn get_problems_with_topic_tag(&self, tag: &str) -> Result<serde_json::Value> {
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

        client::post(self, &self.config.urls.graphql, &body, || None)
    }
}

impl<'a> ServiceProvider<'a> for Leetcode<'a> {
    fn session(&self) -> Option<&Session> {
        self.session.as_ref()
    }

    fn config(&self) -> Result<&Config> {
        Ok(&self.config)
    }

    /// Fetch all problems
    ///
    /// Use cache wherever necessary
    fn fetch_all_problems(&mut self) -> Result<serde_json::value::Value> {
        let problems_res: serde_json::value::Value;
        if let Some(ref val) = self.cache.get(CacheKey::Problems.into())? {
            debug!("Fetching problems from cache...");
            problems_res = serde_json::from_str::<serde_json::value::Value>(val)?;
        } else {
            let url = &self.config.urls.problems_all;
            let session = self.session();
            problems_res = client::get(url, None, session)?
                .json::<serde_json::value::Value>()
                .map_err(LeetUpError::Reqwest)?;
            let res_serialized = serde_json::to_string(&problems_res)?;
            self.cache.set(CacheKey::Problems.into(), res_serialized)?;
        }

        Ok(problems_res)
    }

    fn list_problems(&mut self, list: List) -> Result<()> {
        let problems_res = self.fetch_all_problems()?;
        let mut probs: Vec<Box<dyn ProblemInfo>> = vec![];

        if let Some(ref tag) = list.tag {
            let tag_questions =
                self.get_problems_with_topic_tag(tag)?["data"]["topicTag"]["questions"].clone();
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

            probs.sort_by(|a, b| {
                let mut ordering = Ordering::Equal;
                let id_ordering = a.question_id().cmp(&b.question_id());
                let title_ordering = a.question_title().cmp(&b.question_title());
                let a_difficulty_level: DifficultyType = a.difficulty().into();
                let b_difficulty_level: DifficultyType = b.difficulty().into();
                let diff_ordering = a_difficulty_level.cmp(&b_difficulty_level);

                for order in &orders {
                    match order {
                        OrderBy::IdAsc => ordering = ordering.then(id_ordering),
                        OrderBy::IdDesc => ordering = ordering.then(id_ordering.reverse()),
                        OrderBy::TitleAsc => ordering = ordering.then(title_ordering),
                        OrderBy::TitleDesc => ordering = ordering.then(title_ordering.reverse()),
                        OrderBy::DifficultyAsc => ordering = ordering.then(diff_ordering),
                        OrderBy::DifficultyDesc => {
                            ordering = ordering.then(diff_ordering.reverse())
                        }
                    }
                }

                ordering
            });
        } else {
            probs.sort_by(Ord::cmp);
        }

        if list.query.is_some() || list.keyword.is_some() {
            let filter_predicate = |o: &Box<dyn ProblemInfo>| {
                let default_keyword = String::from("");
                let keyword = list
                    .keyword
                    .as_ref()
                    .unwrap_or(&default_keyword)
                    .to_ascii_lowercase();
                let has_keyword = o.question_title().to_lowercase().contains(&keyword);

                if list.query.is_none() {
                    has_keyword
                } else {
                    let queries: Vec<Query> = Query::from_str(list.query.as_ref().unwrap());
                    has_keyword && Leetcode::apply_queries(&queries, o)
                }
            };

            let mut filtered_problems: Vec<Box<dyn ProblemInfo>> = vec![];
            for prob in probs {
                if filter_predicate(&prob) {
                    filtered_problems.push(prob);
                }
            }
            Leetcode::pretty_list(filtered_problems.iter());
        } else {
            Leetcode::pretty_list(probs.iter());
        }

        Ok(())
    }

    fn pick_problem(&mut self, pick: cmd::Pick) -> Result<()> {
        let probs = self.fetch_problems()?;
        let urls = &self.config.urls;
        let mut single_comment = "";
        let lang = pick.lang.info();

        // TODO should have Single and Multiline comment available?
        let comment = match &lang.comment {
            Comment::C(single, multi) => {
                if let CommentStyle::Single(s) = single {
                    single_comment = s;
                }
                if multi.is_some() {
                    multi.as_ref().unwrap()
                } else {
                    single
                }
            }
            Comment::Python3(single, _) => single,
            Comment::MySQL(single, _) => single,
        };

        if let CommentStyle::Single(s) = comment {
            single_comment = s;
        }

        info!("Comment: {:#?}", comment);

        let problem: Problem = probs
            .iter()
            .find(|item| item.stat.frontend_question_id == pick.id.unwrap())
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

        let response = client::post(self, &urls.graphql, &body, || None)?;
        debug!("Response: {:#?}", response);

        let mut definition = None;
        let mut start_comment = "";
        let line_comment;
        let mut end_comment = "";
        match comment {
            CommentStyle::Single(s) => {
                line_comment = s;
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
            let code = &code_defs.get(&lang.name).unwrap().default_code;
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
            buf.push_str(&code);
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

    fn problem_test(&self, test: cmd::Test) -> Result<()> {
        let problem = service::extract_problem(test.filename)?;
        let test_data = test.test_data.replace("\\n", "\n");
        let body = json!({
                "lang":        problem.lang.to_owned(),
                "question_id": problem.id,
                "test_mode":   true,
                "typed_code":  parse_code(problem.typed_code.as_ref().unwrap()),
                "data_input":  test_data,
                "judge_type":  "large"
        });
        let url = &self.config()?.urls.test;
        let response = self.run_code(url, &problem, body)?;
        let url = self
            .config
            .urls
            .verify
            .replace("$id", &response["interpret_id"].as_str().unwrap());
        let result: SubmissionResult = serde_json::from_value(self.verify_run_code(&url)?)?;
        self.print_judge_result(Some(test_data), result);

        Ok(())
    }

    fn problem_submit(&self, submit: cmd::Submit) -> Result<()> {
        let problem = service::extract_problem(submit.filename)?;
        let body = json!({
            "lang":        problem.lang.to_owned(),
            "question_id": problem.id,
            "test_mode":   false,
            "typed_code":  parse_code(problem.typed_code.as_ref().unwrap()),
            "judge_type": "large",
        });
        let url = &self.config()?.urls.submit;
        let response = self.run_code(url, &problem, body)?;
        let url = self
            .config
            .urls
            .verify
            .replace("$id", &response["submission_id"].to_string());
        let result: SubmissionResult = serde_json::from_value(self.verify_run_code(&url)?)?;
        self.print_judge_result(None, result);

        Ok(())
    }

    fn process_auth(&mut self, user: User) -> Result<()> {
        // cookie login
        if let Some(val) = user.cookie {
            let mut cookie = String::new();

            if let Some(val) = val {
                cookie = val;
            } else {
                println!("Enter Cookie:");
                let stdin = std::io::stdin();
                stdin.read_line(&mut cookie)?;
                cookie = format!(r#"{}"#, cookie.trim_end());
            }

            // filter out all unnecessary cookies
            let session = Session::from_str(&cookie)
                .map_err(|_| LeetUpError::Any(anyhow!("Unable to parse cookie string")))?;
            println!("\n{}", Color::Green("User logged in!").make());
            self.cache_session(session)?;
        }

        // github login
        if let Some(_) = user.github {
            match auth::github_login(self) {
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

#[derive(Deserialize, Debug)]
pub struct Stat {
    pub question_id: usize,

    #[serde(rename = "question__article__live")]
    pub question_article_live: Option<bool>,

    #[serde(rename = "question__article__slug")]
    pub question_article_slug: Option<String>,

    #[serde(rename = "question__title")]
    pub question_title: String,

    #[serde(rename = "question__title_slug")]
    pub question_title_slug: String,

    #[serde(rename = "question__hide")]
    pub question_hide: bool,

    pub total_acs: usize,
    pub total_submitted: usize,
    pub frontend_question_id: usize,
    pub is_new_question: bool,
}

#[derive(Deserialize, Debug)]
pub struct StatStatusPair {
    pub stat: Stat,
    pub status: Option<String>,
    pub difficulty: Difficulty,
    pub paid_only: bool,
    pub is_favor: bool,
    pub frequency: isize,
    pub progress: isize,
}

#[derive(Deserialize, Debug)]
pub struct TopicTagQuestion {
    pub status: Option<String>,
    pub difficulty: Difficulty,
    pub title: String,

    #[serde(rename = "isPaidOnly")]
    pub is_paid_only: bool,

    #[serde(rename = "titleSlug")]
    pub title_slug: String,

    #[serde(rename = "questionFrontendId")]
    pub question_frontend_id: String,
}

#[derive(Deserialize, Debug)]
pub struct ListResponse {
    pub user_name: String,
    pub num_solved: usize,
    pub num_total: usize,
    pub ac_easy: usize,
    pub ac_medium: usize,
    pub ac_hard: usize,
    pub stat_status_pairs: Vec<StatStatusPair>,
    pub frequency_high: usize,
    pub frequency_mid: usize,
    pub category_slug: String,
}

#[derive(Deserialize, Debug)]
struct CodeDefinition {
    value: String,
    text: String,

    #[serde(rename = "defaultCode")]
    default_code: String,
}

#[derive(Deserialize, Debug)]
struct SubmissionResult {
    pub code_output: Option<Either>,
    pub code_answer: Option<Either>,
    pub expected_code_output: Option<Either>,
    pub expected_code_answer: Option<Either>,
    pub compare_result: Option<String>,
    pub compile_error: Option<String>,
    pub elapsed_time: u32,
    pub full_compile_error: Option<String>,
    pub lang: String,
    pub memory: Option<u32>,
    pub memory_percentile: Option<f32>,
    pub pretty_lang: String,
    pub question_id: Option<u32>,
    pub run_success: bool,
    pub runtime_percentile: Option<f32>,
    pub state: String,
    pub status_code: u32,
    pub expected_status_code: Option<u32>,
    pub status_memory: String,
    pub status_msg: String,
    pub status_runtime: String,
    pub submission_id: String,
    pub task_finish_time: i64,
    pub expected_task_finish_time: Option<i64>,
    pub total_correct: Option<u32>,
    pub total_testcases: Option<u32>,
}

impl ProblemInfo for StatStatusPair {
    fn question_title(&self) -> &str {
        self.stat.question_title.as_str()
    }

    fn question_id(&self) -> usize {
        self.stat.frontend_question_id
    }

    fn difficulty(&self) -> &Difficulty {
        &self.difficulty
    }

    fn is_favorite(&self) -> Option<bool> {
        Some(self.is_favor)
    }

    fn is_paid_only(&self) -> bool {
        self.paid_only
    }

    fn status(&self) -> Option<&str> {
        self.status.as_ref().map(String::as_ref)
    }
}

impl ProblemInfo for TopicTagQuestion {
    fn question_title(&self) -> &str {
        self.title.as_str()
    }

    fn question_id(&self) -> usize {
        self.question_frontend_id.parse().unwrap()
    }

    fn difficulty(&self) -> &Difficulty {
        &self.difficulty
    }

    fn is_favorite(&self) -> Option<bool> {
        None
    }

    fn is_paid_only(&self) -> bool {
        self.is_paid_only
    }

    fn status(&self) -> Option<&str> {
        self.status.as_ref().map(String::as_ref)
    }
}
