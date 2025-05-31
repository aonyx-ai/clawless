use clap::{Parser, Subcommand};
use getset::Getters;

#[derive(Debug, Parser, Getters)]
#[command(version)]
pub struct App<Commands>
where
    Commands: Subcommand,
{
    #[command(subcommand)]
    #[getset(get = "pub")]
    pub command: Commands,
}

/// Run an async function in the Clawless runtime
pub fn run_async<F>(future: F)
where
    F: std::future::Future<Output = ()>,
{
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(future);
}

#[macro_export]
macro_rules! clawless {
    // Accept a single argument, which is the enum of commands
    ($commands:ident) => {
        fn main() {
            use clap::Parser;
            let app = $crate::App::<$commands>::parse();

            $crate::run_async(async {
                match app.command {
                    $commands::New(args) => {
                        commands::new::run(&args).await;
                    }
                }
            });
        }
    };
}

#[macro_export]
macro_rules! commands {
    () => {
        include!(concat!(env!("OUT_DIR"), "/commands.rs"));
    };
}
