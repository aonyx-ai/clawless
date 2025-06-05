use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, ItemFn};

use crate::app::AppGenerator;
use crate::command::CommandGenerator;
use crate::inventory::InventoryGenerator;

mod app;
mod command;
mod inventory;

#[proc_macro]
pub fn app(_input: TokenStream) -> TokenStream {
    let app_generator = AppGenerator::new();
    app_generator.app_function().into()
}

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
