//! Event types for structured command output
//!
//! This module defines [`Event`], the structured message type that commands produce and the
//! Presenter consumes. Events decouple output production from rendering: a command emits events
//! through Output, and the Presenter decides how to render them based on its output mode and
//! verbosity settings.
//!
//! The [`Artifact`] trait enables the [`Event::Artifact`] variant to carry a type-erased value that
//! supports both text rendering ([`TextForm`]) and JSON serialization ([`Serialize`]). A blanket
//! implementation covers any type satisfying the required bounds, so command authors derive the
//! usual traits and pass values to Output without manual trait implementation.
//!
//! The [`TextForm`] trait lets a value offer a styled text form next to its plain one. Every type
//! with [`Display`] has one form. The presenter chooses the [`Styling`] of the text that it
//! writes, because only the presenter knows whether that text reaches a terminal.
//!
//! The [`Event::Process`] variant carries the [`ProcessEvent`] of an external program that a
//! command runs. Those events arrive while the program runs, which lets a presenter show progress
//! instead of waiting for the program to end.
//!
//! The [`Event::Prompt`] variant carries a [`PromptRequest`], which is a question for the user.
//! The request arrives after the events that the command sent before it.
//!
//! [`Display`]: std::fmt::Display
//! [`ProcessEvent`]: process::ProcessEvent
//! [`PromptRequest`]: prompt::PromptRequest

use std::fmt::{self, Debug};

use serde::Serialize;

pub use self::channel::{SendError, event_channel};
pub use self::formatted_text::FormattedText;
use self::process::ProcessEvent;
use self::prompt::PromptRequest;
pub use self::receiver::EventReceiver;
pub use self::sender::EventSender;
pub use self::styling::Styling;
pub use self::text_form::TextForm;

pub mod process;
pub mod prompt;

/// Bounded channel that carries events from a command to its presenter
mod channel;
/// The text of a message or a detail, written in every styling
mod formatted_text;
/// The half of the event channel that reads events
mod receiver;
/// The half of the event channel that sends events
mod sender;
/// Whether text may carry terminal styles
mod styling;
/// Values that write their text in a styling that the presenter chooses
mod text_form;

/// Trait for artifact values that can be rendered as text or JSON
///
/// `Artifact` combines a [`TextForm`] (for text rendering), [`Serialize`] (for JSON rendering),
/// and [`Debug`] (for diagnostics). The Presenter uses the appropriate form based on its output
/// mode, and it chooses the [`Styling`] of the text form.
///
/// Command authors do not implement this trait directly. A blanket implementation covers any type
/// that implements [`TextForm`], [`Serialize`], [`Debug`], [`Send`], and [`Sync`]. Every type
/// with [`Display`] has a [`TextForm`], so such a type is an artifact with one text form. A type
/// with a styled form implements [`TextForm`] itself and becomes an artifact the same way.
///
/// A trait object of `Artifact` does not implement [`Display`], so a consumer of an
/// [`Event::Artifact`] chooses a styling with [`TextForm::to_text`] or [`TextForm::fmt_text`].
/// If the trait object implemented [`Display`], a box of it would get its [`TextForm`] from
/// [`Display`], write the plain form for every styling, and hide the styled form of the value in
/// it. `Box<dyn Artifact>` instead forwards [`TextForm`] to the value in it, so a command can
/// pass on an artifact that it received as a box. `Arc<dyn Artifact>` is not an artifact. The
/// coherence rules of Rust reject a forwarding impl for `Arc` next to the blanket implementation
/// of [`TextForm`] for [`Display`] types, because [`Arc`] is not a fundamental type.
///
/// # Examples
///
/// An artifact with one text form:
///
/// ```
/// use std::fmt;
///
/// use clawless_core::event::{Artifact, Styling, TextForm};
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
/// let artifact: Box<dyn Artifact> = Box::new(UserCount { count: 42 });
/// assert_eq!(artifact.to_text(Styling::Plain), "42 users");
/// ```
///
/// An artifact with a styled form and a plain form:
///
/// ```
/// use std::fmt;
///
/// use clawless_core::event::{Artifact, Styling, TextForm};
/// use serde::Serialize;
///
/// #[derive(Clone, Debug, Serialize)]
/// struct Verdict {
///     passed: bool,
/// }
///
/// impl TextForm for Verdict {
///     fn fmt_text(&self, f: &mut fmt::Formatter<'_>, styling: Styling) -> fmt::Result {
///         let word = match self.passed {
///             true => "passed",
///             false => "failed",
///         };
///
///         match styling {
///             Styling::Styled => write!(f, "\x1b[1m{word}\x1b[0m"),
///             Styling::Plain => f.write_str(word),
///         }
///     }
/// }
///
/// // Verdict implements Artifact through its TextForm.
/// let artifact: Box<dyn Artifact> = Box::new(Verdict { passed: true });
/// assert_eq!(artifact.to_text(Styling::Styled), "\x1b[1mpassed\x1b[0m");
/// ```
///
/// [`Arc`]: std::sync::Arc
/// [`Display`]: std::fmt::Display
pub trait Artifact: TextForm + Debug + Send + Sync + erased_serde::Serialize {}

