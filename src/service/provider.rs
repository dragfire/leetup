use crate::{
    cmd::{self, Command, User},
    Result,
};
use cookie::Cookie;
use serde::{Deserialize, Serialize};
use std::str::FromStr;

/// ServiceProvider trait provides all the functionalities required to solve problems
/// on any type of Online Judge through leetup CLI.
pub trait ServiceProvider<'a> {
    fn session(&self) -> Option<&Session>;
    fn config(&self) -> Result<&Config>;
    fn list_problems(&mut self, list: cmd::List) -> Result<()>;
    fn pick_problem(&self, pick: Command) -> Result<()>;
    fn problem_test(&self) -> Result<()>;
    fn problem_submit(&self) -> Result<()>;
    fn process_auth(&mut self, user: User) -> Result<()>;
    fn cache(&mut self) -> Result<&Cache>;
    fn name(&self) -> &'a str;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
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
        let raw_split = raw.split_whitespace();

        // get all cookies in iterator
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

impl From<Session> for String {
    fn from(session: Session) -> Self {
        let mut s = String::new();
        s.push_str(&format!("{}={}; ", "LEETCODE_SESSION", session.id));
        s.push_str(&format!("{}={}", "csrftoken", session.csrf));

        s
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Urls {
    pub base: String,
    pub api: String,
    pub problems_all: String,
    pub github_login: String,
    pub github_login_request: String,
    pub github_session_request: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub urls: Urls,
}

impl Config {
    pub fn new(urls: Urls) -> Self {
        Config { urls }
    }
}

pub struct Cache;
