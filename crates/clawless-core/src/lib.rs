#![cfg_attr(not(doctest),doc = include_str!("../README.md"))]

pub mod cancellation;
pub mod context;
pub mod event;
pub mod exit;
pub mod output;
pub mod process;
pub mod prompt;

#[doc(hidden)]
pub mod signal;

/// A prelude module to easily import Clawless's core types and traits
///
/// This module re-exports the most commonly used items from the clawless-core crate. Importing
/// everything from this module gives users the types and traits to define and run commands,
/// without importing each item individually.
pub mod prelude {
    pub use super::cancellation::Cancellation;
    pub use super::context::{Context, ContextError, CurrentWorkingDirectory, Interactivity};
    pub use super::exit::Exit;
    pub use super::process::{Execution, Invocation, Process};
    pub use super::prompt::{Confirmation, Prompt};
}
