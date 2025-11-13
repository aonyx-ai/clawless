use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::Ident;

use crate::command::CommandGenerator;

const INVENTORY_NAME: &str = "ClawlessSubcommands";

pub struct InventoryGenerator<'a> {
    command_generator: &'a CommandGenerator,
}

impl<'a> InventoryGenerator<'a> {
    pub fn new(command_generator: &'a CommandGenerator) -> Self {
        Self { command_generator }
    }

    pub fn inventory(&self) -> TokenStream {
        let inventory_name = inventory_name();

        quote! {
            struct #inventory_name {
                name: &'static str,
                init: fn() -> clawless::clap::Command,
                func: fn(clawless::clap::ArgMatches) -> std::pin::Pin<Box<dyn std::future::Future<Output = clawless::CommandResult>>>,
            }
            clawless::inventory::collect!(#inventory_name);
        }
    }

    pub fn submit_command(&self) -> TokenStream {
        if self.command_generator.is_root() {
            return quote! {};
        }

        let inventory_name = inventory_name();
        let command = self.command_generator.ident().to_string();
        let init_fn_name = self.command_generator.initialization_function_name();
        let wrapper_fn_name = self.command_generator.wrapper_function_name();

        quote! {
            clawless::inventory::submit!(super::#inventory_name {
                name: #command,
                init: #init_fn_name,
                func: |args| Box::pin(#wrapper_fn_name(args)),
            });
        }
    }
}

pub fn inventory_name() -> Ident {
    format_ident!("{}", INVENTORY_NAME)
}
