//! The way in which a command ends the process
//!
//! A command that returns `()` ends the process with the exit code 0, and a command that returns
//! an error ends it with the exit code 1. A command that needs a different ending returns an
//! [`Exit`], which carries the exit code and an optional text that the command rendered itself.
//!
//! [`Exit`] implements [`Termination`], so the `main` function of an application returns it, and
//! the process writes the text and exits with the code only after `main` has returned.
//!
//! [`Termination`]: std::process::Termination

use std::fmt::Debug;
use std::io::Write;
use std::process::{ExitCode, Termination};

use bon::Builder;
use getset::CopyGetters;

/// The exit code and the text with which a command ends the process
///
/// A wrapper, such as a script or a CI job, reads the exit code of a program to decide what to do
/// next. A command therefore needs to choose the code. The chosen code is not always a failure: a
/// check that finds a problem ends as planned, and it can still exit with a code other than 0 to
/// tell the wrapper what it found.
///
/// The text is what the command wants the user to read last, such as a diagnostic that the
/// application rendered itself. Clawless does not render the text. It writes the text to the
/// standard error once, exactly as it is, and adds no prefix and no line break. End the text with
/// a line break unless the text must not have one.
///
/// Clawless writes the text after it has rendered all the output that the command sent, and the
/// process exits after the text. The text therefore appears last on the standard error. The
/// standard output carries only the result of the command, so JSON output stays valid. The
/// verbosity does not apply to the text, so `--quiet` does not hide it.
///
/// An error that a command returns always ends the process with the exit code 1. A command that
/// needs another code for a failure turns the error into an `Exit` itself, and returns the `Exit`
/// in `Ok`.
///
/// Clawless itself uses these exit codes. A command can still use them, because Clawless reserves
/// none of them.
///
/// | Code | Meaning                                                                    |
/// | ---- | -------------------------------------------------------------------------- |
/// | 0    | The command returned `()`                                                  |
/// | 1    | The command returned an error                                              |
/// | 2    | The command line has an error, which the argument parser reports           |
/// | 101  | The command or Clawless panicked, for example on a closed standard output |
/// | 130  | The user pressed Ctrl+C for the second time                                |
///
/// # Examples
///
/// ```
/// use std::process::ExitCode;
///
/// use clawless_core::exit::Exit;
///
/// let exit = Exit::builder()
///     .code(ExitCode::from(3))
///     .text("error: 0.7.0 is not a stable release\n")
///     .build();
///
/// assert_eq!(exit.code(), ExitCode::from(3));
/// assert_eq!(exit.text(), Some("error: 0.7.0 is not a stable release\n"));
/// ```
///
/// A command that only chooses the code converts an [`ExitCode`] into an `Exit` without text:
///
/// ```
/// use std::process::ExitCode;
///
/// use clawless_core::exit::Exit;
///
/// let exit = Exit::from(ExitCode::from(3));
///
/// assert_eq!(exit.text(), None);
/// ```
///
/// [`ExitCode`]: std::process::ExitCode
#[derive(Clone, PartialEq, Debug, Builder, CopyGetters)]
#[must_use = "an `Exit` ends the process only when `main` returns it"]
pub struct Exit {
    /// The exit code of the process
    #[builder(into)]
    #[getset(get_copy = "pub")]
    code: ExitCode,

    /// The text that the process writes to the standard error before it exits
    #[builder(into)]
    text: Option<String>,
}

impl Exit {
    /// Returns the exit of a command that failed with the given error
    ///
    /// The exit code is 1. The text is `Error: ` and the debug representation of the error, the
    /// same as when the `main` function of a Rust program returns the error.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::process::ExitCode;
    ///
    /// use clawless_core::exit::Exit;
    ///
    /// #[derive(Debug)]
    /// struct MissingFile;
    ///
    /// let exit = Exit::from_error(&MissingFile);
    ///
    /// assert_eq!(exit.code(), ExitCode::FAILURE);
    /// assert_eq!(exit.text(), Some("Error: MissingFile\n"));
    /// ```
    pub fn from_error(error: &impl Debug) -> Self {
        Self {
            code: ExitCode::FAILURE,
            text: Some(format!("Error: {error:?}\n")),
        }
    }

