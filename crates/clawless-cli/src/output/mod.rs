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

use bon::Builder;
use clap::{Arg, ArgAction, ArgMatches};

pub use self::output_mode::OutputMode;
pub use self::verbosity::Verbosity;

mod output_mode;
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
/// let flags = OutputFlags::builder()
///     .verbosity(clawless_cli::output::Verbosity::Default)
///     .mode(clawless_cli::output::OutputMode::Text)
///     .build();
/// assert_eq!(flags.verbosity(), clawless_cli::output::Verbosity::Default);
/// assert_eq!(flags.mode(), clawless_cli::output::OutputMode::Text);
/// ```
///
/// [`Output`]: clawless_core::output::Output
/// [`TerminalPresenter`]: crate::presenter::TerminalPresenter
#[derive(Copy, Clone, Eq, PartialEq, Debug, Builder)]
pub struct OutputFlags {
    /// How much detail the presenter renders, from the `--quiet` and `--verbose` flags
    verbosity: Verbosity,
    /// Whether the presenter renders text or JSON, from the `--json` flag
    mode: OutputMode,
}

impl OutputFlags {
    /// Returns the verbosity level
    ///
    /// # Examples
    ///
    /// ```
    /// use clawless_cli::output::{OutputFlags, OutputMode, Verbosity};
    ///
    /// let flags = OutputFlags::builder()
    ///     .verbosity(Verbosity::Verbose)
    ///     .mode(OutputMode::Text)
    ///     .build();
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
    /// let flags = OutputFlags::builder()
    ///     .verbosity(Verbosity::Default)
    ///     .mode(OutputMode::Json)
    ///     .build();
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

        Self::builder().verbosity(verbosity).mode(mode).build()
    }
}

impl Default for OutputFlags {
    fn default() -> Self {
        Self::builder()
            .verbosity(Verbosity::default())
            .mode(OutputMode::default())
            .build()
    }
}

#[cfg(test)]
mod tests {
    #![allow(
        clippy::missing_panics_doc,
        reason = "an assertion in a test panics by design, so a `# Panics` section would repeat it"
    )]

    use super::*;

    fn test_command() -> clap::Command {
        OutputFlags::augment_command(clap::Command::new("test"))
    }

    #[test]
    fn augment_command_adds_the_global_json_arg() {
        let command = test_command();
        let args: Vec<&str> = command
            .get_arguments()
            .filter(|a| a.is_global_set())
            .map(|a| a.get_id().as_str())
            .collect();

        assert!(args.contains(&"json"));
    }

    #[test]
    fn augment_command_adds_the_global_quiet_arg() {
        let command = test_command();
        let args: Vec<&str> = command
            .get_arguments()
            .filter(|a| a.is_global_set())
            .map(|a| a.get_id().as_str())
            .collect();

        assert!(args.contains(&"quiet"));
    }

    #[test]
    fn augment_command_adds_the_global_verbose_arg() {
        let command = test_command();
        let args: Vec<&str> = command
            .get_arguments()
            .filter(|a| a.is_global_set())
            .map(|a| a.get_id().as_str())
            .collect();

        assert!(args.contains(&"verbose"));
    }

    #[test]
    fn augment_command_adds_three_global_args() {
        let command = test_command();
        let args: Vec<&str> = command
            .get_arguments()
            .filter(|a| a.is_global_set())
            .map(|a| a.get_id().as_str())
            .collect();

        assert_eq!(args.len(), 3);
    }

    #[test]
    fn default_is_default_verbosity() {
        let flags = OutputFlags::default();

        assert_eq!(flags.verbosity(), Verbosity::Default);
    }

    #[test]
    fn default_is_text_mode() {
        let flags = OutputFlags::default();

        assert_eq!(flags.mode(), OutputMode::Text);
    }

    #[test]
    fn from_arg_matches_with_defaults_uses_default_verbosity() {
        let matches = test_command().get_matches_from(vec!["test"]);

        let flags = OutputFlags::from_arg_matches(&matches);

        assert_eq!(flags.verbosity(), Verbosity::Default);
    }

    #[test]
    fn from_arg_matches_with_defaults_uses_text_mode() {
        let matches = test_command().get_matches_from(vec!["test"]);

        let flags = OutputFlags::from_arg_matches(&matches);

        assert_eq!(flags.mode(), OutputMode::Text);
    }

    #[test]
    fn from_arg_matches_with_json_flag_keeps_default_verbosity() {
        let matches = test_command().get_matches_from(vec!["test", "--json"]);

        let flags = OutputFlags::from_arg_matches(&matches);

        assert_eq!(flags.verbosity(), Verbosity::Default);
    }

    #[test]
    fn from_arg_matches_with_json_flag_uses_json_mode() {
        let matches = test_command().get_matches_from(vec!["test", "--json"]);

        let flags = OutputFlags::from_arg_matches(&matches);

        assert_eq!(flags.mode(), OutputMode::Json);
    }

    #[test]
    fn from_arg_matches_with_quiet_flag_keeps_text_mode() {
        let matches = test_command().get_matches_from(vec!["test", "--quiet"]);

        let flags = OutputFlags::from_arg_matches(&matches);

        assert_eq!(flags.mode(), OutputMode::Text);
    }

    #[test]
    fn from_arg_matches_with_quiet_flag_uses_quiet_verbosity() {
        let matches = test_command().get_matches_from(vec!["test", "--quiet"]);

        let flags = OutputFlags::from_arg_matches(&matches);

        assert_eq!(flags.verbosity(), Verbosity::Quiet);
    }

    #[test]
    fn from_arg_matches_with_verbose_flag_keeps_text_mode() {
        let matches = test_command().get_matches_from(vec!["test", "--verbose"]);

        let flags = OutputFlags::from_arg_matches(&matches);

        assert_eq!(flags.mode(), OutputMode::Text);
    }

    #[test]
    fn from_arg_matches_with_verbose_flag_uses_verbose_verbosity() {
        let matches = test_command().get_matches_from(vec!["test", "--verbose"]);

        let flags = OutputFlags::from_arg_matches(&matches);

        assert_eq!(flags.verbosity(), Verbosity::Verbose);
    }

    #[test]
    fn mode_returns_configured_mode() {
        let flags = OutputFlags::builder()
            .verbosity(Verbosity::Default)
            .mode(OutputMode::Json)
            .build();

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
        let flags = OutputFlags::builder()
            .verbosity(Verbosity::Verbose)
            .mode(OutputMode::Text)
            .build();

        assert_eq!(flags.verbosity(), Verbosity::Verbose);
    }
}
