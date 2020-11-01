use crate::{
    client,
    cmd::{self, Command, List, OrderBy, Query, User},
    icon::Icon,
    service::{
        self, auth, CommentStyle, Config, Lang, LangInfo, Problem, ServiceProvider, Session, Urls,
    },
    LeetUpError, Result,
};
use ansi_term::Colour::{Green, Red, Yellow};
use anyhow::anyhow;
use cache::kvstore::KvStore;
use colci::Color;
use html2text::from_read;
use log::{debug, info};
use reqwest::header::{self, HeaderMap, HeaderValue};
use serde::{Deserialize, Serialize};
use serde_json::json;
use spinners::{Spinner, Spinners};
use std::cmp::Ordering;
use std::collections::HashMap;
use std::env;
use std::fs::File;
use std::io::prelude::*;
use std::io::BufWriter;
use std::path::PathBuf;
use std::str::FromStr;

#[derive(Serialize, Deserialize, Debug, Ord, PartialOrd, Eq, PartialEq)]
pub struct Difficulty {
    pub level: usize,
}

impl ToString for Difficulty {
    fn to_string(&self) -> String {
        match self.level {
            1 => Green.paint(String::from("Easy")).to_string(),
            2 => Yellow.paint(String::from("Medium")).to_string(),
            3 => Red.paint(String::from("Hard")).to_string(),
            _ => String::from("UnknownLevel"),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Ord, PartialOrd, Eq, PartialEq)]
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

#[derive(Serialize, Deserialize, Debug, Ord, PartialOrd, Eq, PartialEq)]
pub struct StatStatusPair {
    pub stat: Stat,
    pub status: Option<String>,
    pub difficulty: Difficulty,
    pub paid_only: bool,
    pub is_favor: bool,
    pub frequency: isize,
    pub progress: isize,
}

#[derive(Serialize, Deserialize, Debug)]
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

#[derive(Serialize, Deserialize, Debug)]
struct CodeDefinition {
    value: String,
    text: String,

    #[serde(rename = "defaultCode")]
    default_code: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct SubmissionResult {
    pub code_output: Option<String>,
    pub compare_result: Option<String>,
    pub compile_error: Option<String>,
    pub elapsed_time: u32,
    pub full_compile_error: Option<String>,
    pub lang: String,
    pub memory: u32,
    pub memory_percentile: Option<f32>,
    pub pretty_lang: String,
    pub question_id: u32,
    pub run_success: bool,
    pub runtime_percentile: Option<f32>,
    pub state: String,
    pub status_code: u32,
    pub status_memory: String,
    pub status_msg: String,
    pub status_runtime: String,
    pub submission_id: String,
    pub task_finish_time: i64,
    pub total_correct: Option<u32>,
    pub total_testcases: Option<u32>,
}

fn pretty_list<'a, T: Iterator<Item = &'a StatStatusPair>>(probs: T) {
    for obj in probs {
        let qstat = &obj.stat;

        let starred_icon = if obj.is_favor {
            Yellow.paint(Icon::Star.to_string()).to_string()
        } else {
            Icon::Empty.to_string()
        };

        let locked_icon = if obj.paid_only {
            Red.paint(Icon::Lock.to_string()).to_string()
        } else {
            Icon::Empty.to_string()
        };

        let acd = if obj.status.is_some() {
            Green.paint(Icon::Yes.to_string()).to_string()
        } else {
            Icon::Empty.to_string()
        };

        println!(
            "{} {:2} {} [{:^4}] {:75} {:6}",
            starred_icon,
            locked_icon,
            acd,
            qstat.question_id,
            qstat.question_title,
            obj.difficulty.to_string()
        );
    }
}

fn apply_queries(queries: &Vec<Query>, o: &StatStatusPair) -> bool {
    let mut is_satisfied = true;

    for q in queries {
        match q {
            Query::Easy => is_satisfied &= o.difficulty.level == 1,
            Query::NotEasy => is_satisfied &= o.difficulty.level != 1,
            Query::Medium => is_satisfied &= o.difficulty.level == 2,
            Query::NotMedium => is_satisfied &= o.difficulty.level != 2,
            Query::Hard => is_satisfied &= o.difficulty.level == 3,
            Query::NotHard => is_satisfied &= o.difficulty.level != 3,
            Query::Locked => is_satisfied &= o.paid_only,
            Query::Unlocked => is_satisfied &= !o.paid_only,
            Query::Done => is_satisfied &= o.status.is_some(),
            Query::NotDone => is_satisfied &= o.status.is_none(),
            Query::Starred => is_satisfied &= o.is_favor,
            Query::Unstarred => is_satisfied &= !o.is_favor,
        }
    }

    is_satisfied
}

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
        let base = "https://leetcode.com";
        let urls = Urls {
            base: base.to_owned(),
            api: format!("{}/api", base),
            graphql: format!("{}/graphql", base),
            problems: format!("{}/problems/", base),
            problems_all: format!("{}/api/problems/all", base),
            github_login: format!("{}/accounts/github/login/?next=%2F", base),
            github_login_request: "https://github.com/login".to_string(),
            github_session_request: "https://github.com/session".to_string(),
            test: format!("{}/problems/$slug/interpret_solution/", base),
            submit: format!("{}/problems/$slug/submit/", base),
            submissions: format!("{}/api/submissions/$slug", base),
            submission: format!("{}/submissions/detail/$id", base),
            verify: format!("{}/submissions/detail/$id/check/", base),
        };
        let name = "leetcode";
        let config = Config::new(urls);

