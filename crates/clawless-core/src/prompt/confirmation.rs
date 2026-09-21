/// The answer of a user to a [`Confirm`] prompt
///
/// The answer is an enum and not a `bool`, so that a `match` on it names both answers.
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
///
/// [`Confirm`]: super::Confirm
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub enum Confirmation {
    /// The user agreed
    Yes,
    /// The user declined
    No,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn trait_send() {
        fn assert_send<T: Send>() {}
        assert_send::<Confirmation>();
    }

    #[test]
    fn trait_sync() {
        fn assert_sync<T: Sync>() {}
        assert_sync::<Confirmation>();
    }

    #[test]
    fn trait_unpin() {
        fn assert_unpin<T: Unpin>() {}
        assert_unpin::<Confirmation>();
    }
}
