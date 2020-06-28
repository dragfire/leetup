use crate::{
    cmd::{self, Command},
    Result,
};
use serde::{Deserialize, Serialize};

/// ServiceProvider trait provides all the functionalities required to solve problems
/// on any type of Online Judge through leetup CLI.
/// ```
///     trait ServiceProvider {
///         fn session() -> Result<Session>;
///         fn config() -> Result<Config>;
///         fn list_problems(list: cmd::List) -> Result<()>;
///         fn pick_problem(pick: cmd::Pick) -> Result<()>;
///         fn problem_test() -> Result<()>;
///         fn problem_submit() -> Result<()>;
///         fn login() -> Result<()>;
///         fn logout() -> Result<()>;
///         fn cache() -> Result<Cache>;
///     }
///
///     impl ServiceProvider for Leetcode {
///     }
///
///     impl ServiceProvider for Lintcode {
///     }
///
///     impl ServiceProvider for Cses {
///     }
///
///
///     leet_provider.list_problems();
///     lint_provider.list_problems();
///
/// ```
pub trait ServiceProvider<'a> {
    fn session(&self) -> Option<&Session>;
    fn config(&self) -> Result<&Config>;
    fn list_problems(&self, list: cmd::List) -> Result<()>;
    fn pick_problem(&self, pick: Command) -> Result<()>;
    fn problem_test(&self) -> Result<()>;
    fn problem_submit(&self) -> Result<()>;
    fn login(&mut self) -> Result<()>;
    fn logout(&mut self) -> Result<()>;
    fn cache(&mut self) -> Result<&Cache>;
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Session<'a> {
    pub cookie: &'a str,
}

impl<'a> Session<'a> {
    pub fn new(cookie: &'a str) -> Self {
        Session { cookie }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Urls {
    pub base: String,
    pub api: String,
    pub problems_all: String,
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
