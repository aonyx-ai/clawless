//! Questions that a command asks its user
//!
//! [`Prompt`] is the interface that asks. [`Confirm`] is a question that the user answers with
//! yes or no, and [`Confirmation`] is the answer. [`PromptUserError`] is the reason why a command
//! received no answer.
//!
//! A question reaches the presenter as a [`PromptRequest`], and the presenter answers it.
//! [`AnswerPromptError`] is the reason that the presenter reports when it has no answer.
//!
//! # Examples
//!
//! ```no_run
//! use clawless_core::prompt::{Confirm, Confirmation, Prompt};
//!
//! # async fn example(prompt: Prompt) -> Result<(), Box<dyn std::error::Error>> {
//! let question = Confirm::new("Delete the branch?").with_default(Confirmation::No);
//!
//! match prompt.confirm(question).await? {
//!     Confirmation::Yes => println!("deleting"),
//!     Confirmation::No => println!("keeping"),
//! }
//! # Ok(())
//! # }
//! ```

use bon::Builder;
use getset::CopyGetters;

pub use self::answer_prompt_error::AnswerPromptError;
pub use self::confirm::Confirm;
pub use self::confirmation::Confirmation;
pub use self::prompt_user_error::PromptUserError;
use crate::cancellation::Cancellation;
use crate::context::Interactivity;
use crate::event::prompt::{PendingAnswer, PromptRequest};
use crate::output::Output;

/// The error that ends a prompt without an answer from the user
mod answer_prompt_error;
/// A question that a user answers with yes or no
mod confirm;
/// The answer of a user to a confirmation
mod confirmation;
/// The error returned when a command cannot get an answer from its user
mod prompt_user_error;

/// Asks the user of the application a question and waits for the answer
///
/// `Prompt` holds the [`Output`] that sends a question to the presenter, the [`Cancellation`]
/// that stops the wait, and the [`Interactivity`] of the application. [`Context::prompt`] builds
/// one from the context of a command.
///
/// A question arrives at the presenter after the output that the command sent before it.
///
/// Without a user, a prompt returns [`PromptUserError::AbsentUser`] immediately and sends
/// nothing. It never applies a default. [`interactivity`] reports whether a user is present.
///
/// Cancellation stops the wait, and the prompt returns [`PromptUserError::CancelledPrompt`].
///
/// `Prompt` is cheaply clonable. Cloning produces another handle to the same event channel and
/// the same cancellation token.
///
/// # Examples
///
/// ```no_run
/// use clawless_core::prompt::{Confirmation, Prompt};
///
/// # async fn example(prompt: Prompt) -> Result<(), Box<dyn std::error::Error>> {
/// let answer = prompt.confirm("Delete the branch?").await?;
///
/// let delete = match answer {
///     Confirmation::Yes => true,
///     Confirmation::No => false,
/// };
/// # Ok(())
/// # }
/// ```
///
/// [`Context::prompt`]: crate::context::Context::prompt
/// [`interactivity`]: Prompt::interactivity
#[derive(Clone, Debug, Builder, CopyGetters)]
pub struct Prompt {
    /// The channel that carries a question to the presenter
    output: Output,

    /// The token that stops the wait for an answer
    #[builder(default)]
    cancellation: Cancellation,

    /// Whether a user is present who can answer
    #[builder(default)]
    #[getset(get_copy = "pub")]
    interactivity: Interactivity,
}

impl Prompt {
    /// Asks the user to confirm with yes or no
    ///
    /// The question is a [`Confirm`], or a text that converts into one without a default.
    ///
    /// # Errors
    ///
    /// Returns [`PromptUserError::AbsentUser`] if no user is present, and
    /// [`PromptUserError::CancelledPrompt`] if cancellation stopped the wait. Returns
    /// [`PromptUserError::UndeliverablePrompt`] if the presenter stopped listening, and
    /// [`PromptUserError::UnansweredPrompt`] if the prompt came back without an answer.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use clawless_core::prompt::{Confirm, Confirmation, Prompt};
    ///
    /// # async fn example(prompt: Prompt) -> Result<(), Box<dyn std::error::Error>> {
    /// let question = Confirm::new("Publish the release?").with_default(Confirmation::No);
    ///
    /// let answer = prompt.confirm(question).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn confirm(
        &self,
        question: impl Into<Confirm>,
    ) -> Result<Confirmation, PromptUserError> {
        let question = question.into();
        let text = question.question().clone();
        let (request, pending) = PromptRequest::confirm(question);

        self.ask(text, request, pending).await
    }

    /// Sends a request to the presenter and waits for the answer to it
    ///
    /// The send and the wait both race the cancellation token, and cancellation wins a tie. A
    /// token that is already cancelled therefore stops the prompt before it sends a request. The
    /// send races too, because a full channel makes the sender wait.
    ///
    /// Cancellation drops the pending answer, which tells the presenter that the command has
    /// stopped waiting.
    ///
    /// # Errors
    ///
    /// Returns [`PromptUserError::AbsentUser`] if no user is present, and
    /// [`PromptUserError::CancelledPrompt`] if cancellation stopped the send or the wait. Returns
    /// [`PromptUserError::UndeliverablePrompt`] if the presenter stopped listening, and
    /// [`PromptUserError::UnansweredPrompt`] if the prompt came back without an answer.
    async fn ask<T>(
        &self,
        question: String,
        request: PromptRequest,
        pending: PendingAnswer<T>,
    ) -> Result<T, PromptUserError> {
        match self.interactivity {
            Interactivity::Interactive => {}
            Interactivity::NonInteractive => return Err(PromptUserError::AbsentUser { question }),
        }

        let sent = tokio::select! {
            biased;

            () = self.cancellation.cancelled() => {
                return Err(PromptUserError::CancelledPrompt { question });
            }
            sent = self.output.prompt(request) => sent,
        };
        if let Err(source) = sent {
            return Err(PromptUserError::UndeliverablePrompt { question, source });
        }

        let answer = tokio::select! {
            biased;

            () = self.cancellation.cancelled() => {
                return Err(PromptUserError::CancelledPrompt { question });
            }
            answer = pending.wait() => answer,
        };

        answer.map_err(|source| PromptUserError::UnansweredPrompt { question, source })
    }
}

