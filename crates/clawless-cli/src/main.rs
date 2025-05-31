use clap::Parser;

use crate::app::{App, Commands};

mod app;

#[tokio::main]
async fn main() {
    let app = App::parse();

    match app.command {
        Commands::New { name } => {
            println!("Creating new CLI with name: {}", name);
        }
    }
}
