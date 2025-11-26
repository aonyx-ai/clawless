pub use self::command::*;
pub use self::parameter::*;
pub use self::provider::*;

mod command;
mod parameter;
mod provider;

#[cfg(test)]
mod tests {
    use crate::CommandResult;
    use clap::{Args, Command};

    use super::*;

    #[derive(Args)]
    struct CommandArgs;

    async fn passes<'a>(args: CommandArgs, number: Context<'a, i32>) -> CommandResult {
        Ok(())
    }

    async fn fails<'a>(args: CommandArgs, number: Context<'a, i32>) -> CommandResult {
        anyhow::bail!("failed")
    }

    #[tokio::test]
    async fn run_command_with_context() {
        let mut context = ContextProvider::new();
        context.add_context(42i32);

        let args = Command::new("test").get_matches_from(Vec::new());

        assert!(context.execute(passes, args).await.is_ok());
        assert!(context.execute(fails, args).await.is_err());
    }
}
