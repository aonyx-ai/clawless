use clap::{command, Parser, Subcommand};
use getset::Getters;

#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Parser, Getters)]
#[command(version)]
pub struct App {
    #[command(subcommand)]
    #[getset(get = "pub")]
    pub command: Commands,
}

#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Subcommand)]
pub enum Commands {
    /// Create a new CLI with Clawless
    New { name: String },
}
