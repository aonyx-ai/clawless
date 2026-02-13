use clawless::prelude::*;

#[derive(Debug, Args)]
pub struct GreetArgs {
    name: String,
}

#[command(unknown_attr)]
pub async fn greet(args: GreetArgs, _context: Context) -> CommandResult {
    println!("Hello, {}!", args.name);
    Ok(())
}

fn main() {}
