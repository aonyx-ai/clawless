//! The request that borrows the terminal from the presenter
//!
//! A program such as an editor, a pager, or `ssh` draws on the terminal and reads the keys of the
//! user. The presenter writes to the same terminal, so the two must take turns. A command
//! therefore asks the presenter for the terminal before such a program starts. It gives the
//! terminal back when the program ends.
//!
//! The command sends a [`TerminalRequest`] through the event channel and waits on the matching
//! [`PendingLoan`]. The request arrives after every event that the command sent before it, so the
//! presenter shows that output first. The presenter then calls [`TerminalRequest::lend`], and the
//! command receives a [`TerminalLoan`]. The presenter awaits the future of the call until the
//! command drops the loan, and it renders nothing in the meantime.
//!
//! A presenter that cannot lend the terminal answers with a [`Refusal`]. The command then does
//! not start the program.

use std::future::Future;
use std::process::Stdio;

use tokio::sync::oneshot;

pub use self::pending_loan::PendingLoan;
pub use self::refusal::Refusal;
pub use self::terminal_loan::TerminalLoan;
use crate::process::Invocation;

mod pending_loan;
mod refusal;
mod terminal_loan;

/// A request from a command for the terminal, on behalf of one program
///
/// The request names the [`Invocation`] of the program that takes over the terminal. A presenter
/// can therefore say what holds the terminal.
///
/// A presenter answers the request once. It lends the terminal with [`lend`], which also names
/// where the standard output of the program goes. It refuses with [`refuse`], which names the
/// reason. A presenter that drops the request refuses it with [`Refusal::DroppedRequest`].
///
/// The constructor is public so that the tests of a presenter can build a request.
///
/// # Examples
///
/// ```
/// use std::process::Stdio;
///
/// use clawless_core::event::terminal::TerminalRequest;
/// use clawless_core::process::Invocation;
///
/// # #[tokio::main(flavor = "current_thread")]
/// # async fn main() {
/// let (request, pending) = TerminalRequest::new(Invocation::new("vi").arg("notes.md"));
///
/// let returned = request.lend(Stdio::null());
/// let loan = pending.wait().await.expect("the presenter lent the terminal");
/// drop(loan);
///
/// returned.await;
/// # }
/// ```
///
/// [`lend`]: TerminalRequest::lend
/// [`refuse`]: TerminalRequest::refuse
#[derive(Debug)]
pub struct TerminalRequest {
    /// The program that takes over the terminal
    invocation: Invocation,

    /// The channel that carries the answer of the presenter to the command
    sender: oneshot::Sender<Result<TerminalLoan, Refusal>>,
}

impl TerminalRequest {
    /// Creates a request for the terminal, and the pending loan that answers it
    ///
    /// # Examples
    ///
    /// ```
    /// use clawless_core::event::terminal::TerminalRequest;
    /// use clawless_core::process::Invocation;
    ///
    /// let (request, pending) = TerminalRequest::new(Invocation::new("vi"));
    /// ```
    pub fn new(invocation: Invocation) -> (Self, PendingLoan) {
        let (sender, receiver) = oneshot::channel();

        (Self { invocation, sender }, PendingLoan::new(receiver))
    }

    /// Returns the program that takes over the terminal
    ///
    /// # Examples
    ///
    /// ```
    /// use clawless_core::event::terminal::TerminalRequest;
    /// use clawless_core::process::Invocation;
    ///
    /// let (request, _pending) = TerminalRequest::new(Invocation::new("vi"));
    ///
    /// assert_eq!(request.invocation().to_string(), "vi");
    /// ```
    pub fn invocation(&self) -> &Invocation {
        &self.invocation
    }

