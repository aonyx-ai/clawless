#![cfg_attr(not(doctest),doc = include_str!("../README.md"))]
#![warn(missing_docs)]

/// A prelude module to easily import Clawless essentials
///
/// This module re-exports the most commonly used items from the Clawless crate. By importing
/// everything from this module, users can conveniently access the necessary types and traits to
/// define and run commands without needing to import each item individually.
pub mod prelude {
    pub use clap;
    pub use clap::{Args, FromArgMatches};
    pub use clawless_core::prelude::*;
    pub use clawless_derive::{artifact, command, commands, detail, main, message};
    pub use serde::Serialize;

    pub use super::context::*;
    pub use super::error::{CommandResult, Error, ErrorContext};
    pub use super::output::*;
}

pub use clawless_core::cancellation;
pub use clawless_core::event;
pub use clawless_derive::{artifact, command, commands, detail, main, message};
pub use error::{CommandResult, Error, ErrorContext};

pub mod context;
mod error;
pub mod macros;
pub mod output;
pub mod presenter;

// Re-export the clap crate for use with the `clawless-derive` crate
#[doc(hidden)]
pub use clap;
// Signal-to-cancellation adapter used by the `main!()` macro expansion
#[doc(hidden)]
pub use clawless_core::signal;
// Re-export the inventory crate for use with the `clawless-derive` crate
#[doc(hidden)]
pub use inventory;
// Re-export the tokio crate to run commands in an async runtime
#[doc(hidden)]
pub use tokio;
