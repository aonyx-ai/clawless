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
    pub use clawless_cli::error::{CommandResult, Error, ErrorContext};
    pub use clawless_core::context::*;
    pub use clawless_core::output::Output;
    pub use clawless_core::prelude::*;
    pub use clawless_derive::{artifact, command, commands, detail, main, message};
    pub use serde::Serialize;

    pub use super::output::{OutputMode, Verbosity};
}

pub use clawless_cli::error::{CommandResult, Error, ErrorContext};
pub use clawless_cli::macros;
pub use clawless_cli::presenter;
pub use clawless_cli::runner;
pub use clawless_core::cancellation;
pub use clawless_core::context;
pub use clawless_core::event;
pub use clawless_derive::{artifact, command, commands, detail, main, message};

/// CLI flag configuration and output types
///
/// This module re-exports output types from both [`clawless_cli`] and [`clawless_core`], providing
/// a unified `clawless::output` module that contains everything needed for output configuration.
pub mod output {
    pub use clawless_cli::output::{OutputFlags, OutputMode, Verbosity};
    pub use clawless_core::output::Output;
}

// Re-export the clap crate for use with the `clawless-derive` crate
#[doc(hidden)]
pub use clap;
// Re-export the inventory crate for use with the `clawless-derive` crate
#[doc(hidden)]
pub use inventory;
