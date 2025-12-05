use clawless::prelude::*;

#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Args)]
pub struct GreetArgs {
    #[arg(default_value = "World")]
    name: String,
}

#[command]
pub async fn greet(args: GreetArgs) -> CommandResult {
    println!("Hello, {}!", args.name);
    Ok(())
}
