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

/// Entry point for `cargo clawless`
///
/// This command group dispatches to subcommands like `new` and `generate`. When invoked as
/// `cargo clawless`, Cargo passes `clawless` as the first argument, which matches this
/// subcommand.
#[command(require_subcommand)]
pub async fn clawless(_args: ClawlessArgs, _context: Context) -> CommandResult {
    Ok(())
}
