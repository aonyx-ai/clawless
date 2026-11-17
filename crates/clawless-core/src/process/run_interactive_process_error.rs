//! The error of running a program that takes over the terminal

use std::process::ExitStatus;

use kawauso_process::Invocation;
use kawauso_process::error::RunCommandError;
use thiserror::Error;

use crate::event::SendError;
use crate::event::terminal::Refusal;

/// The error returned when a command cannot run a program that takes over the terminal
///
/// The variants separate what a caller does about the failure. A command that was cancelled
/// stops. A command whose program never started reports why: the presenter refused the terminal,
/// or the program could not be started. A command whose run can no longer be reported has lost
/// its presenter and ends.
///
/// A program that ran and ended without success is no failure of the run. The exit status then
/// tells the caller how the program ended.
///
/// A later release can add variants, and it can add fields to a variant. Match with a wildcard
/// arm, and bind the fields of a variant with `..`.
///
/// # Examples
///
/// ```
/// use clawless_core::event::terminal::Refusal;
/// use clawless_core::process::RunInteractiveProcessError;
///
/// fn advice(error: &RunInteractiveProcessError) -> &'static str {
///     match error {
///         RunInteractiveProcessError::CancelledRun { .. } => "",
///         RunInteractiveProcessError::UnlentTerminal {
///             refusal: Refusal::MissingTerminal,
///             ..
///         } => "run the command in a terminal",
///         _ => "report the error",
///     }
/// }
/// ```
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum RunInteractiveProcessError {
    /// Cancellation stopped the run
    ///
    /// Cancellation that arrives while the command waits for the terminal stops the run before
    /// the program starts. Cancellation that arrives while the program runs ends the program, so
    /// the run has no exit status. A command that treats cancellation as an orderly end matches
    /// this variant and returns without an error.
    #[error("the command `{invocation}` was cancelled before it ended")]
    #[non_exhaustive]
    CancelledRun {
        /// The command that was cancelled
        invocation: Box<Invocation>,
    },

    /// The request for the terminal could not reach the presenter
    ///
    /// The presenter stopped listening before the program started, so the program did not
    /// start.
    #[error("failed to ask the presenter for the terminal for the command `{invocation}`")]
    #[non_exhaustive]
    UndeliverableTerminalRequest {
        /// The command that needed the terminal
        invocation: Box<Invocation>,

        /// The cause of the failure
        source: SendError,
    },

    /// The presenter did not lend the terminal, so the program did not start
    ///
    /// The refusal names the reason, such as an application without a terminal.
    #[error("the terminal was not lent to the command `{invocation}`: {refusal}")]
    #[non_exhaustive]
    UnlentTerminal {
        /// The command that needed the terminal
        invocation: Box<Invocation>,

        /// The reason why the presenter did not lend the terminal
        refusal: Refusal,
    },

    /// The run could not be reported
    ///
    /// The program ended, and the presenter stopped listening before Clawless could report how.
    /// The status tells the caller how the program ended, so an edit that the user saved is not
    /// lost with the report.
    #[error("failed to report the run of the command `{invocation}`")]
    #[non_exhaustive]
    UnreportableRun {
        /// The command whose run could not be reported
        invocation: Box<Invocation>,

        /// The exit status of the program
        status: ExitStatus,

        /// The cause of the failure
        source: SendError,
    },

    /// The program could not be run
    ///
    /// No program answers to the name, the working directory does not exist, or the operating
    /// system refused to start the program. The terminal can also have refused its process group.
    /// Or the run started, and its end could not be waited for. The message is the one of the
    /// underlying failure, because this layer knows nothing that the run does not already state.
    #[error(transparent)]
    #[non_exhaustive]
    UnrunnableCommand {
        /// The cause of the failure
        source: RunCommandError,
    },
}

#[cfg(test)]
mod tests {
    #![allow(
        clippy::missing_panics_doc,
        reason = "an assertion in a test panics by design. A `# Panics` section on every test would repeat that and give the reader no information"
    )]

    #[cfg(unix)]
    use std::os::unix::process::ExitStatusExt;

    use super::*;
    use crate::event::Event;

    #[test]
    fn display_with_a_cancelled_run_names_the_command() {
        let error = RunInteractiveProcessError::CancelledRun {
            invocation: Box::new(Invocation::new("vi").arg("notes.md")),
        };

        let message = error.to_string();

        assert_eq!(
            message,
            "the command `vi notes.md` was cancelled before it ended"
        );
    }

    #[test]
    fn display_with_an_undeliverable_request_names_the_command() {
        let error = RunInteractiveProcessError::UndeliverableTerminalRequest {
            invocation: Box::new(Invocation::new("vi")),
            source: SendError(Event::Message("lost".to_owned())),
        };

        let message = error.to_string();

        assert_eq!(
            message,
            "failed to ask the presenter for the terminal for the command `vi`"
        );
    }

    #[test]
    fn display_with_an_unlent_terminal_names_the_command_and_the_refusal() {
        let error = RunInteractiveProcessError::UnlentTerminal {
            invocation: Box::new(Invocation::new("vi")),
            refusal: Refusal::MissingTerminal,
        };

        let message = error.to_string();

        assert_eq!(
            message,
            "the terminal was not lent to the command `vi`: the application has no terminal for the program"
        );
    }

    #[cfg(unix)]
    #[test]
    fn display_with_an_unreportable_run_names_the_command() {
        let error = RunInteractiveProcessError::UnreportableRun {
            invocation: Box::new(Invocation::new("vi")),
            status: ExitStatus::from_raw(0),
            source: SendError(Event::Message("lost".to_owned())),
        };

        let message = error.to_string();

        assert_eq!(message, "failed to report the run of the command `vi`");
    }

    #[test]
    fn trait_send() {
        fn assert_send<T: Send>() {}
        assert_send::<RunInteractiveProcessError>();
    }

    #[test]
    fn trait_sync() {
        fn assert_sync<T: Sync>() {}
        assert_sync::<RunInteractiveProcessError>();
    }

    #[test]
    fn trait_unpin() {
        fn assert_unpin<T: Unpin>() {}
        assert_unpin::<RunInteractiveProcessError>();
    }
}
