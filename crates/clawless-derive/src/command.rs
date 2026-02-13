use darling::FromMeta;
use darling::ast::NestedMeta;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{Error, Expr, FnArg, Ident, ItemFn, Lit, Meta, PatType, Result, Type};

use crate::inventory::inventory_name;

#[derive(Debug)]
pub struct CommandGenerator {
    attrs: Attributes,
    input: ItemFn,
    ident: Ident,
}

#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, FromMeta, Default)]
struct Attributes {
    /// Require a subcommand; show help if invoked without one
    #[darling(default)]
    require_subcommand: bool,
    #[darling(default)]
    root: bool,
    #[darling(default, multiple)]
    alias: Vec<String>,
}

#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
struct Documentation {
    short: String,
    long: String,
}

impl CommandGenerator {
    pub fn new(attrs: TokenStream, input: ItemFn) -> Result<Self> {
        let attrs = parse_attributes(attrs)?;
        let ident = input.sig.ident.clone();

        // Validate function signature early to catch errors
        extract_function_argument_type(&input)?;

        Ok(Self {
            attrs,
            input,
            ident,
        })
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
            pub fn #function_name() -> clawless::clap::Command {
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
            pub async fn #wrapper_function_name(args: clawless::clap::ArgMatches, context: clawless::context::Context) -> clawless::CommandResult {
                for subcommand in clawless::inventory::iter::<#inventory_name> {
                    if let Some(matches) = args.subcommand_matches(subcommand.name) {
                        return (subcommand.func)(matches.clone(), context).await;
                    }
                }

                #wrapper_function_body
            }
        }
    }

    fn command_new(&self) -> TokenStream {
        let command_name = self.ident.to_string();
        // Safe to unwrap: validated in CommandGenerator::new()
        let args_type = extract_function_argument_type(&self.input)
            .expect("function arguments must be validated in CommandGenerator::new()");
        let docs = extract_function_documentation(&self.input);

        let mut command = quote! {
            #args_type::augment_args(clawless::clap::Command::new(#command_name))
        };

        if self.is_root() {
            command = quote! {
                #command.about(clawless::clap::crate_description!())
            };
        } else if let Some(docs) = docs {
            let Documentation { short, long } = docs;

            command = quote! {
                #command.about(#short).long_about(#long)
            };
        }

        if self.attrs.require_subcommand {
            command = quote! {
                #command.arg_required_else_help(true)
            };
        }

        if !self.attrs.alias.is_empty() {
            let aliases = &self.attrs.alias;
            command = quote! {
                #command.visible_aliases([#(#aliases),*])
            };
        }

        command
    }

    fn wrapper_function_body(&self) -> TokenStream {
        // Safe to unwrap: validated in CommandGenerator::new()
        let args_type = extract_function_argument_type(&self.input)
            .expect("function arguments must be validated in CommandGenerator::new()");
        let command = self.ident();

        quote! {
            use clawless::clap::FromArgMatches;
            let args = #args_type::from_arg_matches(&args).unwrap();
            #command(args, context).await
        }
    }
}

fn parse_attributes(attrs: TokenStream) -> Result<Attributes> {
    let argument_list = NestedMeta::parse_meta_list(attrs.clone()).map_err(|e| {
        Error::new_spanned(
            attrs.clone(),
            format!(
                "invalid attribute syntax: {e}\n\n\
                 = help: use one of the supported attributes\n\n    \
                 #[command]\n    \
                 #[command(alias = \"g\")]\n    \
                 #[command(require_subcommand)]\n    \
                 #[command(alias = \"g\", require_subcommand)]"
            ),
        )
    })?;

    Attributes::from_list(&argument_list).map_err(|e| {
        Error::new_spanned(
            attrs,
            format!(
                "{e}\n\n\
                 = help: supported attributes are `alias` and `require_subcommand`\n\n    \
                 #[command(alias = \"g\", require_subcommand)]"
            ),
        )
    })
}

fn extract_function_argument_type(input_fn: &ItemFn) -> Result<Box<Type>> {
    let mut function_arguments = input_fn.sig.inputs.iter().filter_map(|arg| match arg {
        FnArg::Receiver(_) => None,
        FnArg::Typed(PatType { ty, .. }) => Some(ty.clone()),
    });

    let args = function_arguments.next();
    let context = function_arguments.next();

    let extra = function_arguments.next();

    if extra.is_some() {
        return Err(Error::new_spanned(
            &input_fn.sig,
            "command function has too many parameters\n\n\
             = help: command functions must accept exactly two parameters: an arguments struct and context\n\n    \
             #[command]\n    \
             pub async fn my_command(args: MyArgs, context: Context) -> CommandResult {\n        \
             ...\n    \
             }",
        ));
    }

    match (args, context) {
        (Some(args_type), Some(_)) => Ok(args_type),
        (None, None) => Err(Error::new_spanned(
            &input_fn.sig,
            "command function is missing required parameters\n\n\
             = help: command functions must accept two parameters: an arguments struct and context\n\n    \
             #[derive(Debug, Args)]\n    \
             pub struct MyArgs {}\n\n    \
             #[command]\n    \
             pub async fn my_command(args: MyArgs, context: Context) -> CommandResult {\n        \
             ...\n    \
             }",
        )),
        (Some(_), None) => Err(Error::new_spanned(
            &input_fn.sig,
            "command function is missing the `context` parameter\n\n\
             = help: command functions must accept `context: Context` as the second parameter\n\n    \
             #[command]\n    \
             pub async fn my_command(args: MyArgs, context: Context) -> CommandResult {\n        \
             ...\n    \
             }",
        )),
        (None, Some(_)) => {
            unreachable!("sequential Iterator::next() cannot skip elements")
        }
    }
}

