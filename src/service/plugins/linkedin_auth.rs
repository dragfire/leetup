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
    fn process_auth(&mut self, user: &cmd::User) -> Result<()> {
        if user.linkedin.is_none() {
            return Ok(());
        }
        println!("Processing Linkedin authentication");
        Ok(())
    }

    fn name(&self) -> String {
        self.name.to_owned()
    }
}
