//! The request that carries a prompt to the presenter
//!
//! A command sends a [`PromptRequest`] through the event channel. The request arrives after
//! every event that the command sent before it, so the presenter shows that output first.
//!
//! Events flow from the command to the presenter only. Each request therefore carries its own
//! [`Reply`], which returns exactly one answer to the command. The command waits on the matching
//! [`PendingAnswer`].

pub use self::pending_answer::PendingAnswer;
pub use self::reply::Reply;
use crate::prompt::{Confirm, Confirmation};

/// The answer to one prompt, which has not arrived yet
mod pending_answer;
/// The way back to the command that asked one prompt
mod reply;

/// One question for the user, with the [`Reply`] that answers it
///
/// Each variant pairs a kind of question with a [`Reply`] of the matching answer type.
///
/// A presenter answers a request with [`Reply::answer`], or reports why it has no answer with
/// [`Reply::fail`]. A presenter that cannot ask a user drops the request. The command then
/// receives [`AnswerPromptError::DroppedRequest`] and does not wait.
///
/// The constructors are public so that the tests of a presenter can build a request.
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
///     PromptRequest::Confirm { reply, .. } => reply.answer(Confirmation::No),
/// }
///
/// assert_eq!(pending.wait().await.expect("should answer"), Confirmation::No);
/// # }
/// ```
///
/// [`AnswerPromptError::DroppedRequest`]: crate::prompt::AnswerPromptError::DroppedRequest
#[derive(Debug)]
pub enum PromptRequest {
    /// A question that the user answers with yes or no
    Confirm {
        /// The question to ask
        question: Confirm,

        /// The reply that returns the answer to the command
        reply: Reply<Confirmation>,
    },
}

impl PromptRequest {
    /// Creates a request for a confirmation, and the pending answer to it
    ///
    /// # Examples
    ///
    /// ```
    /// use clawless_core::event::prompt::PromptRequest;
    /// use clawless_core::prompt::Confirm;
    ///
    /// let (request, pending) = PromptRequest::confirm(Confirm::new("Delete the branch?"));
    /// ```
    pub fn confirm(question: Confirm) -> (Self, PendingAnswer<Confirmation>) {
        let (reply, pending) = Reply::channel();

        (Self::Confirm { question, reply }, pending)
    }
}

#[cfg(test)]
mod tests {
    // An assertion in a test panics by design. A `# Panics` section on every test
    // would repeat that and give the reader no information.
    #![allow(clippy::missing_panics_doc)]

    use super::*;
    use crate::prompt::AnswerPromptError;

    #[tokio::test]
    async fn confirm_with_a_dropped_request_reports_the_drop() {
        let (request, pending) = PromptRequest::confirm(Confirm::new("Release?"));
        drop(request);

        let error = pending.wait().await.expect_err("should fail");

        assert!(match error {
            AnswerPromptError::DroppedRequest => true,
            AnswerPromptError::ClosedInput => false,
            AnswerPromptError::UnusableTerminal { .. } => false,
        });
    }

    #[tokio::test]
    async fn confirm_with_an_answer_resolves_the_pending_answer() {
        let (request, pending) = PromptRequest::confirm(Confirm::new("Release?"));
        let PromptRequest::Confirm { reply, .. } = request;
        reply.answer(Confirmation::Yes);

        let answer = pending.wait().await.expect("should answer");

        assert_eq!(answer, Confirmation::Yes);
    }

    #[test]
    fn trait_send() {
        fn assert_send<T: Send>() {}
        assert_send::<PromptRequest>();
    }

    #[test]
    fn trait_sync() {
        fn assert_sync<T: Sync>() {}
        assert_sync::<PromptRequest>();
    }

    #[test]
    fn trait_unpin() {
        fn assert_unpin<T: Unpin>() {}
        assert_unpin::<PromptRequest>();
    }
}
