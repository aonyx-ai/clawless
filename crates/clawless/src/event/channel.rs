use std::fmt;

use tokio::sync::mpsc;

use super::Event;
use super::receiver::EventReceiver;
use super::sender::EventSender;

/// Error returned when sending an event fails
///
/// A send fails when the [`EventReceiver`] has been dropped, meaning the consumer is no longer
/// listening. The error carries the unsent [`Event`] so callers can log or inspect what was lost.
#[derive(Debug)]
pub struct SendError(pub Event);

impl fmt::Display for SendError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("event channel closed")
    }
}

impl std::error::Error for SendError {}

/// Creates a bounded event channel
///
/// Returns a paired sender and receiver. The sender is clonable; the receiver is not. When all
/// senders are dropped, the receiver's [`recv`] method returns `None`.
///
/// The channel is bounded with a capacity of 256 events. If the consumer falls behind, producers
/// will await until space is available, providing natural back-pressure.
///
/// [`recv`]: EventReceiver::recv
pub fn event_channel() -> (EventSender, EventReceiver) {
    let (tx, rx) = mpsc::channel(256);
    (EventSender::new(tx), EventReceiver::new(rx))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn event_channel_recv_drains_buffered_events_after_sender_dropped() {
        let (sender, mut receiver) = event_channel();

        sender
            .send(Event::Message("first".to_string()))
            .await
            .expect("should send");
        sender
            .send(Event::Message("second".to_string()))
            .await
            .expect("should send");
        drop(sender);

        let first = receiver.recv().await.expect("should receive first");
        let second = receiver.recv().await.expect("should receive second");
        let done = receiver.recv().await;

        assert!(matches!(first, Event::Message(ref s) if s == "first"));
        assert!(matches!(second, Event::Message(ref s) if s == "second"));
        assert!(done.is_none());
    }

    #[tokio::test]
    async fn event_channel_recv_returns_none_when_all_senders_dropped() {
        let (sender, mut receiver) = event_channel();

        drop(sender);

        let result = receiver.recv().await;

        assert!(result.is_none());
    }

    #[tokio::test]
    async fn event_channel_send_after_receiver_dropped_returns_error() {
        let (sender, receiver) = event_channel();

        drop(receiver);

        let error = sender
            .send(Event::Message("hello".to_string()))
            .await
            .expect_err("should fail");

        assert!(matches!(error.0, Event::Message(ref s) if s == "hello"));
    }

    #[tokio::test]
    async fn event_channel_send_and_recv_delivers_event() {
        let (sender, mut receiver) = event_channel();

        sender
            .send(Event::Message("hello".to_string()))
            .await
            .expect("should send");

        let event = receiver.recv().await.expect("should receive");

        assert!(matches!(event, Event::Message(ref s) if s == "hello"));
    }

    #[test]
    fn send_error_display_shows_closed_message() {
        let error = SendError(Event::Message("lost".to_string()));

        let message = error.to_string();

        assert_eq!(message, "event channel closed");
    }

    #[test]
    fn send_error_is_std_error() {
        fn assert_error<T: std::error::Error>() {}
        assert_error::<SendError>();
    }

    #[test]
    fn trait_send_error_send() {
        fn assert_send<T: Send>() {}
        assert_send::<SendError>();
    }

    #[test]
    fn trait_send_error_sync() {
        fn assert_sync<T: Sync>() {}
        assert_sync::<SendError>();
    }

    #[test]
    fn trait_send_error_unpin() {
        fn assert_unpin<T: Unpin>() {}
        assert_unpin::<SendError>();
    }
}
