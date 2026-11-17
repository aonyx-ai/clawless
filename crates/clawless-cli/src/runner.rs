//! CLI command runner
//!
//! This module defines [`CommandRunner`], which encapsulates the full lifecycle of a CLI command:
//! event channel creation, context construction, signal handling, and terminal presentation. The
//! `main!()` macro dispatches to `CommandRunner::run` when the resolved leaf is a command.
//!
//! Application authors do not interact with this module directly. The `main!()` macro calls
//! [`CommandRunner::run`] with the resolved matches and the leaf's exec function. The runner takes
//! any callable, so a caller that builds its command tree at run time can dispatch to a closure
//! that owns the command it resolved.

use std::env;
use std::ffi::{OsStr, OsString};
use std::future::Future;
use std::io::IsTerminal;

use bon::Builder;
use clawless_core::cancellation::Cancellation;
use clawless_core::context::{Context, Interactivity};
use clawless_core::event::{Styling, event_channel};
use clawless_core::exit::Exit;
use clawless_core::output::Output;
use clawless_core::signal::wait_for_shutdown;

use crate::error::{CommandResult, ErrorContext};
use crate::output::OutputFlags;
use crate::presenter::{Presenter, TerminalPresenter};

/// Reports whether a user is present who can answer the application
///
/// A user is present when `input` and `display` are both terminals. The user types on the
/// input and reads on the display. The display is the standard error, because the standard
/// output carries the result of the command.
fn detect_interactivity(input: &impl IsTerminal, display: &impl IsTerminal) -> Interactivity {
    match (input.is_terminal(), display.is_terminal()) {
        (true, true) => Interactivity::Interactive,
        (true, false) => Interactivity::NonInteractive,
        (false, true) => Interactivity::NonInteractive,
        (false, false) => Interactivity::NonInteractive,
    }
}

/// The variables of the environment that state a preference for the colors of the output
///
/// Each field holds the value of one variable, or [`None`] when the variable is not set.
/// [`CommandRunner::run`] reads them from the environment of the process, and a test builds
/// them.
#[derive(Clone, Eq, PartialEq, Debug, Builder)]
struct ColorVariables {
    /// The value of `NO_COLOR`, which turns colors off
    #[builder(into)]
    no_color: Option<OsString>,

    /// The value of `CLICOLOR_FORCE`, which turns colors on whatever the stream is
    #[builder(into)]
    clicolor_force: Option<OsString>,

    /// The value of `CLICOLOR`, which turns colors off with `0` and asserts support otherwise
    #[builder(into)]
    clicolor: Option<OsString>,

    /// The value of `TERM`, the kind of terminal, which is `dumb` for one without colors
    #[builder(into)]
    term: Option<OsString>,
}

impl ColorVariables {
    /// Reads the variables from the environment of the process
    fn from_env() -> Self {
        Self {
            no_color: env::var_os("NO_COLOR"),
            clicolor_force: env::var_os("CLICOLOR_FORCE"),
            clicolor: env::var_os("CLICOLOR"),
            term: env::var_os("TERM"),
        }
    }
}

/// What the environment of the process asks for the colors of the output
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
enum ColorPreference {
    /// The environment turns colors off, and the text is plain whatever the stream is
    Disabled,
    /// `CLICOLOR_FORCE` turns colors on, and the text is styled whatever the stream is
    Forced,
    /// The environment states nothing that decides, and the stream decides
    Unstated,
}

