//! Command output interface
//!
//! This module defines [`Output`], the interface commands use to produce events. `Output` wraps an
//! [`EventSender`] and provides typed methods for each event kind, decoupling commands from the
//! rendering strategy. Commands call [`message`], [`detail`], or [`artifact`] to emit events into
//! the channel; the Presenter consumes them and decides how to render.
//!
//! [`process_event`] carries the steps of an external program that a command runs. Commands do not
//! usually call it, because [`Process`] reports a run through it.
//!
//! [`prompt`] sends a question for the user.
//!
//! [`Process`]: crate::process::Process
//! [`artifact`]: Output::artifact
//! [`detail`]: Output::detail
//! [`message`]: Output::message
//! [`process_event`]: Output::process_event
//! [`prompt`]: Output::prompt

use crate::event::process::ProcessEvent;
use crate::event::prompt::PromptRequest;
use crate::event::{Artifact, Event, EventSender, FormattedText, SendError, TextForm};

/// Command output interface that wraps an event channel sender
///
/// `Output` provides typed methods for each event kind, decoupling commands from the event channel
/// API. Commands call [`message`], [`detail`], or [`artifact`] to emit events; the Presenter
/// consumes them for rendering.
///
/// `Output` is cheaply clonable. Cloning an `Output` produces another handle to the same
/// underlying channel, not an independent channel.
///
/// # Examples
///
/// ```
/// use clawless_core::event::event_channel;
/// use clawless_core::output::Output;
///
/// # #[tokio::main]
/// # async fn main() {
/// let (sender, mut receiver) = event_channel();
/// let output = Output::new(sender);
///
/// output.message("hello").await.expect("should send");
/// # }
/// ```
///
/// [`artifact`]: Output::artifact
/// [`detail`]: Output::detail
/// [`message`]: Output::message
#[derive(Clone, Debug)]
pub struct Output {
    /// Channel that carries the emitted events
    sender: EventSender,
}

impl Output {
    /// Creates a new `Output` wrapping the given event sender
    ///
    /// # Examples
    ///
    /// ```
    /// use clawless_core::event::event_channel;
    /// use clawless_core::output::Output;
    ///
    /// let (sender, _receiver) = event_channel();
    /// let output = Output::new(sender);
    /// ```
    pub fn new(sender: EventSender) -> Self {
        Self { sender }
    }

    /// Sends an informational message event
    ///
    /// Writes the [`TextForm`] of the value in every [`Styling`] and sends the text as an
    /// [`Event::Message`]. Every value with [`Display`] has one form. A value with a styled form
    /// implements [`TextForm`] itself, and the presenter chooses which form it writes. The value
    /// does not have to outlive the call, because the text is written before the event is sent.
    ///
    /// # Errors
    ///
    /// Returns [`SendError`] if the [`EventReceiver`] has been dropped.
    ///
    /// # Examples
    ///
    /// ```
    /// use clawless_core::event::event_channel;
    /// use clawless_core::output::Output;
    ///
    /// # #[tokio::main]
    /// # async fn main() {
    /// let (sender, _receiver) = event_channel();
    /// let output = Output::new(sender);
    ///
    /// output.message("processing files").await.expect("should send");
    /// output.message(format!("found {} items", 42)).await.expect("should send");
    /// # }
    /// ```
    ///
    /// [`Display`]: std::fmt::Display
    /// [`EventReceiver`]: crate::event::EventReceiver
    /// [`Styling`]: crate::event::Styling
    /// [`TextForm`]: crate::event::TextForm
    pub async fn message(&self, message: impl TextForm) -> Result<(), SendError> {
        let message = FormattedText::new(&message);

        self.sender.send(Event::Message(message)).await
    }

    /// Sends a supplementary detail event
    ///
    /// Writes the [`TextForm`] of the value in every [`Styling`] and sends the text as an
    /// [`Event::Detail`], as [`message`] does. Details carry lower-priority information that the
    /// Presenter may suppress at default verbosity.
    ///
    /// # Errors
    ///
    /// Returns [`SendError`] if the [`EventReceiver`] has been dropped.
    ///
    /// # Examples
    ///
    /// ```
    /// use clawless_core::event::event_channel;
    /// use clawless_core::output::Output;
    ///
    /// # #[tokio::main]
    /// # async fn main() {
    /// let (sender, _receiver) = event_channel();
    /// let output = Output::new(sender);
    ///
    /// output.detail("reading config from ~/.config/app.toml").await.expect("should send");
    /// # }
    /// ```
    ///
    /// [`EventReceiver`]: crate::event::EventReceiver
    /// [`Styling`]: crate::event::Styling
    /// [`TextForm`]: crate::event::TextForm
    /// [`message`]: Output::message
    pub async fn detail(&self, detail: impl TextForm) -> Result<(), SendError> {
        let detail = FormattedText::new(&detail);

        self.sender.send(Event::Detail(detail)).await
    }

