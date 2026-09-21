use super::Confirmation;

/// One answer in the script of a [`ScriptedUser`]
///
/// Each variant answers one kind of prompt, and it matches no other kind.
///
/// [`ScriptedUser`]: super::ScriptedUser
#[derive(Clone, Eq, PartialEq, Hash, Debug)]
pub enum ScriptedAnswer {
    /// The answer to a confirmation
    Confirm(Confirmation),

    /// The answer to a selection
    ///
    /// The value is the position of the chosen option, where the first option has the position
    /// zero.
    Select(usize),

    /// The answer to a question for one line of text
    Text(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn trait_send() {
        fn assert_send<T: Send>() {}
        assert_send::<ScriptedAnswer>();
    }

    #[test]
    fn trait_sync() {
        fn assert_sync<T: Sync>() {}
        assert_sync::<ScriptedAnswer>();
    }

    #[test]
    fn trait_unpin() {
        fn assert_unpin<T: Unpin>() {}
        assert_unpin::<ScriptedAnswer>();
    }
}
