//! Signal-to-cancellation adapter
//!
//! Infrastructure that maps OS signals (SIGINT, SIGTERM) to a [`Cancellation`] token. This module
//! is consumed by the `main!()` macro expansion and is not part of the public API.
//!
//! The double Ctrl+C pattern follows the standard CLI convention: the first signal triggers
//! graceful cancellation, and the second exits immediately with code 130 (128 + SIGINT).
//!
//! On Unix, OS signal handlers are registered eagerly (at call time, not when the returned future
//! is first polled). This guarantees that signals sent after `wait_for_shutdown` is called are
//! always captured, even if the Tokio runtime has not yet polled the spawned task.
//!
//! [`Cancellation`]: crate::cancellation::Cancellation

use std::future::Future;

use crate::cancellation::Cancellation;

/// Returns a future that waits for shutdown signals and maps them to cancellation
///
/// On the first SIGINT (or SIGTERM on Unix), the `cancellation` token is cancelled, giving
/// in-flight work a chance to complete gracefully. On the second SIGINT, the process exits
/// immediately with code 130 (128 + SIGINT signal number 2).
///
/// On Unix, signal handlers are registered synchronously when this function is called, not when
/// the returned future is first polled. This is important because `main!()` spawns the future as
/// a background task, and without eager registration a signal could arrive before the runtime
/// polls the task, bypassing the handler entirely.
///
/// This function is designed to be spawned as a background Tokio task by `main!()`.
///
/// # Panics
///
/// Panics if the OS signal handler cannot be registered, which indicates system resource
/// exhaustion.
pub fn wait_for_shutdown(cancellation: Cancellation) -> impl Future<Output = ()> + Send {
    let first_signal = first_signal_listener();

    async move {
        first_signal.await;
        cancellation.cancel();

        tokio::signal::ctrl_c()
            .await
            .expect("failed to listen for SIGINT");

        std::process::exit(130);
    }
}

#[cfg(unix)]
fn first_signal_listener() -> impl Future<Output = ()> + Send {
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
    }
}

#[cfg(not(unix))]
fn first_signal_listener() -> impl Future<Output = ()> + Send {
    async {
        tokio::signal::ctrl_c()
            .await
            .expect("failed to listen for SIGINT");
    }
}
