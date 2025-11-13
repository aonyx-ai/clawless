use clawless::prelude::*;

/// Arguments for the `greet` command
///
/// This struct defines the command-line arguments for the `greet` command, which either greets a
/// user by the provided name or defaults to "World" if no name is given.
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Args)]
pub struct GreetArgs {
    /// The name to greet
    #[arg(default_value = "World")]
    name: String,
}

/// Greet the user
///
/// This command prints a greeting message to the console using the provided name. If no name is
/// given, the greeting default to "Hello, World!".
#[command]
pub async fn greet(args: GreetArgs) -> CommandResult {
    println!("Hello, {}!", args.name);
    Ok(())
}
