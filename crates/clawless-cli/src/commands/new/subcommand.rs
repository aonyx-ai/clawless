use clap::Args;
use clawless::{command, Result};

#[derive(Debug, Args)]
pub struct SubcommandArgs {
    #[arg(short, long)]
    name: String,
}

#[command]
pub async fn subcommand(args: SubcommandArgs) -> Result {
    println!("Running a subcommand: {}", args.name);
    Ok(())
}
