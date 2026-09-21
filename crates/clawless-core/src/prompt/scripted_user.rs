use std::collections::VecDeque;

use super::{AnswerPromptError, ScriptedAnswer};
use crate::event::prompt::PromptRequest;
use crate::event::{Event, EventReceiver};

/// A user for tests, who answers every prompt from a script
///
/// `ScriptedUser` reads the events of a command in place of a presenter. It answers each prompt
/// with the next answer of its script, and it uses each answer once.
///
/// A prompt fails with [`AnswerPromptError::UnscriptedPrompt`] when the script has ended, and
/// when the next answer is for another kind of prompt. That answer is used up as well.
///
/// Build the [`Context`] or the [`Prompt`] of the test with [`Interactivity::Interactive`].
/// Otherwise the prompt fails before it sends a request.
///
/// # Examples
///
/// ```
/// use clawless_core::context::Interactivity;
/// use clawless_core::event::event_channel;
/// use clawless_core::output::Output;
/// use clawless_core::prompt::{Confirmation, Prompt, ScriptedAnswer, ScriptedUser};
///
/// # #[tokio::main]
/// # async fn main() {
/// let (sender, receiver) = event_channel();
/// let user = ScriptedUser::new([ScriptedAnswer::Confirm(Confirmation::Yes)]);
/// tokio::spawn(user.attend(receiver));
///
/// let prompt = Prompt::builder()
///     .output(Output::new(sender))
///     .interactivity(Interactivity::Interactive)
///     .build();
///
/// let answer = prompt.confirm("Delete the branch?").await.expect("should answer");
///
/// assert_eq!(answer, Confirmation::Yes);
/// # }
/// ```
///
/// [`Context`]: crate::context::Context
/// [`Interactivity::Interactive`]: crate::context::Interactivity::Interactive
/// [`Prompt`]: super::Prompt
#[derive(Clone, Eq, PartialEq, Hash, Debug, Default)]
pub struct ScriptedUser {
    /// The answers that the user has not given yet
    answers: VecDeque<ScriptedAnswer>,
}

impl ScriptedUser {
    /// Creates a user who gives the answers in the order of the script
    ///
    /// # Examples
    ///
    /// ```
    /// use clawless_core::prompt::{Confirmation, ScriptedAnswer, ScriptedUser};
    ///
    /// let user = ScriptedUser::new([
    ///     ScriptedAnswer::Confirm(Confirmation::Yes),
    ///     ScriptedAnswer::Confirm(Confirmation::No),
    /// ]);
    /// ```
    pub fn new(answers: impl IntoIterator<Item = ScriptedAnswer>) -> Self {
        Self {
            answers: answers.into_iter().collect(),
        }
    }

    /// Reads the events of a command and answers its prompts, until the command is done
    ///
    /// The call returns when the channel closes, which happens when the command drops its
    /// output. It returns every event that was not a prompt, in the order of arrival.
    ///
    /// Run this in a task of its own, because the command waits for each answer.
    pub async fn attend(mut self, mut receiver: EventReceiver) -> Vec<Event> {
        let mut events = Vec::new();

        while let Some(event) = receiver.recv().await {
            match event {
                Event::Prompt(request) => self.answer(*request),
                Event::Message(_) | Event::Detail(_) | Event::Artifact(_) | Event::Process(_) => {
                    events.push(event);
                }
            }
        }

        events
    }

