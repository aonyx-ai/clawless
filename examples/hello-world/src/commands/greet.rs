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
// A command's doc comment is its `--help` text, so an `# Errors` section would render as a
// raw Markdown heading in the terminal rather than documenting an API.
#[allow(clippy::missing_errors_doc)]
pub async fn greet(args: GreetArgs, context: Context) -> CommandResult {
    message!("Hello, {}!", args.name);
    Ok(())
}
