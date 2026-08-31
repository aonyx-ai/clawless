//! Terminal output adapter
//!
//! This module defines [`TerminalPresenter`], the concrete [`Presenter`] adapter for terminal
//! output. `TerminalPresenter` is a stateless presenter: it renders each event as it arrives,
//! writing to stdout or stderr based on its [`OutputMode`] and filtering by [`Verbosity`].
//!
//! `TerminalPresenter` is constructed once via its builder, consumed by a single call to
//! [`present`], and dropped when the command completes. The builder requires an [`EventReceiver`]
//! and defaults [`Verbosity`] to [`Verbosity::Default`] and [`OutputMode`] to
//! [`OutputMode::Text`].
//!
//! [`EventReceiver`]: clawless_core::event::EventReceiver
//! [`OutputMode`]: crate::output::OutputMode
//! [`Presenter`]: super::Presenter
//! [`Verbosity`]: crate::output::Verbosity
//! [`present`]: super::Presenter::present

use std::future::Future;
use std::io::Write;
use std::pin::Pin;

use async_trait::async_trait;
use bon::Builder;
use clawless_core::event::process::ProcessEvent;
use clawless_core::event::{Event, EventReceiver};
use clawless_core::process::Stream;

use super::Presenter;
use crate::error::CommandResult;
use crate::output::OutputMode;
use crate::output::Verbosity;

/// Terminal presenter adapter
///
/// Renders command output to the terminal. In text mode, all output goes to stdout. In JSON
/// mode, messages go to stderr and artifacts are serialized as JSON to stdout. This follows the
/// convention used by `gh`, `kubectl`, and `jq`.
///
/// `TerminalPresenter` is constructed once via its [builder], consumed by a single call to
/// [`present`], and dropped when the command completes. The presenter holds the [`EventReceiver`]
/// alive during command execution so that [`EventSender`]s do not receive errors when sending.
/// After the command completes, the presenter and its receiver are dropped.
///
/// # Examples
///
/// ```
/// use clawless_core::event::event_channel;
/// use clawless_cli::presenter::TerminalPresenter;
///
/// let (_sender, receiver) = event_channel();
/// let presenter = TerminalPresenter::builder().receiver(receiver).build();
/// ```
///
/// [`EventReceiver`]: clawless_core::event::EventReceiver
/// [`EventSender`]: clawless_core::event::EventSender
/// [`present`]: super::Presenter::present
/// [builder]: TerminalPresenter::builder
#[derive(Debug, Builder)]
pub struct TerminalPresenter {
    /// How much detail to render. The presenter drops events below this level
    #[builder(default)]
    verbosity: Verbosity,

    /// Whether to render events as text or as JSON
    #[builder(default)]
    mode: OutputMode,

    /// Stream of events that the command produces
    receiver: EventReceiver,
}

/// Reports whether an event of a run belongs on the error stream
///
/// A program separates its result from its diagnostics, and the presenter keeps that separation:
/// what a program wrote to its standard error is written to the standard error of the
/// application. A reader that redirects one of the two streams therefore sees the same split that
/// running the program by hand would give.
///
/// The start and the end of a run are not output of the program. They belong with the result,
/// which is where a transcript reads in the order that a person expects.
// r[impl process.render.streams]
fn is_diagnostic(event: &ProcessEvent) -> bool {
    match event {
        ProcessEvent::Started { .. } => false,
        ProcessEvent::Finished { .. } => false,
        ProcessEvent::Line { line, .. } => match line.stream() {
            Stream::StandardError => true,
            Stream::StandardOutput => false,
        },
    }
}

