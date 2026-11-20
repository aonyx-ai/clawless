//! Generator for `#[application]` functions

use bon::bon;
use proc_macro2::TokenStream;
use quote::quote;
use syn::{Error, FnArg, Ident, ItemFn, PatType, Result, Type};

use super::{Attributes, Generator, parse_attributes};
use crate::command_name::CommandName;

/// Code generator for `#[application]` functions
///
/// Applications are stateful TUI leaves that receive `(args, context, projection)` and are
/// executed through `ApplicationRunner`, which sets up a pull-based `Projection` instead of a
/// presenter. This generator validates the three-parameter signature at macro expansion time and
/// implements `Generator` to emit `ResolvedLeaf::Application` in the resolve function body. All
/// other code generation is inherited from `Generator`'s default methods, identical to
/// `CommandGenerator`.
#[derive(Debug)]
pub(crate) struct ApplicationGenerator {
    /// Parsed `#[application(...)]` attributes such as `alias` and `require_subcommand`
    attrs: Attributes,
    /// The function that the user wrote, which the macro emits again without a change
    input: ItemFn,
    /// Identifier of the function that the user wrote, which names the generated functions
    ident: Ident,
    /// Name that this leaf has on the command line, which the constructor derives from
    /// `ident`
    command_name: CommandName,
}

impl Generator for ApplicationGenerator {
    fn ident(&self) -> Ident {
        self.ident.clone()
    }

    fn command_name(&self) -> &CommandName {
        &self.command_name
    }

    fn attrs(&self) -> &Attributes {
        &self.attrs
    }

    fn input(&self) -> &ItemFn {
        &self.input
    }

    #[allow(
        clippy::expect_used,
        reason = "`ApplicationGenerator::new` rejects any function whose signature does not match, so by the time this runs the extraction cannot fail."
    )]
    fn args_type(&self) -> Box<Type> {
        extract_application_argument_type(&self.input)
            .expect("function arguments must be validated in ApplicationGenerator::new()")
    }

    fn resolve_function_body(&self) -> TokenStream {
        let args_type = self.args_type();
        let application = self.ident();
        let map_into_exit = self.map_into_exit();

        quote! {
            clawless::resolved_leaf::ResolvedLeaf::Application {
                matches,
                exec: |matches, context, projection| {
                    Box::pin(async move {
                        use clawless::clap::FromArgMatches;
                        let args = #args_type::from_arg_matches(&matches).unwrap();
                        #application(args, context, projection).await #map_into_exit
                    })
                },
            }
        }
    }
}

#[bon]
impl ApplicationGenerator {
    /// Validates the function signature and parses macro attributes
    ///
    /// Signature validation happens eagerly here. Users therefore get a clear compile error
    /// pointing at the function definition, rather than a cryptic error from generated code.
    ///
    /// # Errors
    ///
    /// Returns a compile error if the function does not accept exactly three parameters (args,
    /// context, and projection). Returns a compile error if the macro attributes are invalid.
    /// Returns one as well if the name of the function does not translate into a command name.
    #[builder]
    pub(crate) fn new(attrs: TokenStream, input: ItemFn) -> Result<Self> {
        let attrs = parse_attributes(attrs, "application")?;
        let ident = input.sig.ident.clone();

        extract_application_argument_type(&input)?;

        let command_name = CommandName::try_from(&ident)?;

        Ok(Self {
            attrs,
            input,
            ident,
            command_name,
        })
    }
}

/// Returns the type of the first parameter of an application function
///
/// An application function must take exactly `(args, context, projection)`. This function makes
/// sure that the shape is correct at expansion time. [`Generator::args_type`] can therefore
/// treat the later extraction as infallible.
///
/// # Errors
///
/// Returns [`Error`] if the function does not take exactly three parameters. The error points
/// at the wrong parameter, so that the user sees the mistake in place.
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
    #![allow(
        clippy::missing_panics_doc,
        reason = "an assertion in a test panics by design, so a `# Panics` section would repeat it"
    )]

    use quote::ToTokens;

    use super::*;
    use crate::generator::auto_traits::{AmbiguousIfSend, AmbiguousIfSync};

    fn generator_with_args() -> ApplicationGenerator {
        let input = quote! {
            fn foo(args: Args, context: Context, projection: Projection) {}
        };

        let input_function = syn::parse2::<ItemFn>(input).unwrap();

        ApplicationGenerator::builder()
            .attrs(TokenStream::new())
            .input(input_function)
            .build()
            .unwrap()
    }

    #[test]
    fn application_generator_is_unpin() {
        fn assert_unpin<T: Unpin>() {}
        assert_unpin::<ApplicationGenerator>();
    }

    #[test]
    fn application_generator_new_with_four_params_returns_error() {
        let input = quote! {
            fn foo(args: Args, context: Context, projection: Projection, extra: Extra) {}
        };

        let input_function = syn::parse2::<ItemFn>(input).unwrap();
        let error = ApplicationGenerator::builder()
            .attrs(TokenStream::new())
            .input(input_function)
            .build()
            .expect_err("should fail");

        assert!(error.to_string().contains("too many parameters"));
    }

    #[test]
    fn application_generator_new_with_one_param_returns_error() {
        let input = quote! {
            fn foo(args: Args) {}
        };

        let input_function = syn::parse2::<ItemFn>(input).unwrap();
        let error = ApplicationGenerator::builder()
            .attrs(TokenStream::new())
            .input(input_function)
            .build()
            .expect_err("should fail");

        assert!(
            error
                .to_string()
                .contains("missing the `context` and `projection` parameters")
        );
    }

    #[test]
    fn application_generator_new_with_two_params_returns_error() {
        let input = quote! {
            fn foo(args: Args, context: Context) {}
        };

        let input_function = syn::parse2::<ItemFn>(input).unwrap();
        let error = ApplicationGenerator::builder()
            .attrs(TokenStream::new())
            .input(input_function)
            .build()
            .expect_err("should fail");

        assert!(
            error
                .to_string()
                .contains("missing the `projection` parameter")
        );
    }

    #[test]
    fn application_generator_new_without_params_returns_error() {
        let input = quote! {
            fn foo() {}
        };

        let input_function = syn::parse2::<ItemFn>(input).unwrap();
        let error = ApplicationGenerator::builder()
            .attrs(TokenStream::new())
            .input(input_function)
            .build()
            .expect_err("should fail");

        assert!(error.to_string().contains("missing required parameters"));
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
                        foo(args, context, projection).await.map(clawless::exit::Exit::from)
                    })
                },
            }
        };

        assert_eq!(actual.to_string(), expected.to_string());
    }

    /// `ApplicationGenerator` is not `Send`
    ///
    /// It owns a parsed function, and the syntax tree of the compiler holds handles that no other
    /// thread may use.
    #[test]
    fn trait_not_send_application_generator() {
        <ApplicationGenerator as AmbiguousIfSend<_>>::compiles_only_without_send();
    }

    /// `ApplicationGenerator` is not `Sync`
    ///
    /// It owns a parsed function, and the syntax tree of the compiler holds handles that no other
    /// thread may use.
    #[test]
    fn trait_not_sync_application_generator() {
        <ApplicationGenerator as AmbiguousIfSync<_>>::compiles_only_without_sync();
    }
}
