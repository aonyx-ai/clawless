//! How a terminal asks for one line of text

use clawless_core::prompt::Text;

use super::LineQuestion;

/// Asks for a line of text after the question
///
/// The prompt is the question with a colon. A question that ends with a colon or with a question
/// mark gets no colon.
///
/// Every line is an answer, so `parse` never returns a hint. The answer is the line as the user
/// typed it.
impl LineQuestion for Text {
    type Answer = String;

    fn prompt(&self) -> String {
        let question = self.question();

        if question.ends_with([':', '?']) {
            format!("{question} ")
        } else {
            format!("{question}: ")
        }
    }

    fn parse(&self, line: &str) -> Result<Self::Answer, String> {
        Ok(line.to_owned())
    }
}

#[cfg(test)]
mod tests {
    // An assertion in a test panics by design. A `# Panics` section on every test
    // would repeat that and give the reader no information.
    #![allow(clippy::missing_panics_doc)]

    use clawless_core::prompt::Text;

    use crate::presenter::line_question::LineQuestion;

    #[test]
    fn parse_returns_the_line_as_the_user_typed_it() {
        let question = Text::new("Title");

        let answer = question.parse("  Fix the race ").expect("should parse");

        assert_eq!(answer, "  Fix the race ");
    }

    #[test]
    fn parse_with_an_empty_line_returns_an_empty_text() {
        let question = Text::new("Title");

        let answer = question.parse("").expect("should parse");

        assert_eq!(answer, "");
    }

    #[test]
    fn prompt_with_a_label_ends_it_with_a_colon() {
        let question = Text::new("Title");

        let prompt = question.prompt();

        assert_eq!(prompt, "Title: ");
    }

    #[test]
    fn prompt_with_a_label_that_ends_with_a_colon_adds_no_second_one() {
        let question = Text::new("Title:");

        let prompt = question.prompt();

        assert_eq!(prompt, "Title: ");
    }

    #[test]
    fn prompt_with_a_question_adds_no_colon() {
        let question = Text::new("What is the title?");

        let prompt = question.prompt();

        assert_eq!(prompt, "What is the title? ");
    }
}
