use std::future::Future;
use std::pin::Pin;

use clap::{ArgMatches, Command};

mod commands;

struct SubcommandRegistration {
    name: &'static str,
    init: fn() -> Command,
    func: fn(ArgMatches) -> Pin<Box<dyn Future<Output = ()>>>,
}

inventory::collect!(SubcommandRegistration);

#[tokio::main]
async fn main() {
    let mut app = Command::new("clawless")
        .version(env!("CARGO_PKG_VERSION"))
        .about("A framework for building command-line applications in Rust")
        .arg_required_else_help(true);

    for command in inventory::iter::<SubcommandRegistration> {
        app = app.subcommand((command.init)());
    }

    let args = app.get_matches();

    for command in inventory::iter::<SubcommandRegistration> {
        if let Some(matches) = args.subcommand_matches(command.name) {
            (command.func)(matches.clone()).await;
        }
    }
}
