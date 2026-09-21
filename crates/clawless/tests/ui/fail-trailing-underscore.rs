use clawless::prelude::*;

#[derive(Debug, Args)]
pub struct DeployArgs {}

#[command]
pub async fn deploy_(_args: DeployArgs, _context: Context) -> CommandResult {
    Ok(())
}

fn main() {}
