use std::fmt;
use std::sync::Arc;

use clawless_core::event::Artifact;

/// User-facing representation of a single event
///
/// `Entry` is the projection's public vocabulary for events. TUI consumers query the projection
/// and receive entries, never raw events. Each variant carries the same data as its corresponding
/// [`Event`] variant but uses [`Arc`] for artifact values so that entries can be cheaply cloned
/// when returned from queries.
///
/// [`Event`]: clawless_core::event::Event
// r[impl projection.entry.message]
// r[impl projection.entry.detail]
// r[impl projection.entry.artifact]
#[derive(Clone, Debug)]
pub enum Entry {
    /// Informational message
    Message(String),
    /// Supplementary detail
    Detail(String),
    /// Primary command output
    Artifact(Arc<dyn Artifact>),
}

impl fmt::Display for Entry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Entry::Message(text) => f.write_str(text),
            Entry::Detail(text) => f.write_str(text),
            Entry::Artifact(artifact) => fmt::Display::fmt(artifact, f),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

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

    fn test_artifact() -> TestArtifact {
        TestArtifact {
            value: "result".to_string(),
        }
    }

    #[test]
    fn clone_detail_produces_equal_value() {
        let entry = Entry::Detail("info".to_string());

        let cloned = entry.clone();

        let Entry::Detail(s) = &cloned else {
            panic!("expected Entry::Detail");
        };
        assert_eq!(s, "info");
    }

    #[test]
    fn clone_message_produces_equal_value() {
        let entry = Entry::Message("hello".to_string());

        let cloned = entry.clone();

        let Entry::Message(s) = &cloned else {
            panic!("expected Entry::Message");
        };
        assert_eq!(s, "hello");
    }

    #[test]
    fn display_artifact_renders_via_display_trait() {
        let entry = Entry::Artifact(Arc::new(test_artifact()));

        let text = entry.to_string();

        assert_eq!(text, "result");
    }

    #[test]
    fn display_detail_renders_text() {
        let entry = Entry::Detail("info".to_string());

        let text = entry.to_string();

        assert_eq!(text, "info");
    }

    #[test]
    fn display_message_renders_text() {
        let entry = Entry::Message("hello".to_string());

        let text = entry.to_string();

        assert_eq!(text, "hello");
    }

    #[test]
    fn trait_send() {
        fn assert_send<T: Send>() {}
        assert_send::<Entry>();
    }

    #[test]
    fn trait_sync() {
        fn assert_sync<T: Sync>() {}
        assert_sync::<Entry>();
    }

    #[test]
    fn trait_unpin() {
        fn assert_unpin<T: Unpin>() {}
        assert_unpin::<Entry>();
    }
}
