//! Framework-controlled output for commands
//!
//! Commands use [`Output`] instead of `println!` to produce user-facing text. In return, every
//! command automatically gains `--quiet`, `--verbose`, and `--json` support without any extra
//! work. Call [`Output::print`] for normal messages, [`Output::verbose`] for detail that only
//! appears when the user asks for it, and [`Output::result`] for the primary data a command
//! produces.
//!
//! [`Verbosity`] and [`OutputMode`] are the two axes of configuration. Verbosity controls
//! *whether* a method writes anything; mode controls *where* and *how*. In text mode everything
//! goes to stdout, matching the familiar `println!` behavior. In JSON mode, messages redirect to
//! stderr so that stdout is reserved for machine-readable data — the same convention used by
//! `gh`, `kubectl`, and `jq`.

use std::fmt::Display;
use std::io::Write;
#[cfg(test)]
use std::sync::{Arc, Mutex};

use serde::Serialize;

pub use self::output_mode::OutputMode;
pub use self::verbosity::Verbosity;

mod output_mode;
mod verbosity;

/// Framework-controlled output for commands
///
/// `Output` is the primary way commands communicate with users. By routing all user-facing text
/// through `Output` instead of `println!`, commands automatically support `--quiet`, `--verbose`,
/// and `--json` flags with no additional effort from the command author.
///
/// Three methods cover every output need:
///
/// - [`print`] — informational messages (suppressed by `--quiet`).
/// - [`verbose`] — extra detail (shown only with `--verbose`).
/// - [`result`] — the primary data a command produces (always shown, serialized as JSON in
///   `--json` mode).
///
/// In text mode, all output goes to stdout. In JSON mode, messages redirect to stderr so that
/// stdout stays clean for machine-readable data, matching the convention used by `gh`, `kubectl`,
/// and `jq`.
///
/// # Examples
///
/// ```
/// use clawless::prelude::*;
///
/// let output = Output::new(Verbosity::Default, OutputMode::Text);
/// output.print("processing files");
/// output.verbose("scanning directory: /home/user/project");
/// ```
///
/// [`print`]: Output::print
/// [`verbose`]: Output::verbose
/// [`result`]: Output::result
#[derive(Clone, Debug)]
pub struct Output {
    verbosity: Verbosity,
    mode: OutputMode,
    message_writer: Writer,
    result_writer: Writer,
}

impl Output {
    /// Creates a new [`Output`] with stdout and stderr as writer targets
    ///
    /// In text mode, messages and results both go to stdout. In JSON mode, messages go to stderr
    /// (keeping stdout reserved for machine-readable data) and results go to stdout.
    ///
    /// # Examples
    ///
    /// ```
    /// use clawless::prelude::*;
    ///
    /// let output = Output::new(Verbosity::Default, OutputMode::Text);
    /// assert_eq!(output.verbosity(), Verbosity::Default);
    /// assert_eq!(output.mode(), OutputMode::Text);
    /// ```
    pub fn new(verbosity: Verbosity, mode: OutputMode) -> Self {
        let message_writer = match mode {
            OutputMode::Text => Writer::Stdout,
            OutputMode::Json => Writer::Stderr,
        };

        Self {
            verbosity,
            mode,
            message_writer,
            result_writer: Writer::Stdout,
        }
    }

    /// Writes a message followed by a newline
    ///
    /// The message is written to the message writer (stdout in text mode, stderr in JSON mode).
    /// This method is a no-op if verbosity is [`Verbosity::Quiet`].
    ///
    /// # Panics
    ///
    /// Panics if writing to the underlying writer fails (matching `println!` behavior).
    ///
    /// # Examples
    ///
    /// ```
    /// use clawless::prelude::*;
    ///
    /// let output = Output::new(Verbosity::Default, OutputMode::Text);
    /// output.print("processing 42 files");
    /// ```
    pub fn print(&self, message: impl Display) {
        match self.verbosity {
            Verbosity::Quiet => {}
            Verbosity::Default | Verbosity::Verbose => {
                self.message_writer.write_line(&message.to_string());
            }
        }
    }

