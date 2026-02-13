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
pub async fn wait(_args: WaitArgs, context: Context) -> CommandResult {
    println!("waiting");
    context.cancellation().cancelled().await;
    println!("cancelled");
    Ok(())
}
