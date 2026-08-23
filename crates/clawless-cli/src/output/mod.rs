//! CLI flag configuration for output behavior
//!
//! This module defines [`OutputFlags`], which captures the `--quiet`, `--verbose`, and `--json`
//! flags from the command line. These flags control how the [`TerminalPresenter`] renders events:
//! [`Verbosity`] controls *whether* an event is shown; [`OutputMode`] controls *where* and *how*.
//!
//! Commands produce output through the core [`Output`] type, which sends events into a channel.
//! `OutputFlags` configures the presenter that consumes those events, not the output itself.
//!
//! [`Output`]: clawless_core::output::Output
//! [`TerminalPresenter`]: crate::presenter::TerminalPresenter

use clap::{Arg, ArgAction, ArgMatches};

pub use self::output_mode::OutputMode;
pub use self::verbosity::Verbosity;

/// Whether events render as human-readable text or as JSON
mod output_mode;
/// How much detail the presenter renders
mod verbosity;

/// CLI flag configuration for output behavior
///
/// `OutputFlags` captures the `--quiet`, `--verbose`, and `--json` flags that the `main!()` macro
/// adds to every Clawless application. After parsing, the flags are forwarded to the
/// [`TerminalPresenter`] to control how events are rendered.
///
/// `OutputFlags` does not produce output itself. Commands use the core [`Output`] type to emit
/// events; `OutputFlags` configures the presenter that renders them.
///
/// # Examples
///
/// ```
/// use clawless_cli::output::OutputFlags;
///
/// let flags = OutputFlags::new(
///     clawless_cli::output::Verbosity::Default,
///     clawless_cli::output::OutputMode::Text,
/// );
/// assert_eq!(flags.verbosity(), clawless_cli::output::Verbosity::Default);
/// assert_eq!(flags.mode(), clawless_cli::output::OutputMode::Text);
/// ```
///
/// [`Output`]: clawless_core::output::Output
/// [`TerminalPresenter`]: crate::presenter::TerminalPresenter
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub struct OutputFlags {
    /// How much detail the presenter renders, from the `--quiet` and `--verbose` flags
    verbosity: Verbosity,
    /// Whether the presenter renders text or JSON, from the `--json` flag
    mode: OutputMode,
}

impl OutputFlags {
    /// Creates a new [`OutputFlags`] with the given verbosity and mode
    ///
    /// # Examples
    ///
    /// ```
    /// use clawless_cli::output::{OutputFlags, OutputMode, Verbosity};
    ///
    /// let flags = OutputFlags::new(Verbosity::Default, OutputMode::Text);
    /// assert_eq!(flags.verbosity(), Verbosity::Default);
    /// assert_eq!(flags.mode(), OutputMode::Text);
    /// ```
    pub fn new(verbosity: Verbosity, mode: OutputMode) -> Self {
        Self { verbosity, mode }
    }

    /// Returns the verbosity level
    ///
    /// # Examples
    ///
    /// ```
    /// use clawless_cli::output::{OutputFlags, OutputMode, Verbosity};
    ///
    /// let flags = OutputFlags::new(Verbosity::Verbose, OutputMode::Text);
    /// assert_eq!(flags.verbosity(), Verbosity::Verbose);
    /// ```
    pub fn verbosity(&self) -> Verbosity {
        self.verbosity
    }

    /// Returns the output mode
    ///
    /// # Examples
    ///
    /// ```
    /// use clawless_cli::output::{OutputFlags, OutputMode, Verbosity};
    ///
    /// let flags = OutputFlags::new(Verbosity::Default, OutputMode::Json);
    /// assert_eq!(flags.mode(), OutputMode::Json);
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
    /// let command = clawless_cli::output::OutputFlags::augment_command(command);
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

    /// Constructs an [`OutputFlags`] from parsed CLI flags
    ///
    /// Reads the `--quiet`, `--verbose`, and `--json` flags from the given [`ArgMatches`] and
    /// returns an [`OutputFlags`] configured accordingly. The command must have been augmented with
    /// [`augment_command`] before parsing.
    ///
    /// This method is called by the `main!()` macro expansion. Command authors do not call it
    /// directly.
    ///
    /// # Examples
    ///
    /// ```
    /// let command = clawless_cli::output::OutputFlags::augment_command(
    ///     clap::Command::new("test"),
    /// );
    /// let matches = command.get_matches_from(vec!["test", "--quiet"]);
    /// let flags = clawless_cli::output::OutputFlags::from_arg_matches(&matches);
    /// assert_eq!(flags.verbosity(), clawless_cli::output::Verbosity::Quiet);
    /// ```
    ///
    /// [`augment_command`]: OutputFlags::augment_command
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

impl Default for OutputFlags {
    fn default() -> Self {
        Self::new(Verbosity::default(), OutputMode::default())
    }
}

#[cfg(test)]
mod tests {
    // An assertion in a test panics by design. A `# Panics` section on every test
    // would repeat that and give the reader no information.
    #![allow(clippy::missing_panics_doc)]

    use super::*;

    fn test_command() -> clap::Command {
        OutputFlags::augment_command(clap::Command::new("test"))
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
        let flags = OutputFlags::default();

        assert_eq!(flags.verbosity(), Verbosity::Default);
        assert_eq!(flags.mode(), OutputMode::Text);
    }

    #[test]
    fn from_arg_matches_with_defaults() {
        let matches = test_command().get_matches_from(vec!["test"]);

        let flags = OutputFlags::from_arg_matches(&matches);

        assert_eq!(flags.verbosity(), Verbosity::Default);
        assert_eq!(flags.mode(), OutputMode::Text);
    }

    #[test]
    fn from_arg_matches_with_json_flag() {
        let matches = test_command().get_matches_from(vec!["test", "--json"]);

        let flags = OutputFlags::from_arg_matches(&matches);

        assert_eq!(flags.mode(), OutputMode::Json);
        assert_eq!(flags.verbosity(), Verbosity::Default);
    }

    #[test]
    fn from_arg_matches_with_quiet_flag() {
        let matches = test_command().get_matches_from(vec!["test", "--quiet"]);

        let flags = OutputFlags::from_arg_matches(&matches);

        assert_eq!(flags.verbosity(), Verbosity::Quiet);
        assert_eq!(flags.mode(), OutputMode::Text);
    }

    #[test]
    fn from_arg_matches_with_verbose_flag() {
        let matches = test_command().get_matches_from(vec!["test", "--verbose"]);

        let flags = OutputFlags::from_arg_matches(&matches);

        assert_eq!(flags.verbosity(), Verbosity::Verbose);
        assert_eq!(flags.mode(), OutputMode::Text);
    }

    #[test]
    fn mode_returns_configured_mode() {
        let flags = OutputFlags::new(Verbosity::Default, OutputMode::Json);

        assert_eq!(flags.mode(), OutputMode::Json);
    }

    #[test]
    fn trait_send() {
        fn assert_send<T: Send>() {}
        assert_send::<OutputFlags>();
    }

    #[test]
    fn trait_sync() {
        fn assert_sync<T: Sync>() {}
        assert_sync::<OutputFlags>();
    }

    #[test]
    fn trait_unpin() {
        fn assert_unpin<T: Unpin>() {}
        assert_unpin::<OutputFlags>();
    }

    #[test]
    fn verbosity_returns_configured_verbosity() {
        let flags = OutputFlags::new(Verbosity::Verbose, OutputMode::Text);

        assert_eq!(flags.verbosity(), Verbosity::Verbose);
    }
}
