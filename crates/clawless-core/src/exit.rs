//! The way in which a command ends the process
//!
//! A command that returns `()` ends the process with the exit code 0, and a command that returns
//! an error ends it with the exit code 1. A command that needs a different ending returns an
//! [`Exit`], which carries the exit code and an optional text that the command rendered itself.
//!
//! An [`Exit`] is a plain value. The runner of an application returns it, and the `main`
//! function of the application ends the process with it.

use std::fmt::Debug;
use std::process::ExitCode;

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

#[cfg(test)]
mod tests {
    // An assertion in a test panics by design. A `# Panics` section on every test
    // would repeat that and give the reader no information.
    #![allow(clippy::missing_panics_doc)]

    use super::*;

    /// An error whose debug representation is known
    #[derive(Debug)]
    struct MissingFile;

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
