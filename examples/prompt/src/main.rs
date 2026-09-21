//! Prompt example
//!
//! This example shows how a command asks its user a question. Run it in a terminal to answer the
//! questions. Run it with the input from a file or a pipe to see the behavior without a user.

// This crate compiles to a binary, so nothing in it is reachable from outside the crate
// and `unreachable_pub` would demand `pub(crate)` on every item. The lint earns its keep
// in the library crates, where the public API is a real boundary.
#![allow(unreachable_pub)]

/// The commands of this example
mod commands;

clawless::main!();
