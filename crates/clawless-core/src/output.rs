//! Command output interface
//!
//! This module defines [`Output`], the interface commands use to produce events. `Output` wraps an
//! [`EventSender`] and provides typed methods for each event kind, decoupling commands from the
//! rendering strategy. Commands call [`message`], [`detail`], or [`artifact`] to emit events into
//! the channel; the Presenter consumes them and decides how to render.
//!
//! [`artifact`]: Output::artifact
//! [`detail`]: Output::detail
//! [`message`]: Output::message

use std::fmt::{Debug, Display};

use serde::Serialize;

use crate::event::{Event, EventSender, SendError};

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
// r[impl output.safety.clone]
// r[impl output.safety.send]
// r[impl output.safety.concurrent]
#[derive(Clone, Debug)]
pub struct Output {
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
    /// Converts the value to a string via [`Display`] and sends it as an [`Event::Message`].
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
    /// [`EventReceiver`]: crate::event::EventReceiver
    // r[impl output.send.message]
    // r[impl output.send.async]
    pub async fn message(&self, message: impl Display) -> Result<(), SendError> {
        self.sender.send(Event::Message(message.to_string())).await
    }

    /// Sends a supplementary detail event
    ///
    /// Converts the value to a string via [`Display`] and sends it as an [`Event::Detail`].
    /// Details carry lower-priority information that the Presenter may suppress at default
    /// verbosity.
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
    // r[impl output.send.detail]
    pub async fn detail(&self, detail: impl Display) -> Result<(), SendError> {
        self.sender.send(Event::Detail(detail.to_string())).await
    }

    /// Sends a structured artifact event
    ///
    /// Wraps the value in a [`Box`] and sends it as an [`Event::Artifact`]. The value must
    /// implement [`Display`] (for text rendering), [`Serialize`] (for JSON rendering), and
    /// [`Debug`] (for diagnostics).
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
    /// [`EventReceiver`]: crate::event::EventReceiver
    // r[impl output.send.artifact]
    pub async fn artifact<T>(&self, value: T) -> Result<(), SendError>
    where
        T: Display + Serialize + Debug + Send + Sync + 'static,
    {
        self.sender.send(Event::Artifact(Box::new(value))).await
    }
}

#[cfg(test)]
mod tests {
    use std::fmt;

    use serde::Serialize;

    use super::*;
    use crate::event::event_channel;

    #[derive(Clone, Debug, Serialize)]
    struct TestArtifact {
        value: String,
    }

    impl fmt::Display for TestArtifact {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(f, "{}", self.value)
        }
    }

    // r[verify output.send.artifact]
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

        assert!(matches!(event, Event::Artifact(ref a) if a.to_string() == "result"));
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

    // r[verify output.safety.clone]
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

    // r[verify output.send.detail]
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
    async fn detail_to_closed_channel_returns_error() {
        let (sender, receiver) = event_channel();
        let output = Output::new(sender);
        drop(receiver);

        let error = output.detail("lost").await.expect_err("should fail");

        assert_eq!(error.to_string(), "event channel closed");
    }

    // r[verify output.send.message]
    // r[verify output.send.async]
    #[tokio::test]
    async fn message_sends_message_event() {
        let (sender, mut receiver) = event_channel();
        let output = Output::new(sender);

        output.message("hello").await.expect("should send");

        let event = receiver.recv().await.expect("should receive");

        assert!(matches!(event, Event::Message(ref s) if s == "hello"));
    }

    // r[verify output.send.error]
    #[tokio::test]
    async fn message_to_closed_channel_returns_error() {
        let (sender, receiver) = event_channel();
        let output = Output::new(sender);
        drop(receiver);

        let error = output.message("lost").await.expect_err("should fail");

        assert_eq!(error.to_string(), "event channel closed");
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
