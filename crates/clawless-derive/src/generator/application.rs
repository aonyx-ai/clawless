use proc_macro2::TokenStream;
use quote::quote;
use syn::{Error, FnArg, Ident, ItemFn, PatType, Result, Type};

use super::{Attributes, Generator, parse_attributes};

/// Code generator for `#[application]` functions
///
/// Applications are stateful TUI leaves that receive `(args, context, projection)` and are
/// executed through `ApplicationRunner`, which sets up a pull-based `Projection` instead of a
/// presenter. This generator validates the three-parameter signature at macro expansion time and
/// implements `Generator` to emit `ResolvedLeaf::Application` in the resolve function body. All
/// other code generation is inherited from `Generator`'s default methods, identical to
/// `CommandGenerator`.
#[derive(Debug)]
pub struct ApplicationGenerator {
    attrs: Attributes,
    input: ItemFn,
    ident: Ident,
}

impl Generator for ApplicationGenerator {
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
        extract_application_argument_type(&self.input)
            .expect("function arguments must be validated in ApplicationGenerator::new()")
    }

    fn resolve_function_body(&self) -> TokenStream {
        let args_type = self.args_type();
        let application = self.ident();

        quote! {
            clawless::resolved_leaf::ResolvedLeaf::Application {
                matches,
                exec: |matches, context, projection| {
                    Box::pin(async move {
                        use clawless::clap::FromArgMatches;
                        let args = #args_type::from_arg_matches(&matches).unwrap();
                        #application(args, context, projection).await
                    })
                },
            }
        }
    }
}

impl ApplicationGenerator {
    /// Validates the function signature and parses macro attributes
    ///
    /// Signature validation happens eagerly here so that users get a clear compile error pointing
    /// at the function definition rather than a cryptic error from generated code.
    ///
    /// # Errors
    ///
    /// Returns a compile error if the function does not accept exactly three parameters (args,
    /// context, and projection) or if the macro attributes are invalid.
    pub fn new(attrs: TokenStream, input: ItemFn) -> Result<Self> {
        let attrs = parse_attributes(attrs, "application")?;
        let ident = input.sig.ident.clone();

        extract_application_argument_type(&input)?;

        Ok(Self {
            attrs,
            input,
            ident,
        })
    }
}

fn extract_application_argument_type(input_fn: &ItemFn) -> Result<Box<Type>> {
    let mut function_arguments = input_fn.sig.inputs.iter().filter_map(|arg| match arg {
        FnArg::Receiver(_) => None,
        FnArg::Typed(PatType { ty, .. }) => Some(ty.clone()),
    });

    let args = function_arguments.next();
    let context = function_arguments.next();
    let projection = function_arguments.next();
    let extra = function_arguments.next();

    if extra.is_some() {
        return Err(Error::new_spanned(
            &input_fn.sig,
            "application function has too many parameters\n\n\
             = help: application functions must accept exactly three parameters: arguments, context, and projection\n\n    \
             #[application]\n    \
             pub async fn my_app(args: MyArgs, context: Context, projection: Projection) -> CommandResult {\n        \
             ...\n    \
             }",
        ));
    }

    match (args, context, projection) {
        (Some(args_type), Some(_), Some(_)) => Ok(args_type),
        (None, None, None) => Err(Error::new_spanned(
            &input_fn.sig,
            "application function is missing required parameters\n\n\
             = help: application functions must accept three parameters: arguments, context, and projection\n\n    \
             #[derive(Debug, Args)]\n    \
             pub struct MyArgs {}\n\n    \
             #[application]\n    \
             pub async fn my_app(args: MyArgs, context: Context, projection: Projection) -> CommandResult {\n        \
             ...\n    \
             }",
        )),
        (Some(_), Some(_), None) => Err(Error::new_spanned(
            &input_fn.sig,
            "application function is missing the `projection` parameter\n\n\
             = help: application functions must accept `projection: Projection` as the third parameter\n\n    \
             #[application]\n    \
             pub async fn my_app(args: MyArgs, context: Context, projection: Projection) -> CommandResult {\n        \
             ...\n    \
             }",
        )),
        (Some(_), None, _) => Err(Error::new_spanned(
            &input_fn.sig,
            "application function is missing the `context` and `projection` parameters\n\n\
             = help: application functions must accept `context: Context` and `projection: Projection`\n\n    \
             #[application]\n    \
             pub async fn my_app(args: MyArgs, context: Context, projection: Projection) -> CommandResult {\n        \
             ...\n    \
             }",
        )),
        (None, Some(_), _) | (None, None, Some(_)) => {
            unreachable!("sequential Iterator::next() cannot skip elements")
        }
    }
}

#[cfg(test)]
mod tests {
    use quote::ToTokens;

    use super::*;

    fn generator_with_args() -> ApplicationGenerator {
        let input = quote! {
            fn foo(args: Args, context: Context, projection: Projection) {}
        };

        let input_function = syn::parse2::<ItemFn>(input).unwrap();

        ApplicationGenerator::new(TokenStream::new(), input_function).unwrap()
    }

    #[test]
    fn application_generator_new_with_two_params_returns_error() {
        let input = quote! {
            fn foo(args: Args, context: Context) {}
        };

        let input_function = syn::parse2::<ItemFn>(input).unwrap();
        let result = ApplicationGenerator::new(TokenStream::new(), input_function);

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(
            err.to_string()
                .contains("missing the `projection` parameter")
        );
    }

    #[test]
    fn application_generator_new_with_four_params_returns_error() {
        let input = quote! {
            fn foo(args: Args, context: Context, projection: Projection, extra: Extra) {}
        };

        let input_function = syn::parse2::<ItemFn>(input).unwrap();
        let result = ApplicationGenerator::new(TokenStream::new(), input_function);

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("too many parameters"));
    }

    #[test]
    fn application_generator_new_with_one_param_returns_error() {
        let input = quote! {
            fn foo(args: Args) {}
        };

        let input_function = syn::parse2::<ItemFn>(input).unwrap();
        let result = ApplicationGenerator::new(TokenStream::new(), input_function);

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(
            err.to_string()
                .contains("missing the `context` and `projection` parameters")
        );
    }

    #[test]
    fn application_generator_new_without_params_returns_error() {
        let input = quote! {
            fn foo() {}
        };

        let input_function = syn::parse2::<ItemFn>(input).unwrap();
        let result = ApplicationGenerator::new(TokenStream::new(), input_function);

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("missing required parameters"));
    }

    #[test]
    fn extract_application_argument_type_with_all_params() {
        let input = quote! {
            fn foo(args: Args, context: Context, projection: Projection) {}
        };

        let input_fn = syn::parse2(input).unwrap();
        let args_type = extract_application_argument_type(&input_fn).unwrap();

        assert_eq!("Args", args_type.to_token_stream().to_string());
    }

    #[test]
    fn resolve_function_body() {
        let generator = generator_with_args();

        let actual = generator.resolve_function_body();
        let expected = quote! {
            clawless::resolved_leaf::ResolvedLeaf::Application {
                matches,
                exec: |matches, context, projection| {
                    Box::pin(async move {
                        use clawless::clap::FromArgMatches;
                        let args = Args::from_arg_matches(&matches).unwrap();
                        foo(args, context, projection).await
                    })
                },
            }
        };

        assert_eq!(actual.to_string(), expected.to_string());
    }
}
