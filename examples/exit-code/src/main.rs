//! Exit code example
//!
//! This example shows how a command ends with an exit code and a text that it chooses. A wrapper
//! such as a script or a CI job reads the exit code to decide what to do next.

#![allow(
    unreachable_pub,
    reason = "nothing in a binary crate is reachable from outside it, so the lint would demand `pub(crate)` on every item, and it earns its keep in the library crates, where the public API is a real boundary"
)]

mod commands;

clawless::main!();
