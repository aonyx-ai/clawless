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
//! [`EventReceiver`]: crate::event::EventReceiver
//! [`OutputMode`]: crate::output::OutputMode
//! [`Presenter`]: super::Presenter
//! [`Verbosity`]: crate::output::Verbosity
//! [`present`]: super::Presenter::present

use std::future::Future;
use std::pin::Pin;

use async_trait::async_trait;
use bon::Builder;

use super::Presenter;
use super::writer::Writer;
use crate::error::CommandResult;
use crate::event::{Event, EventReceiver};
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
/// use clawless::event::event_channel;
/// use clawless::presenter::TerminalPresenter;
///
/// let (_sender, receiver) = event_channel();
/// let presenter = TerminalPresenter::builder().receiver(receiver).build();
/// ```
///
/// [`EventReceiver`]: crate::event::EventReceiver
/// [`EventSender`]: crate::event::EventSender
/// [`present`]: super::Presenter::present
/// [builder]: TerminalPresenter::builder
#[derive(Debug, Builder)]
pub struct TerminalPresenter {
    #[builder(default)]
    verbosity: Verbosity,

    #[builder(default)]
    mode: OutputMode,

    receiver: EventReceiver,
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
            receiver,
        } = self;

        let message_writer = match mode {
            OutputMode::Text => Writer::Stdout,
            OutputMode::Json => Writer::Stderr,
        };
        let artifact_writer = Writer::Stdout;

        let render_handle = tokio::spawn(async move {
            render(receiver, verbosity, mode, message_writer, artifact_writer).await;
        });

        let result = command.await;

        render_handle.await.expect("render task panicked");

        result
    }
}

