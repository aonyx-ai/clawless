use clap::{ArgMatches, Args, FromArgMatches};
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

pub async fn exec_run(args: ArgMatches) {
    let args = NewArgs::from_arg_matches(&args).unwrap();
    run(args).await;
}

inventory::submit!(CommandRegistration {
    name: "new",
    func: |args| Box::pin(exec_run(args)),
});
