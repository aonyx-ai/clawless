//! Prompt example
//!
//! This example shows how a command asks its user a question. Run it in a terminal to answer the
//! questions. Run it with the input from a file or a pipe to see the behavior without a user.

#![allow(
    unreachable_pub,
    reason = "nothing in a binary crate is reachable from outside it, so the lint would demand `pub(crate)` on every item, and it earns its keep in the library crates, where the public API is a real boundary"
)]

mod commands;

clawless::main!();
