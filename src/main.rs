use ansi_term::Colour::{Green, Red, Yellow};
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
}

#[derive(Debug, StructOpt)]
pub struct User {
    pub bar: Option<String>,
}

#[derive(Debug, StructOpt)]
pub enum Command {
    /// List questions
    #[structopt(name = "list")]
    List(List),

    #[structopt(name = "user")]
    User(User),
}

#[derive(StructOpt, Debug)]
#[structopt(name = "leetup")]
struct LeetUpArgs {
    #[structopt(subcommand)]
    pub command: Command,
}

enum Icon {
    Yes,
    _No,
    Star,
    _Unstar,
    Lock,
    NoLock,
    Empty,
}

impl ToString for Icon {
    fn to_string(&self) -> String {
        match self {
            Icon::Yes => "✔".to_string(),
            Icon::_No => "✘".to_string(),
            Icon::Star => "★".to_string(),
            Icon::_Unstar => "☆".to_string(),
            Icon::Lock => "🔒".to_string(),
            Icon::NoLock => "  ".to_string(),
            Icon::Empty => "   ".to_string(),
        }
    }
}

fn list_problems(_list: List) -> leetup::Result<()> {
    let mut res = leetup::fetch_all_problems()?;
    let probs = &mut res.stat_status_pairs;
    probs.sort_by(Ord::cmp);
    for obj in &probs[..50] {
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
            "{} {} {} [{:^4}] {:60} {:6}",
            starred_icon,
            locked_icon,
            acd,
            qstat.question_id,
            qstat.question_title,
            obj.difficulty.to_string()
        );
    }
    Ok(())
}

fn main() {
    let opt = LeetUpArgs::from_args();
    match opt.command {
        Command::List(list) => {
            list_problems(list).unwrap();
        }
        _ => (),
    }
}