/// Renders one event to the terminal for the given verbosity and output mode
///
/// In text mode, messages and details go to stdout. They therefore interleave with the
/// artifacts, in the order that the command produced them.
///
/// In JSON mode, messages and details go to stderr instead. Stdout then carries only JSON
/// artifacts, which a caller can pipe into another tool.
///
/// The output of an external program is supplementary, so it follows the same rule as a detail
/// and appears only when the user asks for verbose output. A command that wants a program to be
/// visible at the default verbosity says so itself with a message.
///
/// # Panics
///
/// Panics if the process cannot write to the output stream. A reader that closes the pipe
/// causes this panic.
// Writing to the process output stream fails only when the stream itself is gone, such as a
// pipe the reader has closed. A presenter whose output stream has vanished has nowhere left
// to report the failure, so it fails loudly rather than dropping output silently.
// r[impl process.render.verbosity]
#[allow(clippy::expect_used)]
fn render_event(event: Event, verbosity: Verbosity, mode: OutputMode) {
    match event {
        Event::Message(msg) => match verbosity {
            Verbosity::Quiet => {}
            Verbosity::Default | Verbosity::Verbose => match mode {
                OutputMode::Text => {
                    let mut handle = std::io::stdout().lock();
                    writeln!(handle, "{msg}").expect("should write message");
                }
                OutputMode::Json => {
                    let mut handle = std::io::stderr().lock();
                    writeln!(handle, "{msg}").expect("should write message");
                }
            },
        },
        Event::Detail(msg) => match verbosity {
            Verbosity::Quiet | Verbosity::Default => {}
            Verbosity::Verbose => match mode {
                OutputMode::Text => {
                    let mut handle = std::io::stdout().lock();
                    writeln!(handle, "{msg}").expect("should write detail");
                }
                OutputMode::Json => {
                    let mut handle = std::io::stderr().lock();
                    writeln!(handle, "{msg}").expect("should write detail");
                }
            },
        },
        Event::Artifact(artifact) => {
            let line = match mode {
                OutputMode::Text => artifact.to_string(),
                OutputMode::Json => {
                    serde_json::to_string(&artifact).expect("should serialize artifact to JSON")
                }
            };
            let mut handle = std::io::stdout().lock();
            writeln!(handle, "{line}").expect("should write artifact");
        }
        Event::Process(event) => match verbosity {
            Verbosity::Quiet | Verbosity::Default => {}
            Verbosity::Verbose => {
                let to_stderr = match mode {
                    OutputMode::Json => true,
                    OutputMode::Text => is_diagnostic(&event),
                };

                if to_stderr {
                    let mut handle = std::io::stderr().lock();
                    writeln!(handle, "{event}").expect("should write process event");
                } else {
                    let mut handle = std::io::stdout().lock();
                    writeln!(handle, "{event}").expect("should write process event");
                }
            }
        },
    }
}

#[async_trait(?Send)]
impl Presenter for TerminalPresenter {
    async fn present(
        self,
        command: Pin<Box<dyn Future<Output = CommandResult> + Send>>,
    ) -> CommandResult {
        let Self {
            verbosity,
            mode,
            mut receiver,
        } = self;

        let command_handle = tokio::spawn(command);

        while let Some(event) = receiver.recv().await {
            render_event(event, verbosity, mode);
        }

        // The join fails only if the command task panicked or was aborted. Resuming the
        // panic on this thread preserves the original panic message for the user.
        #[allow(clippy::expect_used)]
        command_handle.await.expect("command task panicked")
    }
}

#[cfg(test)]
mod tests {
    // An assertion in a test panics by design. A `# Panics` section on every test
    // would repeat that and give the reader no information.
    #![allow(clippy::missing_panics_doc)]

    use std::time::Duration;

    use clawless_core::event::event_channel;
    use clawless_core::event::process::{Outcome, RunId};
    use clawless_core::process::Invocation;
    use clawless_core::process::Line;

    use super::*;

    #[test]
    fn builder_with_defaults_uses_default_verbosity_and_mode() {
        let (_sender, receiver) = event_channel();

        let presenter = TerminalPresenter::builder().receiver(receiver).build();

        assert_eq!(presenter.verbosity, Verbosity::Default);
        assert_eq!(presenter.mode, OutputMode::Text);
    }

