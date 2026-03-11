use clawless::prelude::*;

#[derive(Debug, Args)]
pub struct DashboardArgs {}

#[application]
pub async fn dashboard(
    _args: DashboardArgs,
    _context: Context,
) -> CommandResult {
    Ok(())
}

fn main() {}