#[cfg(test)]
mod tests {
    // An assertion in a test panics by design. A `# Panics` section on every test
    // would repeat that and give the reader no information.
    #![allow(clippy::missing_panics_doc)]

    use super::*;
    use crate::event::{Event, EventReceiver, event_channel};

    /// Returns a prompt whose user is present, and the receiver that stands for the presenter
    fn interactive() -> (Prompt, EventReceiver) {
        let (sender, receiver) = event_channel();
        let prompt = Prompt::builder()
            .output(Output::new(sender))
            .interactivity(Interactivity::Interactive)
            .build();

        (prompt, receiver)
    }

    /// Returns the next prompt request that the receiver holds
    async fn next_request(receiver: &mut EventReceiver) -> Option<PromptRequest> {
        while let Some(event) = receiver.recv().await {
            match event {
                Event::Prompt(request) => return Some(*request),
                Event::Message(_) | Event::Detail(_) | Event::Artifact(_) | Event::Process(_) => {}
            }
        }

        None
    }

    /// Returns whether the error reports a prompt without a user
    fn is_absent_user(error: &PromptUserError) -> bool {
        match error {
            PromptUserError::AbsentUser { .. } => true,
            PromptUserError::CancelledPrompt { .. } => false,
            PromptUserError::UnansweredPrompt { .. } => false,
            PromptUserError::UndeliverablePrompt { .. } => false,
        }
    }

    /// Returns whether the error reports a prompt that cancellation stopped
    fn is_cancelled(error: &PromptUserError) -> bool {
        match error {
            PromptUserError::CancelledPrompt { .. } => true,
            PromptUserError::AbsentUser { .. } => false,
            PromptUserError::UnansweredPrompt { .. } => false,
            PromptUserError::UndeliverablePrompt { .. } => false,
        }
    }

    /// Returns whether the error reports a prompt that the presenter dropped
    fn is_dropped(error: &PromptUserError) -> bool {
        match error {
            PromptUserError::UnansweredPrompt { source, .. } => match source {
                AnswerPromptError::DroppedRequest => true,
                AnswerPromptError::ClosedInput => false,
                AnswerPromptError::UnusableTerminal { .. } => false,
            },
            PromptUserError::AbsentUser { .. } => false,
            PromptUserError::CancelledPrompt { .. } => false,
            PromptUserError::UndeliverablePrompt { .. } => false,
        }
    }

    /// Returns whether the error reports a prompt that could not reach the presenter
    fn is_undeliverable(error: &PromptUserError) -> bool {
        match error {
            PromptUserError::UndeliverablePrompt { .. } => true,
            PromptUserError::AbsentUser { .. } => false,
            PromptUserError::CancelledPrompt { .. } => false,
            PromptUserError::UnansweredPrompt { .. } => false,
        }
    }

    #[tokio::test]
    async fn confirm_sends_the_question_with_its_default() {
        let (prompt, mut receiver) = interactive();
        let question = Confirm::new("Release?").with_default(Confirmation::No);
        let asked = question.clone();
        let command = tokio::spawn(async move { prompt.confirm(asked).await });

        let request = next_request(&mut receiver).await;
        command.abort();

        assert_eq!(
            request.map(|request| match request {
                PromptRequest::Confirm { question, .. } => question,
            }),
            Some(question)
        );
    }

