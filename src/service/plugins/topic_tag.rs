use crate::{client, cmd, service::ServiceProvider, Result};

pub struct TopicTag {
    name: String,
}

impl TopicTag {
    pub fn new() -> Self {
        Self {
            name: "topic_tag".to_string(),
        }
    }
}

impl ServiceProvider for TopicTag {
    fn list_problems(&mut self, list: &cmd::List) -> Result<()> {
        if let Some(ref tag) = list.tag {
            println!("Fetching tag: {}", tag);
        }

        Ok(())
    }

    fn name(&self) -> String {
        self.name.to_owned()
    }
}