    /// Sends a structured artifact event
    ///
    /// Wraps the value in a [`Box`] and sends it as an [`Event::Artifact`]. The value must
    /// implement [`Artifact`]. Any value that implements [`Display`] (for text rendering),
    /// [`Serialize`] (for JSON rendering), and [`Debug`] (for diagnostics) does. A value with a
    /// styled text form implements [`TextForm`] instead of [`Display`], and the presenter chooses
    /// which form it writes. A `Box<dyn Artifact>` is an artifact as well, so a command can pass
    /// on an artifact that it received as a box.
    ///
    /// # Errors
    ///
    /// Returns [`SendError`] if the [`EventReceiver`] has been dropped.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::fmt;
    ///
    /// use serde::Serialize;
    ///
    /// use clawless_core::event::event_channel;
    /// use clawless_core::output::Output;
    ///
    /// #[derive(Clone, Debug, Serialize)]
    /// struct UserCount {
    ///     count: usize,
    /// }
    ///
    /// impl fmt::Display for UserCount {
    ///     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    ///         write!(f, "{} users", self.count)
    ///     }
    /// }
    ///
    /// # #[tokio::main]
    /// # async fn main() {
    /// let (sender, _receiver) = event_channel();
    /// let output = Output::new(sender);
    ///
    /// output.artifact(UserCount { count: 42 }).await.expect("should send");
    /// # }
    /// ```
    ///
    /// [`Debug`]: std::fmt::Debug
    /// [`Display`]: std::fmt::Display
    /// [`EventReceiver`]: crate::event::EventReceiver
    /// [`Serialize`]: serde::Serialize
    /// [`TextForm`]: crate::event::TextForm
    pub async fn artifact<T>(&self, value: T) -> Result<(), SendError>
    where
        T: Artifact + 'static,
    {
        self.sender.send(Event::Artifact(Box::new(value))).await
    }

    /// Sends one step in the run of an external program
    ///
    /// [`Process`] calls this for every step of a run, so a command that runs a program through
    /// Clawless does not call it. An application that drives a program itself, and wants its
    /// output to reach the presenter in the same shape, reports the run through this method.
    ///
    /// # Errors
    ///
    /// Returns [`SendError`] if the [`EventReceiver`] has been dropped.
    ///
    /// # Examples
    ///
    /// ```
    /// use clawless_core::event::event_channel;
    /// use clawless_core::event::process::{ProcessEvent, RunId};
    /// use clawless_core::output::Output;
    /// use clawless_core::process::Invocation;
    ///
    /// # #[tokio::main]
    /// # async fn main() {
    /// let (sender, _receiver) = event_channel();
    /// let output = Output::new(sender);
    ///
    /// output
    ///     .process_event(ProcessEvent::Started {
    ///         id: RunId::next(),
    ///         invocation: Invocation::new("git").arg("status"),
    ///         process_id: None,
    ///     })
    ///     .await
    ///     .expect("should send");
    /// # }
    /// ```
    ///
    /// [`EventReceiver`]: crate::event::EventReceiver
    /// [`Process`]: crate::process::Process
    pub async fn process_event(&self, event: ProcessEvent) -> Result<(), SendError> {
        self.sender.send(Event::Process(Box::new(event))).await
    }

    /// Sends a question for the user
    ///
    /// The request arrives after every event that this output sent before it. The answer
    /// arrives at the [`PendingAnswer`] that was created together with the request.
    ///
    /// # Errors
    ///
    /// Returns [`SendError`] if the [`EventReceiver`] has been dropped.
    ///
    /// # Examples
    ///
    /// ```
    /// use clawless_core::event::event_channel;
    /// use clawless_core::event::prompt::PromptRequest;
    /// use clawless_core::output::Output;
    /// use clawless_core::prompt::Confirm;
    ///
    /// # #[tokio::main]
    /// # async fn main() {
    /// let (sender, _receiver) = event_channel();
    /// let output = Output::new(sender);
    /// let (request, _pending) = PromptRequest::confirm(Confirm::new("Delete the branch?"));
    ///
    /// output.prompt(request).await.expect("should send");
    /// # }
    /// ```
    ///
    /// [`EventReceiver`]: crate::event::EventReceiver
    /// [`PendingAnswer`]: crate::event::prompt::PendingAnswer
    pub async fn prompt(&self, request: PromptRequest) -> Result<(), SendError> {
        self.sender.send(Event::Prompt(Box::new(request))).await
    }
}

