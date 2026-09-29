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
        ) -> CommandResult<Exit> {
            Ok(Exit::builder().code(ExitCode::from(3)).build())
        }
    }
}

fn main() {}
