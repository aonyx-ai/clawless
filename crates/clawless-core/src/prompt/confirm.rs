use getset::{CopyGetters, Getters};

use super::Confirmation;

/// A question that a user answers with yes or no
///
/// The default is the answer that the user gives when they submit an empty answer. Without a
/// default, the user must type an answer. A prompt applies the default only when a user submits
/// it, and never when no user is present.
///
/// # Examples
///
/// ```
/// use clawless_core::prompt::{Confirm, Confirmation};
///
/// let question = Confirm::new("Delete the branch?").with_default(Confirmation::No);
///
/// assert_eq!(question.question(), "Delete the branch?");
/// ```
#[derive(Clone, Eq, PartialEq, Hash, Debug, CopyGetters, Getters)]
pub struct Confirm {
    /// The text of the question
    #[getset(get = "pub")]
    question: String,

    /// The answer that the user gives when they confirm without typing one
    #[getset(get_copy = "pub")]
    default: Option<Confirmation>,
}

impl Confirm {
    /// Creates a question without a default
    ///
    /// # Examples
    ///
    /// ```
    /// use clawless_core::prompt::Confirm;
    ///
    /// let question = Confirm::new("Delete the branch?");
    ///
    /// assert_eq!(question.default(), None);
    /// ```
    pub fn new(question: impl Into<String>) -> Self {
        Self {
            question: question.into(),
            default: None,
        }
    }

    /// Sets the answer that the user gives when they submit an empty answer
    ///
    /// # Examples
    ///
    /// ```
    /// use clawless_core::prompt::{Confirm, Confirmation};
    ///
    /// let question = Confirm::new("Delete the branch?").with_default(Confirmation::No);
    ///
    /// assert_eq!(question.default(), Some(Confirmation::No));
    /// ```
    #[must_use]
    pub fn with_default(mut self, default: Confirmation) -> Self {
        self.default = Some(default);
        self
    }
}

impl From<&str> for Confirm {
    fn from(question: &str) -> Self {
        Self::new(question)
    }
}

impl From<String> for Confirm {
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
    fn from_a_string_slice_has_no_default() {
        let question = Confirm::from("Delete the branch?");

        assert_eq!(question, Confirm::new("Delete the branch?"));
    }

    #[test]
    fn new_has_no_default() {
        let question = Confirm::new("Delete the branch?");

        assert_eq!(question.default(), None);
    }

    #[test]
    fn trait_send() {
        fn assert_send<T: Send>() {}
        assert_send::<Confirm>();
    }

    #[test]
    fn trait_sync() {
        fn assert_sync<T: Sync>() {}
        assert_sync::<Confirm>();
    }

    #[test]
    fn trait_unpin() {
        fn assert_unpin<T: Unpin>() {}
        assert_unpin::<Confirm>();
    }

    #[test]
    fn with_default_keeps_the_question() {
        let question = Confirm::new("Delete the branch?").with_default(Confirmation::Yes);

        assert_eq!(question.question(), "Delete the branch?");
    }

    #[test]
    fn with_default_sets_the_default() {
        let question = Confirm::new("Delete the branch?").with_default(Confirmation::Yes);

        assert_eq!(question.default(), Some(Confirmation::Yes));
    }
}
