use proc_macro::TokenStream;

use crate::command::command_impl;

mod command;

const INVENTORY_NAME: &str = "SubcommandRegistration";

#[proc_macro_attribute]
pub fn command(attrs: TokenStream, input: TokenStream) -> TokenStream {
    command_impl(attrs, input)
}
