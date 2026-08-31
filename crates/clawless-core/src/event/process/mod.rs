//! Lifecycle events for the run of an external program
//!
//! A command that runs an external program reports what the program does as it
//! does it. This module defines [`ProcessEvent`], the event that carries that
//! report, and the types that describe a run: [`RunId`] identifies it, and
//! [`Outcome`] says how it ended.
//!
//! A run produces one [`Started`] event, one [`Line`] event per line that the
//! program writes to either of its output streams, and one [`Finished`] event.
//! A consumer can therefore show the program while it runs, keep the last few
//! lines of its output, and drop the whole group when the program ends.
//!
//! [`Finished`]: ProcessEvent::Finished
//! [`Line`]: ProcessEvent::Line
//! [`Started`]: ProcessEvent::Started

use std::fmt::{self, Display};
use std::time::Duration;

use kawauso_process::run::Line;
use kawauso_process::{Invocation, ProcessId};

pub use self::outcome::Outcome;
pub use self::run_id::RunId;

/// The way in which a run ended
mod outcome;
/// The identity that ties the events of one run together
mod run_id;

/// One step in the run of an external program
///
/// Clawless sends these events while a program runs, so that a presenter can
/// render the output of the program as it arrives instead of after the program
/// ends. A build that takes two minutes therefore shows two minutes of
/// progress.
///
/// Every run sends [`Started`] first and [`Finished`] last, with one [`Line`]
/// in between for each line that the program wrote. Two things end a run
/// without a [`Finished`]: a consumer that stopped listening, which cannot be
/// told anything, and a caller that dropped the run instead of cancelling it,
/// which leaves no code to send the event.
///
/// Each variant carries the [`RunId`] of its run. Two programs that run at
/// the same time interleave their lines in the channel, and the identity is
/// what separates them again. [`Started`] and [`Finished`] also carry the
/// command itself, so that a presenter which keeps no state can name the
/// program it is reporting on.
///
/// [`Started`] carries the [`ProcessId`] of the program as well. That value is
/// the name the operating system knows the program by, which is what an
/// operator needs to find it in a process list.
///
/// [`ProcessId`]: crate::process::ProcessId
///
/// # Examples
///
/// ```
/// use clawless_core::event::process::{ProcessEvent, RunId};
/// use clawless_core::process::Invocation;
///
/// let event = ProcessEvent::Started {
///     id: RunId::next(),
///     invocation: Invocation::new("git").arg("status"),
///     process_id: None,
/// };
///
/// assert_eq!(event.to_string(), "$ git status");
/// ```
///
/// [`Finished`]: ProcessEvent::Finished
/// [`Line`]: ProcessEvent::Line
/// [`Started`]: ProcessEvent::Started
// r[impl process.event.lifecycle]
#[derive(Clone, Eq, PartialEq, Debug)]
pub enum ProcessEvent {
    /// The program started
    ///
    /// The event names the command, which is what a consumer shows while the
    /// program runs.
    // r[impl process.event.started]
    Started {
        /// The run that started
        id: RunId,

        /// The command that the run starts
        invocation: Invocation,

        /// The identifier that the operating system gave the program
        ///
        /// `None` when the operating system did not report one. A consumer
        /// that names the program to a tool of the platform, or that writes
        /// the identifier to a log, reads it here.
        // r[impl process.event.started.identifier]
        process_id: Option<ProcessId>,
    },

    /// The program wrote one line
    ///
    /// The line carries the stream that produced it, so that a consumer can
    /// show a diagnostic apart from a result, or drop one of the two. It holds
    /// no characters that end a line.
    // r[impl process.event.line]
    Line {
        /// The run that wrote the line
        id: RunId,

        /// The line that the program wrote
        line: Line,
    },

    /// The program ended
    ///
    /// No further event carries this identity. A consumer that shows running
    /// programs removes the run here.
    // r[impl process.event.finished]
    Finished {
        /// The run that ended
        id: RunId,

        /// The command that ran
        invocation: Invocation,

        /// The way in which the run ended
        outcome: Outcome,

        /// The time from the start of the program to the end of the run
        duration: Duration,
    },
}

