use std::cmp::Ordering;

use ansi_term::Colour::{Green, Red, Yellow};
use async_trait::async_trait;
use leetup_cache::kvstore::KvStore;

use crate::model::DifficultyType::{Easy, Hard, Medium};
use crate::model::{DifficultyType, ProblemInfo};
use crate::service::Session;
use crate::{
    cmd::{self, OrderBy, Query, User},
    icon::Icon,
    Config, Result,
};

/// ServiceProvider trait provides all the functionalities required to solve problems
/// on any type of Online Judge through leetup CLI.
#[async_trait]
pub trait ServiceProvider<'a> {
    fn session(&self) -> Option<&Session>;
    fn config(&self) -> Result<&Config>;
    async fn fetch_all_problems(&mut self) -> Result<serde_json::value::Value>;
    async fn list_problems(&mut self, list: cmd::List) -> Result<()>;
    async fn pick_problem(&mut self, pick: cmd::Pick) -> Result<()>;
    async fn problem_test(&self, test: cmd::Test) -> Result<()>;
    async fn problem_submit(&self, submit: cmd::Submit) -> Result<()>;
    async fn process_auth(&mut self, user: User) -> Result<()>;
    fn cache(&mut self) -> Result<&KvStore>;
    fn name(&self) -> &'a str;

    /// Print list of problems properly.
    fn pretty_list<T: IntoIterator<Item = &'a Box<dyn ProblemInfo + Send>>>(probs: T) {
        for prob in probs {
            let is_favorite = if let Some(is_favor) = prob.is_favorite() {
                is_favor
            } else {
                false
            };
            let starred_icon = if is_favorite {
                Yellow.paint(Icon::Star.to_string()).to_string()
            } else {
                Icon::Empty.to_string()
            };

            let locked_icon = if prob.is_paid_only() {
                Red.paint(Icon::Lock.to_string()).to_string()
            } else {
                Icon::Empty.to_string()
            };

            let acd = if prob.status().is_some() {
                Green.paint(Icon::Yes.to_string()).to_string()
            } else {
                Icon::Empty.to_string()
            };

            println!(
                "{} {:2} {} [{:^4}] {:75} {:6}",
                starred_icon,
                locked_icon,
                acd,
                prob.question_id(),
                prob.question_title(),
                prob.difficulty().to_string()
            );
        }
    }

    /// Filter problems using multiple queries.
    fn apply_queries(queries: &Vec<Query>, o: &Box<dyn ProblemInfo + Send>) -> bool {
        let mut is_satisfied = true;
        let difficulty: DifficultyType = o.difficulty().into();
        let is_favorite = if let Some(is_favor) = o.is_favorite() {
            is_favor
        } else {
            false
        };

        for q in queries {
            match q {
                Query::Easy => is_satisfied &= difficulty == Easy,
                Query::NotEasy => is_satisfied &= difficulty != Easy,
                Query::Medium => is_satisfied &= difficulty == Medium,
                Query::NotMedium => is_satisfied &= difficulty != Medium,
                Query::Hard => is_satisfied &= difficulty == Hard,
                Query::NotHard => is_satisfied &= difficulty != Hard,
                Query::Locked => is_satisfied &= o.is_paid_only(),
                Query::Unlocked => is_satisfied &= !o.is_paid_only(),
                Query::Done => is_satisfied &= o.status().is_some(),
                Query::NotDone => is_satisfied &= o.status().is_none(),
                Query::Starred => is_satisfied &= is_favorite,
                Query::Unstarred => is_satisfied &= !is_favorite,
            }
        }

        is_satisfied
    }

    /// Order problems by Id, Title, Difficulty in Ascending or Descending order
    fn with_ordering(
        orders: &[OrderBy],
        a: &Box<dyn ProblemInfo + Send>,
        b: &Box<dyn ProblemInfo + Send>,
    ) -> Ordering {
        let mut ordering = Ordering::Equal;
        let id_ordering = a.question_id().cmp(&b.question_id());
        let title_ordering = a.question_title().cmp(&b.question_title());
        let a_difficulty_level: DifficultyType = a.difficulty().into();
        let b_difficulty_level: DifficultyType = b.difficulty().into();
        let diff_ordering = a_difficulty_level.cmp(&b_difficulty_level);

        for order in orders {
            match order {
                OrderBy::IdAsc => ordering = ordering.then(id_ordering),
                OrderBy::IdDesc => ordering = ordering.then(id_ordering.reverse()),
                OrderBy::TitleAsc => ordering = ordering.then(title_ordering),
                OrderBy::TitleDesc => ordering = ordering.then(title_ordering.reverse()),
                OrderBy::DifficultyAsc => ordering = ordering.then(diff_ordering),
                OrderBy::DifficultyDesc => ordering = ordering.then(diff_ordering.reverse()),
            }
        }

        ordering
    }
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
