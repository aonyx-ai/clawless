//! Framework-controlled output for commands
//!
//! Commands use [`Output`] instead of `println!` to produce user-facing text. In return, every
//! command automatically gains `--quiet`, `--verbose`, and `--json` support without any extra
//! work. Call [`Output::message`] for normal messages, [`Output::detail`] for detail that only
//! appears when the user asks for it, and [`Output::artifact`] for the primary data a command
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

use clap::{Arg, ArgAction, ArgMatches};
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
/// - [`message`] — informational messages (suppressed by `--quiet`).
/// - [`detail`] — extra detail (shown only with `--verbose`).
/// - [`artifact`] — the primary data a command produces (always shown, serialized as JSON in
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
/// output.message("processing files");
/// output.detail("scanning directory: /home/user/project");
/// ```
///
/// [`message`]: Output::message
/// [`detail`]: Output::detail
/// [`artifact`]: Output::artifact
#[derive(Clone, Debug)]
pub struct Output {
    verbosity: Verbosity,
    mode: OutputMode,
    message_writer: Writer,
    artifact_writer: Writer,
}

impl Output {
    /// Creates a new [`Output`] with stdout and stderr as writer targets
    ///
    /// In text mode, messages and artifacts both go to stdout. In JSON mode, messages go to stderr
    /// (keeping stdout reserved for machine-readable data) and artifacts go to stdout.
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
            artifact_writer: Writer::Stdout,
        }
    }

    /// Writes an informational message followed by a newline
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
    /// output.message("processing 42 files");
    /// ```
    pub fn message(&self, message: impl Display) {
        match self.verbosity {
            Verbosity::Quiet => {}
            Verbosity::Default | Verbosity::Verbose => {
                self.message_writer.write_line(&message.to_string());
            }
        }
    }

    /// Writes a supplementary detail followed by a newline
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
    /// output.detail("scanning directory: /home/user/project");
    /// ```
    pub fn detail(&self, message: impl Display) {
        match self.verbosity {
            Verbosity::Quiet | Verbosity::Default => {}
            Verbosity::Verbose => {
                self.message_writer.write_line(&message.to_string());
            }
        }
    }

    /// Writes an artifact value followed by a newline
    ///
    /// In text mode, the value is formatted via [`Display`] and written to stdout. In JSON mode,
    /// the value is serialized via [`Serialize`] as compact JSON and written to stdout. Artifacts
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
    /// output.artifact(&"hello");
    /// ```
    pub fn artifact<T: Display + Serialize>(&self, value: &T) {
        let line = match self.mode {
            OutputMode::Text => value.to_string(),
            OutputMode::Json => {
                serde_json::to_string(value).expect("failed to serialize artifact to JSON")
            }
        };
        self.artifact_writer.write_line(&line);
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

    /// Attaches `--quiet`, `--verbose`, and `--json` as global flags to a [`clap::Command`]
    ///
    /// The `--quiet` and `--verbose` flags conflict with each other. All three flags are global,
    /// meaning they can appear before or after the subcommand name.
    ///
    /// This method is called by the `main!()` macro expansion. Command authors do not call it
    /// directly.
    ///
    /// # Examples
    ///
    /// ```
    /// let command = clap::Command::new("test");
    /// let command = clawless::output::Output::augment_command(command);
    /// ```
    pub fn augment_command(command: clap::Command) -> clap::Command {
        command
            .arg(
                Arg::new("quiet")
                    .short('q')
                    .long("quiet")
                    .help("Suppress informational messages")
                    .global(true)
                    .action(ArgAction::SetTrue)
                    .conflicts_with("verbose"),
            )
            .arg(
                Arg::new("verbose")
                    .short('v')
                    .long("verbose")
                    .help("Show additional detail")
                    .global(true)
                    .action(ArgAction::SetTrue)
                    .conflicts_with("quiet"),
            )
            .arg(
                Arg::new("json")
                    .long("json")
                    .help("Output results as JSON")
                    .global(true)
                    .action(ArgAction::SetTrue),
            )
    }

    /// Constructs an [`Output`] from parsed CLI flags
    ///
    /// Reads the `--quiet`, `--verbose`, and `--json` flags from the given [`ArgMatches`] and
    /// returns an [`Output`] configured accordingly. The command must have been augmented with
    /// [`augment_command`] before parsing.
    ///
    /// This method is called by the `main!()` macro expansion. Command authors do not call it
    /// directly.
    ///
    /// # Examples
    ///
    /// ```
    /// let command = clawless::output::Output::augment_command(
    ///     clap::Command::new("test"),
    /// );
    /// let matches = command.get_matches_from(vec!["test", "--quiet"]);
    /// let output = clawless::output::Output::from_arg_matches(&matches);
    /// assert_eq!(output.verbosity(), clawless::output::Verbosity::Quiet);
    /// ```
    ///
    /// [`augment_command`]: Output::augment_command
    pub fn from_arg_matches(matches: &ArgMatches) -> Self {
        let quiet = matches.get_flag("quiet");
        let verbose = matches.get_flag("verbose");
        let json = matches.get_flag("json");

        let verbosity = match (quiet, verbose) {
            (true, _) => Verbosity::Quiet,
            (_, true) => Verbosity::Verbose,
            (false, false) => Verbosity::Default,
        };

        let mode = match json {
            true => OutputMode::Json,
            false => OutputMode::Text,
        };

        Self::new(verbosity, mode)
    }
}

