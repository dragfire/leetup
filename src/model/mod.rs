use std::cmp::Ordering;
use std::str::FromStr;

use ansi_term::Color::{Green, Red, Yellow};
use serde::Deserialize;
use serde_repr::{Deserialize_repr, Serialize_repr};

use DifficultyType::*;

use crate::{Either, LeetUpError};

#[derive(Debug)]
pub struct Problem {
    pub id: usize,
    pub slug: String,
    pub lang: String,
    pub link: String,
    pub typed_code: Option<String>,
}

#[derive(Ord, PartialOrd, Eq, PartialEq, Clone, Serialize_repr, Deserialize_repr, Debug)]
#[repr(u8)]
pub enum DifficultyType {
    Easy = 1,
    Medium,
    Hard,
}

impl FromStr for DifficultyType {
    type Err = LeetUpError;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        let easy = Easy.to_string();
        let medium = Medium.to_string();
        let hard = Hard.to_string();
        match s {
            x if x == easy => Ok(Easy),
            x if x == medium => Ok(Medium),
            x if x == hard => Ok(Hard),
            _ => Err(LeetUpError::UnexpectedCommand),
        }
    }
}

impl ToString for DifficultyType {
    fn to_string(&self) -> String {
        match self {
            Easy => "Easy".into(),
            Medium => "Medium".into(),
            Hard => "Hard".into(),
        }
    }
}

#[derive(Deserialize, Debug)]
#[serde(untagged)]
pub enum Difficulty {
    Cardinal { level: DifficultyType },
    String(String),
}

impl<'a> From<&'_ Difficulty> for DifficultyType {
    fn from(difficulty: &Difficulty) -> Self {
        match difficulty {
            Difficulty::Cardinal { level } => level.clone(),
            Difficulty::String(s) => DifficultyType::from_str(s).unwrap(),
        }
    }
}

impl ToString for Difficulty {
    fn to_string(&self) -> String {
        let level: DifficultyType = self.into();
        match level {
            Easy => Green.paint(Easy.to_string()).to_string(),
            Medium => Yellow.paint(Medium.to_string()).to_string(),
            Hard => Red.paint(Hard.to_string()).to_string(),
        }
    }
}

pub type ProblemInfoSeq = Vec<Box<dyn ProblemInfo + Send + 'static>>;

pub trait ProblemInfo {
    fn question_id(&self) -> usize;
    fn question_title(&self) -> &str;
    fn difficulty(&self) -> &Difficulty;
    fn is_favorite(&self) -> Option<bool>;
    fn is_paid_only(&self) -> bool;
    fn status(&self) -> Option<&str>;
}

impl PartialEq<Self> for dyn ProblemInfo + '_ + Send {
    fn eq(&self, other: &Self) -> bool {
        self.question_id().eq(&other.question_id())
    }
}

impl Eq for dyn ProblemInfo + '_ + Send {}

impl PartialOrd for dyn ProblemInfo + '_ + Send {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for dyn ProblemInfo + '_ + Send {
    fn cmp(&self, other: &Self) -> Ordering {
        self.question_id().cmp(&other.question_id())
    }
}

impl PartialEq for dyn ProblemInfo {
    fn eq(&self, other: &Self) -> bool {
        self.question_id().eq(&other.question_id())
    }
}

impl Eq for dyn ProblemInfo {}

impl PartialOrd for dyn ProblemInfo {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for dyn ProblemInfo {
    fn cmp(&self, other: &Self) -> Ordering {
        self.question_id().cmp(&other.question_id())
    }
}

#[derive(Deserialize, Debug)]
pub struct Stat {
    pub question_id: usize,

    #[serde(rename = "question__article__live")]
    pub question_article_live: Option<bool>,

    #[serde(rename = "question__article__slug")]
    pub question_article_slug: Option<String>,

    #[serde(rename = "question__title")]
    pub question_title: String,

    #[serde(rename = "question__title_slug")]
    pub question_title_slug: String,

    #[serde(rename = "question__hide")]
    pub question_hide: bool,

