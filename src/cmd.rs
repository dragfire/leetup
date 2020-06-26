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
pub struct LeetUpArgs {
    #[structopt(subcommand)]
    pub command: Command,
}
