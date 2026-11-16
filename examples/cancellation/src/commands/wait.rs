//! Waits until the process is cancelled

use clawless::prelude::*;

/// Arguments for the `wait` command
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Args)]
pub struct WaitArgs {}

/// Waits for cancellation
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
#[allow(
    clippy::missing_errors_doc,
    reason = "a command's doc comment is its `--help` text, so an `# Errors` section would render as a raw Markdown heading in the terminal"
)]
pub async fn wait(_args: WaitArgs, context: Context) -> CommandResult {
    message!("waiting");
    context.cancellation().cancelled().await;
    message!("cancelled");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_auto_traits<T: Send + Sync + Unpin>() {}

    #[test]
    fn wait_args_is_send_sync_and_unpin() {
        assert_auto_traits::<WaitArgs>();
    }
}