fn extract_function_documentation(input_fn: &ItemFn) -> Option<Documentation> {
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

#[cfg(test)]
mod tests {
    use indoc::indoc;
    use quote::ToTokens;

    use super::*;

    fn generator_with_args() -> CommandGenerator {
        let input = quote! {
            fn foo(args: Args, context: Context) {}
        };

        let input_function = syn::parse2::<ItemFn>(input).unwrap();

        CommandGenerator::new(TokenStream::new(), input_function).unwrap()
    }

    fn generator_with_require_subcommand() -> CommandGenerator {
        let attrs = quote! {
            require_subcommand
        };

        let input = quote! {
            fn foo(args: Args, context: Context) {}
        };

        let input_function = syn::parse2::<ItemFn>(input).unwrap();

        CommandGenerator::new(attrs, input_function).unwrap()
    }

    fn generator_with_single_alias() -> CommandGenerator {
        let attrs = quote! {
            alias = "f"
        };

        let input = quote! {
            fn foo(args: Args, context: Context) {}
        };

        let input_function = syn::parse2::<ItemFn>(input).unwrap();

        CommandGenerator::new(attrs, input_function).unwrap()
    }

    fn generator_with_multiple_aliases() -> CommandGenerator {
        let attrs = quote! {
            alias = "f", alias = "fo"
        };

        let input = quote! {
            fn foo(args: Args, context: Context) {}
        };

        let input_function = syn::parse2::<ItemFn>(input).unwrap();

        CommandGenerator::new(attrs, input_function).unwrap()
    }

    fn generator_with_require_subcommand_and_alias() -> CommandGenerator {
        let attrs = quote! {
            require_subcommand, alias = "f"
        };

        let input = quote! {
            fn foo(args: Args, context: Context) {}
        };

        let input_function = syn::parse2::<ItemFn>(input).unwrap();

        CommandGenerator::new(attrs, input_function).unwrap()
    }

    #[test]
    fn command_generator_new_with_one_param_returns_error() {
        let input = quote! {
            fn foo(args: Args) {}
        };

        let input_function = syn::parse2::<ItemFn>(input).unwrap();
        let result = CommandGenerator::new(TokenStream::new(), input_function);

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("missing the `context` parameter"));
    }

    #[test]
    fn command_generator_new_with_three_params_returns_error() {
        let input = quote! {
            fn foo(args: Args, context: Context, extra: Extra) {}
        };

        let input_function = syn::parse2::<ItemFn>(input).unwrap();
        let result = CommandGenerator::new(TokenStream::new(), input_function);

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("too many parameters"));
    }

    #[test]
    fn extract_function_argument_type_with_all_params() {
        let input = quote! {
            fn foo(args: Args, context: Context) {}
        };

        let input_fn = syn::parse2(input).unwrap();
        let args_type = extract_function_argument_type(&input_fn).unwrap();

        assert_eq!("Args", args_type.to_token_stream().to_string());
    }

    #[test]
    fn extract_function_argument_type_with_one_param_returns_error() {
        let input = quote! {
            fn foo(args: Args) {}
        };

        let input_fn = syn::parse2(input).unwrap();
        let result = extract_function_argument_type(&input_fn);

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("missing the `context` parameter"));
    }

    #[test]
    fn extract_function_argument_type_with_three_params_returns_error() {
        let input = quote! {
            fn foo(args: Args, context: Context, extra: Extra) {}
        };

        let input_fn = syn::parse2(input).unwrap();
        let result = extract_function_argument_type(&input_fn);

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("too many parameters"));
    }

    #[test]
    fn extract_function_argument_type_without_params_returns_error() {
        let input = quote! {
            fn foo() {}
        };

        let input_fn = syn::parse2(input).unwrap();
        let result = extract_function_argument_type(&input_fn);

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("missing required parameters"));
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
            Args::augment_args(clawless::clap::Command::new("foo"))
        };

        assert_eq!(actual.to_string(), expected.to_string());
    }

    #[test]
    fn command_new_with_require_subcommand() {
        let generator = generator_with_require_subcommand();

        let actual = generator.command_new();
        let expected = quote! {
            Args::augment_args(clawless::clap::Command::new("foo")).arg_required_else_help(true)
        };

        assert_eq!(actual.to_string(), expected.to_string());
    }

    #[test]
    fn wrapper_function_body() {
        let generator = generator_with_args();

        let actual = generator.wrapper_function_body();
        let expected = quote! {
            use clawless::clap::FromArgMatches;
            let args = Args::from_arg_matches(&args).unwrap();
            foo(args, context).await
        };

        assert_eq!(actual.to_string(), expected.to_string());
    }

    #[test]
    fn command_new_with_single_alias() {
        let generator = generator_with_single_alias();

        let actual = generator.command_new();
        let expected = quote! {
            Args::augment_args(clawless::clap::Command::new("foo")).visible_aliases(["f"])
        };

        assert_eq!(actual.to_string(), expected.to_string());
    }

    #[test]
    fn command_new_with_multiple_aliases() {
        let generator = generator_with_multiple_aliases();

        let actual = generator.command_new();
        let expected = quote! {
            Args::augment_args(clawless::clap::Command::new("foo")).visible_aliases(["f", "fo"])
        };

        assert_eq!(actual.to_string(), expected.to_string());
    }

    #[test]
    fn command_new_with_require_subcommand_and_alias() {
        let generator = generator_with_require_subcommand_and_alias();

        let actual = generator.command_new();
        let expected = quote! {
            Args::augment_args(clawless::clap::Command::new("foo")).arg_required_else_help(true).visible_aliases(["f"])
        };

        assert_eq!(actual.to_string(), expected.to_string());
    }
}
