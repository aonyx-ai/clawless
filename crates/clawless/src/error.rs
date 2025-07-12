pub use anyhow::{Context, Error};

/// Result type for Clawless commands
///
/// Commands in Clawless execute a piece of logic that might fail for various
/// reasons. If such an error is unrecoverable during the execution of the
/// command, it will cause the CLI to fail and exit with an error message.
///
/// To make it easier to handle errors when implementing commands, every command
/// handler returns a `Result` type. This makes it possible to use the question
/// mark `?` operator and return early when an unrecoverable error occurs.
///
/// The `Result` is a wrapper around the `anyhow::Result` type, which provides
/// a more ergonomic way to handle arbitrary errors. Since it isn't possible to
/// recover the error anyways, we do not need to provide a specific error type
/// that a caller could handle gracefully. Similarly, commands do not need to
/// return a value and thus the `Result` type is always `()`.
pub type Result = anyhow::Result<()>;
