/// Whether text may carry terminal styles
///
/// A presenter chooses the styling of the text that it writes, because only the presenter knows
/// where that text goes. A value receives the choice in [`TextForm::fmt_text`] and writes the form
/// of its text that matches it. A value never asks the terminal itself: it cannot know which
/// stream the presenter writes to, or whether the user turned colors off.
///
/// [`Styling::Plain`] is the default, because plain text is correct on every destination. Styled
/// text in a file or a pipe leaves escape sequences that a reader or a tool must remove.
///
/// # Examples
///
/// ```
/// use clawless_core::event::Styling;
///
/// assert_eq!(Styling::default(), Styling::Plain);
/// ```
///
/// [`TextForm::fmt_text`]: super::TextForm::fmt_text
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Default)]
pub enum Styling {
    /// Text without escape sequences, for a file, a pipe, or a reader that turned colors off
    #[default]
    Plain,
    /// Text that may carry ANSI escape sequences for color and emphasis, for a terminal
    Styled,
}

#[cfg(test)]
mod tests {
    // An assertion in a test panics by design. A `# Panics` section on every test
    // would repeat that and give the reader no information.
    #![allow(clippy::missing_panics_doc)]

    use super::*;

    #[test]
    fn default_is_plain() {
        let styling = Styling::default();

        assert_eq!(styling, Styling::Plain);
    }

    #[test]
    fn trait_send() {
        fn assert_send<T: Send>() {}
        assert_send::<Styling>();
    }

    #[test]
    fn trait_sync() {
        fn assert_sync<T: Sync>() {}
        assert_sync::<Styling>();
    }

    #[test]
    fn trait_unpin() {
        fn assert_unpin<T: Unpin>() {}
        assert_unpin::<Styling>();
    }
}
