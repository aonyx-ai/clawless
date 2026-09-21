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
use crate::prompt::{AnswerPromptError, Confirm, Confirmation, Select, Text};

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
///     PromptRequest::Select { reply, .. } => reply.answer(0),
///     PromptRequest::Text { reply, .. } => reply.answer(String::new()),
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

    /// A question that the user answers with one of several options
    Select {
        /// The question to ask
        question: Select,

        /// The reply that returns the answer to the command
        ///
        /// The answer is the position of the selected option, and the first option has the
        /// position zero.
        reply: Reply<usize>,
    },

    /// A question that the user answers with one line of text
    Text {
        /// The question to ask
        question: Text,

        /// The reply that returns the answer to the command
        reply: Reply<String>,
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

    /// Creates a request for a selection, and the pending answer to it
    ///
    /// # Examples
    ///
    /// ```
    /// use clawless_core::event::prompt::PromptRequest;
    /// use clawless_core::prompt::Select;
    ///
    /// let question = Select::new("Kind of change", ["Added", "Changed", "Fixed"]);
    ///
    /// let (request, pending) = PromptRequest::select(question);
    /// ```
    pub fn select(question: Select) -> (Self, PendingAnswer<usize>) {
        let (reply, pending) = Reply::channel();

        (Self::Select { question, reply }, pending)
    }

    /// Creates a request for a line of text, and the pending answer to it
    ///
    /// # Examples
    ///
    /// ```
    /// use clawless_core::event::prompt::PromptRequest;
    /// use clawless_core::prompt::Text;
    ///
    /// let (request, pending) = PromptRequest::text(Text::new("Title of the change"));
    /// ```
    pub fn text(question: Text) -> (Self, PendingAnswer<String>) {
        let (reply, pending) = Reply::channel();

        (Self::Text { question, reply }, pending)
    }

    /// Returns the text of the question, for every kind of prompt
    ///
    /// # Examples
    ///
    /// ```
    /// use clawless_core::event::prompt::PromptRequest;
    /// use clawless_core::prompt::Text;
    ///
    /// let (request, _pending) = PromptRequest::text(Text::new("Title of the change"));
    ///
    /// assert_eq!(request.question(), "Title of the change");
    /// ```
    pub fn question(&self) -> &str {
        match self {
            Self::Confirm { question, .. } => question.question(),
            Self::Select { question, .. } => question.question(),
            Self::Text { question, .. } => question.question(),
        }
    }

    /// Reports to the command why the prompt has no answer, for every kind of prompt
    ///
    /// # Examples
    ///
    /// ```
    /// use clawless_core::event::prompt::PromptRequest;
    /// use clawless_core::prompt::{AnswerPromptError, Text};
    ///
    /// let (request, pending) = PromptRequest::text(Text::new("Title of the change"));
    ///
    /// request.fail(AnswerPromptError::ClosedInput);
    /// ```
    pub fn fail(self, error: AnswerPromptError) {
        match self {
            Self::Confirm { reply, .. } => reply.fail(error),
            Self::Select { reply, .. } => reply.fail(error),
            Self::Text { reply, .. } => reply.fail(error),
        }
    }
}

#[cfg(test)]
mod tests {
    // An assertion in a test panics by design. A `# Panics` section on every test
    // would repeat that and give the reader no information.
    #![allow(clippy::missing_panics_doc)]

    use super::*;

    #[tokio::test]
    async fn confirm_with_a_dropped_request_reports_the_drop() {
        let (request, pending) = PromptRequest::confirm(Confirm::new("Release?"));
        drop(request);

        let error = pending.wait().await.expect_err("should fail");

        assert!(match error {
            AnswerPromptError::DroppedRequest => true,
            AnswerPromptError::ClosedInput => false,
            AnswerPromptError::UnscriptedPrompt => false,
            AnswerPromptError::UnusableTerminal { .. } => false,
        });
    }

    #[tokio::test]
    async fn confirm_with_an_answer_resolves_the_pending_answer() {
        let (request, pending) = PromptRequest::confirm(Confirm::new("Release?"));
        match request {
            PromptRequest::Confirm { reply, .. } => reply.answer(Confirmation::Yes),
            PromptRequest::Select { .. } | PromptRequest::Text { .. } => {}
        }

        let answer = pending.wait().await.expect("should answer");

        assert_eq!(answer, Confirmation::Yes);
    }

    #[tokio::test]
    async fn fail_tells_the_command_why_the_prompt_has_no_answer() {
        let (request, pending) = PromptRequest::text(Text::new("Title"));
        request.fail(AnswerPromptError::ClosedInput);

        let error = pending.wait().await.expect_err("should fail");

        assert_eq!(
            error.to_string(),
            "the input ended before the user answered"
        );
    }

    #[test]
    fn question_with_a_selection_returns_its_text() {
        let (request, _pending) = PromptRequest::select(Select::new("Category", ["Added"]));

        let question = request.question();

        assert_eq!(question, "Category");
    }

    #[tokio::test]
    async fn select_with_an_answer_resolves_the_pending_answer() {
        let (request, pending) = PromptRequest::select(Select::new("Category", ["Added", "Fixed"]));
        match request {
            PromptRequest::Select { reply, .. } => reply.answer(1),
            PromptRequest::Confirm { .. } | PromptRequest::Text { .. } => {}
        }

        let answer = pending.wait().await.expect("should answer");

        assert_eq!(answer, 1);
    }

    #[tokio::test]
    async fn text_with_an_answer_resolves_the_pending_answer() {
        let (request, pending) = PromptRequest::text(Text::new("Title"));
        match request {
            PromptRequest::Text { reply, .. } => reply.answer("Fix the race".to_owned()),
            PromptRequest::Confirm { .. } | PromptRequest::Select { .. } => {}
        }

        let answer = pending.wait().await.expect("should answer");

        assert_eq!(answer, "Fix the race");
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
