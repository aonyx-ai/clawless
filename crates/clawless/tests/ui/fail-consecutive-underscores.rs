use clawless::prelude::*;

#[derive(Debug, Args)]
pub struct DeployStagingArgs {}

#[command]
pub async fn deploy__staging(_args: DeployStagingArgs, _context: Context) -> CommandResult {
    Ok(())
}

fn main() {}
