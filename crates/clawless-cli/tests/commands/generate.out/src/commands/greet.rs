use clawless::prelude::*;
mod shout;

#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Args)]
pub struct GreetArgs {
    #[arg(default_value = "World")]
    name: String,
}

#[command]
pub async fn greet(args: GreetArgs, context: Context) -> CommandResult {
    context.output().print(format!("Hello, {}!", args.name));
    Ok(())
}
