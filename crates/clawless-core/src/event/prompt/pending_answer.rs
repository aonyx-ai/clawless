//! The answer to one prompt, which has not arrived yet

use tokio::sync::oneshot;

use crate::prompt::AnswerPromptError;

/// The answer to one [`PromptRequest`], which has not arrived yet
///
/// The pending answer resolves when the presenter uses the [`Reply`] of the request, and when
/// the presenter drops it. Dropping the pending answer tells the presenter that the command has
/// stopped waiting.
///
/// # Examples
///
/// ```
/// use clawless_core::event::prompt::PromptRequest;
/// use clawless_core::prompt::{Confirm, Confirmation};
///
/// # #[tokio::main(flavor = "current_thread")]
/// # async fn main() {
/// let (request, pending) = PromptRequest::confirm(Confirm::new("Delete the branch?"));
///
/// match request {
///     PromptRequest::Confirm { reply, .. } => reply.answer(Confirmation::Yes),
///     PromptRequest::Select { .. } | PromptRequest::Text { .. } => {}
/// }
///
/// assert_eq!(pending.wait().await.expect("should answer"), Confirmation::Yes);
/// # }
/// ```
///
/// [`PromptRequest`]: super::PromptRequest
/// [`Reply`]: super::Reply
#[derive(Debug)]
pub struct PendingAnswer<T> {
    /// The channel that carries the one answer to the command
    receiver: oneshot::Receiver<Result<T, AnswerPromptError>>,
}

impl<T> PendingAnswer<T> {
    /// Creates a pending answer over the channel of a reply
    pub(super) fn new(receiver: oneshot::Receiver<Result<T, AnswerPromptError>>) -> Self {
        Self { receiver }
    }

    /// Waits for the answer of the user
    ///
    /// # Errors
    ///
    /// Returns the [`AnswerPromptError`] that the presenter reported. Returns
    /// [`AnswerPromptError::DroppedRequest`] if the presenter dropped the request.
    ///
    /// # Examples
    ///
    /// ```
    /// use clawless_core::event::prompt::PromptRequest;
    /// use clawless_core::prompt::{AnswerPromptError, Text};
    ///
    /// # #[tokio::main(flavor = "current_thread")]
    /// # async fn main() {
    /// let (request, pending) = PromptRequest::text(Text::new("Title of the change"));
    /// drop(request);
    ///
    /// let error = pending.wait().await.expect_err("the request was dropped");
    ///
    /// assert!(matches!(error, AnswerPromptError::DroppedRequest));
    /// # }
    /// ```
    pub async fn wait(self) -> Result<T, AnswerPromptError> {
        match self.receiver.await {
            Ok(answer) => answer,
            Err(_dropped) => Err(AnswerPromptError::DroppedRequest),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::prompt::Confirmation;

    #[test]
    fn trait_send() {
        fn assert_send<T: Send>() {}
        assert_send::<PendingAnswer<Confirmation>>();
    }

    #[test]
    fn trait_sync() {
        fn assert_sync<T: Sync>() {}
        assert_sync::<PendingAnswer<Confirmation>>();
    }

    #[test]
    fn trait_unpin() {
        fn assert_unpin<T: Unpin>() {}
        assert_unpin::<PendingAnswer<Confirmation>>();
    }
}
