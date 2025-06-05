use std::future::Future;
use std::pin::Pin;

use clap::{ArgMatches, Command};

mod commands;

#[allow(dead_code)]
struct SubcommandRegistration {
    name: &'static str,
    init: fn() -> Command,
    func: fn(ArgMatches) -> Pin<Box<dyn Future<Output = ()>>>,
}

inventory::collect!(SubcommandRegistration);

#[tokio::main]
async fn main() {
    let app = commands::clawless_init();
    commands::clawless_exec(app.get_matches()).await;
}
