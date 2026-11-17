//! Values for the tests of the Clawless crates
//!
//! The module is not part of the public API. The tests of `clawless-core` compile it, and the
//! tests of the other crates of the workspace enable the `test-support` feature to reach it.

use std::fmt;

use serde::Serialize;

use crate::event::{Styling, TextForm};

/// A value with a styled text form, which wraps its word in bold
///
/// The plain form is the word itself. The styled form surrounds it with the escape sequences
/// that turn bold on and off, so a test sees which form a presenter chose.
#[derive(Clone, Eq, PartialEq, Debug, Serialize)]
pub struct Bold {
    /// The word that the value writes
    value: String,
}

impl Bold {
    /// Creates a value that writes the given word
    pub fn new(value: impl Into<String>) -> Self {
        Self {
            value: value.into(),
        }
    }
}

impl TextForm for Bold {
    fn fmt_text(&self, f: &mut fmt::Formatter<'_>, styling: Styling) -> fmt::Result {
        match styling {
            Styling::Styled => write!(f, "\x1b[1m{}\x1b[0m", self.value),
            Styling::Plain => f.write_str(&self.value),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn trait_send() {
        fn assert_send<T: Send>() {}
        assert_send::<Bold>();
    }

    #[test]
    fn trait_sync() {
        fn assert_sync<T: Sync>() {}
        assert_sync::<Bold>();
    }

    #[test]
    fn trait_unpin() {
        fn assert_unpin<T: Unpin>() {}
        assert_unpin::<Bold>();
    }
}
