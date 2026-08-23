//! Cooperative cancellation example
//!
//! This example shows how a command reads the cancellation token in [`Context`]. The command
//! then stops in an orderly way after the process receives SIGINT or SIGTERM.
//!
//! [`Context`]: clawless::prelude::Context

// This crate compiles to a binary, so nothing in it is reachable from outside the crate
// and `unreachable_pub` would demand `pub(crate)` on every item. The lint earns its keep
// in the library crates, where the public API is a real boundary.
#![allow(unreachable_pub)]

/// The commands of this example
mod commands;

clawless::main!();
