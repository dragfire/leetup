use crate::{cmd::List, fetch, icon::Icon, LeetUpError, Result};
use ansi_term::Colour::{Green, Red, Yellow};
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;

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

/// -q to query by conditions.
///    e = easy, E = not easy = m + h.
///    m = medium, M = not medium = e + h.
///    h = hard, H = not hard = e + m.
///    d = done = AC-ed, D = not AC-ed.
///    l = locked, L = not locked.
///    s = starred, S = unstarred.
///    mdLs
#[derive(Debug)]
enum Query {
    Easy = 1,
    Medium,
    Hard,
    NotEasy,
    NotMedium,
    NotHard,
    Locked,
    Unlocked,
    Done,
    NotDone,
    Starred,
    Unstarred,
}

impl From<char> for Query {
    fn from(c: char) -> Self {
        match c {
            'e' => Query::Easy,
            'E' => Query::NotEasy,
            'm' => Query::Medium,
            'M' => Query::NotMedium,
            'h' => Query::Hard,
            'H' => Query::NotHard,
            'l' => Query::Locked,
            'L' => Query::Unlocked,
            'd' => Query::Done,
            'D' => Query::NotDone,
            's' => Query::Starred,
            'S' => Query::Unstarred,
            _ => Query::Easy,
        }
    }
}

impl Query {
    fn from_str(q: &str) -> Vec<Query> {
        q.chars().map(Query::from).collect()
    }
}

enum OrderBy {
    /// Order by question Id in Ascending order
    IdAsc,

    /// Order by question Id in Descending order
    IdDesc,
    TitleAsc,
    TitleDesc,
    DifficultyAsc,
    DifficultyDesc,
}

impl From<char> for OrderBy {
    fn from(c: char) -> Self {
        match c {
            'i' => OrderBy::IdAsc,
            'I' => OrderBy::IdDesc,
            't' => OrderBy::TitleAsc,
            'T' => OrderBy::TitleDesc,
            'd' => OrderBy::DifficultyAsc,
            'D' => OrderBy::DifficultyDesc,
            _ => OrderBy::IdAsc,
        }
    }
}

impl OrderBy {
    fn from_str(order: &str) -> Vec<OrderBy> {
        order.chars().map(OrderBy::from).collect()
    }
}

/// Fetch all problems
pub fn fetch_all_problems() -> Result<ListResponse> {
    fetch::fetch_url("/problems/all")?
        .json::<ListResponse>()
        .map_err(LeetUpError::Reqwest)
}

fn pretty_list<'a, T: Iterator<Item = &'a StatStatusPair>>(probs: T) {
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

fn apply_queries(queries: &Vec<Query>, o: &StatStatusPair) -> bool {
    let mut is_satisfied = true;

    for q in queries {
        match q {
            Query::Easy => is_satisfied &= o.difficulty.level == 1,
            Query::NotEasy => is_satisfied &= o.difficulty.level != 1,
            Query::Medium => is_satisfied &= o.difficulty.level == 2,
            Query::NotMedium => is_satisfied &= o.difficulty.level != 2,
            Query::Hard => is_satisfied &= o.difficulty.level == 3,
            Query::NotHard => is_satisfied &= o.difficulty.level != 3,
            Query::Locked => is_satisfied &= o.paid_only,
            Query::Unlocked => is_satisfied &= !o.paid_only,
            Query::Done => is_satisfied &= o.status.is_some(),
            Query::NotDone => is_satisfied &= o.status.is_none(),
            Query::Starred => is_satisfied &= o.is_favor,
            Query::Unstarred => is_satisfied &= !o.is_favor,
        }
    }

    is_satisfied
}

pub fn list_problems(list: List) -> crate::Result<()> {
    let mut res = fetch_all_problems()?;
    let probs = &mut res.stat_status_pairs;

    if list.order.is_some() {
        let orders = OrderBy::from_str(list.order.as_ref().unwrap());

        probs.sort_by(|a, b| {
            let mut ordering = Ordering::Equal;
            let id_ordering = a.stat.question_id.cmp(&b.stat.question_id);
            let title_ordering = a.stat.question_title_slug.cmp(&b.stat.question_title_slug);
            let diff_ordering = a.difficulty.level.cmp(&b.difficulty.level);

            for order in &orders {
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
        });
    } else {
        probs.sort_by(Ord::cmp);
    }

    if list.query.is_some() {
        let default_keyword = String::from("");
        let keyword = list
            .keyword
            .as_ref()
            .unwrap_or(&default_keyword)
            .to_ascii_lowercase();
        let queries: Vec<Query> = Query::from_str(list.query.as_ref().unwrap());
        let filter_predicate = |o: &&StatStatusPair| {
            o.stat.question_title_slug.contains(&keyword) && apply_queries(&queries, o)
        };

        let filtered_probs: Vec<&StatStatusPair> = probs.iter().filter(filter_predicate).collect();

        pretty_list(filtered_probs.into_iter());
    } else {
        pretty_list(probs.iter());
    }

    Ok(())
}
