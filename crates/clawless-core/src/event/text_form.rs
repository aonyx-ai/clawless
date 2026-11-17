use std::fmt::{self, Display};

use super::Styling;

/// Trait for values that write their text in a styling that the presenter chooses
///
/// A value can have two text forms: a styled form with escape sequences for color and emphasis,
/// and a plain form without them. Only the presenter knows whether its text reaches a terminal,
/// so the presenter chooses the [`Styling`], and [`fmt_text`] writes the form that matches it.
///
/// Most values do not implement this trait directly. A blanket implementation covers every type
/// that implements [`Display`]. Such a value has one text form, its [`Display`] output, and writes
/// it in every styling.
///
/// A value with a styled form implements the trait itself, and then does not implement
/// [`Display`]. The two implementations would otherwise overlap. The same overlap stops this crate
/// from forwarding the trait through a generic wrapper, such as `Box<T>`, `Arc<T>`, or `&T`: each
/// of those wrappers implements [`Display`] when the value in it does. A crate that owns a styled
/// type can implement the trait for `Box<Verdict>` or `&Verdict` itself. A value behind an `Arc`
/// needs a newtype of the crate around the `Arc`.
///
/// A styled value has no [`Display`], so it cannot be an argument of `format!` or of a format
/// string of `message!`. Such a value is passed on whole, or [`to_text`] writes it into a string
/// in one styling.
///
/// # Examples
///
/// A value with a styled form and a plain form:
///
/// ```
/// use std::fmt;
///
/// use clawless_core::event::{Styling, TextForm};
///
/// struct Verdict {
///     passed: bool,
/// }
///
/// impl TextForm for Verdict {
///     fn fmt_text(&self, f: &mut fmt::Formatter<'_>, styling: Styling) -> fmt::Result {
///         let word = match self.passed {
///             true => "passed",
///             false => "failed",
///         };
///
///         match styling {
///             Styling::Styled => write!(f, "\x1b[1m{word}\x1b[0m"),
///             Styling::Plain => f.write_str(word),
///         }
///     }
/// }
///
/// let verdict = Verdict { passed: true };
/// assert_eq!(verdict.to_text(Styling::Styled), "\x1b[1mpassed\x1b[0m");
/// assert_eq!(verdict.to_text(Styling::Plain), "passed");
/// ```
///
/// [`fmt_text`]: TextForm::fmt_text
/// [`to_text`]: TextForm::to_text
pub trait TextForm {
    /// Writes the text form of the value in the given styling
    ///
    /// [`Styling::Plain`] asks for text without escape sequences. [`Styling::Styled`] permits
    /// ANSI escape sequences for color and emphasis, and does not require them. A value that
    /// contains another value with a text form passes the styling on, so that the whole text has
    /// one styling.
    ///
    /// # Errors
    ///
    /// Returns [`fmt::Error`] if the formatter fails to write.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::fmt;
    ///
    /// use clawless_core::event::{Styling, TextForm};
    ///
    /// struct Summary {
    ///     count: usize,
    /// }
    ///
    /// impl TextForm for Summary {
    ///     fn fmt_text(&self, f: &mut fmt::Formatter<'_>, styling: Styling) -> fmt::Result {
    ///         f.write_str("found ")?;
    ///         self.count.fmt_text(f, styling)?;
    ///         f.write_str(" files")
    ///     }
    /// }
    ///
    /// assert_eq!(Summary { count: 3 }.to_text(Styling::Styled), "found 3 files");
    /// ```
    fn fmt_text(&self, f: &mut fmt::Formatter<'_>, styling: Styling) -> fmt::Result;

    /// Returns the text form of the value in the given styling
    ///
    /// The text is what [`fmt_text`] writes. A test uses it to compare the forms of a value, and
    /// an application uses it where it needs the text as a [`String`].
    ///
    /// # Panics
    ///
    /// Panics if [`fmt_text`] returns an error, as [`ToString::to_string`] does for a [`Display`]
    /// implementation that fails.
    ///
    /// # Examples
    ///
    /// ```
    /// use clawless_core::event::{Styling, TextForm};
    ///
    /// assert_eq!(42.to_text(Styling::Plain), "42");
    /// ```
    ///
    /// [`fmt_text`]: TextForm::fmt_text
    fn to_text(&self, styling: Styling) -> String {
        InStyling {
            value: self,
            styling,
        }
        .to_string()
    }
}

impl<T> TextForm for T
where
    T: Display + ?Sized,
{
    fn fmt_text(&self, f: &mut fmt::Formatter<'_>, _styling: Styling) -> fmt::Result {
        Display::fmt(self, f)
    }
}

/// The text form of a value in one styling, as a value that implements [`Display`]
struct InStyling<'a, T: ?Sized> {
    /// The value that writes the text
    value: &'a T,

    /// The styling that the text has
    styling: Styling,
}

impl<T> Display for InStyling<'_, T>
where
    T: TextForm + ?Sized,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.value.fmt_text(f, self.styling)
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
    fn to_text_of_a_display_value_with_plain_styling_returns_its_display_output() {
        let text = 42.to_text(Styling::Plain);

        assert_eq!(text, "42");
    }

    #[test]
    fn to_text_of_a_display_value_with_styled_styling_returns_its_display_output() {
        let text = 42.to_text(Styling::Styled);

        assert_eq!(text, "42");
    }

    #[test]
    fn to_text_of_an_unsized_display_value_returns_its_display_output() {
        let text = <str as TextForm>::to_text("hello", Styling::Styled);

        assert_eq!(text, "hello");
    }

    #[test]
    fn to_text_with_plain_styling_returns_the_plain_form() {
        let value = Bold::new("hello");

        let text = value.to_text(Styling::Plain);

        assert_eq!(text, "hello");
    }

    #[test]
    fn to_text_with_styled_styling_returns_the_styled_form() {
        let value = Bold::new("hello");

        let text = value.to_text(Styling::Styled);

        assert_eq!(text, "\x1b[1mhello\x1b[0m");
    }
}
