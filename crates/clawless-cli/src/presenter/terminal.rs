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
use clawless_core::context::Interactivity;
use clawless_core::event::process::ProcessEvent;
use clawless_core::event::prompt::PromptRequest;
use clawless_core::event::{Event, EventReceiver};
use clawless_core::process::Stream;

use super::Presenter;
use super::line_question;
use super::line_reader::LineReader;
use crate::error::CommandResult;
use crate::output::OutputMode;
use crate::output::Verbosity;

/// Terminal presenter adapter
///
/// Renders command output to the terminal. In text mode, all output goes to stdout. In JSON
/// mode, messages go to stderr and artifacts are serialized as JSON to stdout. This follows the
/// convention used by `gh`, `kubectl`, and `jq`.
///
/// The presenter also asks the user the questions of the command. It writes a question to
/// stderr in both modes, and the user answers with one line on stdin. Verbosity and output mode
/// do not suppress a question. A presenter that is not built as interactive drops every
/// question.
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

    /// Whether a user is present who can answer a prompt
    #[builder(default)]
    interactivity: Interactivity,

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

/// Renders one event for the given verbosity and output mode
///
/// `stdout` and `stderr` are the standard output and the standard error of the application. A
/// test passes buffers.
///
/// In text mode, messages and details go to `stdout`. They therefore interleave with the
/// artifacts, in the order that the command produced them.
///
/// In JSON mode, messages and details go to `stderr` instead. `stdout` then carries only JSON
/// artifacts, which a caller can pipe into another tool.
///
/// The output of an external program is supplementary, so it follows the same rule as a detail
/// and appears only when the user asks for verbose output. A command that wants a program to be
/// visible at the default verbosity says so itself with a message.
///
/// The presenter handles a prompt before it renders an event. This function drops a prompt that
/// reaches it.
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
fn render_event(
    event: Event,
    verbosity: Verbosity,
    mode: OutputMode,
    stdout: &mut impl Write,
    stderr: &mut impl Write,
) {
    match event {
        Event::Message(msg) => match verbosity {
            Verbosity::Quiet => {}
            Verbosity::Default | Verbosity::Verbose => match mode {
                OutputMode::Text => writeln!(stdout, "{msg}").expect("should write message"),
                OutputMode::Json => writeln!(stderr, "{msg}").expect("should write message"),
            },
        },
        Event::Detail(msg) => match verbosity {
            Verbosity::Quiet | Verbosity::Default => {}
            Verbosity::Verbose => match mode {
                OutputMode::Text => writeln!(stdout, "{msg}").expect("should write detail"),
                OutputMode::Json => writeln!(stderr, "{msg}").expect("should write detail"),
            },
        },
        Event::Artifact(artifact) => {
            let line = match mode {
                OutputMode::Text => artifact.to_string(),
                OutputMode::Json => {
                    serde_json::to_string(&artifact).expect("should serialize artifact to JSON")
                }
            };
            writeln!(stdout, "{line}").expect("should write artifact");
        }
        Event::Process(event) => match verbosity {
            Verbosity::Quiet | Verbosity::Default => {}
            Verbosity::Verbose => {
                let to_stderr = match mode {
                    OutputMode::Json => true,
                    OutputMode::Text => is_diagnostic(&event),
                };

                if to_stderr {
                    writeln!(stderr, "{event}").expect("should write process event");
                } else {
                    writeln!(stdout, "{event}").expect("should write process event");
                }
            }
        },
        Event::Prompt(request) => drop(request),
    }
}

/// Asks the user the question of a request, on the display and from the input
///
/// The display is the standard error in every output mode, so the standard output carries only
/// the result of the command.
async fn ask_user(request: PromptRequest, input: &mut LineReader, display: &mut impl Write) {
    match request {
        PromptRequest::Confirm { question, reply } => {
            line_question::ask(&question, reply, input, display).await;
        }
        PromptRequest::Text { question, reply } => {
            line_question::ask(&question, reply, input, display).await;
        }
    }
}

