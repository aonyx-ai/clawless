use clap::{command, Parser};
use getset::Getters;

use crate::commands::Commands;

#[derive(Debug, Parser, Getters)]
#[command(version)]
pub struct App {
    #[command(subcommand)]
    #[getset(get = "pub")]
    pub command: Commands,
}
