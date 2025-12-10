//! Code generation commands for Clawless projects

use clawless::prelude::*;

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
/// clawless generate command my-command
/// ```
#[command(noop = true, alias = "g")]
pub async fn generate(_args: GenerateArgs, _context: Context) -> CommandResult {
    Ok(())
}
