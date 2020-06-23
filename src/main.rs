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

fn main() {
    let opt = LeetUpArgs::from_args();
    match opt.command {
        Command::List(list) => {
            println!("{:?}", list);
            leetup::fetch_url("/problems/all").unwrap();
        }
        _ => (),
    }
}
