//! TUI application runner
//!
//! This module defines [`ApplicationRunner`], which encapsulates the full lifecycle of a TUI
//! application. The lifecycle covers event channel creation, context construction, projection
//! setup, signal handling, application execution, and the drain of the events that the
//! application emitted. The `main!()` macro dispatches to `ApplicationRunner::run` when the
//! resolved leaf is an application.
//!
//! Application authors do not interact with this module directly. The `main!()` macro calls
//! [`ApplicationRunner::run`] with the resolved matches and the leaf's exec function. The runner
//! takes any callable. A caller that builds its command tree at run time can therefore dispatch
//! to a closure that owns the application it resolved.

use std::future::Future;

use anyhow::Context as _;
use clawless_core::cancellation::Cancellation;
use clawless_core::context::Context;
use clawless_core::event::event_channel;
use clawless_core::exit::Exit;
use clawless_core::output::Output;
use clawless_core::signal::wait_for_shutdown;

use crate::projection::Projection;

/// TUI application runner
///
/// Encapsulates the lifecycle of running a TUI application. The runner creates the event
/// channel, constructs the [`Context`], builds a [`Projection`] inside a Tokio runtime,
/// registers signal handlers, and calls the application function. Unlike `CommandRunner`,
/// there is no presenter. The application owns its own render loop and queries the projection
/// for accumulated state.
///
/// # Examples
///
/// ```rust,ignore
/// // This is what main!() expands to for applications:
/// ResolvedLeaf::Application { matches, exec } => {
///     clawless::tui::runner::ApplicationRunner::run(matches, exec)
/// }
/// ```
///
/// [`Context`]: clawless_core::context::Context
/// [`Projection`]: crate::projection::Projection
#[derive(Debug)]
pub struct ApplicationRunner;

impl ApplicationRunner {
    /// Runs a TUI application to completion
    ///
    /// Sets up the application lifecycle:
    ///
    /// 1. Creates a [`Cancellation`] token for cooperative shutdown
    /// 2. Creates an event channel and builds the [`Context`]
    /// 3. Creates a Tokio runtime
    /// 4. Inside the runtime, creates a [`Projection`] (which spawns a background drain task)
    /// 5. Spawns the signal handler and calls the application function
    /// 6. Awaits the drain task once the application returns
    ///
    /// The [`Projection`] must be created inside the Tokio runtime because its constructor calls
    /// `tokio::spawn` to start the background event drain.
    ///
    /// Step 6 keeps shutdown whole. Dropping the runtime stops the drain wherever it happens to
    /// be. A runner that returned the moment the application did would therefore leave the events
    /// of the final moments unread. Awaiting the drain instead means the projection accounts for
    /// every event that the application emitted.
    ///
    /// The drain ends because the application's [`Context`] carries the only [`EventSender`], and
    /// returning drops it. The runner drops the future of the application as soon as it returns,
    /// whatever the future still holds. The channel therefore closes before the runner awaits the
    /// drain. An application that gives a clone of its [`Output`] to a task that it never awaits
    /// holds the channel open. This runner then waits for a drain that cannot end.
    /// `CommandRunner` expects the same of a command.
    ///
    /// The runner returns the [`Exit`] of the application. If the application fails, or if the
    /// runner cannot build the [`Context`] or the Tokio runtime, the [`Exit`] has the exit code 1.
    /// Its text is the text of the error. The `main` function returns the [`Exit`] in a
    /// `ProcessExit`. The process writes its text only after the application has returned and the
    /// drain has ended.
    ///
    /// A drain that panicked does not change the [`Exit`]. A join fails only when the drain task
    /// panicked, and the drain panics only on a poisoned lock, which takes a panic elsewhere to
    /// cause. The result of the application is already in hand, and reporting the join failure
    /// in its place would bury the failure that started it.
    ///
    /// # Arguments
    ///
    /// * `matches`: The parsed [`ArgMatches`] for this application leaf, as resolved by the
    ///   subcommand tree walk.
    /// * `exec`: Executes the application with the given matches, context, and projection. The
    ///   `#[application]` macro generates a function for this, and any other callable works. A
    ///   caller that builds its command tree at run time passes a closure that owns the
    ///   application it resolved. The application returns `()` for the exit code 0, or an
    ///   [`Exit`] to choose the exit code and the text.
    ///
    /// # Examples
    ///
    /// ```
    /// use clawless_core::exit::Exit;
    /// use clawless_tui::runner::ApplicationRunner;
    ///
    /// let matches = clap::Command::new("dashboard").get_matches_from(["dashboard"]);
    ///
    /// let exit = ApplicationRunner::run(matches, |_matches, context, projection| async move {
    ///     context.output().message("ready").await?;
    ///     drop(context);
    ///     projection.wait_until_complete().await;
    ///
    ///     assert_eq!(projection.entries().len(), 1);
    ///     Ok(())
    /// });
    ///
    /// assert_eq!(exit, Exit::from(()));
    /// ```
    ///
    /// [`ArgMatches`]: clap::ArgMatches
    /// [`Cancellation`]: clawless_core::cancellation::Cancellation
    /// [`Context`]: clawless_core::context::Context
    /// [`EventSender`]: clawless_core::event::EventSender
    /// [`Exit`]: clawless_core::exit::Exit
    /// [`Output`]: clawless_core::output::Output
    /// [`Projection`]: crate::projection::Projection
    pub fn run<E, F, T>(matches: clap::ArgMatches, exec: E) -> Exit
    where
        E: FnOnce(clap::ArgMatches, Context, Projection) -> F,
        F: Future<Output = anyhow::Result<T>> + Send + 'static,
        T: Into<Exit>,
    {
        Self::try_run(matches, exec).unwrap_or_else(|error| Exit::from_error(&error))
    }

