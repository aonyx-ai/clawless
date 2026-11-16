//! The error of running an external program from a command

use kawauso_process::Invocation;
use kawauso_process::error::RunCommandError;
use thiserror::Error;

use crate::event::SendError;

/// The error returned when a command cannot run an external program
///
/// The variants separate what a caller does about the failure. A command that
/// was cancelled stops. A command whose program never ran reports the program.
/// A command whose output can no longer be reported has lost its presenter and
/// ends.
///
/// A program that ran and ended without success is no failure of the run. The
/// [`Execution`] then carries the status, and [`require_success`][success]
/// turns a status that the caller cannot accept into an error of its own.
///
/// A later release can add variants, and it can add fields to a variant. Match
/// with a wildcard arm, and bind the fields of a variant with `..`.
///
/// # Examples
///
/// ```
/// use clawless_core::event::event_channel;
/// use clawless_core::output::Output;
/// use clawless_core::process::{Invocation, Process, RunProcessError};
///
/// # #[tokio::main]
/// # async fn main() {
/// let (sender, _receiver) = event_channel();
/// let process = Process::builder().output(Output::new(sender)).build();
///
/// let error = process
///     .run(Invocation::new("clawless-no-such-program"))
///     .await
///     .expect_err("no program has this name");
///
/// assert!(matches!(error, RunProcessError::UnrunnableCommand { .. }));
/// # }
/// ```
///
/// [`Execution`]: kawauso_process::Execution
/// [success]: kawauso_process::Execution::require_success
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum RunProcessError {
    /// Cancellation stopped the program before it ended
    ///
    /// Clawless killed the program, so the run has no exit status and no
    /// complete output. A command that treats cancellation as an orderly end
    /// matches this variant and returns without an error.
    #[error("the command `{invocation}` was cancelled before it ended")]
    #[non_exhaustive]
    CancelledRun {
        /// The command that was cancelled
        invocation: Invocation,
    },

    /// The program could not be run
    ///
    /// This variant has four causes. No program answers to the name, or the
    /// working directory does not exist. The operating system refused to start
    /// the program, or the run started and its result could not be collected.
    /// The message is the one of the underlying failure, because this layer
    /// knows nothing that the run does not already state.
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
    #![allow(
        clippy::missing_panics_doc,
        reason = "an assertion in a test panics by design, so a `# Panics` section would repeat it"
    )]

    use super::*;

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
            source: SendError::new(crate::event::Event::Message("lost".to_owned())),
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
