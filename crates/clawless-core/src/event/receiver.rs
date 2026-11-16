//! The half of the event channel that reads events

use tokio::sync::mpsc;

use super::Event;

/// Receiver handle for the event channel
///
/// `EventReceiver` is the consuming end of the event channel. The Presenter holds this handle and
/// reads events for rendering. Because the underlying channel is multi-producer single-consumer,
/// there is exactly one `EventReceiver` per channel.
///
/// `EventReceiver` is [`Send`] and [`Sync`], matching the semantics of
/// [`tokio::sync::mpsc::Receiver`].
///
/// # Examples
///
/// ```
/// use clawless_core::event::{Event, event_channel};
///
/// # #[tokio::main(flavor = "current_thread")]
/// # async fn main() {
/// let (sender, mut receiver) = event_channel();
/// sender.send(Event::Message("hello".to_string())).await.expect("the receiver is alive");
/// drop(sender);
///
/// let mut texts = Vec::new();
/// while let Some(event) = receiver.recv().await {
///     match event {
///         Event::Message(text) => texts.push(text),
///         Event::Detail(_) | Event::Artifact(_) | Event::Process(_) | Event::Prompt(_) => {}
///     }
/// }
///
/// assert_eq!(texts, ["hello"]);
/// # }
/// ```
#[derive(Debug)]
pub struct EventReceiver {
    /// The half of the Tokio channel that reads events
    inner: mpsc::Receiver<Event>,
}

impl EventReceiver {
    /// Wraps a Tokio receiver in an [`EventReceiver`]
    ///
    /// Only [`event_channel`] makes a receiver. Each channel therefore has exactly one
    /// receiver.
    ///
    /// [`event_channel`]: super::event_channel
    pub(super) fn new(inner: mpsc::Receiver<Event>) -> Self {
        Self { inner }
    }

    /// Receives the next event from the channel
    ///
    /// Returns `None` when all [`EventSender`]s have been dropped and the channel is empty.
    ///
    /// # Examples
    ///
    /// ```
    /// use clawless_core::event::{Event, event_channel};
    ///
    /// # #[tokio::main(flavor = "current_thread")]
    /// # async fn main() {
    /// let (sender, mut receiver) = event_channel();
    /// sender.send(Event::Message("hello".to_string())).await.expect("the receiver is alive");
    /// drop(sender);
    ///
    /// assert!(matches!(receiver.recv().await, Some(Event::Message(text)) if text == "hello"));
    /// assert!(receiver.recv().await.is_none());
    /// # }
    /// ```
    ///
    /// [`EventSender`]: super::EventSender
    pub async fn recv(&mut self) -> Option<Event> {
        self.inner.recv().await
    }
}

#[cfg(test)]
mod tests {
    #![allow(
        clippy::missing_panics_doc,
        reason = "an assertion in a test panics by design, so a `# Panics` section would repeat it"
    )]

    use super::*;

    #[test]
    fn trait_send() {
        fn assert_send<T: Send>() {}
        assert_send::<EventReceiver>();
    }

    #[test]
    fn trait_sync() {
        fn assert_sync<T: Sync>() {}
        assert_sync::<EventReceiver>();
    }

    #[test]
    fn trait_unpin() {
        fn assert_unpin<T: Unpin>() {}
        assert_unpin::<EventReceiver>();
    }
}
