//! Exit code example
//!
//! This example shows how a command ends with an exit code and a text that it chooses. A wrapper
//! such as a script or a CI job reads the exit code to decide what to do next.

// This crate compiles to a binary, so nothing in it is reachable from outside the crate
// and `unreachable_pub` would demand `pub(crate)` on every item. The lint earns its keep
// in the library crates, where the public API is a real boundary.
#![allow(unreachable_pub)]

/// The commands of this example
mod commands;

clawless::main!();
