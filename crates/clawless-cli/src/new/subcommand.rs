use clawless::prelude::*;

#[derive(Debug, Args)]
pub struct SubcommandArgs {
    #[arg(short, long)]
    name: String,
}

#[command]
pub async fn subcommand(args: SubcommandArgs) -> CommandResult {
    println!("Running a subcommand: {}", args.name);
    Ok(())
}
