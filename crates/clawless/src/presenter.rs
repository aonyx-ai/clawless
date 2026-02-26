//! Output port for rendering command output
//!
//! This module defines the [`Presenter`] trait, the output port in the hexagonal architecture. A
//! `Presenter` wraps command execution and controls the output lifecycle. The framework calls
//! [`present`] with the command future, and the presenter runs the command, optionally consuming
//! events from its event channel, and returns the command's result.
//!
//! Presenter adapters implement this trait to provide different rendering strategies. A stateless
//! adapter renders each event as it arrives; a stateful adapter queries a surface on each render
//! frame. The command's code is identical regardless of which adapter is in use.
//!
//! [`present`]: Presenter::present

use std::future::Future;
use std::pin::Pin;

use async_trait::async_trait;

pub use self::terminal::TerminalPresenter;
use crate::error::CommandResult;

mod terminal;
mod writer;

/// Output port for rendering command output
///
/// A `Presenter` wraps command execution and controls the output lifecycle. The framework
/// constructs a presenter with its dependencies (such as an [`EventReceiver`] and rendering
/// configuration), then calls [`present`] with the command future. The presenter runs the
/// command, optionally consuming events from its receiver, and returns the command's result.
///
/// `present` takes `self` by value because presentation is a one-shot operation. Each presenter
/// is constructed once, used once, and consumed. This encodes the "call once" invariant in the
/// type system and avoids interior mutability for resources like [`EventReceiver`] that require
/// exclusive access for reading.
///
/// The trait does not require [`Send`] or [`Sync`] because the presenter lives on the main task
/// and is never shared across threads.
///
/// # Examples
///
/// ```rust,ignore
/// use clawless::presenter::{Presenter, TerminalPresenter};
///
/// let presenter = TerminalPresenter::builder().receiver(receiver).build();
/// presenter.present(Box::pin(command_future)).await?;
/// ```
///
/// [`EventReceiver`]: crate::event::EventReceiver
/// [`present`]: Presenter::present
#[async_trait(?Send)]
pub trait Presenter {
    /// Presents the output of a command
    ///
    /// Runs the given command future to completion and returns its result. Implementations may
    /// consume events from their [`EventReceiver`] concurrently with command execution to render
    /// output in real time.
    ///
    /// The command future is boxed and pinned because the concrete future type varies by call
    /// site. The future is [`Send`] because commands execute on Tokio's multi-threaded runtime.
    ///
    /// # Errors
    ///
    /// Returns the command's error if the command fails. Implementations propagate the
    /// command's [`CommandResult`] without modification.
    ///
    /// [`EventReceiver`]: crate::event::EventReceiver
    async fn present(
        self,
        command: Pin<Box<dyn Future<Output = CommandResult> + Send>>,
    ) -> CommandResult;
}
