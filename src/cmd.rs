use crate::{
    service::{self, leetcode::Leetcode, Lang, Problem, ServiceProvider},
    Result,
};
use log::debug;
use spinners::{Spinner, Spinners};
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
pub struct List {
    pub keyword: Option<String>,

    /// Filter by given tag
    #[structopt(short, long)]
    pub tag: Option<String>,

    /// Query by conditions
    #[structopt(short, long)]
    pub query: Option<String>,

    /// Show statistic counter of the output list
    #[structopt(short, long)]
    pub stat: bool,

    /// Order by ProblemId, Question Title, or Difficulty
    #[structopt(short, long)]
    pub order: Option<String>,
}

#[derive(Debug, StructOpt)]
pub struct User {
    /// Login using cookie
    #[structopt(short, long)]
    pub cookie: Option<Option<String>>,

    /// Login using github
    #[structopt(short, long)]
    pub github: Option<Option<String>>,

    /// Logout user
    #[structopt(short, long)]
    pub logout: Option<Option<String>>,
}

#[derive(Debug, StructOpt)]
pub struct Pick {
    /// Show/Pick a problem using ID.
    pub id: Option<usize>,

    /// Generate code if true.
    #[structopt(short)]
    pub generate: bool,

    /// Include problem definition in generated source file.
    #[structopt(short)]
    pub def: bool,

    /// Language used to generate problem's source.
    #[structopt(short, long, default_value = "rust")]
    pub lang: Lang,
}

#[derive(Debug, StructOpt)]
pub struct Submit {
    /// Code filename.
    pub filename: String,
}

#[derive(Debug, StructOpt)]
pub struct Test {
    /// Code filename.
    pub filename: String,

    /// Custom test cases.
    #[structopt(short)]
    pub test_data: String,
}

#[derive(Debug, StructOpt)]
pub enum Command {
    /// List questions
    #[structopt(name = "list")]
    List(List),

    /// User auth
    #[structopt(name = "user")]
    User(User),

    /// Pick a problem
    #[structopt(name = "pick")]
    Pick(Pick),

    /// Submit a problem
    #[structopt(name = "submit")]
    Submit(Submit),

    /// Test a problem
    #[structopt(name = "test")]
    Test(Test),
}

/// -q to query by conditions.
///    e = easy, E = not easy = m + h.
///    m = medium, M = not medium = e + h.
///    h = hard, H = not hard = e + m.
///    d = done = AC-ed, D = not AC-ed.
///    l = locked, L = not locked.
///    s = starred, S = unstarred.
#[derive(Debug)]
pub enum Query {
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
    pub fn from_str(q: &str) -> Vec<Query> {
        q.chars().map(Query::from).collect()
    }
}

pub enum OrderBy {
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
    pub fn from_str(order: &str) -> Vec<OrderBy> {
        order.chars().map(OrderBy::from).collect()
    }
}

#[derive(StructOpt, Debug)]
#[structopt(name = "leetup")]
pub struct LeetUpArgs {
    #[structopt(subcommand)]
    pub command: Command,
}

pub fn process() -> Result<()> {
    let opt = LeetUpArgs::from_args();
    debug!("Options: {:#?}", opt);
    let mut provider = Leetcode::new();

    match opt.command {
        Command::Pick(pick) => {
            provider.pick_problem(pick)?;
        }
        Command::List(list) => {
            provider.list_problems(list)?;
        }
        Command::User(user) => {
            provider.process_auth(user)?;
        }
        Command::Submit(submit) => {
            let sp = Spinner::new(Spinners::Dots9, "Waiting for judge result!".into());
            provider.problem_submit(submit)?;
            sp.stop();
        }
        Command::Test(test) => {
            let sp = Spinner::new(Spinners::Dots9, "Waiting for judge result!".into());
            provider.problem_test(test)?;
            sp.stop();
        }
    }
    Ok(())
}
