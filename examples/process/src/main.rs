//! External program example
//!
//! This example shows how a command runs another program, reports its output while it runs,
//! and reads what the program produced after it ended. Pass `--verbose` to see the output of
//! the program itself, which Clawless streams through the event system as it arrives.

#![allow(
    unreachable_pub,
    reason = "nothing in a binary crate is reachable from outside it, so the lint would demand `pub(crate)` on every item, and it earns its keep in the library crates, where the public API is a real boundary"
)]

mod commands;

clawless::main!();
