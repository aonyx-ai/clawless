use std::fmt;

use clawless::prelude::*;

/// Arguments for the `count` command
///
/// Accepts a sentence as a single positional argument. The command splits the sentence on
/// whitespace and counts the resulting words.
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Args)]
pub struct CountArgs {
    /// Sentence to count words in
    sentence: String,
}

/// Word count produced by the `count` command
///
/// In text mode, displays as a plain number (e.g. `4`). In JSON mode, serializes as a structured
/// object (e.g. `{"words":4}`), demonstrating how [`Output::artifact`] adapts to the output mode.
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Serialize)]
struct Count {
    /// Number of whitespace-separated words in the sentence
    words: usize,
}

impl fmt::Display for Count {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.words)
    }
}

/// Count whitespace-separated words in a sentence
///
/// Demonstrates all three [`Output`] methods: [`Output::detail`] logs the raw input,
/// [`Output::message`] describes the operation, and [`Output::artifact`] emits the word count.
#[command]
// A command's doc comment is its `--help` text, so an `# Errors` section would render as a
// raw Markdown heading in the terminal rather than documenting an API.
#[allow(clippy::missing_errors_doc)]
pub async fn count(args: CountArgs, context: Context) -> CommandResult {
    let count = Count {
        words: args.sentence.split_whitespace().count(),
    };

    detail!("input: {}", args.sentence);
    message!("counting words");
    artifact!(count);

    Ok(())
}
