//! External program example
//!
//! This example shows how a command runs another program, reports its output while it runs,
//! and reads what the program produced after it ended. Pass `--verbose` to see the output of
//! the program itself, which Clawless streams through the event system as it arrives.

// This crate compiles to a binary, so nothing in it is reachable from outside the crate
// and `unreachable_pub` would demand `pub(crate)` on every item. The lint earns its keep
// in the library crates, where the public API is a real boundary.
#![allow(unreachable_pub)]

/// The commands of this example
mod commands;

clawless::main!();
