//! Pull-based, queryable view of the event stream
//!
//! This module defines [`Projection`] and [`Entry`], the pull-based counterpart to the push-based
//! [`TerminalPresenter`]. A projection consumes events from an [`EventReceiver`] in the background
//! and provides read access to accumulated state at any time. TUI applications query the projection
//! on each render frame without interacting with the event system directly.
//!
//! The projection translates internal [`Event`]s into [`Entry`] values so that TUI consumers never
//! interact with the event system directly. Queries return cloned snapshots of the accumulated
//! state. The read lock is held only while cloning the snapshot, keeping contention with the
//! drain task's write lock brief.
//!
//! # Examples
//!
//! ```
//! use clawless_core::event::{Event, event_channel};
//! use clawless_tui::projection::Projection;
//!
//! # #[tokio::main]
//! # async fn main() {
//! let (sender, receiver) = event_channel();
//! let projection = Projection::new(receiver);
//!
//! sender.send(Event::Message("hello".to_string()))
//!     .await
//!     .expect("should send");
//! drop(sender);
//!
//! // Wait for the drain task, which finishes once the dropped sender closes the channel
//! while !projection.is_complete() {
//!     tokio::task::yield_now().await;
//! }
//!
//! assert_eq!(projection.entries().len(), 1);
//! # }
//! ```
//!
//! [`Event`]: clawless_core::event::Event
//! [`EventReceiver`]: clawless_core::event::EventReceiver
//! [`TerminalPresenter`]: https://docs.rs/clawless-cli/latest/clawless_cli/presenter/struct.TerminalPresenter.html

use std::sync::{Arc, RwLock};

use clawless_core::event::{Event, EventReceiver};
use tokio::task::JoinHandle;

pub use self::entry::Entry;
use self::state::ProjectionState;

/// The form of an [`Event`] that a TUI application reads
mod entry;
/// The state that the projection lock protects
mod state;

/// Pull-based, queryable view of the event stream
///
/// `Projection` consumes events from an [`EventReceiver`] in the background and provides read
/// access to accumulated state at any time. TUI applications query the projection on each render
/// frame without interacting with the event system directly.
///
/// Construction starts a background drain task via [`tokio::spawn`]. The task reads events from
/// the receiver, translates each into an [`Entry`], and appends it to internal storage. When the
/// event channel closes (all senders dropped), the task marks the projection as complete.
///
/// Query methods take `&self` and return cloned snapshots of the accumulated state. The drain
/// task and query callers synchronize through a [`RwLock`], allowing concurrent reads from
/// multiple render frames without blocking on each other.
///
/// # Examples
///
/// ```
/// use clawless_core::event::{Event, event_channel};
/// use clawless_tui::projection::Projection;
///
/// # #[tokio::main]
/// # async fn main() {
/// let (sender, receiver) = event_channel();
/// let projection = Projection::new(receiver);
///
/// sender.send(Event::Message("processing".to_string()))
///     .await
///     .expect("should send");
/// drop(sender);
///
/// // Wait for the drain task, which finishes once the dropped sender closes the channel
/// while !projection.is_complete() {
///     tokio::task::yield_now().await;
/// }
///
/// let messages = projection.messages();
/// assert_eq!(messages.len(), 1);
/// # }
/// ```
///
/// [`EventReceiver`]: clawless_core::event::EventReceiver
/// [`RwLock`]: std::sync::RwLock
// r[impl projection.new]
// r[impl projection.new.drain]
// r[impl projection.safety.send]
// r[impl projection.safety.sync]
// r[impl projection.safety.unpin]
#[derive(Debug)]
pub struct Projection {
    /// The entries so far, which the projection shares with the drain task
    state: Arc<RwLock<ProjectionState>>,
    /// Handle to the drain task, which stops when Clawless drops the projection
    _drain_handle: JoinHandle<()>,
}

