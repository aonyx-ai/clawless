//! The state that the projection lock protects

use super::Entry;

/// Internal mutable state shared between the drain task and query methods
///
/// `ProjectionState` holds the accumulated entries and whether the drain has completed. The drain
/// task writes to this state behind a write lock; query methods read behind a read lock.
///
/// [`Projection`]: super::Projection
#[derive(Debug, Default)]
pub(super) struct ProjectionState {
    /// The entries so far, in the order that the drain task received them
    entries: Vec<Entry>,
    /// Whether the event channel has closed and the drain task has processed every event
    completion: Completion,
}

/// Whether the drain task has processed every event
#[derive(Copy, Clone, Eq, PartialEq, Debug, Default)]
enum Completion {
    /// The event channel is open, or the drain task still has events to process
    #[default]
    Incomplete,

    /// The event channel closed and the drain task processed every event
    Complete,
}

impl ProjectionState {
    /// Appends an entry to the end of the entry list
    pub(super) fn push(&mut self, entry: Entry) {
        self.entries.push(entry);
    }

    /// Marks the projection as complete
    ///
    /// Called by the drain task when the event channel closes.
    pub(super) fn set_complete(&mut self) {
        self.completion = Completion::Complete;
    }

    /// Returns whether the event stream has closed and all events have been drained
    pub(super) fn is_complete(&self) -> bool {
        match self.completion {
            Completion::Incomplete => false,
            Completion::Complete => true,
        }
    }

    /// Returns a clone of all accumulated entries in receive order
    pub(super) fn entries(&self) -> Vec<Entry> {
        self.entries.clone()
    }

    /// Returns cloned message entries only
    pub(super) fn messages(&self) -> Vec<Entry> {
        self.entries
            .iter()
            .filter(|entry| match entry {
                Entry::Message(_) => true,
                Entry::Detail(_) => false,
                Entry::Artifact(_) => false,
                Entry::Process(_) => false,
            })
            .cloned()
            .collect()
    }

    /// Returns cloned detail entries only
    pub(super) fn details(&self) -> Vec<Entry> {
        self.entries
            .iter()
            .filter(|entry| match entry {
                Entry::Message(_) => false,
                Entry::Detail(_) => true,
                Entry::Artifact(_) => false,
                Entry::Process(_) => false,
            })
            .cloned()
            .collect()
    }

    /// Returns cloned artifact entries only
    pub(super) fn artifacts(&self) -> Vec<Entry> {
        self.entries
            .iter()
            .filter(|entry| match entry {
                Entry::Message(_) => false,
                Entry::Detail(_) => false,
                Entry::Artifact(_) => true,
                Entry::Process(_) => false,
            })
            .cloned()
            .collect()
    }

    /// Returns cloned process entries only
    pub(super) fn processes(&self) -> Vec<Entry> {
        self.entries
            .iter()
            .filter(|entry| match entry {
                Entry::Message(_) => false,
                Entry::Detail(_) => false,
                Entry::Artifact(_) => false,
                Entry::Process(_) => true,
            })
            .cloned()
            .collect()
    }
}

#[cfg(test)]
mod tests {
    #![allow(
        clippy::missing_panics_doc,
        reason = "an assertion in a test panics by design, so a `# Panics` section would repeat it"
    )]

    use std::fmt;
    use std::sync::Arc;

    use serde::Serialize;

    use super::*;

    #[derive(Clone, Debug, Serialize)]
    struct TestArtifact(String);

    impl fmt::Display for TestArtifact {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            f.write_str(&self.0)
        }
    }

    /// Returns the kind and the text of each entry, in order
    fn describe(entries: &[Entry]) -> Vec<String> {
        entries
            .iter()
            .map(|entry| match entry {
                Entry::Message(text) => format!("message: {text}"),
                Entry::Detail(text) => format!("detail: {text}"),
                Entry::Artifact(artifact) => format!("artifact: {artifact}"),
                Entry::Process(event) => format!("process: {event}"),
            })
            .collect()
    }

    #[test]
    fn artifacts_filters_to_artifact_entries_only() {
        let mut state = ProjectionState::default();
        state.push(Entry::Message("msg".to_string()));
        state.push(Entry::Artifact(Arc::new(TestArtifact("art".to_string()))));

        let artifacts = state.artifacts();

        assert_eq!(describe(&artifacts), ["artifact: art"]);
    }

    #[test]
    fn completion_is_send_sync_and_unpin() {
        fn assert_auto_traits<T: Send + Sync + Unpin>() {}
        assert_auto_traits::<Completion>();
    }

    #[test]
    fn default_is_empty() {
        let state = ProjectionState::default();

        assert!(state.entries().is_empty());
    }

    #[test]
    fn default_is_incomplete() {
        let state = ProjectionState::default();

        assert!(!state.is_complete());
    }

    #[test]
    fn details_filters_to_detail_entries_only() {
        let mut state = ProjectionState::default();
        state.push(Entry::Message("msg".to_string()));
        state.push(Entry::Detail("dtl".to_string()));
        state.push(Entry::Detail("dtl2".to_string()));

        let details = state.details();

        assert_eq!(describe(&details), ["detail: dtl", "detail: dtl2"]);
    }

    #[test]
    fn messages_filters_to_message_entries_only() {
        let mut state = ProjectionState::default();
        state.push(Entry::Message("msg".to_string()));
        state.push(Entry::Detail("dtl".to_string()));
        state.push(Entry::Message("msg2".to_string()));

        let messages = state.messages();

        assert_eq!(describe(&messages), ["message: msg", "message: msg2"]);
    }

    #[test]
    fn push_appends_entry() {
        let mut state = ProjectionState::default();

        state.push(Entry::Message("hello".to_string()));

        let entries = state.entries();
        assert_eq!(describe(&entries), ["message: hello"]);
    }

    #[test]
    fn push_preserves_insertion_order() {
        let mut state = ProjectionState::default();

        state.push(Entry::Message("first".to_string()));
        state.push(Entry::Detail("second".to_string()));
        state.push(Entry::Message("third".to_string()));

        let entries = state.entries();
        assert_eq!(
            describe(&entries),
            ["message: first", "detail: second", "message: third"]
        );
    }

    #[test]
    fn set_complete_marks_state_as_complete() {
        let mut state = ProjectionState::default();

        state.set_complete();

        assert!(state.is_complete());
    }

    #[test]
    fn trait_send() {
        fn assert_send<T: Send>() {}
        assert_send::<ProjectionState>();
    }

    #[test]
    fn trait_sync() {
        fn assert_sync<T: Sync>() {}
        assert_sync::<ProjectionState>();
    }

    #[test]
    fn trait_unpin() {
        fn assert_unpin<T: Unpin>() {}
        assert_unpin::<ProjectionState>();
    }
}
