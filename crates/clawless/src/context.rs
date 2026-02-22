//! Context for Clawless commands
//!
//! This module defines the `Context` struct, which provides information about the environment that
//! Clawless commands are executed in, access to shared resources like loggers, and configuration
//! settings.
//!
//! For information on the context that is available to commands, see the fields and methods of the
//! `Context` struct as well as the types defined in this module.

use anyhow::Result;
use bon::bon;
use getset::Getters;

pub use self::current_working_directory::CurrentWorkingDirectory;
use crate::cancellation::Cancellation;
use crate::output::Output;

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
///     message!("Hello, {}!", args.name);
///     Ok(())
/// }
/// ```
#[derive(Clone, Debug, Getters)]
pub struct Context {
    /// The working directory in which a command was called
    #[getset(get = "pub")]
    current_working_directory: CurrentWorkingDirectory,

    /// The cancellation token for cooperative shutdown
    #[getset(get = "pub")]
    cancellation: Cancellation,

    /// The output handler for user-facing messages
    #[getset(get = "pub")]
    output: Output,
}

#[bon]
impl Context {
    /// Creates a new [`Context`] instance
    ///
    /// When `current_working_directory` is omitted, it is auto-detected from the environment.
    /// When provided explicitly (e.g., in tests), the given value is used directly.
    ///
    /// # Errors
    ///
    /// Returns an error if `current_working_directory` is not provided and the current working
    /// directory cannot be determined from the environment.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// // Production: CWD auto-detected
    /// let context = Context::builder().build()?;
    ///
    /// // Tests: explicit CWD
    /// let context = Context::builder()
    ///     .current_working_directory(tmp.path())
    ///     .build()?;
    /// ```
    #[builder]
    pub fn new(
        #[builder(into)] current_working_directory: Option<CurrentWorkingDirectory>,
        #[builder(default)] cancellation: Cancellation,
        #[builder(default)] output: Output,
    ) -> Result<Self> {
        let current_working_directory = match current_working_directory {
            Some(cwd) => cwd,
            None => CurrentWorkingDirectory::try_from_env()?,
        };

        Ok(Self {
            current_working_directory,
            cancellation,
            output,
        })
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::*;
    use crate::output::{OutputMode, Verbosity};

    #[test]
    fn new_with_cancellation_uses_provided_token() {
        let cancellation = Cancellation::new();
        cancellation.cancel();

        let context = Context::builder()
            .current_working_directory(Path::new("/tmp"))
            .cancellation(cancellation)
            .build()
            .expect("should create context");

        assert!(context.cancellation().is_cancelled());
    }

    #[test]
    fn new_with_cwd_uses_provided_value() {
        let context = Context::builder()
            .current_working_directory(Path::new("/tmp"))
            .build()
            .expect("should create context");

        assert_eq!(context.current_working_directory().get(), Path::new("/tmp"));
    }

    #[test]
    fn new_with_defaults_has_default_output() {
        let context = Context::builder()
            .current_working_directory(Path::new("/tmp"))
            .build()
            .expect("should create context");

        assert_eq!(context.output().verbosity(), Verbosity::Default);
        assert_eq!(context.output().mode(), OutputMode::Text);
    }

    #[test]
    fn new_with_defaults_detects_cwd() {
        let expected = std::env::current_dir().expect("should get current dir");

        let context = Context::builder().build().expect("should create context");

        assert_eq!(context.current_working_directory().get(), expected);
    }

    #[test]
    fn new_with_output_uses_provided_value() {
        let output = Output::new(Verbosity::Verbose, OutputMode::Json);

        let context = Context::builder()
            .current_working_directory(Path::new("/tmp"))
            .output(output)
            .build()
            .expect("should create context");

        assert_eq!(context.output().verbosity(), Verbosity::Verbose);
        assert_eq!(context.output().mode(), OutputMode::Json);
    }

    #[test]
    fn new_with_defaults_has_uncancelled_token() {
        let context = Context::builder()
            .current_working_directory(Path::new("/tmp"))
            .build()
            .expect("should create context");

        assert!(!context.cancellation().is_cancelled());
    }

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
