//! Minimal Clawless application
//!
//! This example is the reference for the framework. One `greet` command shows the command
//! definition, the argument parsing, and the output conventions of a new project.

#![allow(
    unreachable_pub,
    reason = "nothing in a binary crate is reachable from outside it, so the lint would demand `pub(crate)` on every item, and it earns its keep in the library crates, where the public API is a real boundary"
)]

mod commands;

clawless::main!();