impl Default for Output {
    fn default() -> Self {
        Self::new(Verbosity::default(), OutputMode::default())
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
        artifacts: Arc<Mutex<Vec<u8>>>,
    }

    impl TestOutputBuffers {
        fn messages(&self) -> String {
            let guard = self.messages.lock().expect("should lock messages buffer");
            String::from_utf8(guard.clone()).expect("should be valid UTF-8")
        }

        fn artifacts(&self) -> String {
            let guard = self.artifacts.lock().expect("should lock artifacts buffer");
            String::from_utf8(guard.clone()).expect("should be valid UTF-8")
        }
    }

    impl Output {
        fn new_test(verbosity: Verbosity, mode: OutputMode) -> (Self, TestOutputBuffers) {
            let messages = Arc::new(Mutex::new(Vec::new()));
            let artifacts = Arc::new(Mutex::new(Vec::new()));

            let output = Self {
                verbosity,
                mode,
                message_writer: Writer::Buffer(Arc::clone(&messages)),
                artifact_writer: Writer::Buffer(Arc::clone(&artifacts)),
            };

            let buffers = TestOutputBuffers {
                messages,
                artifacts,
            };

            (output, buffers)
        }
    }

    fn test_command() -> clap::Command {
        Output::augment_command(clap::Command::new("test"))
    }

    #[test]
    fn augment_command_adds_three_global_args() {
        let command = test_command();
        let args: Vec<&str> = command
            .get_arguments()
            .filter(|a| a.is_global_set())
            .map(|a| a.get_id().as_str())
            .collect();

        assert!(args.contains(&"quiet"));
        assert!(args.contains(&"verbose"));
        assert!(args.contains(&"json"));
        assert_eq!(args.len(), 3);
    }

    #[test]
    fn default_is_text_with_default_verbosity() {
        let output = Output::default();

        assert_eq!(output.verbosity(), Verbosity::Default);
        assert_eq!(output.mode(), OutputMode::Text);
    }

    #[test]
    fn from_arg_matches_with_defaults() {
        let matches = test_command().get_matches_from(vec!["test"]);

        let output = Output::from_arg_matches(&matches);

        assert_eq!(output.verbosity(), Verbosity::Default);
        assert_eq!(output.mode(), OutputMode::Text);
    }

