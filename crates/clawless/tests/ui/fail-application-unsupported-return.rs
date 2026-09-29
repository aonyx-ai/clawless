mod commands {
    use clawless::prelude::*;
    use clawless::tui::projection::Projection;

    clawless::commands!();

    mod dashboard {
        use clawless::prelude::*;
        use clawless::tui::projection::Projection;

        #[derive(Debug, Args)]
        pub struct DashboardArgs {}

        #[application]
        pub async fn dashboard(
            _args: DashboardArgs,
            _context: Context,
            _projection: Projection,
        ) -> CommandResult<u8> {
            Ok(3)
        }
    }
}

fn main() {}
