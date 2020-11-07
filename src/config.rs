use crate::{LeetUpError, Result};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::File;
use std::io::Read;
use std::path::Path;

type LangInjectCode = HashMap<String, InjectCode>;

// TODO move to ~/.leetup/config.json
#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub urls: Urls,
    pub inject_code: Option<LangInjectCode>,
}

impl Config {
    fn new(urls: Urls, inject_code: Option<LangInjectCode>) -> Self {
        Config { urls, inject_code }
    }

    pub fn get<P: AsRef<Path>>(path: P) -> Result<Self> {
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

        let mut inject_code: Option<LangInjectCode> = None;
        let config: Result<serde_json::Value> = Config::get_config(path);
        if let Ok(config) = config {
            inject_code =
                serde_json::from_value::<LangInjectCode>(config["inject_code"].clone()).ok();
        }

        Ok(Config::new(urls, inject_code))
    }

    fn get_config<P: AsRef<Path>, T: DeserializeOwned>(path: P) -> Result<T> {
        let mut buf = String::new();
        let mut file = File::open(path)?;
        file.read_to_string(&mut buf)?;

        serde_json::from_str(&buf).map_err(LeetUpError::Serde)
    }
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(untagged)]
pub enum Either {
    Sequence(Vec<String>),
    String(String),
}

impl ToString for Either {
    fn to_string(&self) -> String {
        match self {
            Either::String(s) => s.to_owned(),
            Either::Sequence(v) => v.join("\n"),
        }
    }
}

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
pub struct InjectCode {
    pub before_code: Option<Either>,
    pub before_code_exclude: Option<Either>,
    pub after_code: Option<Either>,
    pub before_function_definition: Option<Either>,
}

#[test]
fn test_config() {
    let mut data_dir = std::path::PathBuf::new();
    data_dir.push(dirs::home_dir().expect("Home directory not available!"));
    data_dir.push(".leetup");
    data_dir.push("config.json");
    let config: Config = Config::get(data_dir).unwrap();
    assert!(config.inject_code.is_some());
    assert!(config.urls.base.len() > 0);
}