    #[test]
    fn from_arg_matches_with_json_flag() {
        let matches = test_command().get_matches_from(vec!["test", "--json"]);

        let output = Output::from_arg_matches(&matches);

        assert_eq!(output.mode(), OutputMode::Json);
        assert_eq!(output.verbosity(), Verbosity::Default);
    }

    #[test]
    fn from_arg_matches_with_quiet_flag() {
        let matches = test_command().get_matches_from(vec!["test", "--quiet"]);

        let output = Output::from_arg_matches(&matches);

        assert_eq!(output.verbosity(), Verbosity::Quiet);
        assert_eq!(output.mode(), OutputMode::Text);
    }

    #[test]
    fn from_arg_matches_with_verbose_flag() {
        let matches = test_command().get_matches_from(vec!["test", "--verbose"]);

        let output = Output::from_arg_matches(&matches);

        assert_eq!(output.verbosity(), Verbosity::Verbose);
        assert_eq!(output.mode(), OutputMode::Text);
    }

    #[test]
    fn mode_returns_configured_mode() {
        let (output, _buffers) = Output::new_test(Verbosity::Default, OutputMode::Json);

        assert_eq!(output.mode(), OutputMode::Json);
    }

    #[test]
    fn message_in_default_mode_writes() {
        let (output, buffers) = Output::new_test(Verbosity::Default, OutputMode::Text);

        output.message("hello world");

        assert_eq!(buffers.messages(), "hello world\n");
    }

    #[test]
    fn message_in_quiet_mode_is_noop() {
        let (output, buffers) = Output::new_test(Verbosity::Quiet, OutputMode::Text);

        output.message("hello world");

        assert_eq!(buffers.messages(), "");
    }

    #[test]
    fn message_in_verbose_mode_writes() {
        let (output, buffers) = Output::new_test(Verbosity::Verbose, OutputMode::Text);

        output.message("hello world");

        assert_eq!(buffers.messages(), "hello world\n");
    }

    #[test]
    fn artifact_in_json_mode_serializes_as_json() {
        let (output, buffers) = Output::new_test(Verbosity::Default, OutputMode::Json);

        output.artifact(&"hello world");

        assert_eq!(buffers.artifacts(), "\"hello world\"\n");
    }

    #[test]
    fn artifact_in_quiet_mode_writes() {
        let (output, buffers) = Output::new_test(Verbosity::Quiet, OutputMode::Text);

        output.artifact(&"hello world");

        assert_eq!(buffers.artifacts(), "hello world\n");
    }

    #[test]
    fn artifact_in_text_mode_uses_display() {
        let (output, buffers) = Output::new_test(Verbosity::Default, OutputMode::Text);

        output.artifact(&42);

        assert_eq!(buffers.artifacts(), "42\n");
    }

    #[test]
    fn artifact_with_multiple_calls_produces_multiple_lines() {
        let (output, buffers) = Output::new_test(Verbosity::Default, OutputMode::Text);

        output.artifact(&"first");
        output.artifact(&"second");

        assert_eq!(buffers.artifacts(), "first\nsecond\n");
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
    fn detail_in_default_mode_is_noop() {
        let (output, buffers) = Output::new_test(Verbosity::Default, OutputMode::Text);

        output.detail("extra detail");

        assert_eq!(buffers.messages(), "");
    }

    #[test]
    fn detail_in_quiet_mode_is_noop() {
        let (output, buffers) = Output::new_test(Verbosity::Quiet, OutputMode::Text);

        output.detail("extra detail");

        assert_eq!(buffers.messages(), "");
    }

    #[test]
    fn detail_in_verbose_mode_writes() {
        let (output, buffers) = Output::new_test(Verbosity::Verbose, OutputMode::Text);

        output.detail("extra detail");

        assert_eq!(buffers.messages(), "extra detail\n");
    }

    #[test]
    fn verbosity_returns_configured_verbosity() {
        let (output, _buffers) = Output::new_test(Verbosity::Verbose, OutputMode::Text);

        assert_eq!(output.verbosity(), Verbosity::Verbose);
    }
}
