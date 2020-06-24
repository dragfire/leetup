use crate::{cmd::List, fetch, icon::Icon, LeetUpError, Result};
use ansi_term::Colour::{Green, Red, Yellow};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Ord, PartialOrd, Eq, PartialEq)]
pub struct Difficulty {
    pub level: usize,
}

impl ToString for Difficulty {
    fn to_string(&self) -> String {
        match self.level {
            1 => Green.paint(String::from("Easy")).to_string(),
            2 => Yellow.paint(String::from("Medium")).to_string(),
            3 => Red.paint(String::from("Hard")).to_string(),
            _ => String::from("UnknownLevel"),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Ord, PartialOrd, Eq, PartialEq)]
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

#[derive(Serialize, Deserialize, Debug, Ord, PartialOrd, Eq, PartialEq)]
pub struct StatStatusPair {
    pub stat: Stat,
    pub status: Option<String>,
    pub difficulty: Difficulty,
    pub paid_only: bool,
    pub is_favor: bool,
    pub frequency: isize,
    pub progress: isize,
}

#[derive(Serialize, Deserialize, Debug)]
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

enum Query {
    Easy,
    Medium,
    Hard,
    NotEasy,
    NotMedium,
    NotHard,
    Locked,
    NotLocked,
    Starred,
    UnStarred,
}

/// Fetch all problems
pub fn fetch_all_problems() -> Result<ListResponse> {
    fetch::fetch_url("/problems/all")?
        .json::<ListResponse>()
        .map_err(LeetUpError::Reqwest)
}

fn pretty_list(probs: Vec<&StatStatusPair>) {
    for obj in probs {
        let qstat = &obj.stat;

        let starred_icon = if obj.is_favor {
            Yellow.paint(Icon::Star.to_string()).to_string()
        } else {
            Icon::Empty.to_string()
        };

        let locked_icon = if obj.paid_only {
            Red.paint(Icon::Lock.to_string()).to_string()
        } else {
            Icon::NoLock.to_string()
        };

        let acd = match obj.status {
            Some(_) => Green.paint(Icon::Yes.to_string()).to_string(),
            None => Icon::Empty.to_string(),
        };

        println!(
            "{} {} {} [{:^4}] {:75} {:6}",
            starred_icon,
            locked_icon,
            acd,
            qstat.question_id,
            qstat.question_title,
            obj.difficulty.to_string()
        );
    }
}

pub fn list_problems(list: List) -> crate::Result<()> {
    let mut res = fetch_all_problems()?;
    let probs = &mut res.stat_status_pairs;
    let default_keyword = String::from("");
    let keyword = list
        .keyword
        .as_ref()
        .unwrap_or(&default_keyword)
        .to_ascii_lowercase();
    probs.sort_by(Ord::cmp);

    let filtered_probs: Vec<&StatStatusPair> = probs
        .iter()
        .filter(|o| o.stat.question_title_slug.contains(&keyword))
        .collect();

    pretty_list(filtered_probs);

    Ok(())
}

#[test]
fn test_fetch_url() {
    println!("{:?}", fetch_all_problems().unwrap());
}
