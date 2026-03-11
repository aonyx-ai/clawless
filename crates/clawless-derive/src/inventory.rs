use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::Ident;

use crate::generator::Generator;

const INVENTORY_NAME: &str = "ClawlessSubcommands";

/// Generates the `inventory` crate glue that connects subcommands to their parents at compile time
///
/// Each `#[command]` or `#[application]` expansion emits two pieces of inventory code: a
/// `ClawlessSubcommands` struct definition (via `inventory`) and a `submit!` call that registers
/// the leaf's `_init` and `_resolve` functions with the parent module's collector. This generator
/// is generic over `Generator` so it works identically for both commands and applications.
pub struct InventoryGenerator<'a> {
    generator: &'a dyn Generator,
}

impl<'a> InventoryGenerator<'a> {
    /// Wraps a `Generator` to produce inventory code from its identity and function names
    pub fn new(generator: &'a dyn Generator) -> Self {
        Self { generator }
    }

    // r[impl dispatch.resolve.uniform]
    /// Generates the `ClawlessSubcommands` struct that the `inventory` crate collects at link time
    ///
    /// Every module that uses `#[command]` or `#[application]` gets its own copy of this struct.
    /// The struct carries three function pointers — `name`, `init`, and `resolve` — forming the
    /// uniform interface that the `_init` and `_resolve` functions iterate over to build the clap
    /// tree and walk subcommands.
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

    /// Generates the `inventory::submit!` call that registers this leaf with its parent module
    ///
    /// The submission references `super::ClawlessSubcommands`, connecting this leaf to its parent's
    /// inventory collector. Root commands skip submission because they are the entry point, not a
    /// subcommand of anything.
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

/// Returns the `ClawlessSubcommands` identifier used for all inventory struct definitions
///
/// Centralized here so that `InventoryGenerator` and the `Generator` trait's default methods
/// reference the same name.
pub fn inventory_name() -> Ident {
    format_ident!("{}", INVENTORY_NAME)
}
