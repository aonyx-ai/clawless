use clawless::prelude::*;
use clawless::tui::projection::Projection;

#[derive(Debug, Args)]
pub struct DashboardArgs {}

#[application]
pub async fn dashboard(
    _args: DashboardArgs,
    _context: Context,
    _projection: Projection,
    _extra: String,
) -> CommandResult {
    Ok(())
}

fn main() {}
