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
/// [`EventReceiver`]: super::EventReceiver
// r[impl event.safety.producer-clone]
// r[impl event.safety.producer-concurrent]
#[derive(Clone, Debug)]
pub struct EventSender {
    inner: mpsc::Sender<Event>,
}

impl EventSender {
    pub(super) fn new(inner: mpsc::Sender<Event>) -> Self {
        Self { inner }
    }

    /// Sends an event into the channel
    ///
    /// # Errors
    ///
    /// Returns [`SendError`] if the [`EventReceiver`] has been dropped.
    ///
    /// [`EventReceiver`]: super::EventReceiver
    /// [`SendError`]: super::SendError
    pub async fn send(&self, event: Event) -> Result<(), super::SendError> {
        self.inner
            .send(event)
            .await
            .map_err(|e| super::SendError(e.0))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn trait_send() {
        fn assert_send<T: Send>() {}
        assert_send::<EventSender>();
    }

    // r[verify event.safety.producer-concurrent]
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
