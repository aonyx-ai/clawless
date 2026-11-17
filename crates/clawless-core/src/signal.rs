//! Signal-to-cancellation adapter
//!
//! Infrastructure that maps OS signals (SIGINT, SIGTERM) to a [`Cancellation`] token. The runners
//! of `clawless-cli` and `clawless-tui` consume this module, and it is not part of the public API.
//!
//! The double Ctrl+C pattern follows the standard CLI convention. The first signal triggers
//! graceful cancellation, and the second ends the application with code 130 (128 + SIGINT).
//!
//! The second signal does not exit the process from where it arrives. The runner drops the work
//! of the application instead and returns the exit to `main`. Destructors therefore run, so a
//! program that the application started is killed, and a terminal that it lent is taken back.
//!
//! On Unix, OS signal handlers are registered eagerly (at call time, not when the returned future
//! is first polled). This guarantees that signals sent after the handlers are registered are
//! always captured, even if the Tokio runtime has not yet polled the future.
//!
//! [`Cancellation`]: crate::cancellation::Cancellation

use std::future::Future;
use std::process::ExitCode;
use std::time::Duration;

use tokio::runtime::Runtime;

use crate::cancellation::Cancellation;
use crate::exit::Exit;

/// The exit code of an application that the user interrupted twice, which is 128 + SIGINT
const INTERRUPTED: u8 = 130;

/// The time that the tasks of an interrupted application get to end
///
/// The runtime drops every task of the application when it shuts down, which runs their
/// destructors. A task that blocks a thread cannot be dropped, and the runtime would wait for it
/// without end. The user pressed Ctrl+C twice to end the application. The runtime therefore waits
/// for such a task this long, and then the process exits around it.
const FORCED_SHUTDOWN_TIMEOUT: Duration = Duration::from_secs(1);

/// Runs the work of an application on the runtime until it completes or the user interrupts it
///
/// On the first SIGINT (or SIGTERM on Unix), the `cancellation` token is cancelled, giving
/// in-flight work a chance to complete gracefully. The work then keeps running until it
/// completes.
///
/// On the second SIGINT, the function drops the work and shuts the runtime down, which drops
/// every task that the work spawned. It then returns the [`Exit`] with the code 130, which the
/// runner returns from `main`. The destructors of the work therefore run, which a call to
/// [`std::process::exit`] would skip.
///
/// # Errors
///
/// Returns the [`Exit`] of an interrupted application when a second SIGINT ended the work.
///
/// # Panics
///
/// Panics if the OS signal handler cannot be registered, which indicates system resource
/// exhaustion.
pub fn run_until_interrupted<F: Future>(
    runtime: Runtime,
    cancellation: Cancellation,
    work: F,
) -> Result<F::Output, Exit> {
    let completed = runtime.block_on(async {
        let interrupted = wait_for_interrupts(cancellation);

        tokio::select! {
            biased;

            output = work => Some(output),
            () = interrupted => None,
        }
    });

    let Some(output) = completed else {
        runtime.shutdown_timeout(FORCED_SHUTDOWN_TIMEOUT);

        return Err(Exit::from(ExitCode::from(INTERRUPTED)));
    };

    Ok(output)
}

/// Returns a future that cancels on the first signal and completes on the second SIGINT
///
/// On Unix, signal handlers are registered synchronously when this function is called, not when
/// the returned future is first polled. The second SIGINT arrives on the same handler as the
/// first, so no signal between the two is lost.
///
/// # Panics
///
/// Panics if the operating system cannot register the signal handler. This occurs when the
/// system has no resources left.
// Registering an OS signal handler fails only when the process has exhausted its resources.
// The process cannot honor a shutdown request it is unable to observe, so there is nothing to
// recover to. The `# Panics` section documents this for callers.
#[cfg(unix)]
#[allow(clippy::expect_used)]
fn wait_for_interrupts(cancellation: Cancellation) -> impl Future<Output = ()> + Send {
    use tokio::signal::unix::SignalKind;

    let mut sigint =
        tokio::signal::unix::signal(SignalKind::interrupt()).expect("failed to listen for SIGINT");
    let mut sigterm =
        tokio::signal::unix::signal(SignalKind::terminate()).expect("failed to listen for SIGTERM");

    async move {
        tokio::select! {
            _ = sigint.recv() => {}
            _ = sigterm.recv() => {}
        }
        cancellation.cancel();

        sigint.recv().await;
    }
}

