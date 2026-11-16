//! The half of the event channel that sends events

use tokio::sync::mpsc;

use super::Event;

/// Sender handle for the event channel
///
/// `EventSender` is a clonable handle that commands use to emit events into the channel. The
/// paired [`EventReceiver`] consumes these events for rendering.
///
/// Internally, `EventSender` wraps a [`tokio::sync::mpsc::Sender<Event>`]. Cloning an
/// `EventSender` produces another handle to the same channel, not an independent channel.
///
/// # Examples
///
/// ```
/// use clawless_core::event::{Event, event_channel};
///
/// # #[tokio::main(flavor = "current_thread")]
/// # async fn main() {
/// let (sender, mut receiver) = event_channel();
/// let clone = sender.clone();
///
/// let message = Event::Message("from the sender".to_string());
/// sender.send(message).await.expect("the receiver is alive");
/// let message = Event::Message("from the clone".to_string());
/// clone.send(message).await.expect("the receiver is alive");
///
/// let first = receiver.recv().await;
/// assert!(matches!(first, Some(Event::Message(text)) if text == "from the sender"));
/// let second = receiver.recv().await;
/// assert!(matches!(second, Some(Event::Message(text)) if text == "from the clone"));
/// # }
/// ```
///
/// [`EventReceiver`]: super::EventReceiver
#[derive(Clone, Debug)]
pub struct EventSender {
    /// The half of the Tokio channel that sends events
    inner: mpsc::Sender<Event>,
}

impl EventSender {
    /// Wraps a Tokio sender in an [`EventSender`]
    ///
    /// Only [`event_channel`] makes a sender. Clawless creates the matched receiver at the
    /// same time.
    ///
    /// [`event_channel`]: super::event_channel
    pub(super) fn new(inner: mpsc::Sender<Event>) -> Self {
        Self { inner }
    }

    /// Sends an event into the channel
    ///
    /// # Errors
    ///
    /// Returns [`SendError`] if the [`EventReceiver`] has been dropped.
    ///
    /// # Examples
    ///
    /// ```
    /// use clawless_core::event::{Event, event_channel};
    ///
    /// # #[tokio::main(flavor = "current_thread")]
    /// # async fn main() {
    /// let (sender, receiver) = event_channel();
    /// drop(receiver);
    ///
    /// let result = sender.send(Event::Detail("lost".to_string())).await;
    ///
    /// assert!(result.is_err());
    /// # }
    /// ```
    ///
    /// [`EventReceiver`]: super::EventReceiver
    /// [`SendError`]: super::SendError
    pub async fn send(&self, event: Event) -> Result<(), super::SendError> {
        self.inner
            .send(event)
            .await
            .map_err(|e| super::SendError::new(e.0))
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
        assert_send::<EventSender>();
    }

    #[test]
    fn trait_sync() {
        fn assert_sync<T: Sync>() {}
        assert_sync::<EventSender>();
    }

    #[test]
    fn trait_unpin() {
        fn assert_unpin<T: Unpin>() {}
        assert_unpin::<EventSender>();
    }
}