    #[tokio::test]
    async fn confirm_with_a_cancelled_token_returns_an_error() {
        let cancellation = Cancellation::new();
        let (sender, _receiver) = event_channel();
        let prompt = Prompt::builder()
            .output(Output::new(sender))
            .cancellation(cancellation.clone())
            .interactivity(Interactivity::Interactive)
            .build();
        cancellation.cancel();

        let error = prompt.confirm("Release?").await.expect_err("should fail");

        assert!(is_cancelled(&error));
    }

    #[tokio::test]
    async fn confirm_with_a_cancelled_token_sends_no_request() {
        let cancellation = Cancellation::new();
        let (sender, mut receiver) = event_channel();
        let prompt = Prompt::builder()
            .output(Output::new(sender))
            .cancellation(cancellation.clone())
            .interactivity(Interactivity::Interactive)
            .build();
        cancellation.cancel();

        drop(prompt.confirm("Release?").await);
        drop(prompt);

        assert!(next_request(&mut receiver).await.is_none());
    }

    #[tokio::test]
    async fn confirm_with_a_closed_channel_returns_an_error() {
        let (prompt, receiver) = interactive();
        drop(receiver);

        let error = prompt.confirm("Release?").await.expect_err("should fail");

        assert!(is_undeliverable(&error));
    }

    #[tokio::test]
    async fn confirm_with_a_dropped_request_returns_an_error() {
        let (prompt, mut receiver) = interactive();
        let presenter = tokio::spawn(async move { drop(next_request(&mut receiver).await) });

        let error = prompt.confirm("Release?").await.expect_err("should fail");
        presenter.await.expect("should join");

        assert!(is_dropped(&error));
    }

    #[tokio::test]
    async fn confirm_with_an_answer_returns_it() {
        let (prompt, mut receiver) = interactive();
        let presenter = tokio::spawn(async move {
            match next_request(&mut receiver).await {
                Some(PromptRequest::Confirm { reply, .. }) => reply.answer(Confirmation::Yes),
                None => {}
            }
        });

        let answer = prompt.confirm("Release?").await.expect("should answer");
        presenter.await.expect("should join");

        assert_eq!(answer, Confirmation::Yes);
    }

    #[tokio::test]
    async fn confirm_with_cancellation_while_it_waits_abandons_the_reply() {
        let cancellation = Cancellation::new();
        let (sender, mut receiver) = event_channel();
        let prompt = Prompt::builder()
            .output(Output::new(sender))
            .cancellation(cancellation.clone())
            .interactivity(Interactivity::Interactive)
            .build();
        let command = tokio::spawn(async move { prompt.confirm("Release?").await });
        let request = next_request(&mut receiver).await;
        cancellation.cancel();

        drop(command.await.expect("should join"));

        assert_eq!(
            request.map(|request| match request {
                PromptRequest::Confirm { reply, .. } => reply.is_abandoned(),
            }),
            Some(true)
        );
    }

    #[tokio::test]
    async fn confirm_with_cancellation_while_it_waits_returns_an_error() {
        let cancellation = Cancellation::new();
        let (sender, mut receiver) = event_channel();
        let prompt = Prompt::builder()
            .output(Output::new(sender))
            .cancellation(cancellation.clone())
            .interactivity(Interactivity::Interactive)
            .build();
        let command = tokio::spawn(async move { prompt.confirm("Release?").await });
        let _request = next_request(&mut receiver).await;
        cancellation.cancel();

        let error = command
            .await
            .expect("should join")
            .expect_err("should fail");

        assert!(is_cancelled(&error));
    }

    #[tokio::test]
    async fn confirm_without_a_user_returns_an_error() {
        let (sender, _receiver) = event_channel();
        let prompt = Prompt::builder().output(Output::new(sender)).build();

        let error = prompt.confirm("Release?").await.expect_err("should fail");

        assert!(is_absent_user(&error));
    }

    #[tokio::test]
    async fn confirm_without_a_user_sends_no_request() {
        let (sender, mut receiver) = event_channel();
        let prompt = Prompt::builder().output(Output::new(sender)).build();

        drop(prompt.confirm("Release?").await);
        drop(prompt);

        assert!(next_request(&mut receiver).await.is_none());
    }

    #[test]
    fn interactivity_with_defaults_is_non_interactive() {
        let (sender, _receiver) = event_channel();

        let prompt = Prompt::builder().output(Output::new(sender)).build();

        assert_eq!(prompt.interactivity(), Interactivity::NonInteractive);
    }

    #[test]
    fn trait_send() {
        fn assert_send<T: Send>() {}
        assert_send::<Prompt>();
    }

    #[test]
    fn trait_sync() {
        fn assert_sync<T: Sync>() {}
        assert_sync::<Prompt>();
    }

    #[test]
    fn trait_unpin() {
        fn assert_unpin<T: Unpin>() {}
        assert_unpin::<Prompt>();
    }
}
