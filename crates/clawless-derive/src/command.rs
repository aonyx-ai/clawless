use darling::ast::NestedMeta;
use darling::FromMeta;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{Expr, FnArg, Ident, ItemFn, Lit, Meta, PatType, Type};

use crate::inventory::inventory_name;

pub struct CommandGenerator {
    attrs: Attributes,
    input: ItemFn,
    ident: Ident,
}

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, FromMeta, Default)]
struct Attributes {
    #[darling(default)]
    noop: bool,
    #[darling(default)]
    root: bool,
}

#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
struct Documentation {
    short: String,
    long: String,
}

impl CommandGenerator {
    pub fn new(attrs: TokenStream, input: ItemFn) -> Self {
        let attrs = parse_attributes(attrs);
        let ident = input.sig.ident.clone();

        Self {
            attrs,
            input,
            ident,
        }
    }

    pub fn ident(&self) -> Ident {
        self.ident.clone()
    }

    pub fn is_root(&self) -> bool {
        self.attrs.root
    }

    pub fn initialization_function_name(&self) -> Ident {
        format_ident!("{}_init", self.ident)
    }

    pub fn wrapper_function_name(&self) -> Ident {
        format_ident!("{}_exec", self.ident)
    }

    pub fn initialization_function(&self) -> TokenStream {
        let function_name = self.initialization_function_name();
        let command_new = self.command_new();
        let inventory_name = inventory_name();

        quote! {
            pub fn #function_name() -> clap::Command {
                let mut command = #command_new;

                for subcommand in clawless::inventory::iter::<#inventory_name> {
                    command = command.subcommand((subcommand.init)());
                }

                command
            }
        }
    }

    pub fn wrapper_function(&self) -> TokenStream {
        let wrapper_function_name = self.wrapper_function_name();
        let wrapper_function_body = self.wrapper_function_body();
        let inventory_name = inventory_name();

        quote! {
            pub async fn #wrapper_function_name(args: clap::ArgMatches) -> clawless::CommandResult {
                for subcommand in clawless::inventory::iter::<#inventory_name> {
                    if let Some(matches) = args.subcommand_matches(subcommand.name) {
                        return (subcommand.func)(matches.clone()).await;
                    }
                }

                #wrapper_function_body
            }
        }
    }

    fn command_new(&self) -> TokenStream {
        let command_name = self.ident.to_string();
        let args_type = extract_function_argument_type(&self.input);
        let docs = extract_function_documentation(&self.input);

        let mut command = match args_type {
            Some(ty) => quote! {
                #ty::augment_args(clap::Command::new(#command_name))
            },
            None => quote! {
                clap::Command::new(#command_name)
            },
        };

        if self.is_root() {
            command = quote! {
                #command.about(clap::crate_description!())
            };
        } else if let Some(docs) = docs {
            let Documentation { short, long } = docs;

            command = quote! {
                #command.about(#short).long_about(#long)
            };
        }

        if self.attrs.noop {
            command = quote! {
                #command.arg_required_else_help(true)
            };
        }

        command
    }

    fn wrapper_function_body(&self) -> TokenStream {
        let args_type = extract_function_argument_type(&self.input);
        let command = self.ident();

        match args_type {
            Some(ty) => quote! {
                use clap::FromArgMatches;
                let args = #ty::from_arg_matches(&args).unwrap();
                #command(args).await
            },
            None => quote! {
                #command().await
            },
        }
    }
}

fn parse_attributes(attrs: TokenStream) -> Attributes {
    let argument_list = NestedMeta::parse_meta_list(attrs).unwrap();
    Attributes::from_list(&argument_list).unwrap()
}

fn extract_function_argument_type(input_fn: &ItemFn) -> Option<Box<Type>> {
    input_fn
        .sig
        .inputs
        .iter()
        .filter_map(|arg| match arg {
            FnArg::Receiver(_) => None,
            FnArg::Typed(PatType { ty, .. }) => Some(ty.clone()),
        })
        .next()
}

