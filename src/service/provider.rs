use crate::{
    cmd::{self, OrderBy, Query, User},
    icon::Icon,
    Config, LeetUpError, Result,
};
use ansi_term::Colour::{Green, Red, Yellow};
use async_trait::async_trait;
use cookie::Cookie;
use leetup_cache::kvstore::KvStore;
use serde::{Deserialize, Serialize};
use serde_repr::*;
use std::cmp::Ordering;
use std::str::FromStr;

/// ServiceProvider trait provides all the functionalities required to solve problems
/// on any type of Online Judge through leetup CLI.
#[async_trait]
pub trait ServiceProvider<'a> {
    fn session(&self) -> Option<&Session>;
    fn config(&self) -> Result<&Config>;
    async fn fetch_all_problems(&mut self) -> Result<serde_json::value::Value>;
    async fn list_problems(&mut self, list: cmd::List) -> Result<()>;
    async fn pick_problem(&mut self, pick: cmd::Pick) -> Result<()>;
    async fn problem_test(&self, test: cmd::Test) -> Result<()>;
    async fn problem_submit(&self, submit: cmd::Submit) -> Result<()>;
    async fn process_auth(&mut self, user: User) -> Result<()>;
    fn cache(&mut self) -> Result<&KvStore>;
    fn name(&self) -> &'a str;

    /// Print list of problems properly.
    fn pretty_list<T: IntoIterator<Item = &'a Box<dyn ProblemInfo + Send>>>(probs: T) {
        for prob in probs {
            let is_favorite = if let Some(is_favor) = prob.is_favorite() {
                is_favor
            } else {
                false
            };
            let starred_icon = if is_favorite {
                Yellow.paint(Icon::Star.to_string()).to_string()
            } else {
                Icon::Empty.to_string()
            };

            let locked_icon = if prob.is_paid_only() {
                Red.paint(Icon::Lock.to_string()).to_string()
            } else {
                Icon::Empty.to_string()
            };

            let acd = if prob.status().is_some() {
                Green.paint(Icon::Yes.to_string()).to_string()
            } else {
                Icon::Empty.to_string()
            };

            println!(
                "{} {:2} {} [{:^4}] {:75} {:6}",
                starred_icon,
                locked_icon,
                acd,
                prob.question_id(),
                prob.question_title(),
                prob.difficulty().to_string()
            );
        }
    }

    /// Filter problems using multiple queries.
    fn apply_queries(queries: &Vec<Query>, o: &Box<dyn ProblemInfo + Send>) -> bool {
        let mut is_satisfied = true;
        let difficulty: DifficultyType = o.difficulty().into();
        let is_favorite = if let Some(is_favor) = o.is_favorite() {
            is_favor
        } else {
            false
        };

        for q in queries {
            match q {
                Query::Easy => is_satisfied &= difficulty == Easy,
                Query::NotEasy => is_satisfied &= difficulty != Easy,
                Query::Medium => is_satisfied &= difficulty == Medium,
                Query::NotMedium => is_satisfied &= difficulty != Medium,
                Query::Hard => is_satisfied &= difficulty == Hard,
                Query::NotHard => is_satisfied &= difficulty != Hard,
                Query::Locked => is_satisfied &= o.is_paid_only(),
                Query::Unlocked => is_satisfied &= !o.is_paid_only(),
                Query::Done => is_satisfied &= o.status().is_some(),
                Query::NotDone => is_satisfied &= o.status().is_none(),
                Query::Starred => is_satisfied &= is_favorite,
                Query::Unstarred => is_satisfied &= !is_favorite,
            }
        }

        is_satisfied
    }

    /// Order problems by Id, Title, Difficulty in Ascending or Descending order
    fn with_ordering(
        orders: &[OrderBy],
        a: &Box<dyn ProblemInfo + Send>,
        b: &Box<dyn ProblemInfo + Send>,
    ) -> Ordering {
        let mut ordering = Ordering::Equal;
        let id_ordering = a.question_id().cmp(&b.question_id());
        let title_ordering = a.question_title().cmp(&b.question_title());
        let a_difficulty_level: DifficultyType = a.difficulty().into();
        let b_difficulty_level: DifficultyType = b.difficulty().into();
        let diff_ordering = a_difficulty_level.cmp(&b_difficulty_level);

        for order in orders {
            match order {
                OrderBy::IdAsc => ordering = ordering.then(id_ordering),
                OrderBy::IdDesc => ordering = ordering.then(id_ordering.reverse()),
                OrderBy::TitleAsc => ordering = ordering.then(title_ordering),
                OrderBy::TitleDesc => ordering = ordering.then(title_ordering.reverse()),
                OrderBy::DifficultyAsc => ordering = ordering.then(diff_ordering),
                OrderBy::DifficultyDesc => ordering = ordering.then(diff_ordering.reverse()),
            }
        }

        ordering
    }
}

#[derive(Debug)]
pub struct Problem {
    pub id: usize,
    pub slug: String,
    pub lang: String,
    pub link: String,
    pub typed_code: Option<String>,
}