    /// Runs a TUI application to completion and returns its exit, or the error that ended it
    ///
    /// # Errors
    ///
    /// Returns an error if context construction fails (e.g., the current working directory cannot
    /// be determined). Returns an error if the Tokio runtime cannot be created, or if the
    /// application itself fails.
    fn try_run<E, F, T>(matches: clap::ArgMatches, exec: E) -> anyhow::Result<Exit>
    where
        E: FnOnce(clap::ArgMatches, Context, Projection) -> F,
        F: Future<Output = anyhow::Result<T>> + Send + 'static,
        T: Into<Exit>,
    {
        let cancellation = Cancellation::new();
        let (sender, receiver) = event_channel();
        let output = Output::new(sender);

        let context = Context::builder()
            .cancellation(cancellation.clone())
            .output(output)
            .build()
            .context("build the context of the application")?;

        let rt = tokio::runtime::Runtime::new().context("start the async runtime")?;
        rt.block_on(async {
            let mut projection = Projection::new(receiver);
            let drain = projection.take_drain();

            tokio::spawn(wait_for_shutdown(cancellation));

            let result = exec(matches, context, projection).await;

            if let Some(drain) = drain {
                drop(drain.await);
            }

            result.map(Into::into)
        })
    }
}

#[cfg(test)]
mod tests {
    #![allow(
        clippy::missing_panics_doc,
        reason = "an assertion in a test panics by design, so a `# Panics` section would repeat it"
    )]

    use std::process::ExitCode;
    use std::sync::Arc;
    use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

    use super::*;

    /// Shows that an application reads a complete final frame
    ///
    /// The runner owns the projection and moves it into the application, which leaves a caller
    /// nothing to read afterwards. The count therefore travels out through an atomic. The
    /// application is also the only place worth asserting from. A closing frame is drawn before
    /// the application returns, so that is where completeness has to hold.
    ///
    /// Three hundred events overrun the 256-slot channel. The run therefore also covers a drain
    /// that has to keep pace with a producer the channel is holding back.
    #[test]
    fn run_lets_an_application_read_a_complete_final_frame() {
        let matches = clap::Command::new("test").get_matches_from(["test"]);
        let counted = Arc::new(AtomicUsize::new(0));
        let owned = Arc::clone(&counted);

        drop(ApplicationRunner::run(
            matches,
            move |_matches, context, projection| async move {
                for index in 0..300 {
                    context
                        .output()
                        .message(format!("event {index}"))
                        .await
                        .expect("the projection holds the channel open");
                }
                drop(context);
                projection.wait_until_complete().await;

                owned.store(projection.entries().len(), Ordering::SeqCst);
                Ok(())
            },
        ));

        assert_eq!(counted.load(Ordering::SeqCst), 300);
    }

    #[test]
    fn run_with_a_closure_that_owns_state_executes_the_leaf() {
        let matches = clap::Command::new("test").get_matches_from(["test"]);
        let executed = Arc::new(AtomicBool::new(false));
        let owned = Arc::clone(&executed);

        drop(ApplicationRunner::run(
            matches,
            move |_matches, _context, _projection| async move {
                owned.store(true, Ordering::SeqCst);
                Ok(())
            },
        ));

        assert!(executed.load(Ordering::SeqCst));
    }

    #[test]
    fn run_with_an_application_that_chooses_its_exit_returns_that_exit() {
        let matches = clap::Command::new("test").get_matches_from(["test"]);
        let chosen = Exit::builder()
            .code(ExitCode::from(3))
            .text("error: the session ended early\n")
            .build();
        let returned = chosen.clone();

        let exit =
            ApplicationRunner::run(matches, move |_matches, _context, _projection| async move {
                Ok(returned)
            });

        assert_eq!(exit, chosen);
    }

    #[test]
    fn run_with_an_application_that_fails_returns_the_error_with_code_1() {
        let matches = clap::Command::new("test").get_matches_from(["test"]);

        let exit = ApplicationRunner::run(matches, |_matches, _context, _projection| async {
            Err::<(), _>(anyhow::anyhow!("the terminal is gone"))
        });

        assert_eq!(
            exit,
            Exit::builder()
                .code(ExitCode::FAILURE)
                .text("Error: the terminal is gone\n")
                .build()
        );
    }

    #[test]
    fn run_with_an_application_that_succeeds_returns_success() {
        let matches = clap::Command::new("test").get_matches_from(["test"]);

        let exit =
            ApplicationRunner::run(matches, |_matches, _context, _projection| async { Ok(()) });

        assert_eq!(exit, Exit::from(()));
    }

    #[test]
    fn trait_send() {
        fn assert_send<T: Send>() {}
        assert_send::<ApplicationRunner>();
    }

    #[test]
    fn trait_sync() {
        fn assert_sync<T: Sync>() {}
        assert_sync::<ApplicationRunner>();
    }

    #[test]
    fn trait_unpin() {
        fn assert_unpin<T: Unpin>() {}
        assert_unpin::<ApplicationRunner>();
    }
}