impl ColorPreference {
    /// Reads the preference from the variables of the environment
    ///
    /// A variable that is not set, or is set to the empty string, states nothing. The rules
    /// follow `NO_COLOR` and the `CLICOLOR` convention, in this order:
    ///
    /// 1. `NO_COLOR` turns colors off with any value. It wins over every other variable, so that
    ///    turning colors off always works.
    /// 2. `CLICOLOR_FORCE` turns colors on with any value except `0`.
    /// 3. `CLICOLOR` turns colors off with `0`. Any other value states that the terminal supports
    ///    colors, and the stream decides.
    /// 4. `TERM=dumb` turns colors off, because such a terminal shows escape sequences as text.
    fn from_variables(variables: &ColorVariables) -> Self {
        let ColorVariables {
            no_color,
            clicolor_force,
            clicolor,
            term,
        } = variables;

        if stated(no_color).is_some() {
            return Self::Disabled;
        }
        if stated(clicolor_force).is_some_and(|value| value != "0") {
            return Self::Forced;
        }

        let Some(clicolor) = stated(clicolor) else {
            return match stated(term).is_some_and(|term| term == "dumb") {
                true => Self::Disabled,
                false => Self::Unstated,
            };
        };

        match clicolor == "0" {
            true => Self::Disabled,
            false => Self::Unstated,
        }
    }
}

/// Returns the value of a variable, or [`None`] if the variable is not set or is empty
fn stated(value: &Option<OsString>) -> Option<&OsStr> {
    value.as_deref().filter(|value| !value.is_empty())
}

/// Where the standard output of the process goes
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
enum Destination {
    /// A terminal, which shows escape sequences as styles
    Terminal,
    /// A file, a pipe, or another stream that is not a terminal
    Other,
}

impl Destination {
    /// Reports where the given stream goes
    fn of(stream: &impl IsTerminal) -> Self {
        match stream.is_terminal() {
            true => Self::Terminal,
            false => Self::Other,
        }
    }
}

/// Chooses the styling of the text that the presenter writes to the standard output
///
/// The presenter writes artifacts, messages, and details to the standard output in text mode.
/// Styled text belongs on a terminal. A file or a pipe gets plain text, so that a reader or a
/// tool does not have to remove escape sequences. The preference of the environment overrides
/// the destination.
fn detect_styling(destination: Destination, preference: ColorPreference) -> Styling {
    match preference {
        ColorPreference::Disabled => Styling::Plain,
        ColorPreference::Forced => Styling::Styled,
        ColorPreference::Unstated => match destination {
            Destination::Terminal => Styling::Styled,
            Destination::Other => Styling::Plain,
        },
    }
}

/// CLI command runner
///
/// Encapsulates the lifecycle of running a CLI command: creating the event channel, constructing
/// the [`Context`], registering signal handlers, and presenting output through a
/// [`TerminalPresenter`]. The `main!()` macro dispatches to [`CommandRunner::run`] when the
/// resolved leaf is a `ResolvedLeaf::Command`.
///
/// # Examples
///
/// ```rust,ignore
/// // This is what main!() expands to for commands:
/// ResolvedLeaf::Command { matches, exec } => {
///     clawless::runner::CommandRunner::run(matches, exec)
/// }
/// ```
#[derive(Debug)]
pub struct CommandRunner;

