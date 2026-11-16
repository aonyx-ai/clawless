//! Root subcommand for the `cargo clawless` tool
//!
//! This module provides the top-level `clawless` subcommand that `cargo` dispatches to when
//! invoking `cargo clawless`. It serves as the entry point for all scaffolding commands.

use clawless::prelude::*;

mod generate;
mod new;

/// Arguments for the `cargo clawless` command
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Default, Args)]
pub struct ClawlessArgs {}

/// Dispatches to `cargo clawless` subcommands
///
/// This command group dispatches to subcommands like `new` and `generate`. When invoked as
/// `cargo clawless`, Cargo passes `clawless` as the first argument, which matches this
/// subcommand.
#[command(require_subcommand)]
#[allow(
    clippy::missing_errors_doc,
    reason = "a command's doc comment is its `--help` text, so an `# Errors` section would render as a raw Markdown heading in the terminal rather than documenting an API"
)]
pub async fn clawless(_args: ClawlessArgs, _context: Context) -> CommandResult {
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn trait_send() {
        fn assert_send<T: Send>() {}
        assert_send::<ClawlessArgs>();
    }

    #[test]
    fn trait_sync() {
        fn assert_sync<T: Sync>() {}
        assert_sync::<ClawlessArgs>();
    }

    #[test]
    fn trait_unpin() {
        fn assert_unpin<T: Unpin>() {}
        assert_unpin::<ClawlessArgs>();
    }
}