    #[test]
    fn builder_with_mode_uses_provided_mode() {
        let (_sender, receiver) = event_channel();

        let presenter = TerminalPresenter::builder()
            .receiver(receiver)
            .mode(OutputMode::Json)
            .build();

        assert_eq!(presenter.mode, OutputMode::Json);
    }

    #[test]
    fn builder_with_verbosity_uses_provided_verbosity() {
        let (_sender, receiver) = event_channel();

        let presenter = TerminalPresenter::builder()
            .receiver(receiver)
            .verbosity(Verbosity::Verbose)
            .build();

        assert_eq!(presenter.verbosity, Verbosity::Verbose);
    }

    #[test]
    fn is_diagnostic_with_a_finished_run_returns_false() {
        let event = ProcessEvent::Finished {
            id: RunId::next(),
            invocation: Invocation::new("git"),
            outcome: Outcome::Incomplete,
            duration: Duration::ZERO,
        };

        let diagnostic = is_diagnostic(&event);

        assert!(!diagnostic);
    }

    // r[verify process.render.streams]
    #[test]
    fn is_diagnostic_with_a_standard_error_line_returns_true() {
        let event = ProcessEvent::Line {
            id: RunId::next(),
            line: Line::new(Stream::StandardError, "no such file"),
        };

        let diagnostic = is_diagnostic(&event);

        assert!(diagnostic);
    }

    #[test]
    fn is_diagnostic_with_a_standard_output_line_returns_false() {
        let event = ProcessEvent::Line {
            id: RunId::next(),
            line: Line::new(Stream::StandardOutput, "hello"),
        };

        let diagnostic = is_diagnostic(&event);

        assert!(!diagnostic);
    }

    #[test]
    fn is_diagnostic_with_a_started_run_returns_false() {
        let event = ProcessEvent::Started {
            id: RunId::next(),
            invocation: Invocation::new("git"),
            process_id: None,
        };

        let diagnostic = is_diagnostic(&event);

        assert!(!diagnostic);
    }

    #[tokio::test]
    async fn present_consumes_events_from_channel() {
        let (sender, receiver) = event_channel();
        let presenter = TerminalPresenter::builder().receiver(receiver).build();

        presenter
            .present(Box::pin(async move {
                sender
                    .send(Event::Message("consumed".to_string()))
                    .await
                    .expect("should send");
                Ok(())
            }))
            .await
            .expect("should succeed");
    }

    #[tokio::test]
    async fn present_with_error_propagates_error() {
        let (sender, receiver) = event_channel();
        let presenter = TerminalPresenter::builder().receiver(receiver).build();

        let error = presenter
            .present(Box::pin(async move {
                drop(sender);
                Err(anyhow::anyhow!("command failed"))
            }))
            .await
            .expect_err("should fail");

        assert_eq!(error.to_string(), "command failed");
    }

    #[tokio::test]
    async fn present_with_ok_returns_ok() {
        let (sender, receiver) = event_channel();
        let presenter = TerminalPresenter::builder().receiver(receiver).build();

        presenter
            .present(Box::pin(async move {
                drop(sender);
                Ok(())
            }))
            .await
            .expect("should succeed");
    }

    #[tokio::test]
    async fn present_with_receiver_keeps_channel_open_during_execution() {
        let (sender, receiver) = event_channel();
        let presenter = TerminalPresenter::builder().receiver(receiver).build();

        presenter
            .present(Box::pin(async move {
                sender
                    .send(clawless_core::event::Event::Message("hello".to_string()))
                    .await
                    .expect("should send while presenter holds receiver");
                Ok(())
            }))
            .await
            .expect("should succeed");
    }

    #[test]
    fn trait_send() {
        fn assert_send<T: Send>() {}
        assert_send::<TerminalPresenter>();
    }

    #[test]
    fn trait_unpin() {
        fn assert_unpin<T: Unpin>() {}
        assert_unpin::<TerminalPresenter>();
    }
}
