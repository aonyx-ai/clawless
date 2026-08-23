//! Output, verbosity, and output mode example
//!
//! This example shows the three output kinds of Clawless: messages, details, and artifacts.
//! It also shows how the `--quiet`, `--verbose`, and `--json` flags change what a command
//! emits.

// This crate compiles to a binary, so nothing in it is reachable from outside the crate
// and `unreachable_pub` would demand `pub(crate)` on every item. The lint earns its keep
// in the library crates, where the public API is a real boundary.
#![allow(unreachable_pub)]

/// The commands of this example
mod commands;

clawless::main!();
