use clap::Args;
use clawless::command;

mod subcommand;

#[derive(Debug, Args)]
pub struct NewArgs {
    #[arg(short, long)]
    name: Option<String>,
}

#[command]
pub async fn new(args: NewArgs) {
    println!("Creating new CLI with name: {}", args.name.unwrap());
}
