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
//! A projection is eventually consistent with the event channel. Sending an event puts it in the
//! channel; a separate task then folds it into the projection. A query that runs between those two
//! moments does not show the event. See [`Projection`] for what this means for a render loop and
//! for the last frame that an application draws.
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
//! projection.wait_until_complete().await;
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
use tokio::sync::watch;
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
/// # Visibility
///
/// A projection is eventually consistent with the event channel. Emitting an event puts it in the
/// channel and returns; the drain task folds it into the projection afterwards. A query that runs
/// in between therefore does not show the event, and awaiting the send does not change that. An
/// application that emits an event and queries immediately should expect the entry on a later
/// frame, not on the current one.
///
/// A render loop absorbs this, because the next frame shows what the previous frame missed. The
/// last frame has no next frame, so an application that wants its closing state on screen drops
/// its [`Context`] to close the channel, awaits [`wait_until_complete`], and draws once more:
///
/// ```rust,ignore
/// message!("shutting down");
/// drop(context);
/// projection.wait_until_complete().await;
/// render(&projection.entries());
/// ```
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
/// projection.wait_until_complete().await;
///
/// let messages = projection.messages();
/// assert_eq!(messages.len(), 1);
/// # }
/// ```
///
/// [`Context`]: clawless_core::context::Context
/// [`EventReceiver`]: clawless_core::event::EventReceiver
/// [`RwLock`]: std::sync::RwLock
/// [`wait_until_complete`]: Projection::wait_until_complete
// r[impl projection.new]
// r[impl projection.new.drain]
// r[impl projection.safety.send]
// r[impl projection.safety.sync]
// r[impl projection.safety.unpin]
#[derive(Debug)]
pub struct Projection {
    /// The entries so far, which the projection shares with the drain task
    state: Arc<RwLock<ProjectionState>>,
    /// Carries the completion of the drain, so that a waiter does not poll for it
    completion: watch::Receiver<bool>,
    /// Handle to the drain task, which the runner takes so that it can await the drain
    ///
    /// Dropping a [`JoinHandle`] detaches its task rather than stopping it, so the drain outlives
    /// the projection and ends when the event channel closes or the runtime shuts down.
    drain: Option<JoinHandle<()>>,
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
        let (completed, completion) = watch::channel(false);

        let handle = tokio::spawn(drain(receiver, drain_state, completed));

