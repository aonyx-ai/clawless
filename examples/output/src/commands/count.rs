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
/// object (e.g. `{"words":4}`), demonstrating how [`Output::result`] adapts to the output mode.
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Serialize)]
struct Count {
    words: usize,
}

impl fmt::Display for Count {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.words)
    }
}

/// Count whitespace-separated words in a sentence
///
/// Demonstrates all three [`Output`] methods: [`Output::verbose`] logs the raw input,
/// [`Output::print`] describes the operation, and [`Output::result`] emits the word count.
#[command]
pub async fn count(args: CountArgs, context: Context) -> CommandResult {
    let output = context.output();
    let count = Count {
        words: args.sentence.split_whitespace().count(),
    };

    output.verbose(format!("input: {}", args.sentence));
    output.print("counting words");
    output.result(&count);

    Ok(())
}