fn extract_function_documentation(input_fn: &ItemFn) -> Option<Documentation> {
    let mut docs = Vec::new();

    for attr in input_fn.attrs.iter() {
        if let Meta::NameValue(meta) = &attr.meta {
            if !attr.meta.path().is_ident("doc") {
                continue;
            }

            if let Expr::Lit(expr) = &meta.value {
                if let Lit::Str(lit) = &expr.lit {
                    docs.push(lit.value().trim().to_string());
                }
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

#[cfg(test)]
mod tests {
    use indoc::indoc;
    use quote::ToTokens;

    use super::*;

    fn generator_with_args() -> CommandGenerator {
        let input = quote! {
            fn foo(args: Args) {}
        };

        let input_function = syn::parse2::<ItemFn>(input).unwrap();

        CommandGenerator::new(TokenStream::new(), input_function)
    }

    fn generator_with_args_and_noop() -> CommandGenerator {
        let attrs = quote! {
            noop = true
        };

        let input = quote! {
            fn foo(args: Args) {}
        };

        let input_function = syn::parse2::<ItemFn>(input).unwrap();

        CommandGenerator::new(attrs, input_function)
    }

    fn generator_without_args() -> CommandGenerator {
        let input = quote! {
            fn foo() {}
        };

        let input_function = syn::parse2::<ItemFn>(input).unwrap();

        CommandGenerator::new(TokenStream::new(), input_function)
    }

    #[test]
    fn extract_function_argument_type_with_args() {
        let input = quote! {
            fn foo(args: Args) {}
        };

        let input_fn = syn::parse2(input).unwrap();
        let args_type = extract_function_argument_type(&input_fn);

        assert_eq!("Args", args_type.unwrap().to_token_stream().to_string());
    }

    #[test]
    fn extract_function_argument_type_without_args() {
        let input = quote! {
            fn foo() {}
        };

        let input_fn = syn::parse2(input).unwrap();
        let args_type = extract_function_argument_type(&input_fn);

        assert_eq!(None, args_type);
    }

    #[test]
    fn extract_function_documentation_with_single_line_comment() {
        let input = quote! {
            /// This is a test function
            fn foo() {}
        };

        let input_fn = syn::parse2(input).unwrap();
        let documentation = extract_function_documentation(&input_fn);

        assert_eq!(
            Some(Documentation {
                short: "This is a test function".to_string(),
                long: "This is a test function".to_string(),
            }),
            documentation
        );
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

        assert_eq!(
            Some(Documentation {
                short: "This is a test comment".to_string(),
                long: comment.to_string(),
            }),
            documentation
        );
    }

    #[test]
    fn command_new_with_args() {
        let generator = generator_with_args();

        let actual = generator.command_new();
        let expected = quote! {
            Args::augment_args(clap::Command::new("foo"))
        };

        assert_eq!(actual.to_string(), expected.to_string());
    }

    #[test]
    fn command_new_without_args() {
        let generator = generator_without_args();

        let actual = generator.command_new();
        let expected = quote! {
            clap::Command::new("foo")
        };

        assert_eq!(actual.to_string(), expected.to_string());
    }

    #[test]
    fn command_new_with_args_and_noop() {
        let generator = generator_with_args_and_noop();

        let actual = generator.command_new();
        let expected = quote! {
            Args::augment_args(clap::Command::new("foo")).arg_required_else_help(true)
        };

        assert_eq!(actual.to_string(), expected.to_string());
    }

    #[test]
    fn wrapper_function_body_with_args() {
        let generator = generator_with_args();

        let actual = generator.wrapper_function_body();
        let expected = quote! {
            use clap::FromArgMatches;
            let args = Args::from_arg_matches(&args).unwrap();
            foo(args).await
        };

        assert_eq!(actual.to_string(), expected.to_string());
    }

    #[test]
    fn wrapper_function_body_without_args() {
        let generator = generator_without_args();

        let actual = generator.wrapper_function_body();
        let expected = quote! {
            foo().await
        };

        assert_eq!(actual.to_string(), expected.to_string());
    }
}
