#![cfg_attr(not(doctest),doc = include_str!("../README.md"))]
#![warn(missing_docs)]

pub use clawless_derive::{command, main};

pub use self::error::{Context, Error, Result};

mod error;

// Re-export the inventory crate for use with the `clawless-derive` crate
#[doc(hidden)]
pub use inventory;

// Re-export the tokio crate to run commands in an async runtime
#[doc(hidden)]
pub use tokio;
