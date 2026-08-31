//! The way in which the run of an external program ended

use std::fmt::{self, Display};
use std::process::ExitStatus;

/// The way in which the run of an external program ended
///
/// Every run that starts also ends, and a consumer that shows a running program
/// needs to know how. The variants separate the three ends that a consumer
/// treats differently: the program ran to its end, cancellation stopped it, or
/// Clawless could not collect a result for it.
///
/// A program that ended without success is [`Exited`], not a failure. The check
/// mode of a formatter ends with a non-zero code when it finds a file to
/// format, and that is an answer and not a fault.
///
/// # Examples
///
/// ```
/// use clawless_core::event::process::Outcome;
///
/// assert_eq!(Outcome::Cancelled.to_string(), "was cancelled before it ended");
/// ```
///
/// [`Exited`]: Outcome::Exited
// r[impl process.event.outcome]
#[derive(Clone, Eq, PartialEq, Debug)]
pub enum Outcome {
    /// Cancellation stopped the program before it ended
    ///
    /// Clawless killed the program, so it has no exit status. The output that
    /// reached the consumer before this point is everything the program wrote.
    // r[impl process.event.outcome.cancelled]
    Cancelled,

    /// The program ran to its end
    ///
    /// The status says how it ended. A program that a signal stopped carries no
    /// exit code, which [`ExitStatus::code`] reports as `None`.
    // r[impl process.event.outcome.exited]
    Exited(ExitStatus),

    /// The run started and produced no result
    ///
    /// A stream of the program could not be read, the end of the program could
    /// not be waited for, or the events of the run could no longer be reported.
    /// The program itself may well have done its work.
    // r[impl process.event.outcome.incomplete]
    Incomplete,
}

/// States how the run ended, as the end of a sentence about the command
///
/// The text follows the command in [`ProcessEvent`], which is why it starts
/// with a verb and not with a subject.
///
/// [`ProcessEvent`]: super::ProcessEvent
impl Display for Outcome {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Cancelled => formatter.write_str("was cancelled before it ended"),
            Self::Exited(status) => match status.code() {
                Some(code) => write!(formatter, "exited with code {code}"),
                None => formatter.write_str("ended without an exit code"),
            },
            Self::Incomplete => formatter.write_str("ended without a result"),
        }
    }
}

#[cfg(test)]
mod tests {
    // An assertion in a test panics by design. A `# Panics` section on every test
    // would repeat that and give the reader no information.
    #![allow(clippy::missing_panics_doc)]

    use super::*;

    #[test]
    fn display_with_cancelled_names_the_cancellation() {
        let outcome = Outcome::Cancelled;

        let text = outcome.to_string();

        assert_eq!(text, "was cancelled before it ended");
    }

    // r[verify process.event.outcome.incomplete]
    #[test]
    fn display_with_incomplete_names_the_missing_result() {
        let outcome = Outcome::Incomplete;

        let text = outcome.to_string();

        assert_eq!(text, "ended without a result");
    }

    #[test]
    fn trait_send() {
        fn assert_send<T: Send>() {}
        assert_send::<Outcome>();
    }

    #[test]
    fn trait_sync() {
        fn assert_sync<T: Sync>() {}
        assert_sync::<Outcome>();
    }

    #[test]
    fn trait_unpin() {
        fn assert_unpin<T: Unpin>() {}
        assert_unpin::<Outcome>();
    }
}
