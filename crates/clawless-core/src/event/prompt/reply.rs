//! The way back to the command that asked one prompt

use tokio::sync::oneshot;

use super::PendingAnswer;
use crate::prompt::AnswerPromptError;

/// Returns the one answer to a [`PromptRequest`] to the command that sent it
///
/// `T` is the type of the answer, so a reply accepts only the kind of answer that its question
/// asks for.
///
/// [`answer`] and [`fail`] consume the reply. A reply that is dropped resolves the
/// [`PendingAnswer`] with [`AnswerPromptError::DroppedRequest`], so the command does not wait
/// for a reply that no longer exists.
///
/// # Examples
///
/// ```
/// use clawless_core::event::prompt::PromptRequest;
/// use clawless_core::prompt::Select;
///
/// # #[tokio::main(flavor = "current_thread")]
/// # async fn main() {
/// let question = Select::builder().question("Kind of change").options(["Added", "Fixed"]).build();
/// let (request, pending) = PromptRequest::select(question);
///
/// match request {
///     PromptRequest::Select { reply, .. } => reply.answer(1),
///     PromptRequest::Confirm { .. } | PromptRequest::Text { .. } => {}
/// }
///
/// assert_eq!(pending.wait().await.expect("should answer"), 1);
/// # }
/// ```
///
/// [`PromptRequest`]: super::PromptRequest
/// [`answer`]: Reply::answer
/// [`fail`]: Reply::fail
#[derive(Debug)]
pub struct Reply<T> {
    /// The channel that carries the one answer to the command
    sender: oneshot::Sender<Result<T, AnswerPromptError>>,
}

impl<T> Reply<T> {
    /// Creates a reply and the pending answer that it resolves
    pub(super) fn channel() -> (Self, PendingAnswer<T>) {
        let (sender, receiver) = oneshot::channel();

        (Self { sender }, PendingAnswer::new(receiver))
    }

    /// Returns when the command has stopped waiting for the answer
    ///
    /// A command stops waiting when it drops its [`PendingAnswer`], which cancellation causes. A
    /// presenter awaits this method together with the input of the user, and stops asking when
    /// this method returns.
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
    /// drop(pending);
    ///
    /// match request {
    ///     PromptRequest::Confirm { mut reply, .. } => reply.abandoned().await,
    ///     PromptRequest::Select { .. } | PromptRequest::Text { .. } => {}
    /// }
    /// # }
    /// ```
    pub async fn abandoned(&mut self) {
        self.sender.closed().await;
    }

    /// Sends the answer of the user to the command
    ///
    /// If the command has stopped waiting, this method drops the answer.
    ///
    /// # Examples
    ///
    /// ```
    /// use clawless_core::event::prompt::PromptRequest;
    /// use clawless_core::prompt::Text;
    ///
    /// # #[tokio::main(flavor = "current_thread")]
    /// # async fn main() {
    /// let (request, pending) = PromptRequest::text(Text::new("Title of the change"));
    ///
    /// match request {
    ///     PromptRequest::Text { reply, .. } => reply.answer("Add prompts".to_string()),
    ///     PromptRequest::Confirm { .. } | PromptRequest::Select { .. } => {}
    /// }
    ///
    /// assert_eq!(pending.wait().await.expect("should answer"), "Add prompts");
    /// # }
    /// ```
    pub fn answer(self, answer: T) {
        drop(self.sender.send(Ok(answer)));
    }

    /// Tells the command why the prompt has no answer
    ///
    /// If the command has stopped waiting, this method drops the error.
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
    ///
    /// match request {
    ///     PromptRequest::Text { reply, .. } => reply.fail(AnswerPromptError::ClosedInput),
    ///     PromptRequest::Confirm { .. } | PromptRequest::Select { .. } => {}
    /// }
    ///
    /// let error = pending.wait().await.expect_err("the input closed");
    ///
    /// assert!(matches!(error, AnswerPromptError::ClosedInput));
    /// # }
    /// ```
    pub fn fail(self, error: AnswerPromptError) {
        drop(self.sender.send(Err(error)));
    }

    /// Reports whether the command has stopped waiting for the answer
    ///
    /// # Examples
    ///
    /// ```
    /// use clawless_core::event::prompt::PromptRequest;
    /// use clawless_core::prompt::{Confirm, Confirmation};
    ///
    /// let (request, pending) = PromptRequest::confirm(Confirm::new("Delete the branch?"));
    /// drop(pending);
    ///
    /// match request {
    ///     PromptRequest::Confirm { reply, .. } => assert!(reply.is_abandoned()),
    ///     PromptRequest::Select { .. } | PromptRequest::Text { .. } => {}
    /// }
    /// ```
    pub fn is_abandoned(&self) -> bool {
        self.sender.is_closed()
    }
}

#[cfg(test)]
mod tests {
    #![allow(
        clippy::missing_panics_doc,
        reason = "an assertion in a test panics by design, so a `# Panics` section would repeat it"
    )]

    use super::*;
    use crate::prompt::Confirmation;

    #[tokio::test]
    async fn abandoned_with_a_dropped_pending_answer_returns() {
        let (mut reply, pending) = Reply::<Confirmation>::channel();
        drop(pending);

        reply.abandoned().await;

        assert!(reply.is_abandoned());
    }

    #[tokio::test]
    async fn fail_reports_the_error_to_the_command() {
        let (reply, pending) = Reply::<Confirmation>::channel();

        reply.fail(AnswerPromptError::ClosedInput);

        assert!(match pending.wait().await.expect_err("should fail") {
            AnswerPromptError::ClosedInput => true,
            AnswerPromptError::DroppedRequest => false,
            AnswerPromptError::UnscriptedPrompt => false,
            AnswerPromptError::UnusableTerminal { .. } => false,
        });
    }

    #[test]
    fn is_abandoned_with_a_waiting_command_returns_false() {
        let (reply, _pending) = Reply::<Confirmation>::channel();

        let abandoned = reply.is_abandoned();

        assert!(!abandoned);
    }

    #[test]
    fn trait_send() {
        fn assert_send<T: Send>() {}
        assert_send::<Reply<Confirmation>>();
    }

    #[test]
    fn trait_sync() {
        fn assert_sync<T: Sync>() {}
        assert_sync::<Reply<Confirmation>>();
    }

    #[test]
    fn trait_unpin() {
        fn assert_unpin<T: Unpin>() {}
        assert_unpin::<Reply<Confirmation>>();
    }
}
