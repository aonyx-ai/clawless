use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::Ident;

use crate::generator::Generator;

const INVENTORY_NAME: &str = "ClawlessSubcommands";

pub struct InventoryGenerator<'a> {
    generator: &'a dyn Generator,
}

impl<'a> InventoryGenerator<'a> {
    pub fn new(generator: &'a dyn Generator) -> Self {
        Self { generator }
    }

    // r[impl dispatch.resolve.uniform]
    pub fn inventory(&self) -> TokenStream {
        let inventory_name = inventory_name();

        quote! {
            struct #inventory_name {
                name: &'static str,
                init: fn() -> clawless::clap::Command,
                resolve: fn(clawless::clap::ArgMatches) -> clawless::resolved_leaf::ResolvedLeaf,
            }
            clawless::inventory::collect!(#inventory_name);
        }
    }

    pub fn submit(&self) -> TokenStream {
        if self.generator.is_root() {
            return quote! {};
        }

        let inventory_name = inventory_name();
        let name = self.generator.ident().to_string();
        let init_fn_name = self.generator.initialization_function_name();
        let resolve_fn_name = self.generator.resolve_function_name();

        quote! {
            clawless::inventory::submit!(super::#inventory_name {
                name: #name,
                init: #init_fn_name,
                resolve: #resolve_fn_name,
            });
        }
    }
}

pub fn inventory_name() -> Ident {
    format_ident!("{}", INVENTORY_NAME)
}
