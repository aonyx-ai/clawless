/// Whether a user is present who can answer the application
///
/// The code that builds the [`Context`] sets this value. The default is [`NonInteractive`], so
/// a context reports a user only when that code found one.
///
/// # Examples
///
/// ```
/// use clawless_core::context::Interactivity;
///
/// let interactivity = Interactivity::default();
/// assert_eq!(interactivity, Interactivity::NonInteractive);
/// ```
///
/// [`Context`]: super::Context
/// [`NonInteractive`]: Interactivity::NonInteractive
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Default)]
pub enum Interactivity {
    /// A user is present and can answer
    Interactive,
    /// No user can answer
    #[default]
    NonInteractive,
}

#[cfg(test)]
mod tests {
    // An assertion in a test panics by design. A `# Panics` section on every test
    // would repeat that and give the reader no information.
    #![allow(clippy::missing_panics_doc)]

    use super::*;

    #[test]
    fn default_is_non_interactive() {
        let interactivity = Interactivity::default();

        assert_eq!(interactivity, Interactivity::NonInteractive);
    }

    #[test]
    fn trait_send() {
        fn assert_send<T: Send>() {}
        assert_send::<Interactivity>();
    }

    #[test]
    fn trait_sync() {
        fn assert_sync<T: Sync>() {}
        assert_sync::<Interactivity>();
    }

    #[test]
    fn trait_unpin() {
        fn assert_unpin<T: Unpin>() {}
        assert_unpin::<Interactivity>();
    }
}
