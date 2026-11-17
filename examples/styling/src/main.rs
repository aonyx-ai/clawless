//! Styled output example
//!
//! This example shows values with a styled text form and a plain one. The command offers both
//! forms, and the presenter writes the styled form to a terminal and the plain form to a file or
//! a pipe. `NO_COLOR`, `CLICOLOR_FORCE`, `CLICOLOR`, and `TERM` override that choice.

// This crate compiles to a binary, so nothing in it is reachable from outside the crate
// and `unreachable_pub` would demand `pub(crate)` on every item. The lint earns its keep
// in the library crates, where the public API is a real boundary.
#![allow(unreachable_pub)]

/// The commands of this example
mod commands;

clawless::main!();