impl CommandRunner {
    /// Runs a CLI command to completion
    ///
    /// Sets up the command lifecycle:
    ///
    /// 1. Detects whether a user is present who can answer the application, and whether the
    ///    text of the command is styled
    /// 2. Creates a [`Cancellation`] token for cooperative shutdown
    /// 3. Extracts output flags from the pre-parsed argument matches
    /// 4. Creates an event channel and builds the [`Context`]
    /// 5. Configures a [`TerminalPresenter`] with the parsed output flags
    /// 6. Creates a Tokio runtime, spawns the signal handler, and runs the command through the
    ///    presenter
    ///
    /// A user is present when the standard input and the standard error are both terminals. The
    /// [`Context`] carries the result as its [`Interactivity`].
    ///
    /// In text mode, the presenter writes artifacts, messages, and details styled when the
    /// standard output is a terminal, and plain otherwise. The environment overrides the stream.
    /// A non-empty `NO_COLOR` makes the text plain and wins over every other variable. A
    /// non-empty `CLICOLOR_FORCE` other than `0` makes it styled. `CLICOLOR=0` and `TERM=dumb`
    /// make it plain, and a `CLICOLOR` other than `0` keeps a dumb terminal styled. The command
    /// does not see the [`Styling`]: a value offers its forms, and the presenter chooses one.
    ///
    /// Output flags (`--quiet`, `--verbose`, `--json`) are augmented at the root level by
    /// `main!()` with `.global(true)`, so they are available in every leaf's [`ArgMatches`].
    ///
    /// The runner returns the [`Exit`] of the command, and only after the presenter has rendered
    /// every event of the command. If the command fails, or if the runner cannot build the
    /// [`Context`] or the Tokio runtime, the [`Exit`] has the exit code 1 and the text of the
    /// error. The `main` function returns the [`Exit`] in a `ProcessExit`, and the process then
    /// writes its text and exits with its code. The text therefore comes after all the output of
    /// the command.
    ///
    /// # Arguments
    ///
    /// * `matches` — The parsed [`ArgMatches`] for this command leaf, as resolved by the
    ///   subcommand tree walk.
    /// * `exec` — Executes the command with the given matches and context. The `#[command]` macro
    ///   generates a function for this, and any other callable works. A caller that builds its
    ///   command tree at run time passes a closure that owns the command it resolved. The command
    ///   returns `()` for the exit code 0, or an [`Exit`] to choose the exit code and the text.
    ///
    /// [`ArgMatches`]: clap::ArgMatches
    /// [`Cancellation`]: clawless_core::cancellation::Cancellation
    /// [`Context`]: clawless_core::context::Context
    /// [`Exit`]: clawless_core::exit::Exit
    /// [`Interactivity`]: clawless_core::context::Interactivity
    /// [`Styling`]: clawless_core::event::Styling
    /// [`TerminalPresenter`]: crate::presenter::TerminalPresenter
    pub fn run<E, F, T>(matches: clap::ArgMatches, exec: E) -> Exit
    where
        E: FnOnce(clap::ArgMatches, Context) -> F,
        F: Future<Output = CommandResult<T>> + Send + 'static,
        T: Into<Exit>,
    {
        let interactivity = detect_interactivity(&std::io::stdin(), &std::io::stderr());
        let preference = ColorPreference::from_variables(&ColorVariables::from_env());
        let styling = detect_styling(Destination::of(&std::io::stdout()), preference);

        Self::run_with(matches, exec, interactivity, styling)
    }

    /// Runs a CLI command to completion for the given interactivity and styling
    ///
    /// [`CommandRunner::run`] detects the interactivity and the styling, and calls this function.
    /// A test calls this function directly, because the streams of a test are never terminals.
    fn run_with<E, F, T>(
        matches: clap::ArgMatches,
        exec: E,
        interactivity: Interactivity,
        styling: Styling,
    ) -> Exit
    where
        E: FnOnce(clap::ArgMatches, Context) -> F,
        F: Future<Output = CommandResult<T>> + Send + 'static,
        T: Into<Exit>,
    {
        Self::try_run(matches, exec, interactivity, styling)
            .unwrap_or_else(|error| Exit::from_error(&error))
    }

    /// Runs a CLI command to completion and returns its exit, or the error that ended it
    ///
    /// # Errors
    ///
    /// Returns an error if context construction fails (e.g., the current working directory cannot
    /// be determined), if the Tokio runtime cannot be created, or if the command itself fails.
    fn try_run<E, F, T>(
        matches: clap::ArgMatches,
        exec: E,
        interactivity: Interactivity,
        styling: Styling,
    ) -> CommandResult<Exit>
    where
        E: FnOnce(clap::ArgMatches, Context) -> F,
        F: Future<Output = CommandResult<T>> + Send + 'static,
        T: Into<Exit>,
    {
        let cancellation = Cancellation::new();
        let output_flags = OutputFlags::from_arg_matches(&matches);

        let (sender, receiver) = event_channel();
        let output = Output::new(sender);

        let context = Context::builder()
            .cancellation(cancellation.clone())
            .interactivity(interactivity)
            .output(output)
            .build()
            .context("build the context of the command")?;

        let presenter = TerminalPresenter::builder()
            .receiver(receiver)
            .verbosity(output_flags.verbosity())
            .mode(output_flags.mode())
            .interactivity(interactivity)
            .styling(styling)
            .build();

        let rt = tokio::runtime::Runtime::new().context("start the async runtime")?;
        rt.block_on(async {
            tokio::spawn(wait_for_shutdown(cancellation));

            let command = exec(matches, context);
            presenter
                .present(Box::pin(async move { command.await.map(Into::into) }))
                .await
        })
    }
}

