use crate::{LeetUpError, Result};
use serde::{de::DeserializeOwned, Deserialize};
use std::collections::HashMap;
use std::fs::File;
use std::io::Read;
use std::path::Path;

type LangInjectCode = HashMap<String, InjectCode>;
type PickHookConfig = HashMap<String, PickHook>;

#[derive(Debug, Deserialize)]
pub struct Config {
    #[serde(skip)]
    pub urls: Urls,
    pub inject_code: Option<LangInjectCode>,
    pub pick_hook: Option<PickHookConfig>,
}

impl Config {
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

        let mut config: Result<Config> = Config::get_config(path);

        if let Ok(ref mut config) = config {
            config.urls = urls;
        }

        config
    }

    fn get_config<P: AsRef<Path>, T: DeserializeOwned>(path: P) -> Result<T> {
        let mut buf = String::new();
        let mut file = File::open(path)?;
        file.read_to_string(&mut buf)?;

        serde_json::from_str(&buf).map_err(LeetUpError::Serde)
    }
}

#[derive(Deserialize, Debug)]
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

#[derive(Debug, Default, Deserialize)]
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

#[derive(Debug, Deserialize)]
pub struct InjectCode {
    pub before_code: Option<Either>,
    pub before_code_exclude: Option<Either>,
    pub after_code: Option<Either>,
    pub before_function_definition: Option<Either>,
}

/// Make code generation more flexible with capabilities to run scripts before
/// and after generation.
///
/// Provide the ability to change filenames through certain pre-defined transformation actions.
#[derive(Debug, Deserialize)]
pub struct PickHook {
    working_dir: String,
    script: Option<PickHookScript>,
    transform: Option<PickHookTransform>,
}

#[derive(Debug, Deserialize)]
pub struct PickHookScript {
    pre_generation: Option<Either>,
    post_generation: Option<Either>,
}

#[derive(Debug, Deserialize)]
pub struct PickHookTransform {
    filename: String,
    action: PickHookTransformAction,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PickHookTransformAction {
    Rename,
    RenameCamelCase,
    CamelCase,
}

#[test]
fn test_config() {
    use std::io::Write;

    let data_dir = tempfile::tempdir().unwrap();
    let data = serde_json::json!({
        "inject_code": {},
        "urls": {
            "base": vec![""]
        },
        "pick_hook": {}
    });
    let file_path = data_dir.path().join("config.json");

    let mut file = std::fs::File::create(&file_path).unwrap();
    file.write(data.to_string().as_bytes()).unwrap();

    let config: Config = Config::get(&file_path).unwrap();
    assert!(config.inject_code.is_some());
    assert!(config.urls.base.len() > 0);
    assert!(config.pick_hook.is_some());
    drop(file);
    data_dir.close().unwrap();
}
