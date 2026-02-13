use clawless::prelude::*;

#[derive(Debug, Args)]
pub struct GreetArgs {
    name: String,
}

#[command]
pub async fn greet(args: GreetArgs, context: Context, cancellation: Cancellation, extra: String) -> CommandResult {
    println!("Hello, {}!", args.name);
    Ok(())
}

fn main() {}
