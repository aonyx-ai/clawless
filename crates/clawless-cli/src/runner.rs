//! CLI command runner
//!
//! This module defines [`CommandRunner`], which encapsulates the full lifecycle of a CLI command:
//! event channel creation, context construction, signal handling, and terminal presentation. The
//! `main!()` macro dispatches to `CommandRunner::run` when the resolved leaf is a command.
//!
//! Application authors do not interact with this module directly. The `main!()` macro calls
//! [`CommandRunner::run`] with the resolved matches and the leaf's exec function. The runner takes
//! any callable, so a caller that builds its command tree at run time can dispatch to a closure
//! that owns the command it resolved.

use std::future::Future;

use clawless_core::cancellation::Cancellation;
use clawless_core::context::Context;
use clawless_core::event::event_channel;
use clawless_core::output::Output;
use clawless_core::signal::wait_for_shutdown;

use crate::error::CommandResult;
use crate::output::OutputFlags;
use crate::presenter::{Presenter, TerminalPresenter};

/// CLI command runner
///
/// Encapsulates the lifecycle of running a CLI command: creating the event channel, constructing
/// the [`Context`], registering signal handlers, and presenting output through a
/// [`TerminalPresenter`]. The `main!()` macro dispatches to [`CommandRunner::run`] when the
/// resolved leaf is a `ResolvedLeaf::Command`.
///
/// # Examples
///
/// ```rust,ignore
/// // This is what main!() expands to for commands:
/// ResolvedLeaf::Command { matches, exec } => {
///     clawless::runner::CommandRunner::run(matches, exec)
/// }
/// ```
// r[impl dispatch.exec.command-runner]
#[derive(Debug)]
pub struct CommandRunner;

impl CommandRunner {
    /// Runs a CLI command to completion
    ///
    /// Sets up the command lifecycle:
    ///
    /// 1. Creates a [`Cancellation`] token for cooperative shutdown
    /// 2. Extracts output flags from the pre-parsed argument matches
    /// 3. Creates an event channel and builds the [`Context`]
    /// 4. Configures a [`TerminalPresenter`] with the parsed output flags
    /// 5. Creates a Tokio runtime, spawns the signal handler, and runs the command through the
    ///    presenter
    ///
    /// Output flags (`--quiet`, `--verbose`, `--json`) are augmented at the root level by
    /// `main!()` with `.global(true)`, so they are available in every leaf's [`ArgMatches`].
    ///
    /// # Arguments
    ///
    /// * `matches` — The parsed [`ArgMatches`] for this command leaf, as resolved by the
    ///   subcommand tree walk.
    /// * `exec` — Executes the command with the given matches and context. The `#[command]` macro
    ///   generates a function for this, and any other callable works. A caller that builds its
    ///   command tree at run time passes a closure that owns the command it resolved.
    ///
    /// # Errors
    ///
    /// Returns an error if context construction fails (e.g., the current working directory cannot be
    /// determined), if the Tokio runtime cannot be created, or if the command itself fails.
    ///
    /// [`ArgMatches`]: clap::ArgMatches
    /// [`Cancellation`]: clawless_core::cancellation::Cancellation
    /// [`Context`]: clawless_core::context::Context
    /// [`TerminalPresenter`]: crate::presenter::TerminalPresenter
    // r[impl dispatch.exec.callable]
    pub fn run<E, F>(matches: clap::ArgMatches, exec: E) -> Result<(), Box<dyn std::error::Error>>
    where
        E: FnOnce(clap::ArgMatches, Context) -> F,
        F: Future<Output = CommandResult> + Send + 'static,
    {
        let cancellation = Cancellation::new();
        let output_flags = OutputFlags::from_arg_matches(&matches);

        let (sender, receiver) = event_channel();
        let output = Output::new(sender);

        let context = Context::builder()
            .cancellation(cancellation.clone())
            .output(output)
            .build()?;

        let presenter = TerminalPresenter::builder()
            .receiver(receiver)
            .verbosity(output_flags.verbosity())
            .mode(output_flags.mode())
            .build();

        let rt = tokio::runtime::Runtime::new()?;
        rt.block_on(async {
            tokio::spawn(wait_for_shutdown(cancellation));

            presenter.present(Box::pin(exec(matches, context))).await
        })?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    // An assertion in a test panics by design. A `# Panics` section on every test
    // would repeat that and give the reader no information.
    #![allow(clippy::missing_panics_doc)]

    use std::sync::Arc;
    use std::sync::atomic::{AtomicBool, Ordering};

    use super::*;

    // r[verify dispatch.exec.callable]
    #[test]
    fn run_with_a_closure_that_owns_state_executes_the_leaf() {
        let matches =
            OutputFlags::augment_command(clap::Command::new("test")).get_matches_from(["test"]);
        let executed = Arc::new(AtomicBool::new(false));
        let owned = Arc::clone(&executed);

        CommandRunner::run(matches, move |_matches, _context| async move {
            owned.store(true, Ordering::SeqCst);
            Ok(())
        })
        .expect("the runner runs the leaf to completion");

        assert!(executed.load(Ordering::SeqCst));
    }

    #[test]
    fn trait_send() {
        fn assert_send<T: Send>() {}
        assert_send::<CommandRunner>();
    }

    #[test]
    fn trait_sync() {
        fn assert_sync<T: Sync>() {}
        assert_sync::<CommandRunner>();
    }

    #[test]
    fn trait_unpin() {
        fn assert_unpin<T: Unpin>() {}
        assert_unpin::<CommandRunner>();
    }
}