#[cfg(test)]
mod tests {
    // An assertion in a test panics by design. A `# Panics` section on every test
    // would repeat that and give the reader no information.
    #![allow(clippy::missing_panics_doc)]

    use std::fmt;

    use serde::Serialize;

    use super::*;
    use crate::event::process::RunId;
    use crate::event::{Styling, event_channel};
    use crate::process::Invocation;
    use crate::prompt::Confirm;
    use crate::test_support::Bold;

    #[derive(Clone, Debug, Serialize)]
    struct TestArtifact {
        value: String,
    }

    impl fmt::Display for TestArtifact {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(f, "{}", self.value)
        }
    }

    #[tokio::test]
    async fn artifact_sends_artifact_event() {
        let (sender, mut receiver) = event_channel();
        let output = Output::new(sender);

        output
            .artifact(TestArtifact {
                value: "result".to_string(),
            })
            .await
            .expect("should send");

        let event = receiver.recv().await.expect("should receive");

        let Event::Artifact(artifact) = event else {
            panic!("expected Event::Artifact, received {event:?}");
        };
        assert_eq!(artifact.to_text(Styling::Plain), "result");
    }

    #[tokio::test]
    async fn artifact_with_a_boxed_artifact_sends_the_styled_form_of_the_value() {
        let (sender, mut receiver) = event_channel();
        let output = Output::new(sender);
        let boxed: Box<dyn Artifact> = Box::new(Bold::new("result"));

        output.artifact(boxed).await.expect("should send");

        let event = receiver.recv().await.expect("should receive");
        let Event::Artifact(artifact) = event else {
            panic!("expected Event::Artifact, received {event:?}");
        };
        assert_eq!(artifact.to_text(Styling::Styled), "\x1b[1mresult\x1b[0m");
    }

    #[tokio::test]
    async fn artifact_to_closed_channel_returns_error() {
        let (sender, receiver) = event_channel();
        let output = Output::new(sender);
        drop(receiver);

        let error = output
            .artifact(TestArtifact {
                value: "lost".to_string(),
            })
            .await
            .expect_err("should fail");

        assert_eq!(error.to_string(), "event channel closed");
    }

    #[tokio::test]
    async fn clone_produces_independent_handle() {
        let (sender, mut receiver) = event_channel();
        let output = Output::new(sender);
        let cloned = output.clone();

        output.message("from original").await.expect("should send");
        cloned.message("from clone").await.expect("should send");

        let first = receiver.recv().await.expect("should receive first");
        let second = receiver.recv().await.expect("should receive second");

        assert!(matches!(first, Event::Message(ref s) if s == "from original"));
        assert!(matches!(second, Event::Message(ref s) if s == "from clone"));
    }

    #[tokio::test]
    async fn process_event_sends_process_event() {
        let (sender, mut receiver) = event_channel();
        let output = Output::new(sender);
        let invocation = Invocation::new("git").arg("status");

        output
            .process_event(ProcessEvent::Started {
                id: RunId::next(),
                invocation: invocation.clone(),
                process_id: None,
            })
            .await
            .expect("should send");

        assert_eq!(
            receiver.recv().await.map(|event| match event {
                Event::Process(event) => event.to_string(),
                Event::Message(text) => text.into_form(Styling::Plain),
                Event::Detail(text) => text.into_form(Styling::Plain),
                Event::Artifact(artifact) => artifact.to_text(Styling::Plain),
                Event::Prompt(request) => format!("{request:?}"),
            }),
            Some(format!("$ {invocation}"))
        );
    }

    #[tokio::test]
    async fn prompt_sends_prompt_event() {
        let (sender, mut receiver) = event_channel();
        let output = Output::new(sender);
        let (request, _pending) = PromptRequest::confirm(Confirm::new("Release?"));

        output.prompt(request).await.expect("should send");

        assert_eq!(
            receiver.recv().await.map(|event| match event {
                Event::Prompt(request) => request.question().to_owned(),
                Event::Process(event) => event.to_string(),
                Event::Message(text) => text.into_form(Styling::Plain),
                Event::Detail(text) => text.into_form(Styling::Plain),
                Event::Artifact(artifact) => artifact.to_text(Styling::Plain),
            }),
            Some("Release?".to_owned())
        );
    }

    #[tokio::test]
    async fn detail_sends_detail_event() {
        let (sender, mut receiver) = event_channel();
        let output = Output::new(sender);

        output
            .detail("supplementary info")
            .await
            .expect("should send");

        let event = receiver.recv().await.expect("should receive");

        assert!(matches!(event, Event::Detail(ref s) if s == "supplementary info"));
    }

    #[tokio::test]
    async fn detail_with_a_styled_value_sends_its_styled_form() {
        let (sender, mut receiver) = event_channel();
        let output = Output::new(sender);

        output
            .detail(Bold::new("supplementary"))
            .await
            .expect("should send");

        let event = receiver.recv().await.expect("should receive");
        let Event::Detail(text) = event else {
            panic!("expected Event::Detail, received {event:?}");
        };
        assert_eq!(text.form(Styling::Styled), "\x1b[1msupplementary\x1b[0m");
    }

    #[tokio::test]
    async fn detail_to_closed_channel_returns_error() {
        let (sender, receiver) = event_channel();
        let output = Output::new(sender);
        drop(receiver);

        let error = output.detail("lost").await.expect_err("should fail");

        assert_eq!(error.to_string(), "event channel closed");
    }

    #[tokio::test]
    async fn message_sends_message_event() {
        let (sender, mut receiver) = event_channel();
        let output = Output::new(sender);

        output.message("hello").await.expect("should send");

        let event = receiver.recv().await.expect("should receive");

        assert!(matches!(event, Event::Message(ref s) if s == "hello"));
    }

    #[tokio::test]
    async fn message_to_closed_channel_returns_error() {
        let (sender, receiver) = event_channel();
        let output = Output::new(sender);
        drop(receiver);

        let error = output.message("lost").await.expect_err("should fail");

        assert_eq!(error.to_string(), "event channel closed");
    }

    #[tokio::test]
    async fn message_with_a_borrowed_string_sends_the_string() {
        let (sender, mut receiver) = event_channel();
        let output = Output::new(sender);
        let name = String::from("hello");

        output.message(&name).await.expect("should send");

        let event = receiver.recv().await.expect("should receive");
        let Event::Message(text) = event else {
            panic!("expected Event::Message, received {event:?}");
        };
        assert_eq!(text, FormattedText::from("hello"));
    }

    #[tokio::test]
    async fn message_with_a_styled_value_sends_its_plain_form() {
        let (sender, mut receiver) = event_channel();
        let output = Output::new(sender);

        output
            .message(Bold::new("done"))
            .await
            .expect("should send");

        let event = receiver.recv().await.expect("should receive");
        let Event::Message(text) = event else {
            panic!("expected Event::Message, received {event:?}");
        };
        assert_eq!(text.form(Styling::Plain), "done");
    }

    #[tokio::test]
    async fn message_with_a_styled_value_sends_its_styled_form() {
        let (sender, mut receiver) = event_channel();
        let output = Output::new(sender);

        output
            .message(Bold::new("done"))
            .await
            .expect("should send");

        let event = receiver.recv().await.expect("should receive");
        let Event::Message(text) = event else {
            panic!("expected Event::Message, received {event:?}");
        };
        assert_eq!(text.form(Styling::Styled), "\x1b[1mdone\x1b[0m");
    }

    #[tokio::test]
    async fn message_with_format_args_sends_formatted_string() {
        let (sender, mut receiver) = event_channel();
        let output = Output::new(sender);

        output
            .message(format!("count: {}", 42))
            .await
            .expect("should send");

        let event = receiver.recv().await.expect("should receive");

        assert!(matches!(event, Event::Message(ref s) if s == "count: 42"));
    }

    #[test]
    fn trait_send() {
        fn assert_send<T: Send>() {}
        assert_send::<Output>();
    }

    #[test]
    fn trait_sync() {
        fn assert_sync<T: Sync>() {}
        assert_sync::<Output>();
    }

    #[test]
    fn trait_unpin() {
        fn assert_unpin<T: Unpin>() {}
        assert_unpin::<Output>();
    }
}
