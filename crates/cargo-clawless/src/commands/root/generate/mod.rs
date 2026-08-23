//! Code generation commands for Clawless projects

use clawless::prelude::*;

/// Generates a single command file and registers it with its parent module
mod command;

/// Arguments for the `generate` command group
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Default, Args)]
pub struct GenerateArgs {}

/// Generate code scaffolding for Clawless projects
///
/// This is a command group containing subcommands for generating different
/// types of code. Run with a subcommand to generate specific scaffolding.
///
/// # Examples
///
/// ```shell
/// cargo clawless generate command my-command
/// ```
#[command(require_subcommand, alias = "g")]
// A command's doc comment is its `--help` text, so an `# Errors` section would render as a
// raw Markdown heading in the terminal rather than documenting an API.
#[allow(clippy::missing_errors_doc)]
pub async fn generate(_args: GenerateArgs, _context: Context) -> CommandResult {
    Ok(())
}
