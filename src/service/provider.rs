// TODO refactor this file
use crate::{
    cmd::{self, Command},
    Config, Result,
};
use cookie::Cookie;
use leetup_cache::kvstore::KvStore;
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::str::FromStr;

/// ServiceProvider trait provides all the functionalities required to solve problems
/// on any type of Online Judge through leetup CLI.
pub trait ServiceProvider {
    fn session(&self) -> Option<&Session>;
    fn config(&self) -> Result<&Config>;
    fn fetch_all_problems(&mut self) -> Result<serde_json::value::Value>;
    fn list_problems(&mut self, list: &cmd::List) -> Result<()>;
    fn pick_problem(&mut self, pick: &cmd::Pick) -> Result<()>;
    fn problem_test(&self, test: &cmd::Test) -> Result<()>;
    fn problem_submit(&self, submit: &cmd::Submit) -> Result<()>;
    fn process_auth(&mut self, user: &cmd::User) -> Result<()>;
    fn cache(&mut self) -> Result<&KvStore>;
    fn name(&self) -> String;
}

#[derive(Debug)]
pub struct Problem {
    pub id: usize,
    pub slug: String,
    pub lang: String,
    pub link: String,
    pub typed_code: Option<String>,
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
