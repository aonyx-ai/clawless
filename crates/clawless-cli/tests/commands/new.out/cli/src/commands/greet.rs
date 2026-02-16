use clawless::prelude::*;

#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Args)]
pub struct GreetArgs {
    /// Name of the person to greet
    #[arg(default_value = "World")]
    name: String,
}

#[command]
pub async fn greet(args: GreetArgs, context: Context) -> CommandResult {
    // Print the greeting to the console
    context.output().print(format!("Hello, {}!", args.name));

    // Exit the CLI successfully
    Ok(())
}
