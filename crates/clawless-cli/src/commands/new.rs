use clap::Args;
use getset::Getters;

#[derive(Debug, Args, Getters)]
pub struct NewArgs {
    #[arg(index = 1)]
    #[getset(get = "pub")]
    name: String,
}

pub async fn run(args: &NewArgs) {
    println!("Creating new CLI with name: {}", args.name());
}
