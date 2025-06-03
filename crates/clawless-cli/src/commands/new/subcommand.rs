use clap::{ArgMatches, Args, Command, FromArgMatches};
use getset::Getters;

struct SubcommandRegistration {
    name: &'static str,
    init: fn() -> Command,
    func: fn(ArgMatches) -> std::pin::Pin<Box<dyn std::future::Future<Output = ()>>>,
}
inventory::collect!(SubcommandRegistration);

#[derive(Debug, Args, Getters)]
pub struct SubcommandArgs {
    #[arg(short, long)]
    #[getset(get = "pub")]
    name: String,
}

pub async fn run(args: SubcommandArgs) {
    println!("Running a subcommand: {}", args.name());
}

pub fn init() -> Command {
    let mut command = SubcommandArgs::augment_args(Command::new("subcommand"));

    for subcommand in inventory::iter::<SubcommandRegistration> {
        command = command.subcommand((subcommand.init)());
    }

    command
}

pub async fn exec_run(args: ArgMatches) {
    for subcommand in inventory::iter::<SubcommandRegistration> {
        if args.subcommand_name() == Some(subcommand.name) {
            return (subcommand.func)(args.clone()).await;
        }
    }

    let args = SubcommandArgs::from_arg_matches(&args).unwrap();
    run(args).await;
}

inventory::submit!(super::SubcommandRegistration {
    name: "subcommand",
    init,
    func: |args| Box::pin(exec_run(args)),
});
