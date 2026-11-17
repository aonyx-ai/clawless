use std::fmt;

use clawless::event::{Styling, TextForm};
use clawless::prelude::*;

/// Arguments for the `check` command
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Args)]
pub struct CheckArgs {
    /// Word to check
    word: String,
}

/// The word that the `check` command checks, in bold when styled
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
struct Subject<'a> {
    /// The word as the user typed it
    word: &'a str,
}

impl TextForm for Subject<'_> {
    fn fmt_text(&self, f: &mut fmt::Formatter<'_>, styling: Styling) -> fmt::Result {
        let Subject { word } = self;

        match styling {
            Styling::Styled => write!(f, "checking \x1b[1m{word}\x1b[0m"),
            Styling::Plain => write!(f, "checking {word}"),
        }
    }
}

/// Verdict of the `check` command
///
/// In text mode, the verdict is a green `yes` or a red `no` on a terminal, and the bare word
/// elsewhere. In JSON mode, it serializes as `{"palindrome":true}`.
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Serialize)]
struct Verdict {
    /// Whether the word reads the same backwards
    palindrome: bool,
}

impl TextForm for Verdict {
    fn fmt_text(&self, f: &mut fmt::Formatter<'_>, styling: Styling) -> fmt::Result {
        let (word, color) = match self.palindrome {
            true => ("yes", "32"),
            false => ("no", "31"),
        };

        match styling {
            Styling::Styled => write!(f, "\x1b[{color}m{word}\x1b[0m"),
            Styling::Plain => f.write_str(word),
        }
    }
}

/// Check whether a word is a palindrome
///
/// Demonstrates a styled message, sent with [`message!`] and a value, and a styled artifact.
#[command]
// A command's doc comment is its `--help` text, so an `# Errors` section would render as a
// raw Markdown heading in the terminal rather than documenting an API.
#[allow(clippy::missing_errors_doc)]
pub async fn check(args: CheckArgs, context: Context) -> CommandResult {
    let CheckArgs { word } = args;
    let verdict = Verdict {
        palindrome: word.chars().eq(word.chars().rev()),
    };

    message!(Subject { word: &word });
    artifact!(verdict);

    Ok(())
}
