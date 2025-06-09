use clap::Args;
use clawless::command;

#[derive(Debug, Args)]
pub struct SubcommandArgs {
    #[arg(short, long)]
    name: String,
}

#[command]
pub async fn subcommand(args: SubcommandArgs) {
    println!("Running a subcommand: {}", args.name);
}
