use crate::{
    cmd::{Command, List, OrderBy, Query, User},
    fetch,
    icon::Icon,
    service::{auth, Cache, Config, ServiceProvider, Session, Urls},
    LeetUpError, Result,
};
use ansi_term::Colour::{Green, Red, Yellow};
use cache::kvstore::KvStore;
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::env;
use std::path::PathBuf;

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
        let urls = Urls {
            base: "https://leetcode.com".to_string(),
            api: "https://leetcode.com/api".to_string(),
            problems_all: "https://leetcode.com/api/problems/all".to_string(),
            github_login: "https://leetcode.com/accounts/github/login/?next=%2F".to_string(),
            github_login_request: "https://github.com/login".to_string(),
            github_session_request: "https://github.com/session".to_string(),
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
        let cookie = cache.get("cookie".to_string()).unwrap();

        // Set session if the user is logged in
        if let Some(val) = cookie {
            session = Some(Session::new(val));
        }

        Leetcode {
            session,
            config,
            cache,
            name,
        }
    }
}

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

/// Fetch all problems
pub fn fetch_all_problems<'a, P: ServiceProvider<'a>>(provider: &P) -> Result<ListResponse> {
    let url = &provider.config()?.urls.problems_all;
    let session = provider.session();
    let mut headers = reqwest::header::HeaderMap::new();
    if let Some(sess) = session {
        let cookie = sess.cookie.parse().unwrap();
        headers.insert("Cookie", cookie);
    }
    fetch::get(url, headers)?
        .json::<ListResponse>()
        .map_err(LeetUpError::Reqwest)
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

impl<'a> ServiceProvider<'a> for Leetcode<'a> {
    fn session(&self) -> Option<&Session> {
        self.session.as_ref()
    }

    fn config(&self) -> Result<&Config> {
        Ok(&self.config)
    }

    fn list_problems(&mut self, list: List) -> Result<()> {
        let mut problems_res: ListResponse;
        if let Some(ref val) = self.cache.get("problems".to_string())? {
            problems_res = serde_json::from_str::<ListResponse>(val)?;
        } else {
            problems_res = fetch_all_problems(self)?;
            let res_serialized = serde_json::to_string(&problems_res)?;
            self.cache.set("problems".to_string(), res_serialized)?;
        }

        let probs = &mut problems_res.stat_status_pairs;

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

    fn pick_problem(&self, _pick: Command) -> Result<()> {
        panic!();
    }

    fn problem_test(&self) -> Result<()> {
        panic!();
    }

    fn problem_submit(&self) -> Result<()> {
        panic!();
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

            self.cache.set("cookie".to_string(), cookie)?;

            // remove key `problems`, rebuild problems cache.
            //
            // NOTE: cache.remove throws "Key not found" error
            // so ignore that error if it is thrown.
            if let Err(_) = self.cache.remove("problems".to_string()) {}
        }

        // github login
        if let Some(_) = user.github {
            auth::github_login(self)?;
        }

        Ok(())
    }

    fn cache(&mut self) -> Result<&Cache> {
        panic!();
    }
}
