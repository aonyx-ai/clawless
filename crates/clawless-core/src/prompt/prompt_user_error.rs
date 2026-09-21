use thiserror::Error;

use super::AnswerPromptError;
use crate::event::SendError;

/// The error returned when a command cannot get an answer from its user
///
/// No variant is an answer, so a command must not treat one as a yes, a no, or a default. Every
/// variant carries the text of the question.
///
/// A later release can add variants, and it can add fields to a variant. Match with a wildcard
/// arm, and bind the fields of a variant with `..`.
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum PromptUserError {
    /// No user is present who can answer
    ///
    /// The [`Interactivity`] of the prompt is `NonInteractive`. The prompt sends no request.
    ///
    /// [`Interactivity`]: crate::context::Interactivity
    #[error("no user is present to answer the prompt `{question}`")]
    #[non_exhaustive]
    AbsentUser {
        /// The text of the question
        question: String,
    },

    /// Cancellation stopped the prompt before the user answered
    #[error("the prompt `{question}` was cancelled before the user answered")]
    #[non_exhaustive]
    CancelledPrompt {
        /// The text of the question
        question: String,
    },

    /// The presenter returned no answer
    ///
    /// The source is the reason.
    #[error("failed to get an answer to the prompt `{question}`")]
    #[non_exhaustive]
    UnansweredPrompt {
        /// The text of the question
        question: String,

        /// The cause of the failure
        source: AnswerPromptError,
    },

    /// The prompt could not be sent to the presenter
    ///
    /// The event channel is closed.
    #[error("failed to send the prompt `{question}` to the presenter")]
    #[non_exhaustive]
    UndeliverablePrompt {
        /// The text of the question
        question: String,

        /// The cause of the failure
        source: SendError,
    },
}

#[cfg(test)]
mod tests {
    // An assertion in a test panics by design. A `# Panics` section on every test
    // would repeat that and give the reader no information.
    #![allow(clippy::missing_panics_doc)]

    use super::*;

    #[test]
    fn display_with_a_cancelled_prompt_names_the_question() {
        let error = PromptUserError::CancelledPrompt {
            question: "Release?".to_owned(),
        };

        let message = error.to_string();

        assert_eq!(
            message,
            "the prompt `Release?` was cancelled before the user answered"
        );
    }

    #[test]
    fn display_with_an_absent_user_names_the_question() {
        let error = PromptUserError::AbsentUser {
            question: "Release?".to_owned(),
        };

        let message = error.to_string();

        assert_eq!(
            message,
            "no user is present to answer the prompt `Release?`"
        );
    }

    #[test]
    fn display_with_an_unanswered_prompt_names_the_question() {
        let error = PromptUserError::UnansweredPrompt {
            question: "Release?".to_owned(),
            source: AnswerPromptError::ClosedInput,
        };

        let message = error.to_string();

        assert_eq!(message, "failed to get an answer to the prompt `Release?`");
    }

    #[test]
    fn trait_send() {
        fn assert_send<T: Send>() {}
        assert_send::<PromptUserError>();
    }

    #[test]
    fn trait_sync() {
        fn assert_sync<T: Sync>() {}
        assert_sync::<PromptUserError>();
    }

    #[test]
    fn trait_unpin() {
        fn assert_unpin<T: Unpin>() {}
        assert_unpin::<PromptUserError>();
    }
}
