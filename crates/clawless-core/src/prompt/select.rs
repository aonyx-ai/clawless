use getset::Getters;

/// A question that a user answers with one of several options
///
/// The options are the texts that the user sees, in the order of display. The answer is the
/// position of one option, and the first option has the position zero.
///
/// [`Prompt::select`] builds this type from the `Display` text of its options and sends it to
/// the presenter. A command does not usually build it.
///
/// # Examples
///
/// ```
/// use clawless_core::prompt::Select;
///
/// let question = Select::new("Kind of change", ["Added", "Changed", "Fixed"]);
///
/// assert_eq!(question.options().len(), 3);
/// ```
///
/// [`Prompt::select`]: super::Prompt::select
#[derive(Clone, Eq, PartialEq, Hash, Debug, Getters)]
pub struct Select {
    /// The text of the question
    #[getset(get = "pub")]
    question: String,

    /// The texts of the options, in the order in which they are shown
    #[getset(get = "pub")]
    options: Vec<String>,
}

impl Select {
    /// Creates a question over the texts of its options
    ///
    /// # Examples
    ///
    /// ```
    /// use clawless_core::prompt::Select;
    ///
    /// let question = Select::new("Kind of change", ["Added", "Changed", "Fixed"]);
    /// ```
    pub fn new(
        question: impl Into<String>,
        options: impl IntoIterator<Item = impl Into<String>>,
    ) -> Self {
        Self {
            question: question.into(),
            options: options.into_iter().map(Into::into).collect(),
        }
    }
}

#[cfg(test)]
mod tests {
    // An assertion in a test panics by design. A `# Panics` section on every test
    // would repeat that and give the reader no information.
    #![allow(clippy::missing_panics_doc)]

    use super::*;

    #[test]
    fn new_keeps_the_options_in_their_order() {
        let question = Select::new("Kind of change", ["Added", "Fixed"]);

        assert_eq!(
            question.options(),
            &vec!["Added".to_owned(), "Fixed".to_owned()]
        );
    }

    #[test]
    fn trait_send() {
        fn assert_send<T: Send>() {}
        assert_send::<Select>();
    }

    #[test]
    fn trait_sync() {
        fn assert_sync<T: Sync>() {}
        assert_sync::<Select>();
    }

    #[test]
    fn trait_unpin() {
        fn assert_unpin<T: Unpin>() {}
        assert_unpin::<Select>();
    }
}
