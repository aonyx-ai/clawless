/// Level of output detail requested by the user
///
/// `Verbosity` controls whether [`Output`] methods produce output. It is
/// orthogonal to [`OutputMode`], which controls format and destination.
///
/// Three levels are available:
///
/// - **Quiet**: suppress informational messages; show only results and errors.
/// - **Default**: show normal messages and results.
/// - **Verbose**: show everything including additional detail.
///
/// # Examples
///
/// ```
/// use clawless::prelude::*;
///
/// let verbosity = Verbosity::default();
/// assert_eq!(verbosity, Verbosity::Default);
/// ```
///
/// [`Output`]: super::Output
/// [`OutputMode`]: super::OutputMode
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Default)]
pub enum Verbosity {
    /// Suppress informational messages; show only results and errors
    Quiet,
    /// Show normal messages and results
    #[default]
    Default,
    /// Show everything including additional detail
    Verbose,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_is_default_variant() {
        let verbosity = Verbosity::default();

        assert_eq!(verbosity, Verbosity::Default);
    }

    #[test]
    fn trait_send() {
        fn assert_send<T: Send>() {}
        assert_send::<Verbosity>();
    }

    #[test]
    fn trait_sync() {
        fn assert_sync<T: Sync>() {}
        assert_sync::<Verbosity>();
    }

    #[test]
    fn trait_unpin() {
        fn assert_unpin<T: Unpin>() {}
        assert_unpin::<Verbosity>();
    }
}