/// Consumes events from the receiver and renders them to the appropriate writer
///
/// Runs in a spawned Tokio task, concurrent with the command. The loop terminates when the
/// channel closes (all senders dropped), draining any buffered events before returning.
///
/// Verbosity filtering suppresses events below the configured threshold. Mode-based routing
/// directs messages and details to the message writer and artifacts to the artifact writer.
/// In JSON mode, artifacts are serialized via [`Serialize`] rather than [`Display`].
///
/// [`Display`]: std::fmt::Display
/// [`Serialize`]: serde::Serialize
async fn render(
    mut receiver: EventReceiver,
    verbosity: Verbosity,
    mode: OutputMode,
    message_writer: Writer,
    artifact_writer: Writer,
) {
    while let Some(event) = receiver.recv().await {
        match event {
            Event::Message(text) => match verbosity {
                Verbosity::Quiet => {}
                Verbosity::Default | Verbosity::Verbose => {
                    message_writer.write_line(&text);
                }
            },
            Event::Detail(text) => match verbosity {
                Verbosity::Quiet | Verbosity::Default => {}
                Verbosity::Verbose => {
                    message_writer.write_line(&text);
                }
            },
            Event::Artifact(artifact) => {
                let line = match mode {
                    OutputMode::Text => artifact.to_string(),
                    OutputMode::Json => serde_json::to_string(&artifact)
                        .expect("failed to serialize artifact to JSON"),
                };
                artifact_writer.write_line(&line);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::sync::{Arc, Mutex};

    use serde::Serialize;

    use super::super::writer::Writer;
    use super::*;
    use crate::event::{Event, event_channel};

    #[derive(Clone, Debug, Serialize)]
    struct TestArtifact {
        value: String,
    }

    impl std::fmt::Display for TestArtifact {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{}", self.value)
        }
    }

    struct TestBuffers {
        messages: Arc<Mutex<Vec<u8>>>,
        artifacts: Arc<Mutex<Vec<u8>>>,
    }

    impl TestBuffers {
        fn messages(&self) -> String {
            let guard = self.messages.lock().expect("should lock messages buffer");
            String::from_utf8(guard.clone()).expect("should be valid UTF-8")
        }

        fn artifacts(&self) -> String {
            let guard = self.artifacts.lock().expect("should lock artifacts buffer");
            String::from_utf8(guard.clone()).expect("should be valid UTF-8")
        }
    }

    fn test_writers() -> (Writer, Writer, TestBuffers) {
        let messages = Arc::new(Mutex::new(Vec::new()));
        let artifacts = Arc::new(Mutex::new(Vec::new()));
        let buffers = TestBuffers {
            messages: Arc::clone(&messages),
            artifacts: Arc::clone(&artifacts),
        };
        (Writer::Buffer(messages), Writer::Buffer(artifacts), buffers)
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
                    .send(crate::event::Event::Message("hello".to_string()))
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

    fn test_artifact() -> TestArtifact {
        TestArtifact {
            value: "hello".to_string(),
        }
    }

    #[tokio::test]
    async fn render_artifact_in_json_mode_serializes_as_json() {
        let (sender, receiver) = event_channel();
        let (message_writer, artifact_writer, buffers) = test_writers();

        sender
            .send(Event::Artifact(Box::new(test_artifact())))
            .await
            .expect("should send");
        drop(sender);
        render(
            receiver,
            Verbosity::Default,
            OutputMode::Json,
            message_writer,
            artifact_writer,
        )
        .await;

        assert_eq!(buffers.artifacts(), "{\"value\":\"hello\"}\n");
        assert_eq!(buffers.messages(), "");
    }

    #[tokio::test]
    async fn render_artifact_in_quiet_mode_writes() {
        let (sender, receiver) = event_channel();
        let (message_writer, artifact_writer, buffers) = test_writers();

        sender
            .send(Event::Artifact(Box::new(test_artifact())))
            .await
            .expect("should send");
        drop(sender);
        render(
            receiver,
            Verbosity::Quiet,
            OutputMode::Text,
            message_writer,
            artifact_writer,
        )
        .await;

        assert_eq!(buffers.artifacts(), "hello\n");
    }

    #[tokio::test]
    async fn render_artifact_in_text_mode_uses_display() {
        let (sender, receiver) = event_channel();
        let (message_writer, artifact_writer, buffers) = test_writers();

        sender
            .send(Event::Artifact(Box::new(test_artifact())))
            .await
            .expect("should send");
        drop(sender);
        render(
            receiver,
            Verbosity::Default,
            OutputMode::Text,
            message_writer,
            artifact_writer,
        )
        .await;

        assert_eq!(buffers.artifacts(), "hello\n");
        assert_eq!(buffers.messages(), "");
    }

    #[tokio::test]
    async fn render_detail_in_default_mode_is_noop() {
        let (sender, receiver) = event_channel();
        let (message_writer, artifact_writer, buffers) = test_writers();

        sender
            .send(Event::Detail("extra detail".to_string()))
            .await
            .expect("should send");
        drop(sender);
        render(
            receiver,
            Verbosity::Default,
            OutputMode::Text,
            message_writer,
            artifact_writer,
        )
        .await;

        assert_eq!(buffers.messages(), "");
    }

    #[tokio::test]
    async fn render_detail_in_quiet_mode_is_noop() {
        let (sender, receiver) = event_channel();
        let (message_writer, artifact_writer, buffers) = test_writers();

        sender
            .send(Event::Detail("extra detail".to_string()))
            .await
            .expect("should send");
        drop(sender);
        render(
            receiver,
            Verbosity::Quiet,
            OutputMode::Text,
            message_writer,
            artifact_writer,
        )
        .await;

        assert_eq!(buffers.messages(), "");
    }

    #[tokio::test]
    async fn render_detail_in_verbose_mode_writes_to_message_writer() {
        let (sender, receiver) = event_channel();
        let (message_writer, artifact_writer, buffers) = test_writers();

        sender
            .send(Event::Detail("extra detail".to_string()))
            .await
            .expect("should send");
        drop(sender);
        render(
            receiver,
            Verbosity::Verbose,
            OutputMode::Text,
            message_writer,
            artifact_writer,
        )
        .await;

        assert_eq!(buffers.messages(), "extra detail\n");
    }

    #[tokio::test]
    async fn render_drains_buffered_events_after_sender_dropped() {
        let (sender, receiver) = event_channel();
        let (message_writer, artifact_writer, buffers) = test_writers();

        sender
            .send(Event::Message("first".to_string()))
            .await
            .expect("should send");
        sender
            .send(Event::Message("second".to_string()))
            .await
            .expect("should send");
        drop(sender);
        render(
            receiver,
            Verbosity::Default,
            OutputMode::Text,
            message_writer,
            artifact_writer,
        )
        .await;

        assert_eq!(buffers.messages(), "first\nsecond\n");
    }

    #[tokio::test]
    async fn render_message_in_default_mode_writes_to_message_writer() {
        let (sender, receiver) = event_channel();
        let (message_writer, artifact_writer, buffers) = test_writers();

        sender
            .send(Event::Message("hello world".to_string()))
            .await
            .expect("should send");
        drop(sender);
        render(
            receiver,
            Verbosity::Default,
            OutputMode::Text,
            message_writer,
            artifact_writer,
        )
        .await;

        assert_eq!(buffers.messages(), "hello world\n");
    }

    #[tokio::test]
    async fn render_message_in_json_mode_writes_to_message_writer() {
        let (sender, receiver) = event_channel();
        let (message_writer, artifact_writer, buffers) = test_writers();

        sender
            .send(Event::Message("hello world".to_string()))
            .await
            .expect("should send");
        drop(sender);
        render(
            receiver,
            Verbosity::Default,
            OutputMode::Json,
            message_writer,
            artifact_writer,
        )
        .await;

        assert_eq!(buffers.messages(), "hello world\n");
        assert_eq!(buffers.artifacts(), "");
    }

    #[tokio::test]
    async fn render_message_in_quiet_mode_is_noop() {
        let (sender, receiver) = event_channel();
        let (message_writer, artifact_writer, buffers) = test_writers();

        sender
            .send(Event::Message("hello world".to_string()))
            .await
            .expect("should send");
        drop(sender);
        render(
            receiver,
            Verbosity::Quiet,
            OutputMode::Text,
            message_writer,
            artifact_writer,
        )
        .await;

        assert_eq!(buffers.messages(), "");
    }

    #[tokio::test]
    async fn render_message_in_verbose_mode_writes_to_message_writer() {
        let (sender, receiver) = event_channel();
        let (message_writer, artifact_writer, buffers) = test_writers();

        sender
            .send(Event::Message("hello world".to_string()))
            .await
            .expect("should send");
        drop(sender);
        render(
            receiver,
            Verbosity::Verbose,
            OutputMode::Text,
            message_writer,
            artifact_writer,
        )
        .await;

        assert_eq!(buffers.messages(), "hello world\n");
    }

    #[tokio::test]
    async fn render_preserves_event_order() {
        let (sender, receiver) = event_channel();
        let (message_writer, artifact_writer, buffers) = test_writers();

        sender
            .send(Event::Message("first".to_string()))
            .await
            .expect("should send");
        sender
            .send(Event::Detail("second".to_string()))
            .await
            .expect("should send");
        sender
            .send(Event::Artifact(Box::new(test_artifact())))
            .await
            .expect("should send");
        drop(sender);
        render(
            receiver,
            Verbosity::Verbose,
            OutputMode::Text,
            message_writer,
            artifact_writer,
        )
        .await;

        assert_eq!(buffers.messages(), "first\nsecond\n");
        assert_eq!(buffers.artifacts(), "hello\n");
    }

    #[tokio::test]
    async fn render_with_no_events_returns_immediately() {
        let (sender, receiver) = event_channel();
        let (message_writer, artifact_writer, buffers) = test_writers();

        drop(sender);
        render(
            receiver,
            Verbosity::Default,
            OutputMode::Text,
            message_writer,
            artifact_writer,
        )
        .await;

        assert_eq!(buffers.messages(), "");
        assert_eq!(buffers.artifacts(), "");
    }
}
