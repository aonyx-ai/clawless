//! Bounded channel that carries events from a command to its presenter

use std::fmt;

use getset::Getters;
use tokio::sync::mpsc;

use super::Event;
use super::receiver::EventReceiver;
use super::sender::EventSender;

/// Error returned when sending an event fails
///
/// A send fails when the [`EventReceiver`] has been dropped, meaning the consumer is no longer
/// listening. The error carries the unsent [`Event`] so callers can log or inspect what was lost.
///
/// # Examples
///
/// ```
/// use clawless_core::event::{Event, event_channel};
///
/// # #[tokio::main]
/// # async fn main() {
/// let (sender, receiver) = event_channel();
/// drop(receiver);
///
/// let error = sender
///     .send(Event::Message("hello".to_string()))
///     .await
///     .expect_err("the receiver is gone");
///
/// assert!(matches!(error.event(), Event::Message(text) if text == "hello"));
/// # }
/// ```
#[derive(Debug, Getters)]
pub struct SendError {
    /// The event that the channel did not deliver
    #[getset(get = "pub")]
    event: Event,
}

impl SendError {
    /// Wraps the event that the channel did not deliver
    pub(crate) fn new(event: Event) -> Self {
        Self { event }
    }
}

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
/// # Examples
///
/// ```
/// use clawless_core::event::{Event, event_channel};
///
/// # #[tokio::main]
/// # async fn main() {
/// let (sender, mut receiver) = event_channel();
///
/// sender
///     .send(Event::Message("hello".to_string()))
///     .await
///     .expect("the receiver is alive");
/// drop(sender);
///
/// assert!(matches!(receiver.recv().await, Some(Event::Message(text)) if text == "hello"));
/// assert!(receiver.recv().await.is_none());
/// # }
/// ```
///
/// [`recv`]: EventReceiver::recv
pub fn event_channel() -> (EventSender, EventReceiver) {
    let (tx, rx) = mpsc::channel(256);
    (EventSender::new(tx), EventReceiver::new(rx))
}

#[cfg(test)]
mod tests {
    #![allow(
        clippy::missing_panics_doc,
        reason = "an assertion in a test panics by design, so a `# Panics` section would repeat it"
    )]

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

        assert!(matches!(first, Event::Message(ref s) if s == "first"));
    }

    #[tokio::test]
    async fn event_channel_recv_returns_none_after_draining_buffered_events() {
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

        let _first = receiver.recv().await.expect("should receive first");
        let _second = receiver.recv().await.expect("should receive second");
        let done = receiver.recv().await;

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
    async fn event_channel_recv_returns_second_buffered_event_after_sender_dropped() {
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

        let _first = receiver.recv().await.expect("should receive first");
        let second = receiver.recv().await.expect("should receive second");

        assert!(matches!(second, Event::Message(ref s) if s == "second"));
    }

    #[tokio::test]
    async fn event_channel_send_after_receiver_dropped_returns_error() {
        let (sender, receiver) = event_channel();

        drop(receiver);

        let error = sender
            .send(Event::Message("hello".to_string()))
            .await
            .expect_err("should fail");

        assert!(matches!(error.event(), Event::Message(s) if s == "hello"));
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
        let error = SendError::new(Event::Message("lost".to_string()));

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
