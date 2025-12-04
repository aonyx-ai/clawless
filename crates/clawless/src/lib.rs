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
    pub use clawless_derive::{command, commands, main};

    pub use super::context::*;
    pub use super::error::{CommandResult, Error, ErrorContext};
}

pub use clawless_derive::{command, commands, main};
pub use error::{CommandResult, Error, ErrorContext};

pub mod context;
mod error;

// Re-export the clap crate for use with the `clawless-derive` crate
#[doc(hidden)]
pub use clap;
// Re-export the inventory crate for use with the `clawless-derive` crate
#[doc(hidden)]
pub use inventory;
// Re-export the tokio crate to run commands in an async runtime
#[doc(hidden)]
pub use tokio;