impl<T> Artifact for T where T: TextForm + Serialize + Debug + Send + Sync + 'static {}

erased_serde::serialize_trait_object!(Artifact);

/// Writes the text form of the artifact in the box, in the styling that the presenter chose
impl TextForm for Box<dyn Artifact> {
    fn fmt_text(&self, f: &mut fmt::Formatter<'_>, styling: Styling) -> fmt::Result {
        (**self).fmt_text(f, styling)
    }
}

/// Structured output event produced by commands
///
/// An `Event` represents a single piece of output that a command has produced. Events travel from
/// the producer through an async channel to the Presenter, decoupling production from rendering.
///
/// Five variants:
///
/// - [`Message`] — informational text (shown at default verbosity and above).
/// - [`Detail`] — supplementary text (shown only at verbose verbosity).
/// - [`Event::Artifact`] — the primary data a command produces, carried as a trait object that the
///   Presenter can render via [`TextForm`] or [`Serialize`].
/// - [`Event::Process`] — one step in the run of an external program that the command started.
/// - [`Event::Prompt`] — a question for the user, with the reply that answers it.
///
/// The Presenter decides which events to render based on its verbosity setting. Verbosity does
/// not apply to a prompt, because the command waits for the answer. A presenter that cannot ask
/// the user drops the request.
///
/// [`Detail`]: Event::Detail
/// [`Message`]: Event::Message
#[derive(Debug)]
pub enum Event {
    /// Informational message
    Message(String),
    /// Supplementary detail
    Detail(String),
    /// Primary command output
    Artifact(Box<dyn Artifact>),
    /// One step in the run of an external program
    ///
    /// The event is boxed so that the size of an `Event` stays that of its text variants. A run
    /// names its command in the events that start and end it, and an unboxed variant would make
    /// every message in the channel as large as that name.
    Process(Box<ProcessEvent>),
    /// A question for the user
    ///
    /// The request is boxed so that the size of an `Event` stays that of its text variants.
    Prompt(Box<PromptRequest>),
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
    use crate::prompt::Confirm;
    use crate::test_support::Bold;

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
                Event::Prompt(_) => None,
            },
            Some(id)
        );
    }

    #[test]
    fn prompt_carries_the_request_of_a_command() {
        let (request, _pending) = PromptRequest::confirm(Confirm::new("Release?"));

        let event = Event::Prompt(Box::new(request));

        assert_eq!(
            match event {
                Event::Prompt(request) => Some(request.question().to_owned()),
                Event::Message(_) => None,
                Event::Detail(_) => None,
                Event::Artifact(_) => None,
                Event::Process(_) => None,
            },
            Some("Release?".to_owned())
        );
    }

    #[test]
    fn to_text_of_a_box_in_a_box_returns_the_styled_form_of_the_value() {
        let inner: Box<dyn Artifact> = Box::new(Bold::new("hello"));
        let boxed: Box<dyn Artifact> = Box::new(inner);

        let text = boxed.to_text(Styling::Styled);

        assert_eq!(text, "\x1b[1mhello\x1b[0m");
    }

    #[test]
    fn to_text_of_a_boxed_artifact_returns_the_styled_form_of_the_value() {
        let boxed: Box<dyn Artifact> = Box::new(Bold::new("hello"));

        let text = boxed.to_text(Styling::Styled);

        assert_eq!(text, "\x1b[1mhello\x1b[0m");
    }

    #[test]
    fn to_text_of_a_boxed_display_artifact_returns_its_display_output() {
        let boxed: Box<dyn Artifact> = Box::new(test_artifact());

        let text = boxed.to_text(Styling::Plain);

        assert_eq!(text, "hello");
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