impl TerminalPresenter {
    /// Presents the output of a command on the given streams
    ///
    /// [`Presenter::present`] passes the standard output and the standard error of the
    /// application, and the standard input if a user is present. A test passes buffers and
    /// scripted input.
    ///
    /// Each write locks its stream for one line only, so the presenter does not block other
    /// writers for the whole presentation.
    ///
    /// The presenter handles one event at a time. A question therefore appears after the
    /// output that the command sent before it, and later output appears after the answer.
    /// Verbosity does not apply to a question. Without an input, the presenter drops every
    /// prompt.
    ///
    /// # Errors
    ///
    /// Returns the error of the command if the command fails.
    ///
    /// # Panics
    ///
    /// Panics if the command panicked, and if a stream cannot be written.
    async fn present_on(
        self,
        command: Pin<Box<dyn Future<Output = CommandResult> + Send>>,
        stdout: &mut impl Write,
        stderr: &mut impl Write,
        mut input: Option<LineReader>,
    ) -> CommandResult {
        let Self {
            verbosity,
            mode,
            interactivity: _,
            mut receiver,
        } = self;

        let command_handle = tokio::spawn(command);

        while let Some(event) = receiver.recv().await {
            match event {
                Event::Prompt(request) => match &mut input {
                    Some(input) => ask_user(*request, input, stderr).await,
                    None => drop(request),
                },
                Event::Message(_) | Event::Detail(_) | Event::Artifact(_) | Event::Process(_) => {
                    render_event(event, verbosity, mode, stdout, stderr);
                }
            }
        }

        // The join fails only if the command task panicked or was aborted. Resuming the
        // panic on this thread preserves the original panic message for the user.
        #[allow(clippy::expect_used)]
        command_handle.await.expect("command task panicked")
    }
}

#[async_trait(?Send)]
impl Presenter for TerminalPresenter {
    async fn present(
        self,
        command: Pin<Box<dyn Future<Output = CommandResult> + Send>>,
    ) -> CommandResult {
        let input = match self.interactivity {
            Interactivity::Interactive => Some(LineReader::stdin()),
            Interactivity::NonInteractive => None,
        };

        self.present_on(
            command,
            &mut std::io::stdout(),
            &mut std::io::stderr(),
            input,
        )
        .await
    }
}

#[cfg(test)]
mod tests {
    // An assertion in a test panics by design. A `# Panics` section on every test
    // would repeat that and give the reader no information.
    #![allow(clippy::missing_panics_doc)]

    use std::collections::VecDeque;
    use std::sync::{Arc, Mutex};
    use std::time::Duration;

    use clawless_core::context::Interactivity;
    use clawless_core::event::process::{Outcome, RunId};
    use clawless_core::event::prompt::PromptRequest;
    use clawless_core::event::{EventSender, event_channel};
    use clawless_core::output::Output;
    use clawless_core::process::Invocation;
    use clawless_core::process::Line;
    use clawless_core::prompt::{Confirm, Confirmation, Prompt};

    use super::*;

    /// Collects what a presenter writes to both of its streams, in the order of the writes
    #[derive(Clone, Debug, Default)]
    struct Transcript {
        /// The bytes of every write so far
        bytes: Arc<Mutex<Vec<u8>>>,
    }

    impl Transcript {
        /// Returns everything that was written so far
        fn text(&self) -> String {
            String::from_utf8_lossy(&self.bytes.lock().expect("should lock")).into_owned()
        }
    }

    impl Write for Transcript {
        fn write(&mut self, buffer: &[u8]) -> std::io::Result<usize> {
            self.bytes
                .lock()
                .expect("should lock")
                .extend_from_slice(buffer);

            Ok(buffer.len())
        }

        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }

    /// Returns a command that says what it is about to do, asks, and reports the answer
    fn confirming(sender: EventSender) -> Pin<Box<dyn Future<Output = CommandResult> + Send>> {
        Box::pin(async move {
            let output = Output::new(sender);
            let prompt = Prompt::builder()
                .output(output.clone())
                .interactivity(Interactivity::Interactive)
                .build();

            output.message("About to release 1.4.0.").await?;
            let answer = prompt
                .confirm(Confirm::new("Release?").with_default(Confirmation::No))
                .await?;
            output.message(format!("The user said {answer:?}.")).await?;

            Ok(())
        })
    }

