use proc_macro2::TokenStream;
use quote::quote;
use syn::{Error, FnArg, Ident, ItemFn, PatType, Result, Type};

use super::{Attributes, Generator, parse_attributes};

/// Code generator for `#[command]` functions
///
/// Commands are stateless CLI leaves that receive `(args, context)` and are executed through
/// `CommandRunner`, which sets up a push-based `TerminalPresenter`. This generator validates the
/// two-parameter signature at macro expansion time and implements `Generator` to emit
/// `ResolvedLeaf::Command` in the resolve function body. All other code generation (the `_init`
/// function, `_resolve` function, clap `Command` construction, and inventory submission) is
/// inherited from `Generator`'s default methods.
#[derive(Debug)]
pub struct CommandGenerator {
    attrs: Attributes,
    input: ItemFn,
    ident: Ident,
}

impl Generator for CommandGenerator {
    fn ident(&self) -> Ident {
        self.ident.clone()
    }

    fn attrs(&self) -> &Attributes {
        &self.attrs
    }

    fn input(&self) -> &ItemFn {
        &self.input
    }

    fn args_type(&self) -> Box<Type> {
        extract_command_argument_type(&self.input)
            .expect("function arguments must be validated in CommandGenerator::new()")
    }

    fn resolve_function_body(&self) -> TokenStream {
        let args_type = self.args_type();
        let command = self.ident();

        quote! {
            clawless::resolved_leaf::ResolvedLeaf::Command {
                matches,
                exec: |matches, context| {
                    Box::pin(async move {
                        use clawless::clap::FromArgMatches;
                        let args = #args_type::from_arg_matches(&matches).unwrap();
                        #command(args, context).await
                    })
                },
            }
        }
    }
}

impl CommandGenerator {
    /// Validates the function signature and parses macro attributes
    ///
    /// Signature validation happens eagerly here so that users get a clear compile error pointing
    /// at the function definition rather than a cryptic error from generated code.
    ///
    /// # Errors
    ///
    /// Returns a compile error if the function does not accept exactly two parameters (args and
    /// context) or if the macro attributes are invalid.
    pub fn new(attrs: TokenStream, input: ItemFn) -> Result<Self> {
        let attrs = parse_attributes(attrs, "command")?;
        let ident = input.sig.ident.clone();

        extract_command_argument_type(&input)?;

        Ok(Self {
            attrs,
            input,
            ident,
        })
    }
}

fn extract_command_argument_type(input_fn: &ItemFn) -> Result<Box<Type>> {
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

#[cfg(test)]
mod tests {
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
    fn extract_command_argument_type_with_all_params() {
        let input = quote! {
            fn foo(args: Args, context: Context) {}
        };

        let input_fn = syn::parse2(input).unwrap();
        let args_type = extract_command_argument_type(&input_fn).unwrap();

        assert_eq!("Args", args_type.to_token_stream().to_string());
    }

    #[test]
    fn extract_command_argument_type_with_one_param_returns_error() {
        let input = quote! {
            fn foo(args: Args) {}
        };

        let input_fn = syn::parse2(input).unwrap();
        let result = extract_command_argument_type(&input_fn);

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("missing the `context` parameter"));
    }

    #[test]
    fn extract_command_argument_type_with_three_params_returns_error() {
        let input = quote! {
            fn foo(args: Args, context: Context, extra: Extra) {}
        };

        let input_fn = syn::parse2(input).unwrap();
        let result = extract_command_argument_type(&input_fn);

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("too many parameters"));
    }

    #[test]
    fn extract_command_argument_type_without_params_returns_error() {
        let input = quote! {
            fn foo() {}
        };

        let input_fn = syn::parse2(input).unwrap();
        let result = extract_command_argument_type(&input_fn);

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("missing required parameters"));
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

    #[test]
    fn resolve_function_body() {
        let generator = generator_with_args();

        let actual = generator.resolve_function_body();
        let expected = quote! {
            clawless::resolved_leaf::ResolvedLeaf::Command {
                matches,
                exec: |matches, context| {
                    Box::pin(async move {
                        use clawless::clap::FromArgMatches;
                        let args = Args::from_arg_matches(&matches).unwrap();
                        foo(args, context).await
                    })
                },
            }
        };

        assert_eq!(actual.to_string(), expected.to_string());
    }
}
