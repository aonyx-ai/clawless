use clawless::prelude::*;

#[derive(Debug, Args)]
pub struct GreetArgs {
    name: String,
}

#[command]
pub async fn greet(args: GreetArgs) -> CommandResult {
    println!("Hello, {}!", args.name);
    Ok(())
}

fn main() {}
