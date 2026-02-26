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

// TODO: Remove when the presenter is actually used
#![allow(dead_code)]

use std::future::Future;
use std::pin::Pin;

use async_trait::async_trait;
use bon::Builder;

use super::Presenter;
use crate::error::CommandResult;
use crate::event::EventReceiver;
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
        command.await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::event::event_channel;

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
        let (_sender, receiver) = event_channel();
        let presenter = TerminalPresenter::builder().receiver(receiver).build();

        let error = presenter
            .present(Box::pin(async { Err(anyhow::anyhow!("command failed")) }))
            .await
            .expect_err("should fail");

        assert_eq!(error.to_string(), "command failed");
    }

    #[tokio::test]
    async fn present_with_ok_returns_ok() {
        let (_sender, receiver) = event_channel();
        let presenter = TerminalPresenter::builder().receiver(receiver).build();

        presenter
            .present(Box::pin(async { Ok(()) }))
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
}
