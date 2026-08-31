use super::Entry;

/// Internal mutable state shared between the drain task and query methods
///
/// `ProjectionState` holds the accumulated entries and a completion flag. The drain task writes
/// to this state behind a write lock; query methods read behind a read lock.
///
/// [`Projection`]: super::Projection
#[derive(Debug, Default)]
pub(super) struct ProjectionState {
    /// The entries so far, in the order that the drain task received them
    entries: Vec<Entry>,
    /// True after the event channel closes and the drain task processes every event
    complete: bool,
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
        self.complete = true;
    }

    /// Returns whether the event stream has closed and all events have been drained
    pub(super) fn is_complete(&self) -> bool {
        self.complete
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
    // An assertion in a test panics by design. A `# Panics` section on every test
    // would repeat that and give the reader no information.
    #![allow(clippy::missing_panics_doc)]

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

    #[test]
    fn artifacts_filters_to_artifact_entries_only() {
        let mut state = ProjectionState::default();
        state.push(Entry::Message("msg".to_string()));
        state.push(Entry::Artifact(Arc::new(TestArtifact("art".to_string()))));

        let artifacts = state.artifacts();

        assert_eq!(artifacts.len(), 1);
        let Entry::Artifact(a) = &artifacts[0] else {
            panic!("expected Entry::Artifact");
        };
        assert_eq!(a.to_string(), "art");
    }

    #[test]
    fn default_is_empty_and_incomplete() {
        let state = ProjectionState::default();

        assert!(state.entries().is_empty());
        assert!(!state.is_complete());
    }

    #[test]
    fn details_filters_to_detail_entries_only() {
        let mut state = ProjectionState::default();
        state.push(Entry::Message("msg".to_string()));
        state.push(Entry::Detail("dtl".to_string()));
        state.push(Entry::Detail("dtl2".to_string()));

        let details = state.details();

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

    #[test]
    fn messages_filters_to_message_entries_only() {
        let mut state = ProjectionState::default();
        state.push(Entry::Message("msg".to_string()));
        state.push(Entry::Detail("dtl".to_string()));
        state.push(Entry::Message("msg2".to_string()));

        let messages = state.messages();

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

    #[test]
    fn push_appends_entry() {
        let mut state = ProjectionState::default();

        state.push(Entry::Message("hello".to_string()));

        let entries = state.entries();
        assert_eq!(entries.len(), 1);
        let Entry::Message(s) = &entries[0] else {
            panic!("expected Entry::Message");
        };
        assert_eq!(s, "hello");
    }

    // r[verify projection.entry.order]
    #[test]
    fn push_preserves_insertion_order() {
        let mut state = ProjectionState::default();

        state.push(Entry::Message("first".to_string()));
        state.push(Entry::Detail("second".to_string()));
        state.push(Entry::Message("third".to_string()));

        let entries = state.entries();
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

    // r[verify projection.lifecycle.complete]
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