impl Projection {
    /// Creates a new projection that drains events from the given receiver
    ///
    /// Spawns a background task that reads events from `receiver`, translates each into an
    /// [`Entry`], and appends it to internal storage. The task runs until the event channel
    /// closes, at which point it marks the projection as complete.
    ///
    /// The returned projection is immediately ready to query. Early queries return empty results
    /// until events arrive.
    ///
    /// # Panics
    ///
    /// Panics if called outside of a Tokio runtime.
    ///
    /// # Examples
    ///
    /// ```
    /// use clawless_core::event::event_channel;
    /// use clawless_tui::projection::Projection;
    ///
    /// # #[tokio::main]
    /// # async fn main() {
    /// let (_sender, receiver) = event_channel();
    /// let projection = Projection::new(receiver);
    ///
    /// assert!(projection.entries().is_empty());
    /// # }
    /// ```
    pub fn new(receiver: EventReceiver) -> Self {
        let state = Arc::new(RwLock::new(ProjectionState::default()));
        let drain_state = Arc::clone(&state);

        let handle = tokio::spawn(drain(receiver, drain_state));

        Self {
            state,
            _drain_handle: handle,
        }
    }

    /// Returns all accumulated entries in receive order
    ///
    /// Returns a cloned snapshot of the entry list. The snapshot is consistent: it reflects all
    /// events drained up to the moment the read lock is acquired.
    ///
    /// # Panics
    ///
    /// Panics if the internal lock is poisoned.
    // r[impl projection.query.entries]
    // Panics only on a poisoned lock, as documented above.
    #[allow(clippy::expect_used)]
    pub fn entries(&self) -> Vec<Entry> {
        self.state.read().expect("lock poisoned").entries()
    }

    /// Returns accumulated message entries only
    ///
    /// Equivalent to calling [`entries`] and filtering to [`Entry::Message`] variants.
    ///
    /// # Panics
    ///
    /// Panics if the internal lock is poisoned.
    ///
    /// [`entries`]: Projection::entries
    // r[impl projection.query.messages]
    // Panics only on a poisoned lock, as documented above.
    #[allow(clippy::expect_used)]
    pub fn messages(&self) -> Vec<Entry> {
        self.state.read().expect("lock poisoned").messages()
    }

    /// Returns accumulated detail entries only
    ///
    /// Equivalent to calling [`entries`] and filtering to [`Entry::Detail`] variants.
    ///
    /// # Panics
    ///
    /// Panics if the internal lock is poisoned.
    ///
    /// [`entries`]: Projection::entries
    // r[impl projection.query.details]
    // Panics only on a poisoned lock, as documented above.
    #[allow(clippy::expect_used)]
    pub fn details(&self) -> Vec<Entry> {
        self.state.read().expect("lock poisoned").details()
    }

    /// Returns accumulated artifact entries only
    ///
    /// Equivalent to calling [`entries`] and filtering to [`Entry::Artifact`] variants.
    ///
    /// # Panics
    ///
    /// Panics if the internal lock is poisoned.
    ///
    /// [`entries`]: Projection::entries
    // r[impl projection.query.artifacts]
    // Panics only on a poisoned lock, as documented above.
    #[allow(clippy::expect_used)]
    pub fn artifacts(&self) -> Vec<Entry> {
        self.state.read().expect("lock poisoned").artifacts()
    }

    /// Reports whether the event stream has closed and all buffered events have been drained
    ///
    /// Returns `true` once all [`EventSender`]s have been dropped and the drain task has
    /// processed every buffered event. Before that point, returns `false`.
    ///
    /// # Panics
    ///
    /// Panics if the internal lock is poisoned.
    ///
    /// [`EventSender`]: clawless_core::event::EventSender
    // r[impl projection.lifecycle.complete]
    // Panics only on a poisoned lock, as documented above.
    #[allow(clippy::expect_used)]
    pub fn is_complete(&self) -> bool {
        self.state.read().expect("lock poisoned").is_complete()
    }
}

/// Drains events from the receiver into the shared state
///
/// Runs until `receiver.recv()` returns `None` (all senders dropped, channel empty). Each event
/// is translated into an [`Entry`] and appended to the state. When the loop exits, the state is
/// marked as complete.
///
/// # Panics
///
/// Panics if the internal lock is poisoned.
// The lock is poisoned only if another thread panicked while holding it. The drain task has
// no way to publish events into a state it cannot lock, so it fails loudly instead.
#[allow(clippy::expect_used)]
async fn drain(mut receiver: EventReceiver, state: Arc<RwLock<ProjectionState>>) {
    while let Some(event) = receiver.recv().await {
        let entry = match event {
            Event::Message(text) => Entry::Message(text),
            Event::Detail(text) => Entry::Detail(text),
            Event::Artifact(artifact) => Entry::Artifact(Arc::from(artifact)),
        };
        state.write().expect("lock poisoned").push(entry);
    }
    state.write().expect("lock poisoned").set_complete();
}

