//! Minimal Clawless application
//!
//! This example is the reference for the framework. One `greet` command shows the command
//! definition, the argument parsing, and the output conventions of a new project.

// This crate compiles to a binary, so nothing in it is reachable from outside the crate
// and `unreachable_pub` would demand `pub(crate)` on every item. The lint earns its keep
// in the library crates, where the public API is a real boundary.
#![allow(unreachable_pub)]

/// The commands of this example
mod commands;

clawless::main!();
