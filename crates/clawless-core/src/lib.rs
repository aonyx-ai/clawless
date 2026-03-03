#![cfg_attr(not(doctest),doc = include_str!("../README.md"))]
#![warn(missing_docs)]

pub mod cancellation;

// Signal-to-cancellation adapter used by the `main!()` macro expansion
#[doc(hidden)]
pub mod signal;

/// A prelude module to easily import Clawless's core types and traits
///
/// This module re-exports the most commonly used items from the clawless-core crate. By importing
/// everything from this module, users can conveniently access the necessary types and traits to
/// define and run commands without needing to import each item individually.
pub mod prelude {
    pub use super::cancellation::Cancellation;
}