    /// Lends the terminal to the command, and returns a future that waits until it is back
    ///
    /// `stdout` is where the standard output of the program goes. The presenter passes a
    /// terminal, so that the user sees the program. The program reads the standard input of the
    /// application, and it writes its errors to the standard error of the application.
    ///
    /// The call sends the loan at once, and the future only waits. The presenter writes nothing
    /// until the future completes. It flushes its output before the call, because the program
    /// draws as soon as it starts.
    ///
    /// The command gives the terminal back when its program has ended. It also gives it back when
    /// it stops waiting for the loan, panics, or is dropped. The future therefore completes in
    /// every case in which the command no longer uses the terminal.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::process::Stdio;
    ///
    /// use clawless_core::event::terminal::TerminalRequest;
    /// use clawless_core::process::Invocation;
    ///
    /// # #[tokio::main(flavor = "current_thread")]
    /// # async fn main() {
    /// let (request, pending) = TerminalRequest::new(Invocation::new("vi"));
    /// drop(pending);
    ///
    /// request.lend(Stdio::null()).await;
    /// # }
    /// ```
    pub fn lend(self, stdout: impl Into<Stdio>) -> impl Future<Output = ()> + Send + 'static {
        let (loan, returned) = TerminalLoan::channel(stdout.into());

        drop(self.sender.send(Ok(loan)));

        async move {
            match returned.await {
                Ok(never) => match never {},
                Err(_returned) => {}
            }
        }
    }

    /// Refuses the terminal to the command, for the given reason
    ///
    /// The command receives the refusal and does not start its program.
    ///
    /// # Examples
    ///
    /// ```
    /// use clawless_core::event::terminal::{Refusal, TerminalRequest};
    /// use clawless_core::process::Invocation;
    ///
    /// # #[tokio::main(flavor = "current_thread")]
    /// # async fn main() {
    /// let (request, pending) = TerminalRequest::new(Invocation::new("vi"));
    ///
    /// request.refuse(Refusal::MissingTerminal);
    ///
    /// assert_eq!(pending.wait().await.err(), Some(Refusal::MissingTerminal));
    /// # }
    /// ```
    pub fn refuse(self, refusal: Refusal) {
        drop(self.sender.send(Err(refusal)));
    }
}

#[cfg(test)]
mod tests {
    #![allow(
        clippy::missing_panics_doc,
        reason = "an assertion in a test panics by design. A `# Panics` section on every test would repeat that and give the reader no information"
    )]

    use std::time::Duration;

    use tokio::time::timeout;

    use super::*;

    #[tokio::test]
    async fn lend_holds_the_presenter_until_the_command_returns_the_loan() {
        let (request, pending) = TerminalRequest::new(Invocation::new("vi"));
        let returned = request.lend(Stdio::null());
        let _loan = pending.wait().await.expect("should lend");

        let waited = timeout(Duration::from_millis(50), returned).await;

        assert!(waited.is_err());
    }

    #[tokio::test]
    async fn lend_to_a_command_that_stopped_waiting_returns_at_once() {
        let (request, pending) = TerminalRequest::new(Invocation::new("vi"));
        drop(pending);

        let waited = timeout(Duration::from_secs(5), request.lend(Stdio::null())).await;

        assert!(waited.is_ok());
    }

    #[tokio::test]
    async fn lend_with_a_returned_loan_releases_the_presenter() {
        let (request, pending) = TerminalRequest::new(Invocation::new("vi"));
        let returned = request.lend(Stdio::null());
        let loan = pending.wait().await.expect("should lend");
        drop(loan);

        let waited = timeout(Duration::from_secs(5), returned).await;

        assert!(waited.is_ok());
    }

    #[tokio::test]
    async fn new_with_a_dropped_request_resolves_with_a_dropped_request() {
        let (request, pending) = TerminalRequest::new(Invocation::new("vi"));
        drop(request);

        let refusal = pending.wait().await.err();

        assert_eq!(refusal, Some(Refusal::DroppedRequest));
    }

    #[tokio::test]
    async fn refuse_resolves_with_the_refusal() {
        let (request, pending) = TerminalRequest::new(Invocation::new("vi"));
        request.refuse(Refusal::PendingRead);

        let refusal = pending.wait().await.err();

        assert_eq!(refusal, Some(Refusal::PendingRead));
    }

    #[test]
    fn trait_send() {
        fn assert_send<T: Send>() {}
        assert_send::<TerminalRequest>();
    }

    #[test]
    fn trait_sync() {
        fn assert_sync<T: Sync>() {}
        assert_sync::<TerminalRequest>();
    }

    #[test]
    fn trait_unpin() {
        fn assert_unpin<T: Unpin>() {}
        assert_unpin::<TerminalRequest>();
    }
}
