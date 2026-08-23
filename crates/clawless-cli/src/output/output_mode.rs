/// Output format and destination strategy
///
/// `OutputMode` controls where messages and results are written and how
/// results are formatted. It is orthogonal to [`Verbosity`], which controls
/// whether output is produced at all.
///
/// In text mode, all output goes to stdout. In JSON mode, messages go to
/// stderr (keeping stdout reserved for machine-readable data) and results
/// are serialized as JSON to stdout.
///
/// # Examples
///
/// ```
/// use clawless_cli::output::OutputMode;
///
/// let mode = OutputMode::default();
/// assert_eq!(mode, OutputMode::Text);
/// ```
///
/// [`Verbosity`]: super::Verbosity
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Default)]
pub enum OutputMode {
    /// Human-readable text output; all output goes to stdout
    #[default]
    Text,
    /// Machine-readable JSON output; messages go to stderr, results go to stdout as JSON
    Json,
}

#[cfg(test)]
mod tests {
    // An assertion in a test panics by design. A `# Panics` section on every test
    // would repeat that and give the reader no information.
    #![allow(clippy::missing_panics_doc)]

    use super::*;

    #[test]
    fn default_is_text() {
        let mode = OutputMode::default();

        assert_eq!(mode, OutputMode::Text);
    }

    #[test]
    fn trait_send() {
        fn assert_send<T: Send>() {}
        assert_send::<OutputMode>();
    }

    #[test]
    fn trait_sync() {
        fn assert_sync<T: Sync>() {}
        assert_sync::<OutputMode>();
    }

    #[test]
    fn trait_unpin() {
        fn assert_unpin<T: Unpin>() {}
        assert_unpin::<OutputMode>();
    }
}