    /// Answers one prompt with the next answer of the script
    fn answer(&mut self, request: PromptRequest) {
        match (request, self.answers.pop_front()) {
            (PromptRequest::Confirm { reply, .. }, Some(ScriptedAnswer::Confirm(answer))) => {
                reply.answer(answer);
            }
            (PromptRequest::Text { reply, .. }, Some(ScriptedAnswer::Text(answer))) => {
                reply.answer(answer);
            }
            (request @ PromptRequest::Confirm { .. }, Some(ScriptedAnswer::Text(_)) | None) => {
                request.fail(AnswerPromptError::UnscriptedPrompt);
            }
            (request @ PromptRequest::Text { .. }, Some(ScriptedAnswer::Confirm(_)) | None) => {
                request.fail(AnswerPromptError::UnscriptedPrompt);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    // An assertion in a test panics by design. A `# Panics` section on every test
    // would repeat that and give the reader no information.
    #![allow(clippy::missing_panics_doc)]

    use super::*;
    use crate::context::Interactivity;
    use crate::event::event_channel;
    use crate::output::Output;
    use crate::prompt::{Confirmation, Prompt, PromptUserError};

    /// Returns whether the error reports a prompt for which the script held no answer
    fn is_unscripted(error: &PromptUserError) -> bool {
        match error {
            PromptUserError::UnansweredPrompt { source, .. } => match source {
                AnswerPromptError::UnscriptedPrompt => true,
                AnswerPromptError::ClosedInput => false,
                AnswerPromptError::DroppedRequest => false,
                AnswerPromptError::UnusableTerminal { .. } => false,
            },
            PromptUserError::AbsentUser { .. } => false,
            PromptUserError::CancelledPrompt { .. } => false,
            PromptUserError::UndeliverablePrompt { .. } => false,
        }
    }

    /// Returns a prompt whose user follows the script, and the task in which that user attends
    fn scripted(
        answers: impl IntoIterator<Item = ScriptedAnswer>,
    ) -> (Prompt, tokio::task::JoinHandle<Vec<Event>>) {
        let (sender, receiver) = event_channel();
        let user = tokio::spawn(ScriptedUser::new(answers).attend(receiver));
        let prompt = Prompt::builder()
            .output(Output::new(sender))
            .interactivity(Interactivity::Interactive)
            .build();

        (prompt, user)
    }

    #[tokio::test]
    async fn attend_answers_a_confirmation_from_the_script() {
        let (prompt, _user) = scripted([ScriptedAnswer::Confirm(Confirmation::Yes)]);

        let answer = prompt.confirm("Release?").await.expect("should answer");

        assert_eq!(answer, Confirmation::Yes);
    }

    #[tokio::test]
    async fn attend_answers_in_the_order_of_the_script() {
        let (prompt, _user) = scripted([
            ScriptedAnswer::Confirm(Confirmation::Yes),
            ScriptedAnswer::Confirm(Confirmation::No),
        ]);
        drop(prompt.confirm("Release?").await);

        let answer = prompt.confirm("Publish?").await.expect("should answer");

        assert_eq!(answer, Confirmation::No);
    }

    #[tokio::test]
    async fn attend_answers_a_text_prompt_from_the_script() {
        let (prompt, _user) = scripted([ScriptedAnswer::Text("Fix the race".to_owned())]);

        let answer = prompt.text("Title").await.expect("should answer");

        assert_eq!(answer, "Fix the race");
    }

    #[tokio::test]
    async fn attend_returns_the_events_that_are_no_prompts() {
        let (sender, receiver) = event_channel();
        let user = tokio::spawn(ScriptedUser::default().attend(receiver));
        let output = Output::new(sender);
        output.message("released").await.expect("should send");
        drop(output);

        let events = user.await.expect("should join");

        assert_eq!(
            events
                .into_iter()
                .map(|event| match event {
                    Event::Message(text) => text,
                    Event::Detail(text) => text,
                    Event::Artifact(artifact) => artifact.to_string(),
                    Event::Process(event) => event.to_string(),
                    Event::Prompt(request) => format!("{request:?}"),
                })
                .collect::<Vec<_>>(),
            vec!["released".to_owned()]
        );
    }

    #[tokio::test]
    async fn attend_with_an_answer_for_another_kind_fails_the_prompt() {
        let (prompt, _user) = scripted([ScriptedAnswer::Confirm(Confirmation::Yes)]);

        let error = prompt.text("Title").await.expect_err("should fail");

        assert!(is_unscripted(&error));
    }

    #[tokio::test]
    async fn attend_with_an_empty_script_fails_the_prompt() {
        let (prompt, _user) = scripted([]);

        let error = prompt.confirm("Release?").await.expect_err("should fail");

        assert!(is_unscripted(&error));
    }

    #[test]
    fn trait_send() {
        fn assert_send<T: Send>() {}
        assert_send::<ScriptedUser>();
    }

    #[test]
    fn trait_sync() {
        fn assert_sync<T: Sync>() {}
        assert_sync::<ScriptedUser>();
    }

    #[test]
    fn trait_unpin() {
        fn assert_unpin<T: Unpin>() {}
        assert_unpin::<ScriptedUser>();
    }
}
