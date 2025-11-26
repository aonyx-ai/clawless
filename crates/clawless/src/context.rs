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

    fn passes(args: Args, number: Context<i32>) -> CommandResult {
        Ok(())
    }

    fn fails(args: Args, number: Context<i32>) -> CommandResult {
        anyhow::bail!("failed")
    }

    #[test]
    fn run_command_with_context() {
        let mut context = ContextProvider::new();
        context.add_context(42i32);

        assert!(context.execute(Args, &passes).is_ok());
        assert!(context.execute(Args, &fails).is_err());
    }
}