        // create a data directory: ./data/leetcode/*.log
        let mut data_dir = PathBuf::new();
        data_dir.push(env::current_dir().unwrap());
        data_dir.push("data");
        data_dir.push("leetcode");

        let mut cache = KvStore::open(data_dir).unwrap();
        let mut session: Option<Session> = None;
        let session_val = cache.get("session".to_string()).unwrap();

        // Set session if the user is logged in
        if let Some(ref val) = session_val {
            session =
                Some(serde_json::from_str::<Session>(val).expect("Session format not correct"));
        }

        Leetcode {
            session,
            config,
            cache,
            name,
        }
    }

    fn cache_session(&mut self, session: Session) -> Result<()> {
        let session_str = serde_json::to_string(&session)?;
        self.cache.set("session".to_string(), session_str)?;
        self.session = Some(session);
        // remove key `problems`, rebuild problems cache.
        //
        // NOTE: cache.remove throws "Key not found" error
        // so ignore that error if it is thrown.
        if let Err(_) = self.cache.remove("problems".to_string()) {}
        Ok(())
    }

    pub fn fetch_problems(&mut self) -> Result<Vec<StatStatusPair>> {
        let problems = self.fetch_all_problems()?;
        let problems: Vec<StatStatusPair> =
            serde_json::from_value(problems["stat_status_pairs"].clone())?;

        Ok(problems)
    }

    fn run_code(&self, problem: Problem, body: serde_json::Value) -> Result<serde_json::Value> {
        let url = &self.config()?.urls.submit.replace("$slug", &problem.slug);
        client::post(self, url, &body, || {
            let mut headers = HeaderMap::new();
            headers.insert(
                header::REFERER,
                HeaderValue::from_str(&problem.link).unwrap(),
            );
            Some(headers)
        })
    }

    fn verify_run_code(&self, submission: serde_json::Value) -> Result<serde_json::Value> {
        loop {
            let url = self
                .config
                .urls
                .verify
                .replace("$id", &submission["submission_id"].to_string());
            let response = client::get(&url, None, self.session())?.json::<serde_json::Value>()?;
            if response["state"] == "SUCCESS" {
                return Ok(response);
            }
        }
    }

