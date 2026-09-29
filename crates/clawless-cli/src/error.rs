//! Error types and result aliases for Clawless commands
//!
//! This module re-exports [`anyhow`] error handling primitives and defines the [`CommandResult`]
//! type alias used as the standard return type for command functions.

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
pub use anyhow::Error;

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
/// The `CommandResult` is a type alias for [`anyhow::Result<T>`], which provides
/// a more ergonomic way to handle arbitrary errors. Since it isn't possible to
/// recover from the error, we do not need to provide a specific error type
/// that a caller could handle gracefully.
///
/// The success type defaults to `()`. A command that returns `Ok(())` ends the
/// process with the exit code 0, and a command that returns an error ends it
/// with the exit code 1 and the text of the error. A command that chooses its
/// exit code and its text returns an [`Exit`] instead, as `CommandResult<Exit>`.
///
/// # Examples
///
/// ```
/// use std::process::ExitCode;
///
/// use clawless_cli::error::CommandResult;
/// use clawless_core::exit::Exit;
///
/// fn check(major_version: u64) -> CommandResult<Exit> {
///     if major_version == 0 {
///         return Ok(Exit::builder()
///             .code(ExitCode::from(3))
///             .text("error: the version is not a stable release\n")
///             .build());
///     }
///
///     Ok(Exit::from(()))
/// }
/// ```
///
/// [`Exit`]: clawless_core::exit::Exit
/// [`anyhow::Result<T>`]: anyhow::Result
pub type CommandResult<T = ()> = anyhow::Result<T>;
