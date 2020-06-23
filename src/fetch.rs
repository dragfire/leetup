use crate::Result;
use serde::{Deserialize, Serialize};

const API_URI: &str = "https://leetcode.com/api";

#[derive(Serialize, Deserialize, Debug)]
struct Difficulty {
    level: usize,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Stat {
    question_id: usize,

    #[serde(rename = "question__article__live")]
    question_article_live: Option<bool>,

    #[serde(rename = "question__article__slug")]
    question_article_slug: Option<String>,

    #[serde(rename = "question__title")]
    question_title: String,

    #[serde(rename = "question__title_slug")]
    question_title_slug: String,

    #[serde(rename = "question__hide")]
    question_hide: bool,

    total_acs: usize,
    total_submitted: usize,
    frontend_question_id: usize,
    is_new_question: bool,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct StatStatusPair {
    stat: Stat,
    status: Option<String>,
    difficulty: Difficulty,
    paid_only: bool,
    is_favor: bool,
    frequency: isize,
    progress: isize,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ListResponse {
    user_name: String,
    num_solved: usize,
    num_total: usize,
    ac_easy: usize,
    ac_medium: usize,
    ac_hard: usize,
    stat_status_pairs: Vec<StatStatusPair>,
    frequency_high: usize,
    frequency_mid: usize,
    category_slug: String,
}

#[derive(Debug)]
pub enum Response {
    List(ListResponse),
    User(()),
}

/// Fetch URL
pub fn fetch_url(path: &str) -> Result<Response> {
    let url = API_URI.to_string() + path;
    let res = reqwest::blocking::get(&url)?.json::<ListResponse>()?;
    Ok(Response::List(res))
}

#[test]
fn test_fetch_url() {
    println!("{:?}", fetch_url("/problems/all").unwrap());
}
