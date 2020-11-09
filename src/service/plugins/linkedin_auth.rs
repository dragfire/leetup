use crate::{
    client,
    cmd::{self, Command, List, OrderBy, Query, User},
    icon::Icon,
    service::{
        self, auth, CacheKey, Comment, CommentStyle, Lang, LangInfo, Problem, ServiceProvider,
        Session,
    },
    template::{parse_code, InjectPosition, Pattern},
    Config, Either, InjectCode, LeetUpError, Result, Urls,
};
use leetup_cache::kvstore::KvStore;

pub struct LinkedinAuth {
    name: String,
}

impl LinkedinAuth {
    pub fn new() -> Self {
        Self {
            name: "linkedin_auth".to_string(),
        }
    }
}

impl ServiceProvider for LinkedinAuth {
    fn session(&self) -> Option<&Session> {
        None
    }

    fn config(&self) -> Result<Option<&Config>> {
        Ok(None)
    }

    fn fetch_all_problems(&mut self) -> Result<Option<serde_json::value::Value>> {
        Ok(None)
    }

    fn list_problems(&mut self, list: &cmd::List) -> Result<()> {
        Ok(())
    }

    fn pick_problem(&mut self, pick: &cmd::Pick) -> Result<()> {
        Ok(())
    }

    fn problem_test(&self, test: &cmd::Test) -> Result<()> {
        Ok(())
    }

    fn problem_submit(&self, submit: &cmd::Submit) -> Result<()> {
        Ok(())
    }

    fn process_auth(&mut self, user: &cmd::User) -> Result<()> {
        if user.linkedin.is_none() {
            return Ok(());
        }
        println!("Processing Linkedin authentication");
        Ok(())
    }

    fn cache(&mut self) -> Result<Option<&KvStore>> {
        Ok(None)
    }

    fn name(&self) -> String {
        self.name.to_owned()
    }
}
