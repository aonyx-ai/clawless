use thiserror::Error;

/// Errors that can occur when constructing a [`Context`]
///
/// [`Context`]: super::Context
#[derive(Debug, Error)]
pub enum ContextError {
    /// Failed to auto-detect the current working directory from the environment
    #[error("failed to detect current working directory")]
    CurrentWorkingDirectory(#[from] std::io::Error),
}
