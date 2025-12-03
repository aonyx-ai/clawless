//! Context for Clawless commands
//!
//! This module defines the `Context` struct, which provides information about the environment that
//! Clawless commands are executed in, access to shared resources like loggers, and configuration
//! settings.
//!
//! For information on the context that is available to commands, see the fields and methods of the
//! `Context` struct as well as the types defined in this module.

use anyhow::Result;
use getset::Getters;
use typed_builder::TypedBuilder;

pub use self::current_working_directory::CurrentWorkingDirectory;

mod current_working_directory;

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
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Getters, TypedBuilder)]
pub struct Context {
    /// The working directory in which a command was called
    #[builder(setter(into))]
    #[getset(get = "pub")]
    current_working_directory: CurrentWorkingDirectory,
}

impl Context {
    /// Create a new `Context` instance
    ///
    /// This function initializes a new `Context` with default settings. Since some parts of the
    /// context might fail to initialize, this function returns a `Result`.
    pub fn try_new() -> Result<Self> {
        let current_working_directory = CurrentWorkingDirectory::try_from_env()?;

        Ok(Self {
            current_working_directory,
        })
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
