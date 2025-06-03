use std::future::Future;
use std::pin::Pin;

use clap::{ArgMatches, Command};

mod commands;

struct CommandRegistration {
    name: &'static str,
    init: fn() -> Command,
    func: fn(ArgMatches) -> Pin<Box<dyn Future<Output = ()>>>,
}

inventory::collect!(CommandRegistration);

#[tokio::main]
async fn main() {
    let mut app = Command::new("clawless")
        .version(env!("CARGO_PKG_VERSION"))
        .about("A framework for building command-line applications in Rust")
        .arg_required_else_help(true);

    for command in inventory::iter::<CommandRegistration> {
        app = app.subcommand((command.init)());
    }

    let matches = app.get_matches();

    for command in inventory::iter::<CommandRegistration> {
        if let Some(matches) = matches.subcommand_matches(command.name) {
            (command.func)(matches.clone()).await;
        }
    }
}
