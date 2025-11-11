#![cfg_attr(not(doctest),doc = include_str!("../README.md"))]
#![warn(missing_docs)]

use proc_macro::TokenStream;
use quote::quote;
use syn::{ItemFn, parse_macro_input};

use crate::command::CommandGenerator;
use crate::inventory::InventoryGenerator;

mod command;
mod inventory;

/// Initialize and run a Clawless application
///
/// This macro generates the entry point for a Clawless application and a `main` function to run it.
/// Commands can be implemented in submodules and will be automatically registered as subcommands of
/// the CLI.
///
/// # Example
///
/// ```rust,ignore
/// mod greet;
///
/// clawless::main!();
/// ```
#[proc_macro]
pub fn main(_input: TokenStream) -> TokenStream {
    let output = quote! {
        #[clawless::command(noop = true, root = true)]
        async fn clawless() -> clawless::CommandResult {
            Ok(())
        }

        fn main() {
            let rt = clawless::tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async {
                let app = clawless_init();
                clawless_exec(app.get_matches()).await
            })
            .unwrap();
        }
    };
    output.into()
}

/// Add a command to a Clawless application
///
/// This macro attribute can be used to register a function as a (sub)command in
/// a Clawless application. The name of the function will be used as the name of
/// the command, and it will be automatically registered as a subcommand under
/// its parent module.
///
/// Command functions can optionally accept a single argument, which must be a
/// `clap::Args` struct with arguments that will be passed to the command. If no
/// argument is provided, the command takes no parameters.
///
/// # Example
///
/// ```rust,ignore
/// use clap::Args;
/// use clawless::{command, CommandResult};
///
/// #[derive(Debug, Args)]
/// pub struct CommandArgs {
///     #[arg(short, long)]
///     name: String,
/// }
///
/// #[command]
/// pub async fn command(args: CommandArgs) -> CommandResult {
///     println!("Running a command: {}", args.name);
///     Ok(())
/// }
/// ```
#[proc_macro_attribute]
pub fn command(attrs: TokenStream, input: TokenStream) -> TokenStream {
    let input_function = parse_macro_input!(input as ItemFn);

    let command_generator = CommandGenerator::new(attrs.into(), input_function.clone());
    let inventory_generator = InventoryGenerator::new(&command_generator);

    let inventory_struct_for_subcommands = inventory_generator.inventory();
    let submit_command_to_inventory = inventory_generator.submit_command();

    let initialization_function_for_command = command_generator.initialization_function();
    let wrapper_function_for_command = command_generator.wrapper_function();

    let output = quote! {
        #inventory_struct_for_subcommands

        #input_function

        #initialization_function_for_command

        #wrapper_function_for_command

        #submit_command_to_inventory
    };

    output.into()
}
