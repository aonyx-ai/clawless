use clawless::prelude::*;

#[derive(Debug, Args)]
pub struct DashboardArgs {}

#[application]
pub async fn _dashboard(
    _args: DashboardArgs,
    _context: Context,
    _projection: clawless::tui::projection::Projection,
) -> CommandResult {
    Ok(())
}

fn main() {}
