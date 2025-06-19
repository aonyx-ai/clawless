#![cfg_attr(not(doctest),doc = include_str!("../README.md"))]
#![warn(missing_docs)]

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, ItemFn};

use crate::app::AppGenerator;
use crate::command::CommandGenerator;
use crate::inventory::InventoryGenerator;

mod app;
mod command;
mod inventory;

/// Generate a Clawless application
///
/// This macro generates an empty Clawless application that can be extended with
/// subcommands. It is supposed to be called in a module called `commands`,
/// with commands defined in submodules.
///
/// # Example
///
/// ```rust,ignore
/// mod commands {
///     clawless::app!();
/// }
/// ```
#[proc_macro]
pub fn app(_input: TokenStream) -> TokenStream {
    let app_generator = AppGenerator::new();
    app_generator.app_function().into()
}

/// Add a command to a Clawless application
///
/// This macro attribute can be used to register a function as a (sub)command in
/// Clawless application. The name of the function will be used as the name of
/// the command, and it will be automatically registered as a subcommand under
/// its parent module.
///
/// Command functions expect a single argument, which is a `clap::Args` struct
/// with arguments that will be passed to the command.
///
/// # Example
///
/// ```rust,ignore
/// use clap::Args;
/// use clawless::command;
///
/// #[derive(Debug, Args)]
/// pub struct CommandArgs {
///     #[arg(short, long)]
///     name: String,
/// }
///
/// #[command]
/// pub async fn command(args: CommandArgs) {
///     println!("Running a command: {}", args.name);
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