    pub total_acs: usize,
    pub total_submitted: usize,
    pub frontend_question_id: usize,
    pub is_new_question: bool,
}

#[derive(Deserialize, Debug)]
pub struct StatStatusPair {
    pub stat: Stat,
    pub status: Option<String>,
    pub difficulty: Difficulty,
    pub paid_only: bool,
    pub is_favor: bool,
    pub frequency: f64,
    pub progress: f64,
}

#[derive(Deserialize, Debug)]
pub struct TopicTagQuestion {
    pub status: Option<String>,
    pub difficulty: Difficulty,
    pub title: String,

    #[serde(rename = "isPaidOnly")]
    pub is_paid_only: bool,

    #[serde(rename = "titleSlug")]
    pub title_slug: String,

    #[serde(rename = "questionFrontendId")]
    pub question_frontend_id: String,
}

#[derive(Deserialize, Debug)]
pub struct ListResponse {
    pub user_name: String,
    pub num_solved: usize,
    pub num_total: usize,
    pub ac_easy: usize,
    pub ac_medium: usize,
    pub ac_hard: usize,
    pub stat_status_pairs: Vec<StatStatusPair>,
    pub frequency_high: usize,
    pub frequency_mid: usize,
    pub category_slug: String,
}

#[derive(Deserialize, Debug)]
pub struct CodeDefinition {
    pub value: String,
    pub text: String,

    #[serde(rename = "defaultCode")]
    pub default_code: String,
}

#[derive(Deserialize, Debug)]
pub struct SubmissionResult {
    pub code_output: Option<Either>,
    pub code_answer: Option<Either>,
    pub expected_code_output: Option<Either>,
    pub expected_code_answer: Option<Either>,
    pub compare_result: Option<String>,
    pub compile_error: Option<String>,
    pub elapsed_time: u32,
    pub full_compile_error: Option<String>,
    pub lang: String,
    pub memory: Option<u32>,
    pub memory_percentile: Option<f32>,
    pub pretty_lang: String,
    pub question_id: Option<u32>,
    pub run_success: bool,
    pub runtime_percentile: Option<f32>,
    pub state: String,
    pub status_code: u32,
    pub expected_status_code: Option<u32>,
    pub status_memory: String,
    pub status_msg: String,
    pub status_runtime: String,
    pub submission_id: String,
    pub task_finish_time: i64,
    pub expected_task_finish_time: Option<i64>,
    pub total_correct: Option<u32>,
    pub total_testcases: Option<u32>,
}

impl SubmissionResult {
    pub fn has_compile_error(&self) -> bool {
        self.compile_error.is_some() || self.full_compile_error.is_some()
    }

    pub fn has_runtime_error(&self) -> bool {
        self.status_msg.to_lowercase().contains("error")
    }

    pub fn has_error(&self) -> bool {
        self.total_correct.lt(&self.total_testcases)
    }
}

impl ProblemInfo for StatStatusPair {
    fn question_id(&self) -> usize {
        self.stat.frontend_question_id
    }

    fn question_title(&self) -> &str {
        self.stat.question_title.as_str()
    }

    fn difficulty(&self) -> &Difficulty {
        &self.difficulty
    }

    fn is_favorite(&self) -> Option<bool> {
        Some(self.is_favor)
    }

    fn is_paid_only(&self) -> bool {
        self.paid_only
    }

    fn status(&self) -> Option<&str> {
        self.status.as_ref().map(String::as_ref)
    }
}

impl ProblemInfo for TopicTagQuestion {
    fn question_id(&self) -> usize {
        self.question_frontend_id
            .parse()
            .expect("Expected question_frontend_id")
    }

    fn question_title(&self) -> &str {
        self.title.as_str()
    }

    fn difficulty(&self) -> &Difficulty {
        &self.difficulty
    }

    fn is_favorite(&self) -> Option<bool> {
        None
    }

    fn is_paid_only(&self) -> bool {
        self.is_paid_only
    }

    fn status(&self) -> Option<&str> {
        self.status.as_ref().map(String::as_ref)
    }
}
