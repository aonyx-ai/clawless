//! Signal-to-cancellation adapter
//!
//! Infrastructure that maps OS signals (SIGINT, SIGTERM) to a [`Cancellation`] token. This module
//! is consumed by the `main!()` macro expansion and is not part of the public API.
//!
//! The double Ctrl+C pattern follows the standard CLI convention: the first signal triggers
//! graceful cancellation, and the second exits immediately with code 130 (128 + SIGINT).
//!
//! [`Cancellation`]: crate::cancellation::Cancellation

use crate::cancellation::Cancellation;

/// Waits for shutdown signals and maps them to cancellation
///
/// On the first SIGINT (or SIGTERM on Unix), the `cancellation` token is cancelled, giving
/// in-flight work a chance to complete gracefully. On the second SIGINT, the process exits
/// immediately with code 130 (128 + SIGINT signal number 2).
///
/// This function is designed to be spawned as a background Tokio task by `main!()`.
///
/// # Panics
///
/// Panics if the OS signal handler cannot be registered, which indicates system resource
/// exhaustion.
pub async fn wait_for_shutdown(cancellation: Cancellation) {
    wait_for_first_signal().await;
    cancellation.cancel();

    tokio::signal::ctrl_c()
        .await
        .expect("failed to listen for SIGINT");

    std::process::exit(130);
}

#[cfg(unix)]
async fn wait_for_first_signal() {
    use tokio::signal::unix::SignalKind;

    let mut sigterm =
        tokio::signal::unix::signal(SignalKind::terminate()).expect("failed to listen for SIGTERM");

    tokio::select! {
        result = tokio::signal::ctrl_c() => {
            result.expect("failed to listen for SIGINT");
        }
        _ = sigterm.recv() => {}
    }
}

#[cfg(not(unix))]
async fn wait_for_first_signal() {
    tokio::signal::ctrl_c()
        .await
        .expect("failed to listen for SIGINT");
}