#[cfg(test)]
mod tests {
    // An assertion in a test panics by design. A `# Panics` section on every test
    // would repeat that and give the reader no information.
    #![allow(clippy::missing_panics_doc)]

    use std::fs::File;
    use std::path::Path;
    use std::process::ExitCode;
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::{Arc, Mutex};

    use super::*;

    /// Returns a file that is not a terminal
    fn file() -> File {
        let manifest = Path::new(env!("CARGO_MANIFEST_DIR")).join("Cargo.toml");

        File::open(manifest).expect("should open the manifest")
    }

    /// Returns the matches of a command line without arguments
    fn matches() -> clap::ArgMatches {
        OutputFlags::augment_command(clap::Command::new("test")).get_matches_from(["test"])
    }

    #[test]
    fn color_preference_from_variables_with_a_dumb_terminal_returns_disabled() {
        let variables = ColorVariables::builder().term("dumb").build();

        let preference = ColorPreference::from_variables(&variables);

        assert_eq!(preference, ColorPreference::Disabled);
    }

    #[test]
    fn color_preference_from_variables_with_a_terminal_with_colors_returns_unstated() {
        let variables = ColorVariables::builder().term("xterm-256color").build();

        let preference = ColorPreference::from_variables(&variables);

        assert_eq!(preference, ColorPreference::Unstated);
    }

    #[test]
    fn color_preference_from_variables_with_clicolor_and_a_dumb_terminal_returns_unstated() {
        let variables = ColorVariables::builder().clicolor("1").term("dumb").build();

        let preference = ColorPreference::from_variables(&variables);

        assert_eq!(preference, ColorPreference::Unstated);
    }

    #[test]
    fn color_preference_from_variables_with_clicolor_empty_returns_unstated() {
        let variables = ColorVariables::builder().clicolor("").build();

        let preference = ColorPreference::from_variables(&variables);

        assert_eq!(preference, ColorPreference::Unstated);
    }

    #[test]
    fn color_preference_from_variables_with_clicolor_force_and_a_dumb_terminal_returns_forced() {
        let variables = ColorVariables::builder()
            .clicolor_force("1")
            .term("dumb")
            .build();

        let preference = ColorPreference::from_variables(&variables);

        assert_eq!(preference, ColorPreference::Forced);
    }

    #[test]
    fn color_preference_from_variables_with_clicolor_force_and_clicolor_zero_returns_forced() {
        let variables = ColorVariables::builder()
            .clicolor_force("1")
            .clicolor("0")
            .build();

        let preference = ColorPreference::from_variables(&variables);

        assert_eq!(preference, ColorPreference::Forced);
    }

    #[test]
    fn color_preference_from_variables_with_clicolor_force_empty_returns_unstated() {
        let variables = ColorVariables::builder().clicolor_force("").build();

        let preference = ColorPreference::from_variables(&variables);

        assert_eq!(preference, ColorPreference::Unstated);
    }

    #[test]
    fn color_preference_from_variables_with_clicolor_force_returns_forced() {
        let variables = ColorVariables::builder().clicolor_force("1").build();

        let preference = ColorPreference::from_variables(&variables);

        assert_eq!(preference, ColorPreference::Forced);
    }

