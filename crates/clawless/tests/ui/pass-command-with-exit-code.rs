mod commands {
    use clawless::prelude::*;

    clawless::commands!();

    mod check {
        use clawless::prelude::*;

        #[derive(Debug, Args)]
        pub struct CheckArgs {}

        #[command]
        pub async fn check(_args: CheckArgs, _context: Context) -> CommandResult<ExitCode> {
            Ok(ExitCode::from(3))
        }
    }
}

fn main() {}
