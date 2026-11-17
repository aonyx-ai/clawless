use std::fmt;

use super::{Styling, TextForm};

/// The text of a message or a detail, written in every styling
///
/// A command sends a message before the presenter writes it, and the value that the message came
/// from may not live that long. A `FormattedText` therefore holds the [`TextForm`] of the value
/// in each [`Styling`], written when the text is created, and the presenter picks the form that
/// it needs. A value without a styled form, such as a [`String`], has one form, and the text
/// keeps it once.
///
/// `FormattedText` implements [`TextForm`], so a consumer that forwards a message keeps both of
/// its forms. It does not implement [`Display`], for the reason that [`TextForm`] explains: a
/// [`Display`] implementation would replace its styled form with the plain one.
///
/// # Examples
///
/// ```
/// use clawless_core::event::{FormattedText, Styling};
///
/// let text = FormattedText::from("deploying");
///
/// assert_eq!(text.form(Styling::Styled), "deploying");
/// ```
///
/// [`Display`]: std::fmt::Display
#[derive(Clone, Eq, PartialEq, Hash, Debug, Default)]
pub struct FormattedText {
    /// The forms of the text
    ///
    /// The forms are behind a [`Box`], so that a message in the event channel takes one pointer
    /// and an `Event` stays as small as its other variants.
    forms: Box<Forms>,
}

/// The forms of a [`FormattedText`]
#[derive(Clone, Eq, PartialEq, Hash, Debug, Default)]
struct Forms {
    /// The plain form of the text
    plain: String,

    /// The styled form of the text, if it differs from the plain form
    styled: Option<String>,
}

impl FormattedText {
    /// Writes the text form of the value in every styling
    ///
    /// The function calls [`TextForm::fmt_text`] once for each [`Styling`].
    ///
    /// # Panics
    ///
    /// Panics if [`TextForm::fmt_text`] returns an error, as [`TextForm::to_text`] does.
    ///
    /// # Examples
    ///
    /// ```
    /// use clawless_core::event::{FormattedText, Styling};
    ///
    /// let text = FormattedText::new(&42);
    ///
    /// assert_eq!(text.form(Styling::Plain), "42");
    /// ```
    pub fn new<T>(value: &T) -> Self
    where
        T: TextForm + ?Sized,
    {
        let plain = value.to_text(Styling::Plain);
        let styled = value.to_text(Styling::Styled);
        let styled = match styled == plain {
            true => None,
            false => Some(styled),
        };

        Self {
            forms: Box::new(Forms { plain, styled }),
        }
    }

    /// Returns the form of the text in the given styling
    ///
    /// # Examples
    ///
    /// ```
    /// use clawless_core::event::{FormattedText, Styling};
    ///
    /// let text = FormattedText::from("hello");
    ///
    /// assert_eq!(text.form(Styling::Plain), "hello");
    /// ```
    pub fn form(&self, styling: Styling) -> &str {
        match styling {
            Styling::Plain => &self.forms.plain,
            Styling::Styled => self.forms.styled.as_deref().unwrap_or(&self.forms.plain),
        }
    }

    /// Returns the form of the text in the given styling, and drops the other form
    ///
    /// # Examples
    ///
    /// ```
    /// use clawless_core::event::{FormattedText, Styling};
    ///
    /// let text = FormattedText::from("hello");
    ///
    /// assert_eq!(text.into_form(Styling::Plain), "hello");
    /// ```
    pub fn into_form(self, styling: Styling) -> String {
        let Forms { plain, styled } = *self.forms;

        match styling {
            Styling::Plain => plain,
            Styling::Styled => styled.unwrap_or(plain),
        }
    }
}

impl TextForm for FormattedText {
    fn fmt_text(&self, f: &mut fmt::Formatter<'_>, styling: Styling) -> fmt::Result {
        f.write_str(self.form(styling))
    }
}

impl From<String> for FormattedText {
    fn from(text: String) -> Self {
        Self {
            forms: Box::new(Forms {
                plain: text,
                styled: None,
            }),
        }
    }
}

impl From<&str> for FormattedText {
    fn from(text: &str) -> Self {
        Self::from(text.to_owned())
    }
}

/// Compares the text with a string that has one form
///
/// The text equals the string when both of its forms are that string. A text with a styled form
/// equals no string, because a string has no styled form.
impl PartialEq<str> for FormattedText {
    fn eq(&self, other: &str) -> bool {
        match &self.forms.styled {
            Some(_) => false,
            None => self.forms.plain == other,
        }
    }
}

#[cfg(test)]
mod tests {
    // An assertion in a test panics by design. A `# Panics` section on every test
    // would repeat that and give the reader no information.
    #![allow(clippy::missing_panics_doc)]

    use super::*;
    use crate::test_support::Bold;

    #[test]
    fn eq_with_a_string_of_its_one_form_returns_true() {
        let text = FormattedText::from("hello");

        let equal = text == *"hello";

        assert!(equal);
    }

    #[test]
    fn eq_with_a_styled_form_returns_false() {
        let text = FormattedText::new(&Bold::new("hello"));

        let equal = text == *"hello";

        assert!(!equal);
    }

    #[test]
    fn form_of_a_display_value_with_styled_styling_returns_its_display_output() {
        let text = FormattedText::new(&42);

        let form = text.form(Styling::Styled);

        assert_eq!(form, "42");
    }

    #[test]
    fn form_with_plain_styling_returns_the_plain_form() {
        let text = FormattedText::new(&Bold::new("hello"));

        let form = text.form(Styling::Plain);

        assert_eq!(form, "hello");
    }

    #[test]
    fn form_with_styled_styling_returns_the_styled_form() {
        let text = FormattedText::new(&Bold::new("hello"));

        let form = text.form(Styling::Styled);

        assert_eq!(form, "\x1b[1mhello\x1b[0m");
    }

    #[test]
    fn forms_trait_send() {
        fn assert_send<T: Send>() {}
        assert_send::<Forms>();
    }

    #[test]
    fn forms_trait_sync() {
        fn assert_sync<T: Sync>() {}
        assert_sync::<Forms>();
    }

    #[test]
    fn forms_trait_unpin() {
        fn assert_unpin<T: Unpin>() {}
        assert_unpin::<Forms>();
    }

    #[test]
    fn into_form_with_plain_styling_returns_the_plain_form() {
        let text = FormattedText::new(&Bold::new("hello"));

        let form = text.into_form(Styling::Plain);

        assert_eq!(form, "hello");
    }

    #[test]
    fn into_form_with_styled_styling_returns_the_styled_form() {
        let text = FormattedText::new(&Bold::new("hello"));

        let form = text.into_form(Styling::Styled);

        assert_eq!(form, "\x1b[1mhello\x1b[0m");
    }

    #[test]
    fn new_of_a_display_value_keeps_one_form() {
        let value = 42;

        let text = FormattedText::new(&value);

        assert_eq!(text, FormattedText::from("42"));
    }

    #[test]
    fn to_text_with_styled_styling_returns_the_styled_form() {
        let text = FormattedText::new(&Bold::new("hello"));

        let form = text.to_text(Styling::Styled);

        assert_eq!(form, "\x1b[1mhello\x1b[0m");
    }

    #[test]
    fn trait_send() {
        fn assert_send<T: Send>() {}
        assert_send::<FormattedText>();
    }

    #[test]
    fn trait_sync() {
        fn assert_sync<T: Sync>() {}
        assert_sync::<FormattedText>();
    }

    #[test]
    fn trait_unpin() {
        fn assert_unpin<T: Unpin>() {}
        assert_unpin::<FormattedText>();
    }
}