    /// Returns an input over the lines that a user types
    fn typed(lines: &[&str]) -> LineReader {
        let mut lines = lines
            .iter()
            .map(|line| (*line).to_owned())
            .collect::<VecDeque<_>>();

        LineReader::new(move |buffer| {
            let line = lines.pop_front().unwrap_or_default();
            buffer.push_str(&line);
            Ok(line.len())
        })
    }

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
    async fn present_on_with_a_prompt_asks_after_the_earlier_output() {
        let (sender, receiver) = event_channel();
        let presenter = TerminalPresenter::builder().receiver(receiver).build();
        let transcript = Transcript::default();

        presenter
            .present_on(
                confirming(sender),
                &mut transcript.clone(),
                &mut transcript.clone(),
                Some(typed(&["y\n"])),
            )
            .await
            .expect("should succeed");

        assert_eq!(
            transcript.text(),
            "About to release 1.4.0.\nRelease? [y/N] The user said Yes.\n"
        );
    }

    #[tokio::test]
    async fn present_on_with_a_prompt_at_quiet_verbosity_still_asks() {
        let (sender, receiver) = event_channel();
        let presenter = TerminalPresenter::builder()
            .receiver(receiver)
            .verbosity(Verbosity::Quiet)
            .build();
        let (mut stdout, mut stderr) = (Vec::new(), Vec::new());

        presenter
            .present_on(
                confirming(sender),
                &mut stdout,
                &mut stderr,
                Some(typed(&["y\n"])),
            )
            .await
            .expect("should succeed");

        assert_eq!(String::from_utf8_lossy(&stderr), "Release? [y/N] ");
    }

    #[tokio::test]
    async fn present_on_with_a_prompt_in_json_mode_keeps_the_question_off_stdout() {
        let (sender, receiver) = event_channel();
        let presenter = TerminalPresenter::builder()
            .receiver(receiver)
            .mode(OutputMode::Json)
            .build();
        let (mut stdout, mut stderr) = (Vec::new(), Vec::new());

        presenter
            .present_on(
                confirming(sender),
                &mut stdout,
                &mut stderr,
                Some(typed(&["y\n"])),
            )
            .await
            .expect("should succeed");

        assert_eq!(String::from_utf8_lossy(&stdout), "");
    }

    #[tokio::test]
    async fn present_on_with_a_prompt_in_text_mode_asks_on_stderr() {
        let (sender, receiver) = event_channel();
        let presenter = TerminalPresenter::builder().receiver(receiver).build();
        let (mut stdout, mut stderr) = (Vec::new(), Vec::new());

        presenter
            .present_on(
                confirming(sender),
                &mut stdout,
                &mut stderr,
                Some(typed(&["y\n"])),
            )
            .await
            .expect("should succeed");

        assert_eq!(String::from_utf8_lossy(&stderr), "Release? [y/N] ");
    }

    #[tokio::test]
    async fn present_on_with_a_text_prompt_sends_the_line_to_the_command() {
        let (sender, receiver) = event_channel();
        let presenter = TerminalPresenter::builder().receiver(receiver).build();
        let transcript = Transcript::default();

        presenter
            .present_on(
                Box::pin(async move {
                    let output = Output::new(sender);
                    let prompt = Prompt::builder()
                        .output(output.clone())
                        .interactivity(Interactivity::Interactive)
                        .build();

                    let title = prompt.text("Title").await?;
                    output.message(format!("The title is {title}.")).await?;

                    Ok(())
                }),
                &mut transcript.clone(),
                &mut transcript.clone(),
                Some(typed(&["Fix the race\n"])),
            )
            .await
            .expect("should succeed");

        assert_eq!(transcript.text(), "Title: The title is Fix the race.\n");
    }

    #[tokio::test]
    async fn present_on_without_an_input_drops_the_prompt() {
        let (sender, receiver) = event_channel();
        let presenter = TerminalPresenter::builder().receiver(receiver).build();
        let (mut stdout, mut stderr) = (Vec::new(), Vec::new());

        let error = presenter
            .present_on(confirming(sender), &mut stdout, &mut stderr, None)
            .await
            .expect_err("should fail");

        assert_eq!(
            error.root_cause().to_string(),
            "the prompt was dropped without an answer"
        );
    }

