//! The reason why a presenter does not lend the terminal

use std::fmt::{Display, Formatter, Result as FmtResult};

/// The reason why a presenter does not lend the terminal
///
/// A command that receives a refusal does not start its program. Each reason tells the user what
/// to change: run the command in a terminal, or answer the open question first.
///
/// A later release can add variants. Match with a wildcard arm.
///
/// # Examples
///
/// ```
/// use clawless_core::event::terminal::Refusal;
///
/// assert_eq!(
///     Refusal::MissingTerminal.to_string(),
///     "the application has no terminal for the program"
/// );
/// ```
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
#[non_exhaustive]
pub enum Refusal {
    /// The presenter dropped the request without an answer
    ///
    /// A presenter that has no terminal to lend does that. The projection of a TUI application
    /// draws on the terminal itself, and the scripted user of a test has no terminal.
    DroppedRequest,

    /// The application has no terminal for the program
    ///
    /// The standard input is not a terminal, or neither output stream is one. A program that
    /// takes over the terminal would then wait for a user who is not there, as in CI or in a
    /// pipe.
    MissingTerminal,

    /// A read of the terminal is still pending
    ///
    /// A prompt that the command abandoned leaves its read open until the user ends the line.
    /// That read would take the keys that the user types into the program.
    PendingRead,
}

/// Renders the reason as a phrase that completes the message of an error
impl Display for Refusal {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            Self::DroppedRequest => f.write_str("the presenter does not lend the terminal"),
            Self::MissingTerminal => f.write_str("the application has no terminal for the program"),
            Self::PendingRead => f.write_str("an abandoned prompt still reads the terminal"),
        }
    }
}

#[cfg(test)]
mod tests {
    #![allow(
        clippy::missing_panics_doc,
        reason = "an assertion in a test panics by design. A `# Panics` section on every test would repeat that and give the reader no information"
    )]

    use super::*;

    #[test]
    fn display_with_a_dropped_request_names_the_presenter() {
        let message = Refusal::DroppedRequest.to_string();

        assert_eq!(message, "the presenter does not lend the terminal");
    }

    #[test]
    fn display_with_a_missing_terminal_names_the_program() {
        let message = Refusal::MissingTerminal.to_string();

        assert_eq!(message, "the application has no terminal for the program");
    }

    #[test]
    fn display_with_a_pending_read_names_the_prompt() {
        let message = Refusal::PendingRead.to_string();

        assert_eq!(message, "an abandoned prompt still reads the terminal");
    }

    #[test]
    fn trait_send() {
        fn assert_send<T: Send>() {}
        assert_send::<Refusal>();
    }

    #[test]
    fn trait_sync() {
        fn assert_sync<T: Sync>() {}
        assert_sync::<Refusal>();
    }

    #[test]
    fn trait_unpin() {
        fn assert_unpin<T: Unpin>() {}
        assert_unpin::<Refusal>();
    }
}
