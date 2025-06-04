use darling::ast::NestedMeta;
use darling::FromMeta;
use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{parse_macro_input, FnArg, Ident, ItemFn, PatType, Type};

use crate::INVENTORY_NAME;

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, FromMeta, Default)]
struct Attributes {
    #[darling(default)]
    noop: bool,
}

pub fn command_impl(attrs: TokenStream, input: TokenStream) -> TokenStream {
    let input_fn = parse_macro_input!(input as ItemFn);

    let command_ident = input_fn.sig.ident.clone();
    let command_name = command_ident.to_string();
    let init_fn_name = format_ident!("{}_init", command_ident);
    let wrapper_fn_name = format_ident!("{}_exec", command_ident);

    let inventory_name = format_ident!("{}", INVENTORY_NAME);

    let macro_attrs = parse_attributes(attrs);

    let args_type = extract_function_argument_type(&input_fn);
    let command_initialization =
        generate_command_initialization(&args_type, &command_name, &macro_attrs);
    let wrapper_function_body = generate_wrapper_fn_body(&args_type, &command_ident);

    let output = quote! {
        struct #inventory_name {
            name: &'static str,
            init: fn() -> clap::Command,
            func: fn(clap::ArgMatches) -> std::pin::Pin<Box<dyn std::future::Future<Output = ()>>>,
        }
        inventory::collect!(#inventory_name);

        #input_fn

        pub fn #init_fn_name() -> clap::Command {
            let mut command = #command_initialization;

            for subcommand in inventory::iter::<#inventory_name> {
                command = command.subcommand((subcommand.init)());
            }

            command
        }

        pub async fn #wrapper_fn_name(args: clap::ArgMatches) {
            for subcommand in inventory::iter::<#inventory_name> {
                if let Some(matches) = args.subcommand_matches(subcommand.name) {
                    return (subcommand.func)(matches.clone()).await;
                }
            }

            #wrapper_function_body
        }

        inventory::submit!(super::#inventory_name {
            name: #command_name,
            init: #init_fn_name,
            func: |args| Box::pin(#wrapper_fn_name(args)),
        });
    };

    output.into()
}

fn parse_attributes(attrs: TokenStream) -> Attributes {
    let argument_list = NestedMeta::parse_meta_list(attrs.into()).unwrap();
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

fn generate_command_initialization(
    args_type: &Option<Box<Type>>,
    command_name: &str,
    macro_attrs: &Attributes,
) -> proc_macro2::TokenStream {
    let mut command = match args_type {
        Some(ty) => quote! {
            #ty::augment_args(clap::Command::new(#command_name))
        },
        None => quote! {
            clap::Command::new(#command_name)
        },
    };

    if macro_attrs.noop {
        command = quote! {
            #command.arg_required_else_help(true)
        };
    }

    command
}

fn generate_wrapper_fn_body(
    args_type: &Option<Box<Type>>,
    command_ident: &Ident,
) -> proc_macro2::TokenStream {
    match args_type {
        Some(ty) => quote! {
            use clap::FromArgMatches;
            let args = #ty::from_arg_matches(&args).unwrap();
            #command_ident(args).await;
        },
        None => quote! {
            #command_ident().await;
        },
    }
}

#[cfg(test)]
mod tests {
    use quote::ToTokens;
    use syn::{Ident, Path, TypePath};

    use super::*;

    // #[test]
    // fn parse_attributes_with_noop() {
    //     let attrs = quote! {
    //         #[command(noop = true)]
    //         pub async fn foo() {}
    //     };

    //     let actual = parse_attributes(attrs.into());
    //     let expected = Attributes { noop: true };

    //     assert_eq!(actual, expected);
    // }

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
    fn generate_command_initialization_with_args() {
        let args_type = Some(Box::new(Type::Path(TypePath {
            qself: None,
            path: Path::from(Ident::new("Args", proc_macro2::Span::call_site())),
        })));
        let command_name = "foo";

        let actual =
            generate_command_initialization(&args_type, command_name, &Attributes { noop: false });
        let expected = quote! {
            Args::augment_args(clap::Command::new("foo"))
        };

        assert_eq!(actual.to_string(), expected.to_string());
    }

    #[test]
    fn generate_command_initialization_without_args() {
        let args_type = None;
        let command_name = "foo";

        let actual =
            generate_command_initialization(&args_type, command_name, &Attributes { noop: true });
        let expected = quote! {
            clap::Command::new("foo").arg_required_else_help(true)
        };

        assert_eq!(actual.to_string(), expected.to_string());
    }

    #[test]
    fn generate_wrapper_fn_body_with_args() {
        let args_type = Some(Box::new(Type::Path(TypePath {
            qself: None,
            path: Path::from(Ident::new("Args", proc_macro2::Span::call_site())),
        })));
        let command_name = format_ident!("foo");

        let actual = generate_wrapper_fn_body(&args_type, &command_name);
        let expected = quote! {
            use clap::FromArgMatches;
            let args = Args::from_arg_matches(&args).unwrap();
            foo(args).await;
        };

        assert_eq!(actual.to_string(), expected.to_string());
    }

    #[test]
    fn generate_wrapper_fn_body_without_args() {
        let args_type = None;
        let command_name = Ident::new("foo", proc_macro2::Span::call_site());

        let actual = generate_wrapper_fn_body(&args_type, &command_name);
        let expected = quote! {
            foo().await;
        };

        assert_eq!(actual.to_string(), expected.to_string());
    }
}