    /// Returns the text that the process writes to the standard error before it exits
    ///
    /// # Examples
    ///
    /// ```
    /// use std::process::ExitCode;
    ///
    /// use clawless_core::exit::Exit;
    ///
    /// let exit = Exit::builder()
    ///     .code(ExitCode::from(3))
    ///     .text("error: the check failed\n")
    ///     .build();
    ///
    /// assert_eq!(exit.text(), Some("error: the check failed\n"));
    /// ```
    pub fn text(&self) -> Option<&str> {
        self.text.as_deref()
    }

    /// Flushes the output, writes the text to the display, and returns the exit code
    ///
    /// `output` is the standard output, and `display` is the standard error. A command can leave
    /// a line on the standard output without a line break, and the process would flush it only
    /// at exit, after the text. Flushing first keeps the text last.
    ///
    /// A text that cannot be written is lost, and the exit code stays the same. The code is what
    /// a wrapper reads, and a display that is gone has no place left to report the failure.
    fn report_to(self, output: &mut impl Write, display: &mut impl Write) -> ExitCode {
        let Self { code, text } = self;

        drop(output.flush());

        if let Some(text) = text {
            drop(display.write_all(text.as_bytes()));
        }

        code
    }
}

/// Returns the exit of a command that ended without a choice of its own
///
/// The exit code is 0, and the process writes no text.
impl From<()> for Exit {
    fn from((): ()) -> Self {
        Self {
            code: ExitCode::SUCCESS,
            text: None,
        }
    }
}

/// Returns the exit with the given code and without text
impl From<ExitCode> for Exit {
    fn from(code: ExitCode) -> Self {
        Self { code, text: None }
    }
}

/// Flushes the standard output, writes the text to the standard error, and returns the exit code
///
/// The trait is what lets the `main` function of an application return an `Exit`. The process
/// calls it after `main` has returned, which is after every event of the command is rendered.
impl Termination for Exit {
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

    /// An error whose debug representation is known
    #[derive(Debug)]
    struct MissingFile;

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

    #[test]
    fn builder_without_text_has_no_text() {
        let exit = Exit::builder().code(ExitCode::from(3)).build();

        assert_eq!(exit.text(), None);
    }

    #[test]
    fn from_error_returns_the_debug_representation_with_a_prefix() {
        let error = MissingFile;

        let exit = Exit::from_error(&error);

        assert_eq!(exit.text(), Some("Error: MissingFile\n"));
    }

    #[test]
    fn from_error_returns_the_failure_code() {
        let exit = Exit::from_error(&"broken");

        assert_eq!(exit.code(), ExitCode::FAILURE);
    }

    #[test]
    fn from_exit_code_returns_the_code_without_text() {
        let exit = Exit::from(ExitCode::from(3));

        assert_eq!(exit, Exit::builder().code(ExitCode::from(3)).build());
    }

    #[test]
    fn from_unit_returns_success_without_text() {
        let exit = Exit::from(());

        assert_eq!(exit, Exit::builder().code(ExitCode::SUCCESS).build());
    }

    #[test]
    fn report_to_a_closed_display_returns_the_code() {
        let exit = Exit::builder()
            .code(ExitCode::from(3))
            .text("lost\n")
            .build();

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
        let exit = Exit::builder()
            .code(ExitCode::from(3))
            .text("error\n")
            .build();

        exit.report_to(&mut output, &mut display);

        assert_eq!(*transcript.borrow(), b"partial lineerror\n");
    }

    #[test]
    fn report_to_with_text_returns_the_code() {
        let exit = Exit::builder()
            .code(ExitCode::from(3))
            .text("error\n")
            .build();

        let code = exit.report_to(&mut Vec::new(), &mut Vec::new());

        assert_eq!(code, ExitCode::from(3));
    }

    #[test]
    fn report_to_with_text_writes_the_text_as_it_is() {
        let exit = Exit::builder()
            .code(ExitCode::from(3))
            .text("error: no line break")
            .build();
        let mut display = Vec::new();

        exit.report_to(&mut Vec::new(), &mut display);

        assert_eq!(display, b"error: no line break");
    }

    #[test]
    fn report_to_without_text_writes_nothing() {
        let exit = Exit::from(());
        let mut display = Vec::new();

        exit.report_to(&mut Vec::new(), &mut display);

        assert!(display.is_empty());
    }

    #[test]
    fn trait_send() {
        fn assert_send<T: Send>() {}
        assert_send::<Exit>();
    }

    #[test]
    fn trait_sync() {
        fn assert_sync<T: Sync>() {}
        assert_sync::<Exit>();
    }

    #[test]
    fn trait_unpin() {
        fn assert_unpin<T: Unpin>() {}
        assert_unpin::<Exit>();
    }
}
