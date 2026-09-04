//! TUI application runner
//!
//! This module defines [`ApplicationRunner`], which encapsulates the full lifecycle of a TUI
//! application: event channel creation, context construction, projection setup, signal handling,
//! application execution, and the drain of the events that the application emitted. The `main!()`
//! macro dispatches to `ApplicationRunner::run` when the resolved leaf is an application.
//!
//! Application authors do not interact with this module directly. The `main!()` macro calls
//! [`ApplicationRunner::run`] with the resolved matches and the leaf's exec function. The runner
//! takes any callable, so a caller that builds its command tree at run time can dispatch to a
//! closure that owns the application it resolved.

use std::future::Future;

use clawless_core::cancellation::Cancellation;
use clawless_core::context::Context;
use clawless_core::event::event_channel;
use clawless_core::output::Output;
use clawless_core::signal::wait_for_shutdown;

use crate::projection::Projection;

/// TUI application runner
///
/// Encapsulates the lifecycle of running a TUI application: creating the event channel,
/// constructing the [`Context`], building a [`Projection`] inside a Tokio runtime, registering
/// signal handlers, and calling the application function. Unlike `CommandRunner`, there is no
/// presenter — the application owns its own render loop and queries the projection for accumulated
/// state.
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
// r[impl dispatch.exec.application-runner]
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
    /// be, so a runner that returned the moment the application did would leave the events of the
    /// final moments unread. Awaiting the drain instead means the projection accounts for every
    /// event that the application emitted.
    ///
    /// The drain ends because the application's [`Context`] carries the only [`EventSender`], and
    /// returning drops it. An application that gives a clone of its [`Output`] to a task that it
    /// never awaits holds the channel open, and this runner then waits for a drain that cannot
    /// end. `CommandRunner` expects the same of a command.
    ///
    /// # Arguments
    ///
    /// * `matches` — The parsed [`ArgMatches`] for this application leaf, as resolved by the
    ///   subcommand tree walk.
    /// * `exec` — Executes the application with the given matches, context, and projection. The
    ///   `#[application]` macro generates a function for this, and any other callable works. A
    ///   caller that builds its command tree at run time passes a closure that owns the
    ///   application it resolved.
    ///
    /// # Errors
    ///
    /// Returns an error if context construction fails (e.g., the current working directory cannot be
    /// determined), if the Tokio runtime cannot be created, or if the application itself fails.
    ///
    /// [`ArgMatches`]: clap::ArgMatches
    /// [`Cancellation`]: clawless_core::cancellation::Cancellation
    /// [`Context`]: clawless_core::context::Context
    /// [`EventSender`]: clawless_core::event::EventSender
    /// [`Output`]: clawless_core::output::Output
    /// [`Projection`]: crate::projection::Projection
    // r[impl dispatch.exec.callable]
    // r[impl dispatch.exec.application-drain]
    pub fn run<E, F>(matches: clap::ArgMatches, exec: E) -> Result<(), Box<dyn std::error::Error>>
    where
        E: FnOnce(clap::ArgMatches, Context, Projection) -> F,
        F: Future<Output = anyhow::Result<()>> + Send + 'static,
    {
        let cancellation = Cancellation::new();
        let (sender, receiver) = event_channel();
        let output = Output::new(sender);

        let context = Context::builder()
            .cancellation(cancellation.clone())
            .output(output)
            .build()?;

        let rt = tokio::runtime::Runtime::new()?;
        rt.block_on(async {
            let mut projection = Projection::new(receiver);
            let drain = projection.take_drain();

            tokio::spawn(wait_for_shutdown(cancellation));

            // Binding the result ends the statement that owns the application's future, so the
            // future is dropped here whatever it still holds, and with it the `Context` that
            // carries the only `EventSender`. That closes the channel, which is what lets the
            // drain below finish rather than wait for an event that can no longer arrive.
            let result = exec(matches, context, projection).await;

            if let Some(drain) = drain {
                // A join fails when the task panicked, and the drain panics only on a poisoned
                // lock, which takes a panic elsewhere to cause. The application's own result is
                // already in hand, and reporting a consequence in its place would bury the
                // failure that started it.
                drop(drain.await);
            }

            result
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
    use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

    use super::*;

    // The count travels out through an atomic because the runner owns the projection and moves it
    // into the application, which leaves a caller nothing to read afterwards. The application is
    // also the only place worth asserting from: a closing frame is drawn before the application
    // returns, so that is where completeness has to hold.
    //
    // Three hundred events overrun the 256-slot channel, so the run also covers a drain that has
    // to keep pace with a producer the channel is holding back.
    #[test]
    fn run_lets_an_application_read_a_complete_final_frame() {
        let matches = clap::Command::new("test").get_matches_from(["test"]);
        let counted = Arc::new(AtomicUsize::new(0));
        let owned = Arc::clone(&counted);

        ApplicationRunner::run(matches, move |_matches, context, projection| async move {
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
        })
        .expect("the runner runs the leaf to completion");

        assert_eq!(counted.load(Ordering::SeqCst), 300);
    }

    // r[verify dispatch.exec.callable]
    #[test]
    fn run_with_a_closure_that_owns_state_executes_the_leaf() {
        let matches = clap::Command::new("test").get_matches_from(["test"]);
        let executed = Arc::new(AtomicBool::new(false));
        let owned = Arc::clone(&executed);

        ApplicationRunner::run(matches, move |_matches, _context, _projection| async move {
            owned.store(true, Ordering::SeqCst);
            Ok(())
        })
        .expect("the runner runs the leaf to completion");

        assert!(executed.load(Ordering::SeqCst));
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
