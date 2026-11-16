//! Output, verbosity, and output mode example
//!
//! This example shows the three output kinds of Clawless: messages, details, and artifacts.
//! It also shows how the `--quiet`, `--verbose`, and `--json` flags change what a command
//! emits.

#![allow(
    unreachable_pub,
    reason = "nothing in a binary crate is reachable from outside it, so the lint would demand `pub(crate)` on every item, and it earns its keep in the library crates, where the public API is a real boundary"
)]

mod commands;

clawless::main!();
