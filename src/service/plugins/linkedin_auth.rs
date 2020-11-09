use crate::{cmd, service::ServiceProvider, LeetUpError, Result};
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
