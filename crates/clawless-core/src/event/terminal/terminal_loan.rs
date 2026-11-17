//! The terminal as the command holds it

use std::convert::Infallible;
use std::process::Stdio;

use tokio::sync::oneshot;

/// The terminal as the command holds it
///
/// The command holds the loan while its program runs, and it drops the loan when the program
/// has ended. Dropping the loan gives the terminal back, and the presenter renders again. A
/// command that panics or is dropped while it holds the loan gives the terminal back as well.
///
/// The loan carries the stream that the standard output of the program goes to. The presenter
/// chose that stream when it lent the terminal.
///
/// Do not wait for the presenter while you hold the loan. The presenter handles no event until
/// the loan returns, so a send into a full channel would wait forever.
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
/// let returned = request.lend(Stdio::null());
///
/// let loan = pending.wait().await.expect("the presenter lent the terminal");
/// drop(loan);
///
/// returned.await;
/// # }
/// ```
#[derive(Debug)]
pub struct TerminalLoan {
    /// Where the standard output of the program goes
    stdout: Stdio,

    /// The channel whose end tells the presenter that the terminal is back
    ///
    /// Nothing is ever sent on it, which its type states. Only dropping it has a meaning.
    hold: oneshot::Sender<Infallible>,
}

impl TerminalLoan {
    /// Creates a loan over the stream for the standard output, and the channel that it holds
    ///
    /// The receiver ends when the loan is dropped, which tells the presenter that the terminal
    /// is back.
    pub(super) fn channel(stdout: Stdio) -> (Self, oneshot::Receiver<Infallible>) {
        let (hold, returned) = oneshot::channel();

        (Self { stdout, hold }, returned)
    }

    /// Separates the stream for the standard output from the hold on the terminal
    ///
    /// Starting the program consumes the stream, while the command holds the terminal until the
    /// program ends. Dropping the second value gives the terminal back.
    pub(crate) fn into_parts(self) -> (Stdio, oneshot::Sender<Infallible>) {
        let Self { stdout, hold } = self;

        (stdout, hold)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn trait_send() {
        fn assert_send<T: Send>() {}
        assert_send::<TerminalLoan>();
    }

    #[test]
    fn trait_sync() {
        fn assert_sync<T: Sync>() {}
        assert_sync::<TerminalLoan>();
    }

    #[test]
    fn trait_unpin() {
        fn assert_unpin<T: Unpin>() {}
        assert_unpin::<TerminalLoan>();
    }
}
