//! Errors that occur when Clawless builds a [`Context`]

use thiserror::Error;

/// Errors that can occur when constructing a [`Context`]
///
/// # Examples
///
/// ```
/// use clawless_core::context::ContextError;
///
/// let error = ContextError::from(std::io::Error::other("the directory was removed"));
///
/// assert_eq!(error.to_string(), "failed to detect current working directory");
/// ```
///
/// [`Context`]: super::Context
#[derive(Debug, Error)]
pub enum ContextError {
    /// Failed to auto-detect the current working directory from the environment
    #[error("failed to detect current working directory")]
    CurrentWorkingDirectory {
        /// The error that the operating system reported
        #[from]
        source: std::io::Error,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_auto_traits<T: Send + Sync + Unpin>() {}

    #[test]
    fn context_error_is_send_sync_and_unpin() {
        assert_auto_traits::<ContextError>();
    }
}
