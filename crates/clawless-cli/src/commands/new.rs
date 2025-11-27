use clawless::prelude::*;

mod subcommand;

#[derive(Debug, Args)]
pub struct NewArgs {}

/// Create a new project
///
/// This command creates a new project and sets it up for clawless.
#[command(noop = true)]
pub async fn new(_args: NewArgs, _context: Context) -> CommandResult {
    Ok(())
}