/// Returns a future that cancels on the first Ctrl+C and completes on the second
///
/// Platforms other than Unix have no equivalent of SIGTERM. This variant therefore waits for
/// Ctrl+C alone. It registers the handler when the caller first polls the future.
///
/// # Panics
///
/// Panics if the operating system cannot register the signal handler. This occurs when the
/// system has no resources left.
// Registering an OS signal handler fails only when the process has exhausted its resources.
// The process cannot honor a shutdown request it is unable to observe, so there is nothing to
// recover to. The `# Panics` section documents this for callers.
#[cfg(not(unix))]
#[allow(clippy::expect_used)]
fn wait_for_interrupts(cancellation: Cancellation) -> impl Future<Output = ()> + Send {
    async move {
        tokio::signal::ctrl_c()
            .await
            .expect("failed to listen for SIGINT");
        cancellation.cancel();

        tokio::signal::ctrl_c()
            .await
            .expect("failed to listen for SIGINT");
    }
}

#[cfg(test)]
mod tests {
    #![allow(
        clippy::missing_panics_doc,
        reason = "an assertion in a test panics by design. A `# Panics` section on every test would repeat that and give the reader no information"
    )]

    use std::future::pending;
    #[cfg(unix)]
    use std::process::Command;
    use std::sync::Arc;
    use std::sync::atomic::{AtomicBool, Ordering};

    use super::*;

    /// Records in its flag that it was dropped
    struct DropFlag(Arc<AtomicBool>);

    impl Drop for DropFlag {
        fn drop(&mut self) {
            self.0.store(true, Ordering::SeqCst);
        }
    }

    /// Sends a SIGINT to this process
    #[cfg(unix)]
    fn interrupt() {
        let status = Command::new("kill")
            .args(["-s", "INT", &std::process::id().to_string()])
            .status()
            .expect("should run kill");

        assert!(status.success());
    }

    #[test]
    fn run_until_interrupted_with_completed_work_returns_its_output() {
        let runtime = Runtime::new().expect("should start a runtime");

        let output = run_until_interrupted(runtime, Cancellation::new(), async { 42 });

        assert_eq!(output, Ok(42));
    }

    /// Covers both interrupts in one test
    ///
    /// A SIGINT reaches every listener of the process. It would end a second test that ran at the
    /// same time.
    #[cfg(unix)]
    #[test]
    fn run_until_interrupted_with_two_interrupts_drops_the_work_and_returns_130() {
        let runtime = Runtime::new().expect("should start a runtime");
        let cancellation = Cancellation::new();
        let observed = cancellation.clone();
        let work_dropped = Arc::new(AtomicBool::new(false));
        let task_dropped = Arc::new(AtomicBool::new(false));
        let work_flag = DropFlag(Arc::clone(&work_dropped));
        let task_flag = DropFlag(Arc::clone(&task_dropped));

        let exit = run_until_interrupted(runtime, cancellation, async move {
            let _work_flag = work_flag;
            tokio::spawn(async move {
                let _task_flag = task_flag;
                pending::<()>().await;
            });
            interrupt();
            observed.cancelled().await;
            interrupt();
            pending::<()>().await;
        });

        assert_eq!(
            (
                exit,
                work_dropped.load(Ordering::SeqCst),
                task_dropped.load(Ordering::SeqCst)
            ),
            (Err(Exit::from(ExitCode::from(INTERRUPTED))), true, true)
        );
    }
}