#[cfg(test)]
mod tests {
    // An assertion in a test panics by design. A `# Panics` section on every test
    // would repeat that and give the reader no information.
    #![allow(clippy::missing_panics_doc)]

    use std::fmt;

    use clawless_core::event::event_channel;
    use serde::Serialize;

    use super::*;

    #[derive(Clone, Debug, Serialize)]
    struct TestArtifact {
        value: String,
    }

    impl fmt::Display for TestArtifact {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(f, "{}", self.value)
        }
    }

    // r[verify projection.query.artifacts]
    #[tokio::test]
    async fn artifacts_returns_artifact_entries_only() {
        let (sender, receiver) = event_channel();
        let projection = Projection::new(receiver);

        sender
            .send(Event::Message("msg".to_string()))
            .await
            .expect("should send");
        sender
            .send(Event::Artifact(Box::new(TestArtifact {
                value: "art".to_string(),
            })))
            .await
            .expect("should send");
        drop(sender);
        tokio::task::yield_now().await;

        let artifacts = projection.artifacts();

        assert_eq!(artifacts.len(), 1);
        let Entry::Artifact(a) = &artifacts[0] else {
            panic!("expected Entry::Artifact");
        };
        assert_eq!(a.to_string(), "art");
    }

    // r[verify projection.query.details]
    #[tokio::test]
    async fn details_returns_detail_entries_only() {
        let (sender, receiver) = event_channel();
        let projection = Projection::new(receiver);

        sender
            .send(Event::Detail("dtl".to_string()))
            .await
            .expect("should send");
        sender
            .send(Event::Message("msg".to_string()))
            .await
            .expect("should send");
        sender
            .send(Event::Detail("dtl2".to_string()))
            .await
            .expect("should send");
        drop(sender);
        tokio::task::yield_now().await;

        let details = projection.details();

        assert_eq!(details.len(), 2);
        let Entry::Detail(s) = &details[0] else {
            panic!("expected Entry::Detail");
        };
        assert_eq!(s, "dtl");
        let Entry::Detail(s) = &details[1] else {
            panic!("expected Entry::Detail");
        };
        assert_eq!(s, "dtl2");
    }

    #[tokio::test]
    async fn drain_processes_buffered_events_before_completing() {
        let (sender, receiver) = event_channel();

        sender
            .send(Event::Message("buffered".to_string()))
            .await
            .expect("should send");
        drop(sender);

        let projection = Projection::new(receiver);
        tokio::task::yield_now().await;

        assert!(projection.is_complete());
        let entries = projection.entries();
        assert_eq!(entries.len(), 1);
        let Entry::Message(s) = &entries[0] else {
            panic!("expected Entry::Message");
        };
        assert_eq!(s, "buffered");
    }

    // r[verify projection.entry.artifact]
    #[tokio::test]
    async fn entries_returns_artifact_events_as_entries() {
        let (sender, receiver) = event_channel();
        let projection = Projection::new(receiver);

        sender
            .send(Event::Artifact(Box::new(TestArtifact {
                value: "result".to_string(),
            })))
            .await
            .expect("should send");
        drop(sender);
        tokio::task::yield_now().await;

        let entries = projection.entries();

        assert_eq!(entries.len(), 1);
        let Entry::Artifact(a) = &entries[0] else {
            panic!("expected Entry::Artifact");
        };
        assert_eq!(a.to_string(), "result");
    }

    // r[verify projection.entry.detail]
    #[tokio::test]
    async fn entries_returns_detail_events_as_entries() {
        let (sender, receiver) = event_channel();
        let projection = Projection::new(receiver);

        sender
            .send(Event::Detail("info".to_string()))
            .await
            .expect("should send");
        drop(sender);
        tokio::task::yield_now().await;

        let entries = projection.entries();

        assert_eq!(entries.len(), 1);
        let Entry::Detail(s) = &entries[0] else {
            panic!("expected Entry::Detail");
        };
        assert_eq!(s, "info");
    }

    // r[verify projection.entry.message]
    // r[verify projection.query.entries]
    #[tokio::test]
    async fn entries_returns_message_events_as_entries() {
        let (sender, receiver) = event_channel();
        let projection = Projection::new(receiver);

        sender
            .send(Event::Message("hello".to_string()))
            .await
            .expect("should send");
        drop(sender);
        tokio::task::yield_now().await;

        let entries = projection.entries();

        assert_eq!(entries.len(), 1);
        let Entry::Message(s) = &entries[0] else {
            panic!("expected Entry::Message");
        };
        assert_eq!(s, "hello");
    }

    // r[verify projection.entry.order]
    #[tokio::test]
    async fn entries_preserves_receive_order() {
        let (sender, receiver) = event_channel();
        let projection = Projection::new(receiver);

        sender
            .send(Event::Message("first".to_string()))
            .await
            .expect("should send");
        sender
            .send(Event::Detail("second".to_string()))
            .await
            .expect("should send");
        sender
            .send(Event::Message("third".to_string()))
            .await
            .expect("should send");
        drop(sender);
        tokio::task::yield_now().await;

        let entries = projection.entries();

        assert_eq!(entries.len(), 3);
        let Entry::Message(s) = &entries[0] else {
            panic!("expected Entry::Message");
        };
        assert_eq!(s, "first");
        let Entry::Detail(s) = &entries[1] else {
            panic!("expected Entry::Detail");
        };
        assert_eq!(s, "second");
        let Entry::Message(s) = &entries[2] else {
            panic!("expected Entry::Message");
        };
        assert_eq!(s, "third");
    }

    #[tokio::test]
    async fn is_complete_returns_false_while_channel_open() {
        let (_sender, receiver) = event_channel();
        let projection = Projection::new(receiver);

        tokio::task::yield_now().await;

        assert!(!projection.is_complete());
    }

    // r[verify projection.query.messages]
    #[tokio::test]
    async fn messages_returns_message_entries_only() {
        let (sender, receiver) = event_channel();
        let projection = Projection::new(receiver);

        sender
            .send(Event::Message("msg".to_string()))
            .await
            .expect("should send");
        sender
            .send(Event::Detail("dtl".to_string()))
            .await
            .expect("should send");
        sender
            .send(Event::Message("msg2".to_string()))
            .await
            .expect("should send");
        drop(sender);
        tokio::task::yield_now().await;

        let messages = projection.messages();

        assert_eq!(messages.len(), 2);
        let Entry::Message(s) = &messages[0] else {
            panic!("expected Entry::Message");
        };
        assert_eq!(s, "msg");
        let Entry::Message(s) = &messages[1] else {
            panic!("expected Entry::Message");
        };
        assert_eq!(s, "msg2");
    }

    // r[verify projection.new]
    #[tokio::test]
    async fn new_returns_empty_projection() {
        let (_sender, receiver) = event_channel();

        let projection = Projection::new(receiver);

        assert!(projection.entries().is_empty());
        assert!(!projection.is_complete());
    }

    // r[verify projection.new.drain]
    // r[verify projection.lifecycle.complete]
    #[tokio::test]
    async fn new_starts_drain_that_completes_when_channel_closes() {
        let (sender, receiver) = event_channel();
        let projection = Projection::new(receiver);

        drop(sender);
        tokio::task::yield_now().await;

        assert!(projection.is_complete());
    }

    // r[verify projection.safety.send]
    #[test]
    fn trait_send() {
        fn assert_send<T: Send>() {}
        assert_send::<Projection>();
    }

    // r[verify projection.safety.sync]
    #[test]
    fn trait_sync() {
        fn assert_sync<T: Sync>() {}
        assert_sync::<Projection>();
    }

    // r[verify projection.safety.unpin]
    #[test]
    fn trait_unpin() {
        fn assert_unpin<T: Unpin>() {}
        assert_unpin::<Projection>();
    }
}
