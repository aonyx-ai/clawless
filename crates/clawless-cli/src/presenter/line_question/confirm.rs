//! How a terminal asks for a confirmation on one line

use clawless_core::prompt::{Confirm, Confirmation};

use super::LineQuestion;

/// The hint for a line that is neither a yes nor a no
const HINT: &str = r#"Please answer "y" or "n"."#;

/// Asks for a yes or a no on one line
///
/// The choices follow the question in brackets, with the default in upper case: `[y/N]`. The
/// user answers with `y`, `yes`, `n`, or `no`, in any case. An empty line is the default. Without
/// a default, an empty line is no answer.
impl LineQuestion for Confirm {
    type Answer = Confirmation;

    fn prompt(&self) -> String {
        let choices = match self.default() {
            Some(Confirmation::Yes) => "[Y/n]",
            Some(Confirmation::No) => "[y/N]",
            None => "[y/n]",
        };

        format!("{} {choices} ", self.question())
    }

    fn parse(&self, line: &str) -> Result<Self::Answer, String> {
        let word = line.trim().to_lowercase();

        match word.as_str() {
            "y" | "yes" => Ok(Confirmation::Yes),
            "n" | "no" => Ok(Confirmation::No),
            "" => self.default().ok_or_else(|| HINT.to_owned()),
            _ => Err(HINT.to_owned()),
        }
    }
}

#[cfg(test)]
mod tests {
    // An assertion in a test panics by design. A `# Panics` section on every test
    // would repeat that and give the reader no information.
    #![allow(clippy::missing_panics_doc)]

    use clawless_core::prompt::{Confirm, Confirmation};

    use crate::presenter::line_question::LineQuestion;

    #[test]
    fn parse_with_a_word_in_upper_case_returns_the_answer() {
        let question = Confirm::new("Release?");

        let answer = question.parse("YES").expect("should parse");

        assert_eq!(answer, Confirmation::Yes);
    }

    #[test]
    fn parse_with_an_empty_line_and_a_default_returns_the_default() {
        let question = Confirm::new("Release?").with_default(Confirmation::No);

        let answer = question.parse("").expect("should parse");

        assert_eq!(answer, Confirmation::No);
    }

    #[test]
    fn parse_with_an_empty_line_and_no_default_returns_a_hint() {
        let question = Confirm::new("Release?");

        let hint = question.parse("").expect_err("should not parse");

        assert_eq!(hint, r#"Please answer "y" or "n"."#);
    }

    #[test]
    fn parse_with_an_unknown_word_returns_a_hint() {
        let question = Confirm::new("Release?").with_default(Confirmation::Yes);

        let hint = question.parse("maybe").expect_err("should not parse");

        assert_eq!(hint, r#"Please answer "y" or "n"."#);
    }

    #[test]
    fn parse_with_n_returns_no() {
        let question = Confirm::new("Release?");

        let answer = question.parse("n").expect("should parse");

        assert_eq!(answer, Confirmation::No);
    }

    #[test]
    fn parse_with_space_around_the_word_returns_the_answer() {
        let question = Confirm::new("Release?");

        let answer = question.parse("  no ").expect("should parse");

        assert_eq!(answer, Confirmation::No);
    }

    #[test]
    fn parse_with_y_returns_yes() {
        let question = Confirm::new("Release?");

        let answer = question.parse("y").expect("should parse");

        assert_eq!(answer, Confirmation::Yes);
    }

    #[test]
    fn prompt_with_a_default_of_no_marks_it_in_upper_case() {
        let question = Confirm::new("Release?").with_default(Confirmation::No);

        let prompt = question.prompt();

        assert_eq!(prompt, "Release? [y/N] ");
    }

    #[test]
    fn prompt_with_a_default_of_yes_marks_it_in_upper_case() {
        let question = Confirm::new("Release?").with_default(Confirmation::Yes);

        let prompt = question.prompt();

        assert_eq!(prompt, "Release? [Y/n] ");
    }

    #[test]
    fn prompt_without_a_default_marks_no_answer() {
        let question = Confirm::new("Release?");

        let prompt = question.prompt();

        assert_eq!(prompt, "Release? [y/n] ");
    }
}
