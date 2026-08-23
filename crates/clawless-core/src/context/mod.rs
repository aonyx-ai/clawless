//! Context for Clawless commands
//!
//! This module defines the [`Context`] struct, which provides information about the environment
//! that Clawless commands are executed in, access to shared resources like loggers, and
//! configuration settings.
//!
//! For information on the context that is available to commands, see the fields and methods of the
//! [`Context`] struct as well as the types defined in this module.

use bon::bon;
use getset::Getters;

pub use self::current_working_directory::CurrentWorkingDirectory;
pub use self::error::ContextError;
use crate::cancellation::Cancellation;
use crate::output::Output;

/// Newtype for the directory that a command runs in
mod current_working_directory;
/// Errors that occur when Clawless builds a [`Context`]
mod error;

/// Context for Clawless commands
///
/// This struct provides information about the environment that Clawless commands are executed in,
/// access to shared resources, and configuration settings. It is passed to each command by the
/// Clawless runtime when executing commands.
///
/// # Examples
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
// r[impl context.safety.send]
// r[impl context.safety.sync]
// r[impl context.safety.unpin]
#[derive(Clone, Debug, Getters)]
pub struct Context {
    /// The working directory in which a command was called
    // r[impl context.field.cwd]
    #[getset(get = "pub")]
    current_working_directory: CurrentWorkingDirectory,

    /// The cancellation token for cooperative shutdown
    // r[impl cancel.context.field]
    // r[impl context.field.cancellation]
    #[getset(get = "pub")]
    cancellation: Cancellation,

    /// The output handler for sending events to the presenter
    // r[impl context.field.output]
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
    /// Returns [`ContextError::CurrentWorkingDirectory`] if `current_working_directory` is not
    /// provided and the current working directory cannot be determined from the environment.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// // Production: CWD auto-detected
    /// let context = Context::builder()
    ///     .output(output)
    ///     .build()?;
    ///
    /// // Tests: explicit CWD
    /// let context = Context::builder()
    ///     .current_working_directory(tmp.path())
    ///     .output(output)
    ///     .build()?;
    /// ```
    // r[impl context.new]
    // r[impl context.new.error]
    // r[impl cancel.context.default]
    // r[impl cancel.context.injectable]
    #[builder]
    pub fn new(
        #[builder(into)] current_working_directory: Option<CurrentWorkingDirectory>,
        #[builder(default)] cancellation: Cancellation,
        output: Output,
    ) -> Result<Self, ContextError> {
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
    // An assertion in a test panics by design. A `# Panics` section on every test
    // would repeat that and give the reader no information.
    #![allow(clippy::missing_panics_doc)]

    use std::path::Path;

    use super::*;
    use crate::event::event_channel;

    fn test_output() -> Output {
        let (sender, _receiver) = event_channel();
        Output::new(sender)
    }

    // r[verify context.field.cancellation]
    // r[verify cancel.context.injectable]
    // r[verify cancel.context.field]
    #[test]
    fn new_with_cancellation_uses_provided_token() {
        let cancellation = Cancellation::new();
        cancellation.cancel();

        let context = Context::builder()
            .current_working_directory(Path::new("/tmp"))
            .cancellation(cancellation)
            .output(test_output())
            .build()
            .expect("should create context");

        assert!(context.cancellation().is_cancelled());
    }

    // r[verify context.field.cwd]
    #[test]
    fn new_with_cwd_uses_provided_value() {
        let context = Context::builder()
            .current_working_directory(Path::new("/tmp"))
            .output(test_output())
            .build()
            .expect("should create context");

        assert_eq!(context.current_working_directory().get(), Path::new("/tmp"));
    }

    // r[verify context.new]
    // r[verify context.field.output]
    #[test]
    fn new_with_defaults_detects_cwd() {
        let expected = std::env::current_dir().expect("should get current dir");

        let context = Context::builder()
            .output(test_output())
            .build()
            .expect("should create context");

        assert_eq!(context.current_working_directory().get(), expected);
    }

    // r[verify cancel.context.default]
    #[test]
    fn new_with_defaults_has_uncancelled_token() {
        let context = Context::builder()
            .current_working_directory(Path::new("/tmp"))
            .output(test_output())
            .build()
            .expect("should create context");

        assert!(!context.cancellation().is_cancelled());
    }

    // r[verify context.safety.send]
    #[test]
    fn trait_send() {
        fn assert_send<T: Send>() {}
        assert_send::<Context>();
    }

    // r[verify context.safety.sync]
    #[test]
    fn trait_sync() {
        fn assert_sync<T: Sync>() {}
        assert_sync::<Context>();
    }

    // r[verify context.safety.unpin]
    #[test]
    fn trait_unpin() {
        fn assert_unpin<T: Unpin>() {}
        assert_unpin::<Context>();
    }
}
