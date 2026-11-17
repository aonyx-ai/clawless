//! The loan of the terminal, which has not arrived yet

use tokio::sync::oneshot;

use super::{Refusal, TerminalLoan};

/// The loan of the terminal for one [`TerminalRequest`], which has not arrived yet
///
/// The pending loan resolves when the presenter lends the terminal, when it refuses it, and when
/// it drops the request. Dropping the pending loan tells the presenter that the command has
/// stopped waiting, and a loan that arrives afterwards returns at once.
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
/// drop(request);
///
/// assert_eq!(pending.wait().await.err(), Some(Refusal::DroppedRequest));
/// # }
/// ```
///
/// [`TerminalRequest`]: super::TerminalRequest
#[derive(Debug)]
pub struct PendingLoan {
    /// The channel that carries the answer of the presenter to the command
    receiver: oneshot::Receiver<Result<TerminalLoan, Refusal>>,
}

impl PendingLoan {
    /// Creates a pending loan over the channel of a request
    pub(super) fn new(receiver: oneshot::Receiver<Result<TerminalLoan, Refusal>>) -> Self {
        Self { receiver }
    }

    /// Waits for the presenter to lend the terminal
    ///
    /// # Errors
    ///
    /// Returns the [`Refusal`] of the presenter. Returns [`Refusal::DroppedRequest`] if the
    /// presenter dropped the request without an answer.
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
    /// let _lent = request.lend(Stdio::null());
    ///
    /// assert!(pending.wait().await.is_ok());
    /// # }
    /// ```
    pub async fn wait(self) -> Result<TerminalLoan, Refusal> {
        match self.receiver.await {
            Ok(answer) => answer,
            Err(_dropped) => Err(Refusal::DroppedRequest),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn trait_send() {
        fn assert_send<T: Send>() {}
        assert_send::<PendingLoan>();
    }

    #[test]
    fn trait_sync() {
        fn assert_sync<T: Sync>() {}
        assert_sync::<PendingLoan>();
    }

    #[test]
    fn trait_unpin() {
        fn assert_unpin<T: Unpin>() {}
        assert_unpin::<PendingLoan>();
    }
}