    #[tokio::test]
    async fn present_on_writes_the_events_of_the_command_to_the_streams() {
        let (sender, receiver) = event_channel();
        let presenter = TerminalPresenter::builder().receiver(receiver).build();
        let (mut stdout, mut stderr) = (Vec::new(), Vec::new());

        presenter
            .present_on(
                Box::pin(async move {
                    sender
                        .send(Event::Message("hello".to_owned()))
                        .await
                        .expect("should send");
                    Ok(())
                }),
                &mut stdout,
                &mut stderr,
                None,
            )
            .await
            .expect("should succeed");

        assert_eq!((stdout, stderr), (b"hello\n".to_vec(), Vec::new()));
    }

    #[tokio::test]
    async fn present_with_a_prompt_drops_the_request() {
        let (sender, receiver) = event_channel();
        let presenter = TerminalPresenter::builder().receiver(receiver).build();

        let error = presenter
            .present(Box::pin(async move {
                let (request, pending) = PromptRequest::confirm(Confirm::new("Release?"));
                sender
                    .send(Event::Prompt(Box::new(request)))
                    .await
                    .expect("should send");
                pending.wait().await?;
                Ok(())
            }))
            .await
            .expect_err("should fail");

        assert_eq!(
            error.to_string(),
            "the prompt was dropped without an answer"
        );
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
    fn render_event_with_a_detail_at_default_verbosity_writes_nothing() {
        let (mut stdout, mut stderr) = (Vec::new(), Vec::new());

        render_event(
            Event::Detail("reading".to_owned()),
            Verbosity::Default,
            OutputMode::Text,
            &mut stdout,
            &mut stderr,
        );

        assert_eq!((stdout, stderr), (Vec::new(), Vec::new()));
    }

    #[test]
    fn render_event_with_a_detail_at_verbose_verbosity_writes_to_stdout() {
        let (mut stdout, mut stderr) = (Vec::new(), Vec::new());

        render_event(
            Event::Detail("reading".to_owned()),
            Verbosity::Verbose,
            OutputMode::Text,
            &mut stdout,
            &mut stderr,
        );

        assert_eq!((stdout, stderr), (b"reading\n".to_vec(), Vec::new()));
    }

    #[test]
    fn render_event_with_a_message_at_quiet_verbosity_writes_nothing() {
        let (mut stdout, mut stderr) = (Vec::new(), Vec::new());

        render_event(
            Event::Message("hello".to_owned()),
            Verbosity::Quiet,
            OutputMode::Text,
            &mut stdout,
            &mut stderr,
        );

        assert_eq!((stdout, stderr), (Vec::new(), Vec::new()));
    }

    #[test]
    fn render_event_with_a_message_in_json_mode_writes_to_stderr() {
        let (mut stdout, mut stderr) = (Vec::new(), Vec::new());

        render_event(
            Event::Message("hello".to_owned()),
            Verbosity::Default,
            OutputMode::Json,
            &mut stdout,
            &mut stderr,
        );

        assert_eq!((stdout, stderr), (Vec::new(), b"hello\n".to_vec()));
    }

    #[test]
    fn render_event_with_a_message_in_text_mode_writes_to_stdout() {
        let (mut stdout, mut stderr) = (Vec::new(), Vec::new());

        render_event(
            Event::Message("hello".to_owned()),
            Verbosity::Default,
            OutputMode::Text,
            &mut stdout,
            &mut stderr,
        );

        assert_eq!((stdout, stderr), (b"hello\n".to_vec(), Vec::new()));
    }

    #[test]
    fn render_event_with_a_standard_error_line_in_text_mode_writes_to_stderr() {
        let (mut stdout, mut stderr) = (Vec::new(), Vec::new());

        render_event(
            Event::Process(Box::new(ProcessEvent::Line {
                id: RunId::next(),
                line: Line::new(Stream::StandardError, "no such file"),
            })),
            Verbosity::Verbose,
            OutputMode::Text,
            &mut stdout,
            &mut stderr,
        );

        assert_eq!((stdout, stderr), (Vec::new(), b"no such file\n".to_vec()));
    }

    #[test]
    fn render_event_with_an_artifact_in_json_mode_writes_json_to_stdout() {
        let (mut stdout, mut stderr) = (Vec::new(), Vec::new());

        render_event(
            Event::Artifact(Box::new(42)),
            Verbosity::Quiet,
            OutputMode::Json,
            &mut stdout,
            &mut stderr,
        );

        assert_eq!((stdout, stderr), (b"42\n".to_vec(), Vec::new()));
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
