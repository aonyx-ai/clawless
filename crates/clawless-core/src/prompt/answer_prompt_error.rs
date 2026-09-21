use thiserror::Error;

/// The error that ends a prompt without an answer from the user
///
/// The presenter reports this error through the [`Reply`] of a request. No variant is an
/// answer, so a command must not treat one as a yes, a no, or a default.
///
/// A later release can add variants. Match with a wildcard arm.
///
/// [`Reply`]: crate::event::prompt::Reply
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum AnswerPromptError {
    /// The user closed the input instead of answering
    ///
    /// On a terminal, Ctrl+D closes the input.
    #[error("the input ended before the user answered")]
    ClosedInput,

    /// The presenter dropped the request without an answer
    ///
    /// A presenter that cannot ask a user drops every request.
    #[error("the prompt was dropped without an answer")]
    DroppedRequest,

    /// The script of a test holds no answer that fits the prompt
    ///
    /// A [`ScriptedUser`] reports this when its script has ended, and when the next answer is
    /// for another kind of prompt.
    ///
    /// [`ScriptedUser`]: super::ScriptedUser
    #[error("the script holds no answer that fits the prompt")]
    UnscriptedPrompt,

    /// The terminal could not show the prompt or read the answer
    #[error("failed to use the terminal")]
    UnusableTerminal {
        /// The cause of the failure
        source: std::io::Error,
    },
}

#[cfg(test)]
mod tests {
    // An assertion in a test panics by design. A `# Panics` section on every test
    // would repeat that and give the reader no information.
    #![allow(clippy::missing_panics_doc)]

    use super::*;

    #[test]
    fn display_with_a_closed_input_states_the_condition() {
        let error = AnswerPromptError::ClosedInput;

        let message = error.to_string();

        assert_eq!(message, "the input ended before the user answered");
    }

    #[test]
    fn trait_send() {
        fn assert_send<T: Send>() {}
        assert_send::<AnswerPromptError>();
    }

    #[test]
    fn trait_sync() {
        fn assert_sync<T: Sync>() {}
        assert_sync::<AnswerPromptError>();
    }

    #[test]
    fn trait_unpin() {
        fn assert_unpin<T: Unpin>() {}
        assert_unpin::<AnswerPromptError>();
    }
}
