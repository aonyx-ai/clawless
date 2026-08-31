//! The error of running an external program from a command

use kawauso_process::Invocation;
use kawauso_process::error::RunCommandError;
use thiserror::Error;

use crate::event::SendError;

/// The error returned when a command cannot run an external program
///
/// The variants separate what a caller does about the failure. A command that
/// was cancelled stops, a command whose program never ran reports the program,
/// and a command whose output can no longer be reported has lost its presenter
/// and ends.
///
/// A program that ran and ended without success is no failure of the run. The
/// [`Execution`] then carries the status, and [`require_success`][success] turns
/// a status that the caller cannot accept into an error of its own.
///
/// A later release can add variants, and it can add fields to a variant. Match
/// with a wildcard arm, and bind the fields of a variant with `..`.
///
/// [`Execution`]: kawauso_process::Execution
/// [success]: kawauso_process::Execution::require_success
// r[impl process.error]
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum RunProcessError {
    /// Cancellation stopped the program before it ended
    ///
    /// Clawless killed the program, so the run has no exit status and no
    /// complete output. A command that treats cancellation as an orderly end
    /// matches this variant and returns without an error.
    // r[impl process.run.cancel.error]
    #[error("the command `{invocation}` was cancelled before it ended")]
    #[non_exhaustive]
    CancelledRun {
        /// The command that was cancelled
        invocation: Invocation,
    },

    /// The program could not be run
    ///
    /// No program answers to the name, the working directory does not exist,
    /// the operating system refused to start the program, or the run started
    /// and its result could not be collected. The message is the one of the
    /// underlying failure, because this layer knows nothing that the run does
    /// not already state.
    // r[impl process.run.error]
    #[error(transparent)]
    #[non_exhaustive]
    UnrunnableCommand {
        /// The cause of the failure
        source: RunCommandError,
    },

    /// The output of the program could not be reported
    ///
    /// The presenter stopped listening while the program ran, so the run has no
    /// consumer left. The program itself was killed with the run.
    // r[impl process.run.report.error]
    #[error("failed to report the output of the command `{invocation}`")]
    #[non_exhaustive]
    UnreportableOutput {
        /// The command whose output could not be reported
        invocation: Invocation,

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

    // r[verify process.error]
    #[test]
    fn display_with_a_cancelled_run_names_the_command() {
        let error = RunProcessError::CancelledRun {
            invocation: Invocation::new("git").arg("status"),
        };

        let message = error.to_string();

        assert_eq!(
            message,
            "the command `git status` was cancelled before it ended"
        );
    }

    #[test]
    fn display_with_unreportable_output_names_the_command() {
        let error = RunProcessError::UnreportableOutput {
            invocation: Invocation::new("git").arg("status"),
            source: SendError(crate::event::Event::Message("lost".to_owned())),
        };

        let message = error.to_string();

        assert_eq!(
            message,
            "failed to report the output of the command `git status`"
        );
    }

    #[test]
    fn trait_send() {
        fn assert_send<T: Send>() {}
        assert_send::<RunProcessError>();
    }

    #[test]
    fn trait_sync() {
        fn assert_sync<T: Sync>() {}
        assert_sync::<RunProcessError>();
    }

    #[test]
    fn trait_unpin() {
        fn assert_unpin<T: Unpin>() {}
        assert_unpin::<RunProcessError>();
    }
}
