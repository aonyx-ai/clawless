use clap::{ArgMatches, Args, Command, FromArgMatches};
use getset::Getters;

use crate::CommandRegistration;

#[derive(Debug, Args, Getters)]
pub struct NewArgs {
    #[arg(index = 1)]
    #[getset(get = "pub")]
    name: String,
}

pub async fn run(args: NewArgs) {
    println!("Creating new CLI with name: {}", args.name());
}

pub fn init() -> Command {
    NewArgs::augment_args(Command::new("new"))
}

pub async fn exec_run(args: ArgMatches) {
    let args = NewArgs::from_arg_matches(&args).unwrap();
    run(args).await;
}

inventory::submit!(CommandRegistration {
    name: "new",
    init,
    func: |args| Box::pin(exec_run(args)),
});
