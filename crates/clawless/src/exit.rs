//! The way in which a command ends the process
//!
//! A command returns an [`Exit`] to choose the exit code and the text of the process. The
//! `main` function that `main!()` generates returns a [`ProcessExit`], which writes that text and
//! ends the process with that code after `main` has returned.

use std::io::Write;
use std::process::{ExitCode, Termination};

pub use clawless_core::exit::Exit;

/// The exit with which the `main` function of a Clawless application ends the process
///
/// `main!()` generates a `main` function that returns a `ProcessExit`. The process calls
/// [`Termination::report`] after `main` has returned, which is after the runner has rendered
/// every event of the command. `report` then flushes the standard output, writes the text of the
/// [`Exit`] to the standard error, and returns the exit code.
///
/// The flush comes first because a command can leave a line on the standard output without a line
/// break. The process would flush that line only at exit, after the text. Flushing first keeps the
/// text last.
///
/// A text that cannot be written is lost, and the exit code stays the same. The code is what a
/// wrapper reads, and a standard error that is gone has no place left to report the failure.
///
/// A `main` function that calls a runner itself, instead of through `main!()`, returns a
/// `ProcessExit` in the same way.
///
/// # Examples
///
/// ```rust,ignore
/// use clawless::exit::ProcessExit;
/// use clawless::runner::CommandRunner;
///
/// fn main() -> ProcessExit {
///     let matches = build_command().get_matches();
///
///     ProcessExit::from(CommandRunner::run(matches, |_matches, _context| async { Ok(()) }))
/// }
/// ```
///
/// [`Termination::report`]: std::process::Termination::report
#[derive(Clone, PartialEq, Debug)]
#[must_use = "a `ProcessExit` ends the process only when `main` returns it"]
pub struct ProcessExit(Exit);

impl ProcessExit {
    /// Flushes the output, writes the text to the display, and returns the exit code
    ///
    /// `output` is the standard output, and `display` is the standard error.
    fn report_to(self, output: &mut impl Write, display: &mut impl Write) -> ExitCode {
        let Self(exit) = self;

        drop(output.flush());

        if let Some(text) = exit.text() {
            drop(display.write_all(text.as_bytes()));
        }

        exit.code()
    }
}

/// Returns the process exit that ends the process with the given exit
impl From<Exit> for ProcessExit {
    fn from(exit: Exit) -> Self {
        Self(exit)
    }
}

/// Flushes the standard output, writes the text to the standard error, and returns the exit code
impl Termination for ProcessExit {
    fn report(self) -> ExitCode {
        self.report_to(&mut std::io::stdout(), &mut std::io::stderr())
    }
}

#[cfg(test)]
mod tests {
    // An assertion in a test panics by design. A `# Panics` section on every test
    // would repeat that and give the reader no information.
    #![allow(clippy::missing_panics_doc)]

    use std::cell::RefCell;
    use std::io;
    use std::rc::Rc;

    use super::*;

    /// An output that holds its bytes until a flush, like a standard output with a partial line
    #[derive(Debug)]
    struct BufferedOutput {
        /// The bytes that wait for a flush
        pending: Vec<u8>,
        /// Where a flush puts the bytes
        transcript: Rc<RefCell<Vec<u8>>>,
    }

    impl Write for BufferedOutput {
        fn write(&mut self, buffer: &[u8]) -> io::Result<usize> {
            self.pending.extend_from_slice(buffer);
            Ok(buffer.len())
        }

        fn flush(&mut self) -> io::Result<()> {
            self.transcript.borrow_mut().append(&mut self.pending);
            Ok(())
        }
    }

    /// A display that writes straight to a transcript that an output shares
    #[derive(Debug)]
    struct SharedDisplay(Rc<RefCell<Vec<u8>>>);

    impl Write for SharedDisplay {
        fn write(&mut self, buffer: &[u8]) -> io::Result<usize> {
            self.0.borrow_mut().extend_from_slice(buffer);
            Ok(buffer.len())
        }

        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }

    /// A display that fails every write, like a standard error that the reader has closed
    #[derive(Debug)]
    struct ClosedDisplay;

    impl Write for ClosedDisplay {
        fn write(&mut self, _buffer: &[u8]) -> io::Result<usize> {
            Err(io::Error::from(io::ErrorKind::BrokenPipe))
        }

        fn flush(&mut self) -> io::Result<()> {
            Err(io::Error::from(io::ErrorKind::BrokenPipe))
        }
    }

    /// Returns a process exit with the exit code 3 and the given text
    fn with_text(text: &str) -> ProcessExit {
        ProcessExit::from(Exit::builder().code(ExitCode::from(3)).text(text).build())
    }

    #[test]
    fn report_to_a_closed_display_returns_the_code() {
        let exit = with_text("lost\n");

        let code = exit.report_to(&mut Vec::new(), &mut ClosedDisplay);

        assert_eq!(code, ExitCode::from(3));
    }

    #[test]
    fn report_to_with_text_flushes_the_output_before_the_text() {
        let transcript = Rc::new(RefCell::new(Vec::new()));
        let mut output = BufferedOutput {
            pending: b"partial line".to_vec(),
            transcript: Rc::clone(&transcript),
        };
        let mut display = SharedDisplay(Rc::clone(&transcript));
        let exit = with_text("error\n");

        let _code = exit.report_to(&mut output, &mut display);

        assert_eq!(*transcript.borrow(), b"partial lineerror\n");
    }

    #[test]
    fn report_to_with_text_returns_the_code() {
        let exit = with_text("error\n");

        let code = exit.report_to(&mut Vec::new(), &mut Vec::new());

        assert_eq!(code, ExitCode::from(3));
    }

    #[test]
    fn report_to_with_text_writes_the_text_as_it_is() {
        let exit = with_text("error: no line break");
        let mut display = Vec::new();

        let _code = exit.report_to(&mut Vec::new(), &mut display);

        assert_eq!(display, b"error: no line break");
    }

    #[test]
    fn report_to_without_text_writes_nothing() {
        let exit = ProcessExit::from(Exit::from(()));
        let mut display = Vec::new();

        let _code = exit.report_to(&mut Vec::new(), &mut display);

        assert!(display.is_empty());
    }

    #[test]
    fn trait_send() {
        fn assert_send<T: Send>() {}
        assert_send::<ProcessExit>();
    }

    #[test]
    fn trait_sync() {
        fn assert_sync<T: Sync>() {}
        assert_sync::<ProcessExit>();
    }

    #[test]
    fn trait_unpin() {
        fn assert_unpin<T: Unpin>() {}
        assert_unpin::<ProcessExit>();
    }
}
