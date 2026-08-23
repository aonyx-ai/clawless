use tokio::sync::mpsc;

use super::Event;

/// Receiver handle for the event channel
///
/// `EventReceiver` is the consuming end of the event channel. The Presenter holds this handle and
/// reads events for rendering. Because the underlying channel is multi-producer single-consumer,
/// there is exactly one `EventReceiver` per channel.
///
/// `EventReceiver` is [`Send`] but not [`Sync`], matching the semantics of
/// [`tokio::sync::mpsc::Receiver`].
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
    /// [`EventSender`]: super::EventSender
    pub async fn recv(&mut self) -> Option<Event> {
        self.inner.recv().await
    }
}

#[cfg(test)]
mod tests {
    // An assertion in a test panics by design. A `# Panics` section on every test
    // would repeat that and give the reader no information.
    #![allow(clippy::missing_panics_doc)]

    use super::*;

    #[test]
    fn trait_send() {
        fn assert_send<T: Send>() {}
        assert_send::<EventReceiver>();
    }

    #[test]
    fn trait_unpin() {
        fn assert_unpin<T: Unpin>() {}
        assert_unpin::<EventReceiver>();
    }
}
