use clap::Args;
use clawless::command;

mod subcommand;

#[derive(Debug, Args)]
pub struct NewArgs {}

#[command(noop = true)]
pub async fn new(_args: NewArgs) {}
