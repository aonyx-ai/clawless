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
    pub async fn abandoned(&mut self) {
        self.sender.closed().await;
    }

    /// Sends the answer of the user to the command
    ///
    /// If the command has stopped waiting, this method drops the answer.
    pub fn answer(self, answer: T) {
        drop(self.sender.send(Ok(answer)));
    }

    /// Tells the command why the prompt has no answer
    ///
    /// If the command has stopped waiting, this method drops the error.
    pub fn fail(self, error: AnswerPromptError) {
        drop(self.sender.send(Err(error)));
    }

    /// Reports whether the command has stopped waiting for the answer
    pub fn is_abandoned(&self) -> bool {
        self.sender.is_closed()
    }
}

#[cfg(test)]
mod tests {
    // An assertion in a test panics by design. A `# Panics` section on every test
    // would repeat that and give the reader no information.
    #![allow(clippy::missing_panics_doc)]

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
