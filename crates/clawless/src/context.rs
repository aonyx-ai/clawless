pub use self::command::*;
pub use self::parameter::*;
pub use self::provider::*;

mod command;
mod parameter;
mod provider;

#[cfg(test)]
mod tests {
    use crate::CommandResult;

    use super::*;

    fn passes(_number: Context<i32>) -> CommandResult {
        Ok(())
    }

    fn fails(_number: Context<i32>) -> CommandResult {
        anyhow::bail!("failed")
    }

    #[test]
    fn run_command_with_context() {
        let mut context = ContextProvider::new();
        context.add_context(42i32);

        assert!(context.execute(&passes).is_ok());
        assert!(context.execute(&fails).is_err());
    }
}
