use serde::{Deserialize, Serialize};
use std::path::Path;

// TODO move to ~/.leetup/config.json
#[derive(Debug, Serialize, Deserialize)]
pub struct Urls {
    pub base: String,
    pub api: String,
    pub graphql: String,
    pub problems: String,
    pub problems_all: String,
    pub github_login: String,
    pub github_login_request: String,
    pub github_session_request: String,
    pub test: String,
    pub submit: String,
    pub submissions: String,
    pub submission: String,
    pub verify: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub urls: Urls,
}

impl Config {
    fn new(urls: Urls) -> Self {
        Config { urls }
    }

    pub fn get<P: AsRef<Path>>(path: P) -> Self {
        // TODO read from ~/.leetup/config.json
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

        Config::new(urls)
    }
}
