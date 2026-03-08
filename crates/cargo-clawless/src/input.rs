//! Input types and parsers for the CLI
//!
//! This module contains types used for parsing and validating command-line
//! input, such as command names with support for nested hierarchies.

pub use self::command_name::*;

mod command_name;