    #[test]
    fn color_preference_from_variables_with_clicolor_force_zero_returns_unstated() {
        let variables = ColorVariables::builder().clicolor_force("0").build();

        let preference = ColorPreference::from_variables(&variables);

        assert_eq!(preference, ColorPreference::Unstated);
    }

    #[test]
    fn color_preference_from_variables_with_clicolor_returns_unstated() {
        let variables = ColorVariables::builder().clicolor("1").build();

        let preference = ColorPreference::from_variables(&variables);

        assert_eq!(preference, ColorPreference::Unstated);
    }

    #[test]
    fn color_preference_from_variables_with_clicolor_zero_returns_disabled() {
        let variables = ColorVariables::builder().clicolor("0").build();

        let preference = ColorPreference::from_variables(&variables);

        assert_eq!(preference, ColorPreference::Disabled);
    }

    #[test]
    fn color_preference_from_variables_with_no_color_and_clicolor_force_returns_disabled() {
        let variables = ColorVariables::builder()
            .no_color("1")
            .clicolor_force("1")
            .build();

        let preference = ColorPreference::from_variables(&variables);

        assert_eq!(preference, ColorPreference::Disabled);
    }

    #[test]
    fn color_preference_from_variables_with_no_color_and_clicolor_returns_disabled() {
        let variables = ColorVariables::builder()
            .no_color("1")
            .clicolor("1")
            .build();

        let preference = ColorPreference::from_variables(&variables);

        assert_eq!(preference, ColorPreference::Disabled);
    }

    #[test]
    fn color_preference_from_variables_with_no_color_empty_returns_unstated() {
        let variables = ColorVariables::builder().no_color("").build();

        let preference = ColorPreference::from_variables(&variables);

        assert_eq!(preference, ColorPreference::Unstated);
    }

    #[test]
    fn color_preference_from_variables_with_no_color_returns_disabled() {
        let variables = ColorVariables::builder().no_color("1").build();

        let preference = ColorPreference::from_variables(&variables);

        assert_eq!(preference, ColorPreference::Disabled);
    }

    #[test]
    fn color_preference_from_variables_with_no_color_zero_returns_disabled() {
        let variables = ColorVariables::builder().no_color("0").build();

        let preference = ColorPreference::from_variables(&variables);

        assert_eq!(preference, ColorPreference::Disabled);
    }

    #[test]
    fn color_preference_from_variables_without_variables_returns_unstated() {
        let variables = ColorVariables::builder().build();

        let preference = ColorPreference::from_variables(&variables);

        assert_eq!(preference, ColorPreference::Unstated);
    }

    #[test]
    fn color_preference_trait_send() {
        fn assert_send<T: Send>() {}
        assert_send::<ColorPreference>();
    }

    #[test]
    fn color_preference_trait_sync() {
        fn assert_sync<T: Sync>() {}
        assert_sync::<ColorPreference>();
    }

    #[test]
    fn color_preference_trait_unpin() {
        fn assert_unpin<T: Unpin>() {}
        assert_unpin::<ColorPreference>();
    }

    #[test]
    fn color_variables_trait_send() {
        fn assert_send<T: Send>() {}
        assert_send::<ColorVariables>();
    }

    #[test]
    fn color_variables_trait_sync() {
        fn assert_sync<T: Sync>() {}
        assert_sync::<ColorVariables>();
    }

    #[test]
    fn color_variables_trait_unpin() {
        fn assert_unpin<T: Unpin>() {}
        assert_unpin::<ColorVariables>();
    }

    #[test]
    fn detect_interactivity_without_a_terminal_returns_non_interactive() {
        let (input, display) = (file(), file());

        let interactivity = detect_interactivity(&input, &display);

        assert_eq!(interactivity, Interactivity::NonInteractive);
    }

    #[test]
    fn destination_of_a_file_returns_other() {
        let destination = Destination::of(&file());

        assert_eq!(destination, Destination::Other);
    }

    #[test]
    fn destination_trait_send() {
        fn assert_send<T: Send>() {}
        assert_send::<Destination>();
    }

