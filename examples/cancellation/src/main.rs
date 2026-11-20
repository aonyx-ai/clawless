//! Cooperative cancellation example
//!
//! This example shows how a command reads the cancellation token in [`Context`]. The command
//! then stops in an orderly way after the process receives SIGINT or SIGTERM.
//!
//! [`Context`]: clawless::prelude::Context

#![allow(
    unreachable_pub,
    reason = "nothing in a binary crate is reachable from outside it, so the lint would demand `pub(crate)` on every item, and it earns its keep in the library crates, where the public API is a real boundary"
)]

mod commands;

clawless::main!();
