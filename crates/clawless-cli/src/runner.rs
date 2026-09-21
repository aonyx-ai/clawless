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
use std::io::IsTerminal;

use clawless_core::cancellation::Cancellation;
use clawless_core::context::{Context, Interactivity};
use clawless_core::event::event_channel;
use clawless_core::output::Output;
use clawless_core::signal::wait_for_shutdown;

use crate::error::CommandResult;
use crate::output::OutputFlags;
use crate::presenter::{Presenter, TerminalPresenter};

/// Reports whether a user is present who can answer the application
///
/// A user is present when `input` and `display` are both terminals. The user types on the
/// input and reads on the display. The display is the standard error, because the standard
/// output carries the result of the command.
fn detect_interactivity(input: &impl IsTerminal, display: &impl IsTerminal) -> Interactivity {
    match (input.is_terminal(), display.is_terminal()) {
        (true, true) => Interactivity::Interactive,
        (true, false) => Interactivity::NonInteractive,
        (false, true) => Interactivity::NonInteractive,
        (false, false) => Interactivity::NonInteractive,
    }
}

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
    /// 1. Detects whether a user is present who can answer the application
    /// 2. Creates a [`Cancellation`] token for cooperative shutdown
    /// 3. Extracts output flags from the pre-parsed argument matches
    /// 4. Creates an event channel and builds the [`Context`]
    /// 5. Configures a [`TerminalPresenter`] with the parsed output flags
    /// 6. Creates a Tokio runtime, spawns the signal handler, and runs the command through the
    ///    presenter
    ///
    /// A user is present when the standard input and the standard error are both terminals. The
    /// [`Context`] carries the result as its [`Interactivity`].
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
    /// [`Interactivity`]: clawless_core::context::Interactivity
    /// [`TerminalPresenter`]: crate::presenter::TerminalPresenter
    // r[impl dispatch.exec.callable]
    pub fn run<E, F>(matches: clap::ArgMatches, exec: E) -> Result<(), Box<dyn std::error::Error>>
    where
        E: FnOnce(clap::ArgMatches, Context) -> F,
        F: Future<Output = CommandResult> + Send + 'static,
    {
        let interactivity = detect_interactivity(&std::io::stdin(), &std::io::stderr());

        Self::run_with(matches, exec, interactivity)
    }

    /// Runs a CLI command to completion for the given interactivity
    ///
    /// [`CommandRunner::run`] detects the interactivity and calls this function. A test calls
    /// this function directly, because the streams of a test are never terminals.
    ///
    /// # Errors
    ///
    /// Returns an error if context construction fails (e.g., the current working directory cannot be
    /// determined), if the Tokio runtime cannot be created, or if the command itself fails.
    fn run_with<E, F>(
        matches: clap::ArgMatches,
        exec: E,
        interactivity: Interactivity,
    ) -> Result<(), Box<dyn std::error::Error>>
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
            .interactivity(interactivity)
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

    use std::fs::File;
    use std::path::Path;
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::{Arc, Mutex};

    use super::*;

    #[test]
    fn detect_interactivity_without_a_terminal_returns_non_interactive() {
        let manifest = Path::new(env!("CARGO_MANIFEST_DIR")).join("Cargo.toml");
        let input = File::open(&manifest).expect("should open the manifest");
        let display = File::open(&manifest).expect("should open the manifest");

        let interactivity = detect_interactivity(&input, &display);

        assert_eq!(interactivity, Interactivity::NonInteractive);
    }

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
    fn run_with_interactivity_passes_it_to_the_context() {
        let matches =
            OutputFlags::augment_command(clap::Command::new("test")).get_matches_from(["test"]);
        let seen = Arc::new(Mutex::new(None));
        let owned = Arc::clone(&seen);

        CommandRunner::run_with(
            matches,
            move |_matches, context| async move {
                *owned.lock().expect("should lock") = Some(context.interactivity());
                Ok(())
            },
            Interactivity::Interactive,
        )
        .expect("the runner runs the leaf to completion");

        assert_eq!(
            *seen.lock().expect("should lock"),
            Some(Interactivity::Interactive)
        );
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