    #[test]
    fn destination_trait_sync() {
        fn assert_sync<T: Sync>() {}
        assert_sync::<Destination>();
    }

    #[test]
    fn destination_trait_unpin() {
        fn assert_unpin<T: Unpin>() {}
        assert_unpin::<Destination>();
    }

    #[test]
    fn detect_styling_with_a_disabled_preference_on_a_terminal_returns_plain() {
        let styling = detect_styling(Destination::Terminal, ColorPreference::Disabled);

        assert_eq!(styling, Styling::Plain);
    }

    #[test]
    fn detect_styling_with_a_forced_preference_without_a_terminal_returns_styled() {
        let styling = detect_styling(Destination::Other, ColorPreference::Forced);

        assert_eq!(styling, Styling::Styled);
    }

    #[test]
    fn detect_styling_with_an_unstated_preference_on_a_terminal_returns_styled() {
        let styling = detect_styling(Destination::Terminal, ColorPreference::Unstated);

        assert_eq!(styling, Styling::Styled);
    }

    #[test]
    fn detect_styling_with_an_unstated_preference_without_a_terminal_returns_plain() {
        let styling = detect_styling(Destination::Other, ColorPreference::Unstated);

        assert_eq!(styling, Styling::Plain);
    }

    #[test]
    fn run_with_a_closure_that_owns_state_executes_the_leaf() {
        let executed = Arc::new(AtomicBool::new(false));
        let owned = Arc::clone(&executed);

        drop(CommandRunner::run(
            matches(),
            move |_matches, _context| async move {
                owned.store(true, Ordering::SeqCst);
                Ok(())
            },
        ));

        assert!(executed.load(Ordering::SeqCst));
    }

    #[test]
    fn run_with_a_command_that_chooses_its_exit_returns_that_exit() {
        let chosen = Exit::builder()
            .code(ExitCode::from(3))
            .text("error: 0.7.0 is not a stable release\n")
            .build();
        let returned = chosen.clone();

        let exit = CommandRunner::run_with(
            matches(),
            move |_matches, _context| async move { Ok(returned) },
            Interactivity::NonInteractive,
            Styling::Plain,
        );

        assert_eq!(exit, chosen);
    }

    #[test]
    fn run_with_a_command_that_fails_returns_the_error_with_code_1() {
        let exit = CommandRunner::run_with(
            matches(),
            |_matches, _context| async { Err::<(), _>(anyhow::anyhow!("the file is missing")) },
            Interactivity::NonInteractive,
            Styling::Plain,
        );

        assert_eq!(
            exit,
            Exit::builder()
                .code(ExitCode::FAILURE)
                .text("Error: the file is missing\n")
                .build()
        );
    }

    #[test]
    fn run_with_a_command_that_succeeds_returns_success() {
        let exit = CommandRunner::run_with(
            matches(),
            |_matches, _context| async { Ok(()) },
            Interactivity::NonInteractive,
            Styling::Plain,
        );

        assert_eq!(exit, Exit::from(()));
    }

    #[test]
    fn run_with_interactivity_passes_it_to_the_context() {
        let seen = Arc::new(Mutex::new(None));
        let owned = Arc::clone(&seen);

        drop(CommandRunner::run_with(
            matches(),
            move |_matches, context| async move {
                *owned.lock().expect("should lock") = Some(context.interactivity());
                Ok(())
            },
            Interactivity::Interactive,
            Styling::Plain,
        ));

        assert_eq!(
            *seen.lock().expect("should lock"),
            Some(Interactivity::Interactive)
        );
    }

    #[test]
    fn trait_send() {
        fn assert_send<T: Send>() {}
        assert_send::<CommandRunner>();
    }

    #[test]
    fn trait_sync() {
        fn assert_sync<T: Sync>() {}
        assert_sync::<CommandRunner>();
    }

    #[test]
    fn trait_unpin() {
        fn assert_unpin<T: Unpin>() {}
        assert_unpin::<CommandRunner>();
    }
}
