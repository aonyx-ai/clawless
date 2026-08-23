use clawless::prelude::*;

/// Arguments for the `wait` command
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Args)]
pub struct WaitArgs {}

/// Wait for cancellation
///
/// This command blocks until a cancellation signal is received, then prints a confirmation message
/// and exits gracefully. It demonstrates how commands observe the cancellation token for
/// cooperative shutdown.
///
/// Run this command and press Ctrl+C to trigger cancellation:
///
/// ```shell
/// cancellation wait
/// ```
#[command]
// A command's doc comment is its `--help` text, so an `# Errors` section would render as a
// raw Markdown heading in the terminal rather than documenting an API.
#[allow(clippy::missing_errors_doc)]
pub async fn wait(_args: WaitArgs, context: Context) -> CommandResult {
    message!("waiting");
    context.cancellation().cancelled().await;
    message!("cancelled");
    Ok(())
}
