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
    inner: mpsc::Receiver<Event>,
}

impl EventReceiver {
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
