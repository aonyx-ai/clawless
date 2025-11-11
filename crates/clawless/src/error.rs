pub use anyhow::Error;

/// Trait for adding context to errors
///
/// This is a re-export of `anyhow::Context` that provides the `.context()` method
/// for adding contextual information to errors. It's renamed to avoid conflicts
/// with a future `clawless::Context` type for application state.
///
/// # Example
///
/// ```rust,ignore
/// use clawless::ErrorContext;
///
/// let result = some_operation()
///     .context("Failed to perform operation")?;
/// ```
pub use anyhow::Context as ErrorContext;

/// Result type for Clawless commands
///
/// Commands in Clawless execute a piece of logic that might fail for various
/// reasons. If such an error is unrecoverable during the execution of the
/// command, it will cause the CLI to fail and exit with an error message.
///
/// To make it easier to handle errors when implementing commands, every command
/// handler returns a `CommandResult` type. This makes it possible to use the
/// question mark `?` operator and return early when an unrecoverable error
/// occurs.
///
/// The `CommandResult` is a type alias for `anyhow::Result<()>`, which provides
/// a more ergonomic way to handle arbitrary errors. Since it isn't possible to
/// recover from the error, we do not need to provide a specific error type
/// that a caller could handle gracefully. Similarly, commands do not need to
/// return a value, thus the result is always `Result<()>`.
pub type CommandResult = anyhow::Result<()>;