pub trait ProblemInfo {
    fn question_id(&self) -> usize;
    fn question_title(&self) -> &str;
    fn difficulty(&self) -> &Difficulty;
    fn is_favorite(&self) -> Option<bool>;
    fn is_paid_only(&self) -> bool;
    fn status(&self) -> Option<&str>;
}

impl PartialEq for dyn ProblemInfo + '_ + Send {
    fn eq(&self, other: &Self) -> bool {
        self.question_id().eq(&other.question_id())
    }
}

impl Eq for dyn ProblemInfo + '_ + Send {}

impl PartialOrd for dyn ProblemInfo + '_ + Send {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for dyn ProblemInfo + '_ + Send {
    fn cmp(&self, other: &Self) -> Ordering {
        self.question_id().cmp(&other.question_id())
    }
}

impl PartialEq for dyn ProblemInfo {
    fn eq(&self, other: &Self) -> bool {
        self.question_id().eq(&other.question_id())
    }
}

impl Eq for dyn ProblemInfo {}

impl PartialOrd for dyn ProblemInfo {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for dyn ProblemInfo {
    fn cmp(&self, other: &Self) -> Ordering {
        self.question_id().cmp(&other.question_id())
    }
}

#[derive(Ord, PartialOrd, Eq, PartialEq, Clone, Serialize_repr, Deserialize_repr, Debug)]
#[repr(u8)]
pub enum DifficultyType {
    Easy = 1,
    Medium,
    Hard,
}

use DifficultyType::*;

impl FromStr for DifficultyType {
    type Err = LeetUpError;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        let easy = Easy.to_string();
        let medium = Medium.to_string();
        let hard = Hard.to_string();
        match s {
            x if x == easy => Ok(Easy),
            x if x == medium => Ok(Medium),
            x if x == hard => Ok(Hard),
            _ => Err(LeetUpError::UnexpectedCommand),
        }
    }
}

impl ToString for DifficultyType {
    fn to_string(&self) -> String {
        match self {
            Easy => "Easy".into(),
            Medium => "Medium".into(),
            Hard => "Hard".into(),
        }
    }
}

#[derive(Deserialize, Debug)]
#[serde(untagged)]
pub enum Difficulty {
    Cardinal { level: DifficultyType },
    String(String),
}

impl<'a> From<&'_ Difficulty> for DifficultyType {
    fn from(difficulty: &Difficulty) -> Self {
        match difficulty {
            Difficulty::Cardinal { level } => level.clone(),
            Difficulty::String(s) => DifficultyType::from_str(s).unwrap(),
        }
    }
}

impl ToString for Difficulty {
    fn to_string(&self) -> String {
        let level: DifficultyType = self.into();
        match level {
            Easy => Green.paint(Easy.to_string()).to_string(),
            Medium => Yellow.paint(Medium.to_string()).to_string(),
            Hard => Red.paint(Hard.to_string()).to_string(),
        }
    }
}

pub enum CacheKey<'a> {
    Session,
    Problems,
    Problem(&'a str),
}

impl<'a> From<CacheKey<'_>> for String {
    fn from(key: CacheKey) -> Self {
        match key {
            CacheKey::Session => "session".to_string(),
            CacheKey::Problems => "problems".to_string(),
            CacheKey::Problem(id) => format!("problem_{}", id),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Session {
    pub id: String,
    pub csrf: String,
}

impl Session {
    pub fn new(id: String, csrf: String) -> Self {
        Session { id, csrf }
    }
}

impl FromStr for Session {
    type Err = cookie::ParseError;

    fn from_str(raw: &str) -> std::result::Result<Self, Self::Err> {
        let raw_split = raw.split("; ");

        let cookies = raw_split.map(Cookie::parse).collect::<Vec<_>>();
        let mut id = String::new();
        let mut csrf = String::new();

        for cookie in cookies {
            let cookie = cookie?;
            let name = cookie.name();
            match name {
                "LEETCODE_SESSION" => id = cookie.value().to_string(),
                "csrftoken" => csrf = cookie.value().to_string(),
                _ => (),
            }
        }

        Ok(Session { id, csrf })
    }
}

fn session_to_cookie(id: &str, csrf: &str) -> String {
    let mut s = String::new();
    s.push_str(&format!("{}={}; ", "LEETCODE_SESSION", id));
    s.push_str(&format!("{}={}", "csrftoken", csrf));

    s
}

impl From<Session> for String {
    fn from(session: Session) -> Self {
        session_to_cookie(&session.id, &session.csrf)
    }
}

impl From<&Session> for String {
    fn from(session: &Session) -> Self {
        session_to_cookie(&session.id, &session.csrf)
    }
}

#[test]
fn test_cookie_parser() {
    let cookie = "csrftoken=asdsd; LEETCODE_SESSION=asdasd";
    let session: Session = Session::from_str(cookie).unwrap();

    assert!(!session.csrf.is_empty());
    assert!(!session.id.is_empty());
}
