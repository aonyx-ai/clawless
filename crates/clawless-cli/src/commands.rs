//! Command implementations for the Clawless CLI
//!
//! This module contains all the commands that the Clawless CLI provides:
//! - `new` - Create a new Clawless project
//! - `generate` - Generate code scaffolding (subcommands for different generators)

mod generate;
mod new;

clawless::commands!();
