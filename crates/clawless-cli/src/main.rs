use clap::Parser;

use crate::app::App;
use crate::commands::Commands;

mod app;
mod commands;

#[tokio::main]
async fn main() {
    let app = App::parse();

    match app.command {
        Commands::New(args) => {
            crate::commands::new::run(&args).await;
        }
    }
}
