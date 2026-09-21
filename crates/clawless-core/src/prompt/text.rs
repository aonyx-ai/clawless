use getset::Getters;

/// A question that a user answers with one line of text
///
/// The answer is the line as the user typed it, without the characters that end the line. A
/// prompt does not trim or validate the answer, and the answer can be empty.
///
/// # Examples
///
/// ```
/// use clawless_core::prompt::Text;
///
/// let question = Text::new("Title of the change");
///
/// assert_eq!(question.question(), "Title of the change");
/// ```
#[derive(Clone, Eq, PartialEq, Hash, Debug, Getters)]
pub struct Text {
    /// The text of the question
    #[getset(get = "pub")]
    question: String,
}

impl Text {
    /// Creates a question
    ///
    /// # Examples
    ///
    /// ```
    /// use clawless_core::prompt::Text;
    ///
    /// let question = Text::new("Title of the change");
    /// ```
    pub fn new(question: impl Into<String>) -> Self {
        Self {
            question: question.into(),
        }
    }
}

impl From<&str> for Text {
    fn from(question: &str) -> Self {
        Self::new(question)
    }
}

impl From<String> for Text {
    fn from(question: String) -> Self {
        Self::new(question)
    }
}

#[cfg(test)]
mod tests {
    // An assertion in a test panics by design. A `# Panics` section on every test
    // would repeat that and give the reader no information.
    #![allow(clippy::missing_panics_doc)]

    use super::*;

    #[test]
    fn from_a_string_slice_keeps_the_question() {
        let question = Text::from("Title");

        assert_eq!(question, Text::new("Title"));
    }

    #[test]
    fn trait_send() {
        fn assert_send<T: Send>() {}
        assert_send::<Text>();
    }

    #[test]
    fn trait_sync() {
        fn assert_sync<T: Sync>() {}
        assert_sync::<Text>();
    }

    #[test]
    fn trait_unpin() {
        fn assert_unpin<T: Unpin>() {}
        assert_unpin::<Text>();
    }
}
