//! Integration tests for the name that a command has on the command line
//!
//! A function name travels two separate paths before a command runs: clap learns the name when
//! the macros build the command tree, and the generated dispatch code looks the same name up
//! after clap has parsed the arguments. These tests walk both paths, because a name that reaches
//! only one of them produces a command that the help shows but that never runs.

// This test is a crate root that nothing links against, so `unreachable_pub` would demand
// `pub(crate)` on every item that the macros generate without telling the reader anything.
#![allow(unreachable_pub)]
// An assertion in a test panics by design. A `# Panics` section on every test
// would repeat that and give the reader no information.
#![allow(clippy::missing_panics_doc)]

use clawless::clap::error::ErrorKind;
use clawless::resolved_leaf::ResolvedLeaf;

/// The commands of the application under test
mod commands {
    clawless::commands!();

    /// A command whose name holds more than one word
    mod deploy_staging {
        use clawless::prelude::*;

        /// Arguments for the `deploy_staging` command
        #[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Args)]
        pub struct DeployStagingArgs {
            /// The target to deploy to
            target: String,
        }

        /// Deploy to staging
        #[command]
        // A command's doc comment is its `--help` text, so an `# Errors` section would render
        // as a raw Markdown heading in the terminal rather than documenting an API.
        #[allow(clippy::missing_errors_doc)]
        pub async fn deploy_staging(args: DeployStagingArgs, context: Context) -> CommandResult {
            message!("Deploying to {}!", args.target);
            Ok(())
        }
    }

    /// A command whose name is a raw identifier
    mod r#type {
        use clawless::prelude::*;

        /// Arguments for the `type` command
        #[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Args)]
        pub struct TypeArgs {
            /// The file to read
            file: String,
        }

        /// Print the contents of a file
        #[command]
        // A command's doc comment is its `--help` text, so an `# Errors` section would render
        // as a raw Markdown heading in the terminal rather than documenting an API.
        #[allow(clippy::missing_errors_doc)]
        pub async fn r#type(args: TypeArgs, context: Context) -> CommandResult {
            message!("Reading {}!", args.file);
            Ok(())
        }
    }
}

/// Returns the names under which clap knows the subcommands of the application under test
fn subcommand_names() -> Vec<String> {
    commands::clawless_init()
        .get_subcommands()
        .map(|subcommand| subcommand.get_name().to_string())
        .collect()
}

#[test]
fn init_with_multiple_word_command_uses_hyphenated_name() {
    let names = subcommand_names();

    assert!(names.contains(&"deploy-staging".to_string()));
}

#[test]
fn init_with_raw_identifier_command_uses_name_without_prefix() {
    let names = subcommand_names();

    assert!(names.contains(&"type".to_string()));
}

#[test]
fn parse_with_underscored_name_returns_error() {
    let command = commands::clawless_init();

    let error = command
        .try_get_matches_from(["app", "deploy_staging", "production"])
        .expect_err("should fail");

    assert_eq!(ErrorKind::InvalidSubcommand, error.kind());
}

#[test]
fn resolve_with_hyphenated_name_returns_the_arguments_of_the_command() {
    let matches =
        commands::clawless_init().get_matches_from(["app", "deploy-staging", "production"]);

    let leaf = commands::clawless_resolve(matches);

    let target = match leaf {
        ResolvedLeaf::Command { matches, .. } => matches
            .try_get_one::<String>("target")
            .ok()
            .flatten()
            .cloned(),
        ResolvedLeaf::Application { .. } => None,
    };
    assert_eq!(Some("production".to_string()), target);
}
