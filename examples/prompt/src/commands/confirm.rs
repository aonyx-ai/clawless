use clawless::prelude::*;
use clawless::prompt::{Confirm, PromptUserError};

/// Arguments for the `confirm` command
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Args)]
pub struct ConfirmArgs {
    /// Version to release
    #[arg(default_value = "1.4.0")]
    version: String,
}

/// Ask the user to confirm before a step that cannot be undone
///
/// The command pretends to release a version. The default answer is No. Press Enter for the
/// default, or type `y` and press Enter to release. Press Ctrl+C to cancel.
///
/// Without a terminal, the command reports that nobody can answer:
///
/// ```shell
/// prompt confirm < /dev/null
/// ```
#[command]
// A command's doc comment is its `--help` text, so an `# Errors` section would render as a
// raw Markdown heading in the terminal rather than documenting an API.
#[allow(clippy::missing_errors_doc)]
pub async fn confirm(args: ConfirmArgs, context: Context) -> CommandResult {
    let ConfirmArgs { version } = args;

    message!("About to release version {version}.");

    match ask(&context.prompt(), &version).await {
        Ok(Confirmation::Yes) => message!("Released version {version}."),
        Ok(Confirmation::No) => message!("Nothing was released."),
        Err(PromptUserError::AbsentUser { .. }) => {
            message!("Nobody can confirm the release, because no terminal is present.");
        }
        Err(PromptUserError::CancelledPrompt { .. }) => {
            message!("Cancelled. Nothing was released.")
        }
        Err(error) => return Err(error).context("ask for the confirmation"),
    }

    Ok(())
}

/// Asks the user whether to release the version
///
/// The function is separate from the command so that a test can answer it from a script.
///
/// # Errors
///
/// Returns the error of the prompt if the user gives no answer.
async fn ask(prompt: &Prompt, version: &str) -> Result<Confirmation, PromptUserError> {
    let question =
        Confirm::new(format!("Release version {version}?")).with_default(Confirmation::No);

    prompt.confirm(question).await
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
    async fn ask_with_a_user_who_agrees_returns_yes() {
        let (sender, receiver) = event_channel();
        let user = ScriptedUser::new([ScriptedAnswer::Confirm(Confirmation::Yes)]);
        tokio::spawn(user.attend(receiver));
        let prompt = Prompt::builder()
            .output(Output::new(sender))
            .interactivity(Interactivity::Interactive)
            .build();

        let answer = ask(&prompt, "1.4.0").await.expect("should answer");

        assert_eq!(answer, Confirmation::Yes);
    }
}