    fn print_judge_result(&self, result: SubmissionResult) {
        match result.status_code {
            10 => {
                // Accepted
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
 Failed Test: {}
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
        if let Some(ref val) = self.cache.get("problems".to_string())? {
            debug!("Fetching problems from cache...");
            problems_res = serde_json::from_str::<serde_json::value::Value>(val)?;
        } else {
            let url = &self.config.urls.problems_all;
            let session = self.session();
            problems_res = client::get(url, None, session)?
                .json::<serde_json::value::Value>()
                .map_err(LeetUpError::Reqwest)?;
            let res_serialized = serde_json::to_string(&problems_res)?;
            self.cache.set("problems".to_string(), res_serialized)?;
        }

        Ok(problems_res)
    }

    fn list_problems(&mut self, list: List) -> Result<()> {
        let problems_res = self.fetch_all_problems()?;
        let mut probs: Vec<StatStatusPair> =
            serde_json::from_value(problems_res["stat_status_pairs"].clone())?;

        if list.order.is_some() {
            let orders = OrderBy::from_str(list.order.as_ref().unwrap());

            probs.sort_by(|a, b| {
                let mut ordering = Ordering::Equal;
                let id_ordering = a.stat.question_id.cmp(&b.stat.question_id);
                let title_ordering = a.stat.question_title_slug.cmp(&b.stat.question_title_slug);
                let diff_ordering = a.difficulty.level.cmp(&b.difficulty.level);

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
            let filter_predicate = |o: &&StatStatusPair| {
                let default_keyword = String::from("");
                let keyword = list
                    .keyword
                    .as_ref()
                    .unwrap_or(&default_keyword)
                    .to_ascii_lowercase();
                let has_keyword = o.stat.question_title_slug.contains(&keyword);

                if list.query.is_none() {
                    has_keyword
                } else {
                    let queries: Vec<Query> = Query::from_str(list.query.as_ref().unwrap());
                    has_keyword && apply_queries(&queries, o)
                }
            };

            let filtered_probs: Vec<&StatStatusPair> =
                probs.iter().filter(filter_predicate).collect();

            pretty_list(filtered_probs.into_iter());
        } else {
            pretty_list(probs.iter());
        }

        Ok(())
    }

    fn pick_problem(&mut self, pick: cmd::Pick) -> Result<()> {
        let probs = self.fetch_problems()?;
        let urls = &self.config.urls;
        let lang = match pick.lang.clone() {
            Lang::Rust(info) => info,
            Lang::Java(info) => info,
            Lang::Javascript(info) => info,
            Lang::Python3(info) => info,
            Lang::MySQL(info) => info,
        };
        let problem: Problem = probs
            .iter()
            .find(|item| item.stat.question_id == pick.id.unwrap())
            .map(|item| Problem {
                id: item.stat.question_id,
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

        if let Some(content) = &response["data"]["question"]["content"].as_str() {
            let content = from_read(content.as_bytes(), 80);
            let content = content.replace("**", "");
            let content = content
                .split('\n')
                .map(|s| format!("// {}", s))
                .collect::<Vec<String>>()
                .join("\n");
            let content = format!(
                "// @leetup id={} lang={} slug={}\n\n{}",
                problem_id, lang.name, slug, content
            );
            debug!("Content: {}", content);
            definition = Some(content);
        }

        let mut filename = env::current_dir()?;
        filename.push(slug);
        filename.set_extension(&lang.extension);

        if let Some(code_defs) = &response["data"]["question"]["codeDefinition"].as_str() {
            let code_defs: Vec<(String, CodeDefinition)> =
                serde_json::from_str::<Vec<CodeDefinition>>(code_defs)?
                    .into_iter()
                    .map(|def| (def.value.to_owned(), def))
                    .collect();
            let code_defs: HashMap<_, _> = code_defs.into_iter().collect();
            let mut writer = BufWriter::new(File::create(&filename)?);
            if let Some(definition) = definition {
                writer.write_all(definition.as_bytes())?;
            }
            let code = &code_defs.get(&lang.name).unwrap().default_code;
            debug!("Code: {}", code);
            writer.write(b"\n\n\n")?;
            writer.write_all(code.as_bytes())?;
            writer.flush()?;
            println!(
                "Generated: {}",
                Color::Magenta(filename.to_str().unwrap()).make()
            );
        }

        Ok(())
    }

    fn problem_test(&self, test: cmd::Test) -> Result<()> {
        let problem = service::extract_problem(test.filename)?;
        let body = json!({
                "lang":        problem.lang.to_owned(),
                "question_id": problem.id,
                "test_mode":   true,
                "typed_code":  problem.typed_code.as_ref().unwrap()
        });
        let response = self.run_code(problem, body)?;
        let response = self.verify_run_code(response)?;
        debug!("Verification result: {:#?}", response);

        Ok(())
    }

    fn problem_submit(&self, submit: cmd::Submit) -> Result<()> {
        let problem = service::extract_problem(submit.filename)?;
        let body = json!({
            "lang":        problem.lang.to_owned(),
            "question_id": problem.id,
            "test_mode":   false,
            "typed_code":  problem.typed_code.as_ref().unwrap(),
            "judge_type": "large",
        });
        let sp = Spinner::new(Spinners::Dots9, "Waiting for judge result!".into());
        let response = self.run_code(problem, body)?;
        let result: SubmissionResult = serde_json::from_value(self.verify_run_code(response)?)?;
        sp.stop();
        self.print_judge_result(result);

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
                cookie = String::from(cookie.trim_end());
            }

            // filter out all unnecessary cookies
            let session = Session::from_str(&cookie)
                .map_err(|_| LeetUpError::Any(anyhow!("Unable to parse cookie string")))?;
            self.cache_session(session)?;
        }

        // github login
        if let Some(_) = user.github {
            match auth::github_login(self) {
                Ok(session) => {
                    println!("{}", Color::Green("User logged in!").make());
                    self.cache_session(session)?;
                }
                Err(_) => {
                    println!("{}", Color::Red("Github login failed!").make());
                }
            }
        }

        if user.logout.is_some() {
            if let Err(_) = self.cache.remove("session".to_string()) {
                println!("User not logged in!");
                return Ok(());
            }
            if let Err(_) = self.cache.remove("problems".to_string()) {}
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