    /// Writes a verbose message followed by a newline
    ///
    /// The message is written to the message writer (stdout in text mode, stderr in JSON mode).
    /// This method only produces output when verbosity is [`Verbosity::Verbose`].
    ///
    /// # Panics
    ///
    /// Panics if writing to the underlying writer fails (matching `println!` behavior).
    ///
    /// # Examples
    ///
    /// ```
    /// use clawless::prelude::*;
    ///
    /// let output = Output::new(Verbosity::Verbose, OutputMode::Text);
    /// output.verbose("scanning directory: /home/user/project");
    /// ```
    pub fn verbose(&self, message: impl Display) {
        match self.verbosity {
            Verbosity::Quiet | Verbosity::Default => {}
            Verbosity::Verbose => {
                self.message_writer.write_line(&message.to_string());
            }
        }
    }

    /// Writes a result value followed by a newline
    ///
    /// In text mode, the value is formatted via [`Display`] and written to stdout. In JSON mode,
    /// the value is serialized via [`Serialize`] as compact JSON and written to stdout. Results
    /// are always written regardless of verbosity — they are the primary output of a command.
    ///
    /// # Panics
    ///
    /// Panics if writing to the underlying writer fails (matching `println!` behavior) or if
    /// JSON serialization fails. Serialization failures for types that implement [`Serialize`]
    /// are programming errors, not runtime conditions that callers should handle.
    ///
    /// # Examples
    ///
    /// ```
    /// use clawless::prelude::*;
    ///
    /// let output = Output::new(Verbosity::Default, OutputMode::Text);
    /// output.result(&"hello");
    /// ```
    pub fn result<T: Display + Serialize>(&self, value: &T) {
        let line = match self.mode {
            OutputMode::Text => value.to_string(),
            OutputMode::Json => {
                serde_json::to_string(value).expect("failed to serialize result to JSON")
            }
        };
        self.result_writer.write_line(&line);
    }

    /// Returns the verbosity level
    ///
    /// # Examples
    ///
    /// ```
    /// use clawless::prelude::*;
    ///
    /// let output = Output::new(Verbosity::Verbose, OutputMode::Text);
    /// assert_eq!(output.verbosity(), Verbosity::Verbose);
    /// ```
    pub fn verbosity(&self) -> Verbosity {
        self.verbosity
    }

    /// Returns the output mode
    ///
    /// # Examples
    ///
    /// ```
    /// use clawless::prelude::*;
    ///
    /// let output = Output::new(Verbosity::Default, OutputMode::Json);
    /// assert_eq!(output.mode(), OutputMode::Json);
    /// ```
    pub fn mode(&self) -> OutputMode {
        self.mode
    }
}

#[derive(Clone, Debug)]
enum Writer {
    Stdout,
    Stderr,
    #[cfg(test)]
    Buffer(Arc<Mutex<Vec<u8>>>),
}

