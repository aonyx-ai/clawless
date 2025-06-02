use clap::{Args, Command, FromArgMatches};

use crate::commands::new::NewArgs;

mod commands;

#[tokio::main]
async fn main() {
    let new = NewArgs::augment_args(Command::new("new"));

    let app = Command::new("clawless")
        .version(env!("CARGO_PKG_VERSION"))
        .about("A framework for building command-line applications in Rust")
        .arg_required_else_help(true)
        .subcommand(new);

    let matches = app.get_matches();

    match matches.subcommand() {
        Some(("new", matches)) => {
            let args = NewArgs::from_arg_matches(matches).unwrap();
            commands::new::run(&args).await;
        }
        _ => {}
    }
}
