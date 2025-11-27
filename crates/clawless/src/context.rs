//! Context for Clawless commands
//!
//! This module defines the `Context` struct, which provides information about the environment that
//! Clawless commands are executed in, access to shared resources like loggers, and configuration
//! settings.
//!
//! For information on the context that is available to commands, see the fields and methods of the
//! `Context` struct as well as the types defined in this module.

use anyhow::Result;

/// Context for Clawless commands
///
/// This struct provides information about the environment that Clawless commands are executed in,
/// access to shared resources, and configuration settings. It is passed to each command by the
/// Clawless runtime when executing commands.
///
/// ```rust,ignore
/// #[derive(Debug, Args)]
/// pub struct GreetArgs {
///     name: String,
/// }
///
/// #[command]
/// pub async fn greet(args: GreetArgs, context: Context) -> CommandResult {
///     println!("Hello, {}!", args.name);
///     Ok(())
/// }
/// ```
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Default)]
pub struct Context {}

impl Context {
    /// Create a new `Context` instance
    ///
    /// This function initializes a new `Context` with default settings. Since some parts of the
    /// context might fail to initialize, this function returns a `Result`.
    pub fn try_new() -> Result<Self> {
        Ok(Self::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn trait_send() {
        fn assert_send<T: Send>() {}
        assert_send::<Context>();
    }

    #[test]
    fn trait_sync() {
        fn assert_sync<T: Sync>() {}
        assert_sync::<Context>();
    }

    #[test]
    fn trait_unpin() {
        fn assert_unpin<T: Unpin>() {}
        assert_unpin::<Context>();
    }
}
