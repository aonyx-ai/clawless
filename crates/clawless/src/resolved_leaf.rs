//! Resolved subcommand leaf
//!
//! After parsing CLI arguments, the subcommand tree is walked to find the leaf node.
//! [`ResolvedLeaf`] carries typed information about whether the leaf is a stateless CLI command or a
//! stateful TUI application, along with the parsed argument matches and a function pointer to
//! execute it.
//!
//! This type is used by the `main!()` macro expansion to dispatch to the appropriate runner.
//! Application authors do not interact with it directly.

use std::future::Future;
use std::pin::Pin;

use clawless_cli::error::CommandResult;
use clawless_core::context::Context;
use clawless_tui::projection::Projection;

/// Function pointer type for executing a resolved command
///
/// Accepts parsed [`ArgMatches`] and a [`Context`], returning a pinned future that produces a
/// [`CommandResult`].
///
/// [`ArgMatches`]: clap::ArgMatches
/// [`Context`]: clawless_core::context::Context
pub type CommandExec =
    fn(clap::ArgMatches, Context) -> Pin<Box<dyn Future<Output = CommandResult> + Send>>;

/// Function pointer type for executing a resolved application
///
/// Accepts parsed [`ArgMatches`], a [`Context`], and a [`Projection`], returning a pinned future
/// that produces a [`CommandResult`].
///
/// [`ArgMatches`]: clap::ArgMatches
/// [`Context`]: clawless_core::context::Context
/// [`Projection`]: clawless_tui::projection::Projection
pub type ApplicationExec = fn(
    clap::ArgMatches,
    Context,
    Projection,
) -> Pin<Box<dyn Future<Output = CommandResult> + Send>>;

/// Resolved subcommand leaf
///
/// The result of walking the subcommand tree after argument parsing. Each variant carries the
/// parsed [`ArgMatches`] for the leaf and a function pointer that executes the leaf with the
/// appropriate arguments.
///
/// `main!()` matches on this enum to choose between [`CommandRunner`] (for commands) and
/// [`ApplicationRunner`] (for applications). This separation allows each leaf type to have its own
/// lifecycle without requiring a uniform function signature in the inventory.
///
/// [`ArgMatches`]: clap::ArgMatches
/// [`ApplicationRunner`]: clawless_tui::runner::ApplicationRunner
/// [`CommandRunner`]: clawless_cli::runner::CommandRunner
// r[impl dispatch.safety.send]
// r[impl dispatch.safety.sync]
// r[impl dispatch.safety.unpin]
pub enum ResolvedLeaf {
    // r[impl dispatch.leaf.command]
    // r[impl dispatch.leaf.matches]
    // r[impl dispatch.leaf.exec]
    /// A stateless CLI command rendered through a push-based presenter
    ///
    /// The command receives parsed arguments and a [`Context`] for emitting events. A
    /// [`CommandRunner`] creates the event channel, presenter, and Tokio runtime.
    ///
    /// [`CommandRunner`]: clawless_cli::runner::CommandRunner
    /// [`Context`]: clawless_core::context::Context
    Command {
        /// Parsed argument matches for this command
        matches: clap::ArgMatches,
        /// Executes the command with the given matches and context
        exec: CommandExec,
    },
    // r[impl dispatch.leaf.application]
    /// A stateful TUI application that queries a pull-based projection
    ///
    /// The application receives parsed arguments, a [`Context`] for emitting events, and a
    /// [`Projection`] for querying accumulated state. An [`ApplicationRunner`] creates the event
    /// channel, projection, and Tokio runtime.
    ///
    /// [`ApplicationRunner`]: clawless_tui::runner::ApplicationRunner
    /// [`Context`]: clawless_core::context::Context
    /// [`Projection`]: clawless_tui::projection::Projection
    Application {
        /// Parsed argument matches for this application
        matches: clap::ArgMatches,
        /// Executes the application with the given matches, context, and projection
        exec: ApplicationExec,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    // r[verify dispatch.safety.send]
    #[test]
    fn trait_send() {
        fn assert_send<T: Send>() {}
        assert_send::<ResolvedLeaf>();
    }

    // r[verify dispatch.safety.sync]
    #[test]
    fn trait_sync() {
        fn assert_sync<T: Sync>() {}
        assert_sync::<ResolvedLeaf>();
    }

    // r[verify dispatch.safety.unpin]
    #[test]
    fn trait_unpin() {
        fn assert_unpin<T: Unpin>() {}
        assert_unpin::<ResolvedLeaf>();
    }
}
