use clawless::prelude::*;
use clawless::prompt::PromptUserError;

/// Arguments for the `text` command
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Args)]
pub struct TextArgs {}

/// Ask the user for one line of text
///
/// The command asks for the title of a change. An empty line is an answer too, so the command
/// asks again until the title has a character that is not a space.
#[command]
// A command's doc comment is its `--help` text, so an `# Errors` section would render as a
// raw Markdown heading in the terminal rather than documenting an API.
#[allow(clippy::missing_errors_doc)]
pub async fn text(_args: TextArgs, context: Context) -> CommandResult {
    match ask(&context).await {
        Ok(title) => message!("The title is \"{title}\"."),
        Err(PromptUserError::AbsentUser { .. }) => {
            message!("Nobody can type a title, because no terminal is present.");
        }
        Err(PromptUserError::CancelledPrompt { .. }) => message!("Cancelled."),
        Err(error) => return Err(error).context("ask for the title"),
    }

    Ok(())
}

/// Asks the user for the title of a change, until the user gives one
///
/// The function sends the message about an empty title before the next question, so the user
/// reads the message first. The function is separate from the command so that a test can answer
/// it from a script.
///
/// # Errors
///
/// Returns the error of the prompt if the user gives no answer.
async fn ask(context: &Context) -> Result<String, PromptUserError> {
    loop {
        let title = context.prompt().text("Title of the change").await?;
        let title = title.trim();

        if !title.is_empty() {
            return Ok(title.to_owned());
        }

        message!("A title needs at least one character.");
    }
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
    async fn ask_with_an_empty_answer_asks_again() {
        let (sender, receiver) = event_channel();
        let user = ScriptedUser::new([
            ScriptedAnswer::Text(String::new()),
            ScriptedAnswer::Text("Fix the race".to_owned()),
        ]);
        tokio::spawn(user.attend(receiver));
        let context = Context::builder()
            .output(Output::new(sender))
            .interactivity(Interactivity::Interactive)
            .build()
            .expect("should build the context");

        let title = ask(&context).await.expect("should answer");

        assert_eq!(title, "Fix the race");
    }
}
