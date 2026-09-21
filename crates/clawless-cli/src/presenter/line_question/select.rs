//! How a terminal asks the user to select one of several options

use clawless_core::prompt::Select;

use super::{LineQuestion, label};

/// Lists the options with a number each, and asks for the number
///
/// The numbers start at one and are aligned to the right. The answer is the position of the
/// option, which starts at zero.
///
/// Only a number in the range is an answer. An empty line and the text of an option are no
/// answers.
impl LineQuestion for Select {
    type Answer = usize;

    fn introduction(&self) -> Option<String> {
        let width = self.options().len().to_string().len();
        let mut introduction = format!("{}\n", label(self.question()));

        for (index, option) in self.options().iter().enumerate() {
            let number = index + 1;
            introduction.push_str(&format!("  {number:>width$}) {option}\n"));
        }

        Some(introduction)
    }

    fn prompt(&self) -> String {
        match self.options().len() {
            0 | 1 => "Enter the number [1]: ".to_owned(),
            count => format!("Enter a number [1-{count}]: "),
        }
    }

    fn parse(&self, line: &str) -> Result<Self::Answer, String> {
        let count = self.options().len();
        let hint = || match count {
            0 | 1 => "Please enter the number 1.".to_owned(),
            count => format!("Please enter a number from 1 to {count}."),
        };

        let Ok(number) = line.trim().parse::<usize>() else {
            return Err(hint());
        };

        if (1..=count).contains(&number) {
            Ok(number - 1)
        } else {
            Err(hint())
        }
    }
}

#[cfg(test)]
mod tests {
    // An assertion in a test panics by design. A `# Panics` section on every test
    // would repeat that and give the reader no information.
    #![allow(clippy::missing_panics_doc)]

    use clawless_core::prompt::Select;

    use crate::presenter::line_question::LineQuestion;

    fn kinds() -> Select {
        Select::new("Kind of change", ["Added", "Changed", "Fixed"])
    }

    #[test]
    fn introduction_lists_the_options_with_their_numbers() {
        let question = kinds();

        let introduction = question.introduction();

        assert_eq!(
            introduction,
            Some("Kind of change:\n  1) Added\n  2) Changed\n  3) Fixed\n".to_owned())
        );
    }

    #[test]
    fn introduction_with_ten_options_aligns_the_numbers() {
        let question = Select::new("Digit", (0..10).map(|digit| digit.to_string()));

        let introduction = question.introduction().unwrap_or_default();

        assert_eq!(
            introduction.lines().skip(9).collect::<Vec<_>>(),
            vec!["   9) 8", "  10) 9"]
        );
    }

    #[test]
    fn parse_with_a_number_beyond_the_options_returns_a_hint() {
        let question = kinds();

        let hint = question.parse("4").expect_err("should not parse");

        assert_eq!(hint, "Please enter a number from 1 to 3.");
    }

    #[test]
    fn parse_with_a_number_returns_the_position_of_the_option() {
        let question = kinds();

        let answer = question.parse("3").expect("should parse");

        assert_eq!(answer, 2);
    }

    #[test]
    fn parse_with_an_empty_line_returns_a_hint() {
        let question = kinds();

        let hint = question.parse("").expect_err("should not parse");

        assert_eq!(hint, "Please enter a number from 1 to 3.");
    }

    #[test]
    fn parse_with_space_around_the_number_returns_the_position() {
        let question = kinds();

        let answer = question.parse(" 1 ").expect("should parse");

        assert_eq!(answer, 0);
    }

    #[test]
    fn parse_with_text_returns_a_hint() {
        let question = kinds();

        let hint = question.parse("Fixed").expect_err("should not parse");

        assert_eq!(hint, "Please enter a number from 1 to 3.");
    }

    #[test]
    fn parse_with_zero_returns_a_hint() {
        let question = kinds();

        let hint = question.parse("0").expect_err("should not parse");

        assert_eq!(hint, "Please enter a number from 1 to 3.");
    }

    #[test]
    fn prompt_names_the_range_of_the_numbers() {
        let question = kinds();

        let prompt = question.prompt();

        assert_eq!(prompt, "Enter a number [1-3]: ");
    }

    #[test]
    fn prompt_with_one_option_names_that_number() {
        let question = Select::new("Kind of change", ["Added"]);

        let prompt = question.prompt();

        assert_eq!(prompt, "Enter the number [1]: ");
    }
}
