use std::fmt;

use clawless::prelude::*;
use clawless::prompt::PromptUserError;

/// Arguments for the `select` command
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Args)]
pub struct SelectArgs {}

/// The kind of a change, as a changelog groups them
///
/// The user sees the `Display` text of each kind, and the prompt returns the variant.
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
enum Kind {
    /// A new feature
    Added,

    /// A change to an existing feature
    Changed,

    /// A fix for a fault
    Fixed,
}

impl Kind {
    /// Every kind, in the order in which the user sees them
    const ALL: [Self; 3] = [Self::Added, Self::Changed, Self::Fixed];
}

impl fmt::Display for Kind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Added => write!(f, "Added"),
            Self::Changed => write!(f, "Changed"),
            Self::Fixed => write!(f, "Fixed"),
        }
    }
}

/// Ask the user to select one of several options
///
/// The command asks for the kind of a change. The options are the variants of an enum. Type the
/// number of an option and press Enter.
#[command]
// A command's doc comment is its `--help` text, so an `# Errors` section would render as a
// raw Markdown heading in the terminal rather than documenting an API.
#[allow(clippy::missing_errors_doc)]
pub async fn select(_args: SelectArgs, context: Context) -> CommandResult {
    match ask(&context.prompt()).await {
        Ok(kind) => message!("The kind of the change is \"{kind}\"."),
        Err(PromptUserError::AbsentUser { .. }) => {
            message!("Nobody can select a kind, because no terminal is present.");
        }
        Err(PromptUserError::CancelledPrompt { .. }) => message!("Cancelled."),
        Err(error) => return Err(error).context("ask for the kind of the change"),
    }

    Ok(())
}

/// Asks the user for the kind of a change
///
/// The function is separate from the command so that a test can answer it from a script.
///
/// # Errors
///
/// Returns the error of the prompt if the user gives no answer.
async fn ask(prompt: &Prompt) -> Result<Kind, PromptUserError> {
    prompt.select("Kind of change", Kind::ALL).await
}

#[cfg(test)]
mod tests {
    // An assertion in a test panics by design. A `# Panics` section on every test
    // would repeat that and give the reader no information.
    #![allow(clippy::missing_panics_doc)]

    use clawless::event::event_channel;
    use clawless::prompt::{ScriptedAnswer, ScriptedUser};

    use super::*;

    #[tokio::test]
    async fn ask_with_a_user_who_selects_the_last_option_returns_fixed() {
        let (sender, receiver) = event_channel();
        let user = ScriptedUser::new([ScriptedAnswer::Select(2)]);
        tokio::spawn(user.attend(receiver));
        let prompt = Prompt::builder()
            .output(Output::new(sender))
            .interactivity(Interactivity::Interactive)
            .build();

        let kind = ask(&prompt).await.expect("should answer");

        assert_eq!(kind, Kind::Fixed);
    }
}
