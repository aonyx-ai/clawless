//! Event types for structured command output
//!
//! This module defines [`Event`], the structured message type that commands produce and the
//! Presenter consumes. Events decouple output production from rendering: a command emits events
//! through [`Output`], and the Presenter decides how to render them based on its output mode and
//! verbosity settings.
//!
//! The [`Artifact`] trait enables the [`Event::Artifact`] variant to carry a type-erased value that
//! supports both text rendering ([`Display`]) and JSON serialization ([`Serialize`]). A blanket
//! implementation covers any type satisfying the required bounds, so command authors derive the
//! usual traits and pass values to [`Output::artifact`] without manual trait implementation.
//!
//! [`Output`]: crate::output::Output
//! [`Output::artifact`]: crate::output::Output::artifact

use std::fmt::{Debug, Display};

use serde::Serialize;

pub use self::channel::{SendError, event_channel};
pub use self::receiver::EventReceiver;
pub use self::sender::EventSender;

mod channel;
mod receiver;
mod sender;

/// Trait for artifact values that can be rendered as text or JSON
///
/// `Artifact` combines [`Display`] (for text rendering), [`Serialize`] (for JSON rendering), and
/// [`Debug`] (for diagnostics). The Presenter uses the appropriate trait based on its output mode.
///
/// Command authors do not implement this trait directly. A blanket implementation covers any type
/// that satisfies the required bounds.
///
/// # Examples
///
/// ```
/// use std::fmt;
///
/// use serde::Serialize;
///
/// #[derive(Clone, Debug, Serialize)]
/// struct UserCount {
///     count: usize,
/// }
///
/// impl fmt::Display for UserCount {
///     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
///         write!(f, "{} users", self.count)
///     }
/// }
///
/// // UserCount automatically implements Artifact — no manual impl needed.
/// let artifact: Box<dyn clawless::event::Artifact> = Box::new(UserCount { count: 42 });
/// assert_eq!(artifact.to_string(), "42 users");
/// ```
///
pub trait Artifact: Display + Debug + Send + Sync + erased_serde::Serialize {}

impl<T> Artifact for T where T: Display + Serialize + Debug + Send + Sync + 'static {}

erased_serde::serialize_trait_object!(Artifact);

/// Structured output event produced by commands
///
/// An `Event` represents a single piece of output that a command has produced. Events travel from
/// the producer ([`Output`]) through an async channel to the Presenter, decoupling production from
/// rendering.
///
/// Three variants mirror [`Output`]'s methods:
///
/// - [`Message`] — informational text (shown at default verbosity and above).
/// - [`Detail`] — supplementary text (shown only at verbose verbosity).
/// - [`Event::Artifact`] — the primary data a command produces, carried as a trait object that the
///   Presenter can render via [`Display`] or [`Serialize`].
///
/// The Presenter decides which events to render based on its [`Verbosity`] setting. [`Output`]
/// emits all events unconditionally.
///
/// [`Detail`]: Event::Detail
/// [`Message`]: Event::Message
/// [`Output`]: crate::output::Output
/// [`Verbosity`]: crate::output::Verbosity
#[derive(Debug)]
pub enum Event {
    /// Informational message
    Message(String),
    /// Supplementary detail
    Detail(String),
    /// Primary command output
    Artifact(Box<dyn Artifact>),
}

#[cfg(test)]
mod tests {
    use std::fmt;

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
            value: "hello".to_string(),
        }
    }

    #[test]
    fn artifact_debug_delegates_to_inner_type() {
        let event = Event::Artifact(Box::new(test_artifact()));

        let debug = format!("{event:?}");

        assert!(debug.contains("hello"));
    }

    #[test]
    fn artifact_display_renders_via_display_trait() {
        let boxed: Box<dyn Artifact> = Box::new(test_artifact());

        let display = boxed.to_string();

        assert_eq!(display, "hello");
    }

    #[test]
    fn artifact_serializes_via_erased_serde() {
        let boxed: Box<dyn Artifact> = Box::new(test_artifact());

        let json = serde_json::to_string(&boxed).expect("should serialize");

        assert_eq!(json, r#"{"value":"hello"}"#);
    }

    #[test]
    fn detail_with_empty_string_is_valid() {
        let event = Event::Detail(String::new());

        let debug = format!("{event:?}");

        assert!(debug.contains("Detail"));
    }

    #[test]
    fn message_with_empty_string_is_valid() {
        let event = Event::Message(String::new());

        let debug = format!("{event:?}");

        assert!(debug.contains("Message"));
    }

    #[test]
    fn trait_send() {
        fn assert_send<T: Send>() {}
        assert_send::<Event>();
    }

    #[test]
    fn trait_sync() {
        fn assert_sync<T: Sync>() {}
        assert_sync::<Event>();
    }

    #[test]
    fn trait_unpin() {
        fn assert_unpin<T: Unpin>() {}
        assert_unpin::<Event>();
    }
}