impl ProcessEvent {
    /// Returns the run that the event belongs to
    ///
    /// A consumer that groups the events of one program reads the identity
    /// here instead of matching on the variant.
    ///
    /// # Examples
    ///
    /// ```
    /// use clawless_core::event::process::{ProcessEvent, RunId};
    /// use clawless_core::process::Invocation;
    ///
    /// let id = RunId::next();
    /// let event = ProcessEvent::Started {
    ///     id,
    ///     invocation: Invocation::new("git"),
    ///     process_id: None,
    /// };
    ///
    /// assert_eq!(event.id(), id);
    /// ```
    // r[impl process.event.correlation.accessor]
    pub fn id(&self) -> RunId {
        match self {
            Self::Started { id, .. } => *id,
            Self::Line { id, .. } => *id,
            Self::Finished { id, .. } => *id,
        }
    }
}

/// Renders the event as one line of a transcript
///
/// The start of a run reads as a shell prompt, a line of output reads as the
/// program wrote it, and the end of a run states what became of the command.
/// A consumer that wants another shape matches on the variant and builds its
/// own.
///
/// The text never names the stream that produced a line. A presenter that
/// separates the two writes them to different places instead of labelling them,
/// which keeps the output of the program readable.
// r[impl process.event.display]
impl Display for ProcessEvent {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Started { invocation, .. } => write!(formatter, "$ {invocation}"),
            Self::Line { line, .. } => Display::fmt(line, formatter),
            Self::Finished {
                invocation,
                outcome,
                ..
            } => write!(formatter, "{invocation} {outcome}"),
        }
    }
}

#[cfg(test)]
mod tests {
    // An assertion in a test panics by design. A `# Panics` section on every test
    // would repeat that and give the reader no information.
    #![allow(clippy::missing_panics_doc)]

    use kawauso_process::run::Stream;

    use super::*;

    #[test]
    fn display_with_finished_names_the_command_and_the_outcome() {
        let event = ProcessEvent::Finished {
            id: RunId::next(),
            invocation: Invocation::new("git").arg("status"),
            outcome: Outcome::Cancelled,
            duration: Duration::from_secs(1),
        };

        let text = event.to_string();

        assert_eq!(text, "git status was cancelled before it ended");
    }

    #[test]
    fn display_with_line_returns_the_line_alone() {
        let event = ProcessEvent::Line {
            id: RunId::next(),
            line: Line::new(Stream::StandardError, "no such file"),
        };

        let text = event.to_string();

        assert_eq!(text, "no such file");
    }

    // r[verify process.event.display]
    #[test]
    fn display_with_started_reads_as_a_prompt() {
        let event = ProcessEvent::Started {
            id: RunId::next(),
            invocation: Invocation::new("git").arg("status"),
            process_id: None,
        };

        let text = event.to_string();

        assert_eq!(text, "$ git status");
    }

    #[test]
    fn id_with_finished_returns_the_run() {
        let id = RunId::next();
        let event = ProcessEvent::Finished {
            id,
            invocation: Invocation::new("git"),
            outcome: Outcome::Incomplete,
            duration: Duration::ZERO,
        };

        let found = event.id();

        assert_eq!(found, id);
    }

    // r[verify process.event.correlation.accessor]
    #[test]
    fn id_with_line_returns_the_run() {
        let id = RunId::next();
        let event = ProcessEvent::Line {
            id,
            line: Line::new(Stream::StandardOutput, "hello"),
        };

        let found = event.id();

        assert_eq!(found, id);
    }

    #[test]
    fn id_with_started_returns_the_run() {
        let id = RunId::next();
        let event = ProcessEvent::Started {
            id,
            invocation: Invocation::new("git"),
            process_id: None,
        };

        let found = event.id();

        assert_eq!(found, id);
    }

    #[test]
    fn trait_send() {
        fn assert_send<T: Send>() {}
        assert_send::<ProcessEvent>();
    }

    #[test]
    fn trait_sync() {
        fn assert_sync<T: Sync>() {}
        assert_sync::<ProcessEvent>();
    }

    #[test]
    fn trait_unpin() {
        fn assert_unpin<T: Unpin>() {}
        assert_unpin::<ProcessEvent>();
    }
}