impl Writer {
    fn write_line(&self, message: &str) {
        match self {
            Writer::Stdout => {
                let mut handle = std::io::stdout().lock();
                writeln!(handle, "{message}").expect("should write to stdout");
            }
            Writer::Stderr => {
                let mut handle = std::io::stderr().lock();
                writeln!(handle, "{message}").expect("should write to stderr");
            }
            #[cfg(test)]
            Writer::Buffer(buffer) => {
                let mut guard = buffer.lock().expect("should lock buffer");
                writeln!(guard, "{message}").expect("should write to buffer");
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestOutputBuffers {
        messages: Arc<Mutex<Vec<u8>>>,
        results: Arc<Mutex<Vec<u8>>>,
    }

    impl TestOutputBuffers {
        fn messages(&self) -> String {
            let guard = self.messages.lock().expect("should lock messages buffer");
            String::from_utf8(guard.clone()).expect("should be valid UTF-8")
        }

        fn results(&self) -> String {
            let guard = self.results.lock().expect("should lock results buffer");
            String::from_utf8(guard.clone()).expect("should be valid UTF-8")
        }
    }

    impl Output {
        fn new_test(verbosity: Verbosity, mode: OutputMode) -> (Self, TestOutputBuffers) {
            let messages = Arc::new(Mutex::new(Vec::new()));
            let results = Arc::new(Mutex::new(Vec::new()));

            let output = Self {
                verbosity,
                mode,
                message_writer: Writer::Buffer(Arc::clone(&messages)),
                result_writer: Writer::Buffer(Arc::clone(&results)),
            };

            let buffers = TestOutputBuffers { messages, results };

            (output, buffers)
        }
    }

    #[test]
    fn mode_returns_configured_mode() {
        let (output, _buffers) = Output::new_test(Verbosity::Default, OutputMode::Json);

        assert_eq!(output.mode(), OutputMode::Json);
    }

    #[test]
    fn print_in_default_mode_writes_message() {
        let (output, buffers) = Output::new_test(Verbosity::Default, OutputMode::Text);

        output.print("hello world");

        assert_eq!(buffers.messages(), "hello world\n");
    }

    #[test]
    fn print_in_quiet_mode_is_noop() {
        let (output, buffers) = Output::new_test(Verbosity::Quiet, OutputMode::Text);

        output.print("hello world");

        assert_eq!(buffers.messages(), "");
    }

    #[test]
    fn print_in_verbose_mode_writes_message() {
        let (output, buffers) = Output::new_test(Verbosity::Verbose, OutputMode::Text);

        output.print("hello world");

        assert_eq!(buffers.messages(), "hello world\n");
    }

    #[test]
    fn result_in_json_mode_serializes_as_json() {
        let (output, buffers) = Output::new_test(Verbosity::Default, OutputMode::Json);

        output.result(&"hello world");

        assert_eq!(buffers.results(), "\"hello world\"\n");
    }

    #[test]
    fn result_in_quiet_mode_writes() {
        let (output, buffers) = Output::new_test(Verbosity::Quiet, OutputMode::Text);

        output.result(&"hello world");

        assert_eq!(buffers.results(), "hello world\n");
    }

    #[test]
    fn result_in_text_mode_uses_display() {
        let (output, buffers) = Output::new_test(Verbosity::Default, OutputMode::Text);

        output.result(&42);

        assert_eq!(buffers.results(), "42\n");
    }

    #[test]
    fn result_with_multiple_calls_produces_multiple_lines() {
        let (output, buffers) = Output::new_test(Verbosity::Default, OutputMode::Text);

        output.result(&"first");
        output.result(&"second");

        assert_eq!(buffers.results(), "first\nsecond\n");
    }

    #[test]
    fn trait_send() {
        fn assert_send<T: Send>() {}
        assert_send::<Output>();
    }

    #[test]
    fn trait_sync() {
        fn assert_sync<T: Sync>() {}
        assert_sync::<Output>();
    }

    #[test]
    fn trait_unpin() {
        fn assert_unpin<T: Unpin>() {}
        assert_unpin::<Output>();
    }

    #[test]
    fn verbose_in_default_mode_is_noop() {
        let (output, buffers) = Output::new_test(Verbosity::Default, OutputMode::Text);

        output.verbose("extra detail");

        assert_eq!(buffers.messages(), "");
    }

    #[test]
    fn verbose_in_quiet_mode_is_noop() {
        let (output, buffers) = Output::new_test(Verbosity::Quiet, OutputMode::Text);

        output.verbose("extra detail");

        assert_eq!(buffers.messages(), "");
    }

    #[test]
    fn verbose_in_verbose_mode_writes_message() {
        let (output, buffers) = Output::new_test(Verbosity::Verbose, OutputMode::Text);

        output.verbose("extra detail");

        assert_eq!(buffers.messages(), "extra detail\n");
    }

    #[test]
    fn verbosity_returns_configured_verbosity() {
        let (output, _buffers) = Output::new_test(Verbosity::Verbose, OutputMode::Text);

        assert_eq!(output.verbosity(), Verbosity::Verbose);
    }
}
