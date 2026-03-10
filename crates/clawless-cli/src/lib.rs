//! CLI presentation layer for the Clawless framework
//!
//! This crate provides the CLI-specific types and traits for building command-line applications
//! with Clawless. It includes the command execution context, output configuration, and the
//! presenter abstraction for rendering command output to the terminal.
//!
//! Most users should depend on the [`clawless`] facade crate instead of using this crate directly.
//! The facade re-exports everything from this crate alongside [`clawless-core`] and
//! [`clawless-derive`], providing a single dependency for CLI applications.
//!
//! [`clawless`]: https://docs.rs/clawless
//! [`clawless-core`]: https://docs.rs/clawless-core
//! [`clawless-derive`]: https://docs.rs/clawless-derive
#![warn(missing_docs)]

pub mod error;
pub mod macros;
pub mod output;
pub mod presenter;
pub mod runner;
