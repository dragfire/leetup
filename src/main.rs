use leetup::{
    cmd::{Command, LeetUpArgs},
    service,
};
use structopt::StructOpt;

fn main() {
    let opt = LeetUpArgs::from_args();
    match opt.command {
        Command::List(list) => {
            service::list::list_problems(list).unwrap();
        }
        _ => (),
    }
}
