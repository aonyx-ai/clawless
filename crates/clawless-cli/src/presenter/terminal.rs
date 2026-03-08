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
use clawless_core::event::{Event, EventReceiver};

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
    #[builder(default)]
    verbosity: Verbosity,

    #[builder(default)]
    mode: OutputMode,

    receiver: EventReceiver,
}

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

        command_handle.await.expect("command task panicked")
    }
}

#[cfg(test)]
mod tests {
    use clawless_core::event::event_channel;

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
