mod commands {
    use clawless::prelude::*;

    clawless::commands!();

    mod check {
        use clawless::prelude::*;

        #[derive(Debug, Args)]
        pub struct CheckArgs {}

        #[command]
        pub async fn check(_args: CheckArgs, _context: Context) -> CommandResult<Exit> {
            Ok(Exit::builder().code(ExitCode::from(3)).build())
        }
    }
}

fn main() {}
