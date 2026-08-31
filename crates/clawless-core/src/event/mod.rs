//! Event types for structured command output
//!
//! This module defines [`Event`], the structured message type that commands produce and the
//! Presenter consumes. Events decouple output production from rendering: a command emits events
//! through Output, and the Presenter decides how to render them based on its output mode and
//! verbosity settings.
//!
//! The [`Artifact`] trait enables the [`Event::Artifact`] variant to carry a type-erased value that
//! supports both text rendering ([`Display`]) and JSON serialization ([`Serialize`]). A blanket
//! implementation covers any type satisfying the required bounds, so command authors derive the
//! usual traits and pass values to Output without manual trait implementation.
//!
//! The [`Event::Process`] variant carries the [`ProcessEvent`] of an external program that a
//! command runs. Those events arrive while the program runs, which lets a presenter show progress
//! instead of waiting for the program to end.
//!
//! [`ProcessEvent`]: process::ProcessEvent

use std::fmt::{Debug, Display};

use serde::Serialize;

pub use self::channel::{SendError, event_channel};
use self::process::ProcessEvent;
pub use self::receiver::EventReceiver;
pub use self::sender::EventSender;

pub mod process;

/// Bounded channel that carries events from a command to its presenter
mod channel;
/// The half of the event channel that reads events
mod receiver;
/// The half of the event channel that sends events
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
/// let artifact: Box<dyn clawless_core::event::Artifact> = Box::new(UserCount { count: 42 });
/// assert_eq!(artifact.to_string(), "42 users");
/// ```
///
// r[impl event.artifact.structured]
// r[impl event.artifact.text]
pub trait Artifact: Display + Debug + Send + Sync + erased_serde::Serialize {}

// r[impl event.artifact.zero-cost]
impl<T> Artifact for T where T: Display + Serialize + Debug + Send + Sync + 'static {}

erased_serde::serialize_trait_object!(Artifact);

/// Structured output event produced by commands
///
/// An `Event` represents a single piece of output that a command has produced. Events travel from
/// the producer through an async channel to the Presenter, decoupling production from rendering.
///
/// Four variants:
///
/// - [`Message`] — informational text (shown at default verbosity and above).
/// - [`Detail`] — supplementary text (shown only at verbose verbosity).
/// - [`Event::Artifact`] — the primary data a command produces, carried as a trait object that the
///   Presenter can render via [`Display`] or [`Serialize`].
/// - [`Event::Process`] — one step in the run of an external program that the command started.
///
/// The Presenter decides which events to render based on its verbosity setting.
///
/// [`Detail`]: Event::Detail
/// [`Message`]: Event::Message
// r[impl event.safety.event-send]
#[derive(Debug)]
pub enum Event {
    /// Informational message
    // r[impl event.output.message]
    Message(String),
    /// Supplementary detail
    // r[impl event.output.detail]
    Detail(String),
    /// Primary command output
    // r[impl event.output.artifact]
    Artifact(Box<dyn Artifact>),
    /// One step in the run of an external program
    ///
    /// The event is boxed so that the size of an `Event` stays that of its text variants. A run
    /// names its command in the events that start and end it, and an unboxed variant would make
    /// every message in the channel as large as that name.
    // r[impl event.output.process]
    Process(Box<ProcessEvent>),
}

#[cfg(test)]
mod tests {
    // An assertion in a test panics by design. A `# Panics` section on every test
    // would repeat that and give the reader no information.
    #![allow(clippy::missing_panics_doc)]

    use std::fmt;

    use serde::Serialize;

    use super::*;
    use crate::event::process::RunId;
    use crate::process::Invocation;

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

    // r[verify event.artifact.text]
    #[test]
    fn artifact_display_renders_via_display_trait() {
        let boxed: Box<dyn Artifact> = Box::new(test_artifact());

        let display = boxed.to_string();

        assert_eq!(display, "hello");
    }

    // r[verify event.artifact.structured]
    #[test]
    fn artifact_serializes_via_erased_serde() {
        let boxed: Box<dyn Artifact> = Box::new(test_artifact());

        let json = serde_json::to_string(&boxed).expect("should serialize");

        assert_eq!(json, r#"{"value":"hello"}"#);
    }

    // r[verify event.output.detail]
    #[test]
    fn detail_with_empty_string_is_valid() {
        let event = Event::Detail(String::new());

        let debug = format!("{event:?}");

        assert!(debug.contains("Detail"));
    }

    // r[verify event.output.message]
    #[test]
    fn message_with_empty_string_is_valid() {
        let event = Event::Message(String::new());

        let debug = format!("{event:?}");

        assert!(debug.contains("Message"));
    }

    // r[verify event.output.process]
    #[test]
    fn process_carries_the_event_of_a_run() {
        let id = RunId::next();

        let event = Event::Process(Box::new(ProcessEvent::Started {
            id,
            invocation: Invocation::new("git"),
            process_id: None,
        }));

        assert_eq!(
            match event {
                Event::Process(event) => Some(event.id()),
                Event::Message(_) => None,
                Event::Detail(_) => None,
                Event::Artifact(_) => None,
            },
            Some(id)
        );
    }

    // r[verify event.safety.event-send]
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
