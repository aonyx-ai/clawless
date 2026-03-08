use clawless::prelude::*;

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Args)]
pub struct ShoutArgs {
    // Define command arguments here
}

#[command]
pub async fn shout(args: ShoutArgs, context: Context) -> CommandResult {
    // Command implementation goes here
    Ok(())
}
