use clawless::prelude::*;

#[derive(Debug, Args)]
pub struct SubcommandArgs {
    #[arg(short, long)]
    name: String,
}

#[command]
pub async fn subcommand(args: SubcommandArgs, _context: Context) -> CommandResult {
    println!("Running a subcommand: {}", args.name);
    Ok(())
}
