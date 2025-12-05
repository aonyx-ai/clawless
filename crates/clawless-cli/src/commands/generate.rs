use clawless::prelude::*;

mod command;

#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Default, Args)]
pub struct GenerateArgs {}

#[command(noop = true)]
pub async fn generate(_args: GenerateArgs, _context: Context) -> CommandResult {
    Ok(())
}
