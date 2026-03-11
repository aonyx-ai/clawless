pub mod application;
pub mod command;

pub use application::ApplicationGenerator;
pub use command::CommandGenerator;
use darling::FromMeta;
use darling::ast::NestedMeta;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{Error, Expr, Ident, ItemFn, Lit, Meta, Result, Type};

use crate::inventory::inventory_name;

/// Shared behavior for command and application code generators
///
/// Both `CommandGenerator` and `ApplicationGenerator` produce the same structural output: an `_init`
/// function (clap command tree), a `_resolve` function (subcommand dispatch), and an inventory
/// submission. The generators differ only in how they validate function parameters and which
/// `ResolvedLeaf` variant they produce.
///
/// Required methods supply the generator-specific parts. Default methods wire them together into the
/// common code generation patterns.
pub trait Generator {
    /// Returns the function identifier (e.g., `greet`, `dashboard`)
    fn ident(&self) -> Ident;

    /// Returns the parsed macro attributes
    fn attrs(&self) -> &Attributes;

    /// Returns the original input function
    fn input(&self) -> &ItemFn;

    /// Extracts the arguments type from the validated function signature
    fn args_type(&self) -> Box<Type>;

    /// Generates the leaf-specific body of the resolve function
    ///
    /// `CommandGenerator` returns `ResolvedLeaf::Command`, `ApplicationGenerator` returns
    /// `ResolvedLeaf::Application`.
    fn resolve_function_body(&self) -> TokenStream;

    /// Returns whether this is the root command
    fn is_root(&self) -> bool {
        self.attrs().root()
    }

    /// Returns the name of the generated `_init` function
    fn initialization_function_name(&self) -> Ident {
        format_ident!("{}_init", self.ident())
    }

    /// Returns the name of the generated `_resolve` function
    fn resolve_function_name(&self) -> Ident {
        format_ident!("{}_resolve", self.ident())
    }

    /// Generates the clap `Command` constructor expression
    fn command_new(&self) -> TokenStream {
        build_command(
            &self.ident(),
            &self.args_type(),
            extract_function_documentation(self.input()).as_ref(),
            self.attrs(),
        )
    }

    /// Generates the `_init` function that builds the clap command tree
    fn initialization_function(&self) -> TokenStream {
        let function_name = self.initialization_function_name();
        let command_new = self.command_new();
        let inventory_name = inventory_name();

        quote! {
            pub fn #function_name() -> clawless::clap::Command {
                let mut command = #command_new;

                for subcommand in clawless::inventory::iter::<#inventory_name> {
                    command = command.subcommand((subcommand.init)());
                }

                command
            }
        }
    }

    // r[impl dispatch.resolve.sync]
    /// Generates the `_resolve` function that walks subcommands and returns a `ResolvedLeaf`
    fn resolve_function(&self) -> TokenStream {
        let resolve_function_name = self.resolve_function_name();
        let resolve_function_body = self.resolve_function_body();
        let inventory_name = inventory_name();

        // r[impl dispatch.resolve.delegate]
        quote! {
            pub fn #resolve_function_name(matches: clawless::clap::ArgMatches) -> clawless::resolved_leaf::ResolvedLeaf {
                for subcommand in clawless::inventory::iter::<#inventory_name> {
                    if let Some(sub_matches) = matches.subcommand_matches(subcommand.name) {
                        return (subcommand.resolve)(sub_matches.clone());
                    }
                }

                #resolve_function_body
            }
        }
    }
}

#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Default, FromMeta)]
pub struct Attributes {
    #[darling(default)]
    require_subcommand: bool,
    #[darling(default)]
    root: bool,
    #[darling(default, multiple)]
    alias: Vec<String>,
}

impl Attributes {
    pub fn require_subcommand(&self) -> bool {
        self.require_subcommand
    }

    pub fn root(&self) -> bool {
        self.root
    }

    pub fn alias(&self) -> &[String] {
        &self.alias
    }
}

#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub struct Documentation {
    short: String,
    long: String,
}

impl Documentation {
    pub fn short(&self) -> &str {
        &self.short
    }

    pub fn long(&self) -> &str {
        &self.long
    }
}

pub fn parse_attributes(attrs: TokenStream, macro_name: &str) -> Result<Attributes> {
    let argument_list = NestedMeta::parse_meta_list(attrs.clone()).map_err(|e| {
        Error::new_spanned(
            attrs.clone(),
            format!(
                "invalid attribute syntax: {e}\n\n\
                 = help: use one of the supported attributes\n\n    \
                 #[{macro_name}]\n    \
                 #[{macro_name}(alias = \"g\")]\n    \
                 #[{macro_name}(require_subcommand)]\n    \
                 #[{macro_name}(alias = \"g\", require_subcommand)]"
            ),
        )
    })?;

    Attributes::from_list(&argument_list).map_err(|e| {
        Error::new_spanned(
            attrs,
            format!(
                "{e}\n\n\
                 = help: supported attributes are `alias` and `require_subcommand`\n\n    \
                 #[{macro_name}(alias = \"g\", require_subcommand)]"
            ),
        )
    })
}

pub fn extract_function_documentation(input_fn: &ItemFn) -> Option<Documentation> {
    let mut docs = Vec::new();

    for attr in input_fn.attrs.iter() {
        if let Meta::NameValue(meta) = &attr.meta {
            if !attr.meta.path().is_ident("doc") {
                continue;
            }

            if let Expr::Lit(expr) = &meta.value
                && let Lit::Str(lit) = &expr.lit
            {
                docs.push(lit.value().trim().to_string());
            }
        }
    }

    if docs.is_empty() {
        None
    } else {
        Some(Documentation {
            short: docs[0].clone(),
            long: docs.join("\n"),
        })
    }
}

fn build_command(
    ident: &Ident,
    args_type: &Type,
    docs: Option<&Documentation>,
    attrs: &Attributes,
) -> TokenStream {
    let command_name = ident.to_string();

    let mut command = quote! {
        #args_type::augment_args(clawless::clap::Command::new(#command_name))
    };

    if attrs.root() {
        command = quote! {
            #command.about(clawless::clap::crate_description!())
        };
    } else if let Some(docs) = docs {
        let short = docs.short();
        let long = docs.long();

        command = quote! {
            #command.about(#short).long_about(#long)
        };
    }

    if attrs.require_subcommand() {
        command = quote! {
            #command.arg_required_else_help(true)
        };
    }

    let alias = attrs.alias();
    if !alias.is_empty() {
        command = quote! {
            #command.visible_aliases([#(#alias),*])
        };
    }

    command
}

#[cfg(test)]
mod tests {
    use indoc::indoc;

    use super::*;

    #[test]
    fn extract_function_documentation_with_single_line_comment() {
        let input = quote! {
            /// This is a test function
            fn foo() {}
        };

        let input_fn = syn::parse2(input).unwrap();
        let documentation = extract_function_documentation(&input_fn);

        let documentation = documentation.expect("should have documentation");
        assert_eq!("This is a test function", documentation.short());
        assert_eq!("This is a test function", documentation.long());
    }

    #[test]
    fn extract_function_documentation_with_multiple_line_comment() {
        let comment = indoc! { r#"
            This is a test comment
            with multiple lines"#
        };

        let input = quote! {
            /// This is a test comment
            /// with multiple lines
            fn foo() {}
        };

        let input_fn = syn::parse2(input).unwrap();
        let documentation = extract_function_documentation(&input_fn);

        let documentation = documentation.expect("should have documentation");
        assert_eq!("This is a test comment", documentation.short());
        assert_eq!(comment, documentation.long());
    }
}