        Self {
            state,
            completion,
            drain: Some(handle),
        }
    }

    /// Takes the handle of the drain task out of the projection
    ///
    /// The runner calls this before it moves the projection into the application, so that it
    /// still has something to await once the application returns. The drain task itself is
    /// unaffected: it keeps running, because dropping a [`JoinHandle`] detaches a task instead of
    /// stopping it. A second call returns [`None`].
    pub(crate) fn take_drain(&mut self) -> Option<JoinHandle<()>> {
        self.drain.take()
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

    /// Returns accumulated entries for the runs of external programs only
    ///
    /// The entries arrive while a program runs, so a view that shows the last few lines of a
    /// build reads them from here and takes the tail it has room for. Every entry carries the
    /// [`ProcessId`] of its run, which is what separates two programs that run at the same time.
    ///
    /// # Panics
    ///
    /// Panics if the internal lock is poisoned.
    ///
    /// [`ProcessId`]: clawless_core::event::process::RunId
    // r[impl projection.query.processes]
    // Panics only on a poisoned lock, as documented above.
    #[allow(clippy::expect_used)]
    pub fn processes(&self) -> Vec<Entry> {
        self.state.read().expect("lock poisoned").processes()
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

    /// Waits until the event stream has closed and every buffered event has been drained
    ///
    /// Returns as soon as [`is_complete`] would report `true`, and returns straight away if that
    /// is already the case. An application awaits this before it draws its closing frame, so that
    /// the frame shows the events it emitted last. Because completion needs the channel to close,
    /// the application drops its [`Context`] first; otherwise its own [`EventSender`] holds the
    /// channel open and this never returns.
    ///
    /// This is not a way to read back a single event during a run. It reports the end of the
    /// stream, not the arrival of one entry.
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
    /// sender.send(Event::Message("last word".to_string()))
    ///     .await
    ///     .expect("should send");
    /// drop(sender);
    ///
    /// projection.wait_until_complete().await;
    ///
    /// assert_eq!(projection.entries().len(), 1);
    /// # }
    /// ```
    ///
    /// [`Context`]: clawless_core::context::Context
    /// [`EventSender`]: clawless_core::event::EventSender
    /// [`is_complete`]: Projection::is_complete
    // r[impl projection.lifecycle.wait]
    pub async fn wait_until_complete(&self) {
        let mut completion = self.completion.clone();

        loop {
            let complete = *completion.borrow_and_update();
            if complete {
                return;
            }

            // A failure here means the sender is gone. The sender lives in the drain task and the
            // drain announces completion before it ends, so its absence without an announcement
            // takes a panic. Waiting for word that can no longer come would never return.
            if completion.changed().await.is_err() {
                return;
            }
        }
    }
}

/// Drains events from the receiver into the shared state
///
/// Runs until `receiver.recv()` returns `None` (all senders dropped, channel empty). Each event
/// is translated into an [`Entry`] and appended to the state. When the loop exits, the state is
/// marked as complete and `completed` announces it to any waiter.
///
/// The state is marked before the announcement, and never the other way round. A waiter that the
/// announcement wakes therefore finds every entry already in place, rather than a projection that
/// calls itself complete while the last entry is still missing.
///
/// # Panics
///
/// Panics if the internal lock is poisoned.
// The lock is poisoned only if another thread panicked while holding it. The drain task has
// no way to publish events into a state it cannot lock, so it fails loudly instead.
// r[impl projection.lifecycle.order]
#[allow(clippy::expect_used)]
async fn drain(
    mut receiver: EventReceiver,
    state: Arc<RwLock<ProjectionState>>,
    completed: watch::Sender<bool>,
) {
    while let Some(event) = receiver.recv().await {
        let entry = match event {
            Event::Message(text) => Entry::Message(text),
            Event::Detail(text) => Entry::Detail(text),
            Event::Artifact(artifact) => Entry::Artifact(Arc::from(artifact)),
            Event::Process(event) => Entry::Process(Arc::from(event)),
        };
        state.write().expect("lock poisoned").push(entry);
    }
    state.write().expect("lock poisoned").set_complete();

    // `send_replace` rather than `send`, because a drain that ends after its projection was
    // dropped has no receiver left and `send` would call that an error. There is nothing to
    // recover: the announcement has no audience, and the previous value is of no interest.
    completed.send_replace(true);
}

#[cfg(test)]
mod tests {
    // An assertion in a test panics by design. A `# Panics` section on every test
    // would repeat that and give the reader no information.
    #![allow(clippy::missing_panics_doc)]

    use std::fmt;

    use clawless_core::event::event_channel;
    use clawless_core::event::process::{ProcessEvent, RunId};
    use clawless_core::process::{Line, Stream};
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
        projection.wait_until_complete().await;

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
        projection.wait_until_complete().await;

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
        projection.wait_until_complete().await;

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
        projection.wait_until_complete().await;

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
        projection.wait_until_complete().await;

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
        projection.wait_until_complete().await;

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
        projection.wait_until_complete().await;

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
        projection.wait_until_complete().await;

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
        projection.wait_until_complete().await;

        assert!(projection.is_complete());
    }

    // r[verify projection.query.processes]
    #[tokio::test]
    async fn processes_returns_process_entries_only() {
        let (sender, receiver) = event_channel();
        let projection = Projection::new(receiver);

        sender
            .send(Event::Message("msg".to_string()))
            .await
            .expect("should send");
        sender
            .send(Event::Process(Box::new(ProcessEvent::Line {
                id: RunId::next(),
                line: Line::new(Stream::StandardOutput, "compiling"),
            })))
            .await
            .expect("should send");
        drop(sender);
        projection.wait_until_complete().await;

        let processes = projection.processes();

        assert_eq!(
            processes.iter().map(Entry::to_string).collect::<Vec<_>>(),
            vec!["compiling".to_string()]
        );
    }

    #[tokio::test]
    async fn take_drain_with_a_second_call_returns_none() {
        let (_sender, receiver) = event_channel();
        let mut projection = Projection::new(receiver);
        drop(projection.take_drain());

        let handle = projection.take_drain();

        assert!(handle.is_none());
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

    // r[verify projection.lifecycle.wait]
    #[tokio::test]
    async fn wait_until_complete_after_the_drain_finished_returns_immediately() {
        let (sender, receiver) = event_channel();
        let projection = Projection::new(receiver);
        drop(sender);
        projection.wait_until_complete().await;

        projection.wait_until_complete().await;

        assert!(projection.is_complete());
    }

    // The announcement of completion is what wakes the waiter, so this is where an announcement
    // that ran ahead of the state would show: the waiter would return and find a projection that
    // still calls itself incomplete. The multi-threaded flavor is what gives the check teeth,
    // because the drain runs on another worker and the two writes can be observed apart. A
    // current-thread runtime hides the difference.
    // r[verify projection.lifecycle.order]
    #[tokio::test(flavor = "multi_thread")]
    async fn wait_until_complete_reports_completion_when_it_returns() {
        let mut torn = 0;

        for _ in 0..1000 {
            let (sender, receiver) = event_channel();
            let projection = Projection::new(receiver);
            sender
                .send(Event::Message("last word".to_string()))
                .await
                .expect("should send");
            drop(sender);

            projection.wait_until_complete().await;

            if !projection.is_complete() {
                torn += 1;
            }
        }

        assert_eq!(torn, 0);
    }

    // Awaiting the send only puts an event in the channel, so a projection read straight after it
    // is almost always still empty. This is the await that closes that gap, and the count is what
    // a closing frame depends on.
    // r[verify projection.lifecycle.wait]
    #[tokio::test(flavor = "multi_thread")]
    async fn wait_until_complete_shows_the_event_that_preceded_it() {
        let mut incomplete = 0;

        for _ in 0..1000 {
            let (sender, receiver) = event_channel();
            let projection = Projection::new(receiver);
            sender
                .send(Event::Message("last word".to_string()))
                .await
                .expect("should send");
            drop(sender);

            projection.wait_until_complete().await;

            if projection.entries().len() != 1 {
                incomplete += 1;
            }
        }

        assert_eq!(incomplete, 0);
    }
}
