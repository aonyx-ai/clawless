pub use self::command::*;
pub use self::parameter::*;
pub use self::provider::*;

mod command;
mod parameter;
mod provider;

#[cfg(test)]
mod tests {
    use crate::{CommandArguments, CommandResult};

    use super::*;

    struct Args;
    impl CommandArguments for Args {}

    async fn passes<'a>(args: Args, number: Context<'a, i32>) -> CommandResult {
        Ok(())
    }

    async fn fails<'a>(args: Args, number: Context<'a, i32>) -> CommandResult {
        anyhow::bail!("failed")
    }

    #[tokio::test]
    async fn run_command_with_context() {
        let mut context = ContextProvider::new();
        context.add_context(42i32);

        assert!(context.execute(Args, &passes).await.is_ok());
        assert!(context.execute(Args, &fails).await.is_err());
    }
}
