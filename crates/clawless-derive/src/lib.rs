#![cfg_attr(not(doctest),doc = include_str!("../README.md"))]
#![warn(missing_docs)]

use proc_macro::TokenStream;
use quote::quote;
use syn::{ItemFn, parse_macro_input};

use crate::generator::{ApplicationGenerator, CommandGenerator, Generator};
use crate::inventory::InventoryGenerator;

mod generator;
mod inventory;

/// Writes an informational message via the [`Output`] on [`Context`]
///
/// Expands to `context.output().message(format!(...)).await.expect("event channel closed")`,
/// where `context` resolves to the local variable in the calling function's scope. This is the
/// macro equivalent of [`Output::message`].
///
/// The macro must be called from an async function because the core [`Output`] methods are async.
///
/// # Examples
///
/// ```rust,ignore
/// use clawless::prelude::*;
///
/// #[command]
/// pub async fn deploy(args: DeployArgs, context: Context) -> CommandResult {
///     message!("deploying to {}", args.target);
///     Ok(())
/// }
/// ```
///
/// [`Context`]: clawless::context::Context
/// [`Output`]: clawless_core::output::Output
/// [`Output::message`]: clawless_core::output::Output::message
#[proc_macro]
pub fn message(input: TokenStream) -> TokenStream {
    output_format_macro(input, "message")
}

/// Writes a supplementary detail via the [`Output`] on [`Context`]
///
/// Expands to `context.output().detail(format!(...)).await.expect("event channel closed")`,
/// where `context` resolves to the local variable in the calling function's scope. Detail is only
/// shown when the user passes `--verbose`. This is the macro equivalent of [`Output::detail`].
///
/// The macro must be called from an async function because the core [`Output`] methods are async.
///
/// # Examples
///
/// ```rust,ignore
/// use clawless::prelude::*;
///
/// #[command]
/// pub async fn deploy(args: DeployArgs, context: Context) -> CommandResult {
///     detail!("config loaded from {}", args.config_path);
///     Ok(())
/// }
/// ```
///
/// [`Context`]: clawless::context::Context
/// [`Output`]: clawless_core::output::Output
/// [`Output::detail`]: clawless_core::output::Output::detail
#[proc_macro]
pub fn detail(input: TokenStream) -> TokenStream {
    output_format_macro(input, "detail")
}

/// Writes an artifact value via the [`Output`] on [`Context`]
///
/// Expands to `context.output().artifact(...).await.expect("event channel closed")`, where
/// `context` resolves to the local variable in the calling function's scope. Unlike [`message!`]
/// and [`detail!`], this macro does not use `format!` — it takes an expression that implements
/// [`Display`], [`Serialize`], and [`Debug`]. This is the macro equivalent of
/// [`Output::artifact`].
///
/// The macro must be called from an async function because the core [`Output`] methods are async.
///
/// # Examples
///
/// ```rust,ignore
/// use clawless::prelude::*;
///
/// #[command]
/// pub async fn count(args: CountArgs, context: Context) -> CommandResult {
///     let count = WordCount { words: 42 };
///     artifact!(count);
///     Ok(())
/// }
/// ```
///
/// [`Context`]: clawless::context::Context
/// [`Debug`]: std::fmt::Debug
/// [`Display`]: std::fmt::Display
/// [`Output`]: clawless_core::output::Output
/// [`Output::artifact`]: clawless_core::output::Output::artifact
/// [`Serialize`]: serde::Serialize
#[proc_macro]
pub fn artifact(input: TokenStream) -> TokenStream {
    let input = proc_macro2::TokenStream::from(input);
    let context = proc_macro2::Ident::new("context", proc_macro2::Span::call_site());
    quote! {
        #context.output().artifact(#input)
            .await
            .expect("event channel closed")
    }
    .into()
}

fn output_format_macro(input: TokenStream, method: &str) -> TokenStream {
    let input = proc_macro2::TokenStream::from(input);
    let context = proc_macro2::Ident::new("context", proc_macro2::Span::call_site());
    let method = proc_macro2::Ident::new(method, proc_macro2::Span::call_site());
    quote! {
        #context.output().#method(format!(#input))
            .await
            .expect("event channel closed")
    }
    .into()
}

/// Set up the commands module for a Clawless application
///
/// This macro generates the root command for the command-line application and allows subcommands to
/// be registered under it. It should be called inside `src/commands.rs` or `src/commands/mod.rs` to
/// follow Clawless's convention.
///
/// # Example
///
/// ```rust,ignore
/// // src/commands.rs
/// mod greet;
/// mod deploy;
///
/// clawless::commands!();
/// ```
#[proc_macro]
pub fn commands(_input: TokenStream) -> TokenStream {
    let output = quote! {
        use clawless::prelude::*;
        #[derive(Debug, clawless::clap::Args)]
        struct ClawlessEntryPoint {}

        #[clawless::command(require_subcommand, root = true)]
        pub async fn clawless(_args: ClawlessEntryPoint, _context: clawless::context::Context) -> clawless::CommandResult {
            Ok(())
        }
    };
    output.into()
}

/// Initialize and run a Clawless application
///
/// This macro generates the `main` function for a Clawless application. It uses two-phase
/// dispatch: first it parses arguments and resolves the subcommand tree to find the leaf, then
/// it matches on the [`ResolvedLeaf`] variant to delegate to the appropriate runner.
///
/// # Example
///
/// ```rust,ignore
/// // src/main.rs
/// mod commands;
///
/// clawless::main!();
/// ```
///
/// [`ResolvedLeaf`]: clawless::resolved_leaf::ResolvedLeaf
#[proc_macro]
pub fn main(_input: TokenStream) -> TokenStream {
    let output = quote! {
        fn main() -> Result<(), Box<dyn std::error::Error>> {
            let app = clawless::output::OutputFlags::augment_command(commands::clawless_init());
            let matches = app.get_matches();
            let leaf = commands::clawless_resolve(matches);

            match leaf {
                clawless::resolved_leaf::ResolvedLeaf::Command { matches, exec } => {
                    clawless::runner::CommandRunner::run(matches, exec)
                }
                clawless::resolved_leaf::ResolvedLeaf::Application { matches, exec } => {
                    clawless::tui::runner::ApplicationRunner::run(matches, exec)
                }
            }
        }
    };
    output.into()
}

/// Add a command to a Clawless application
///
/// This macro attribute can be used to register a function as a (sub)command in
/// a Clawless application. The name of the function will be used as the name of
/// the command, and it will be automatically registered as a subcommand under
/// its parent module.
///
/// Command functions must accept exactly two parameters:
/// 1. An `args` parameter: a `clap::Args` struct with the command's arguments
/// 2. A `context` parameter: the `Context` providing access to the application environment
///    and the cancellation token for cooperative shutdown
///
/// # Attributes
///
/// - `alias = "name"` - Add a visible alias for the command. Can be repeated for multiple aliases.
/// - `require_subcommand` - Require a subcommand; show help if the command is invoked without one.
///
/// # Requiring Subcommands
///
/// Use `require_subcommand` to create a command that serves as a container for subcommands. When
/// this attribute is set, invoking the command without a subcommand will display help instead of
/// running the command body. This is useful for organizing related commands under a common prefix.
///
/// For example, a CLI might have `db migrate`, `db seed`, and `db reset` commands, where `db`
/// itself requires a subcommand and doesn't perform any action on its own.
///
/// # Examples
///
/// Basic command:
///
/// ```rust,ignore
/// use clawless::prelude::*;
///
/// #[derive(Debug, Args)]
/// pub struct GreetArgs {
///     #[arg(short, long)]
///     name: String,
/// }
///
/// #[command]
/// pub async fn greet(args: GreetArgs, context: Context) -> CommandResult {
///     message!("Hello, {}!", args.name);
///     Ok(())
/// }
/// ```
///
/// Command with alias:
///
/// ```rust,ignore
/// use clawless::prelude::*;
///
/// #[derive(Debug, Args)]
/// pub struct GenerateArgs {}
///
/// // Users can run `mycli generate` or `mycli g`
/// #[command(alias = "g")]
/// pub async fn generate(args: GenerateArgs, context: Context) -> CommandResult {
///     Ok(())
/// }
/// ```
///
/// Command that requires a subcommand:
///
/// ```rust,ignore
/// use clawless::prelude::*;
///
/// #[derive(Debug, Args)]
/// pub struct DbArgs {}
///
/// // Running `mycli db` shows help; users must specify a subcommand like `mycli db migrate`
/// #[command(require_subcommand, alias = "d")]
/// pub async fn db(args: DbArgs, context: Context) -> CommandResult {
///     Ok(())
/// }
/// ```
#[proc_macro_attribute]
pub fn command(attrs: TokenStream, input: TokenStream) -> TokenStream {
    let input_function = parse_macro_input!(input as ItemFn);

    let command_generator = match CommandGenerator::new(attrs.into(), input_function.clone()) {
        Ok(generator) => generator,
        Err(e) => return e.into_compile_error().into(),
    };
    let inventory_generator = InventoryGenerator::new(&command_generator);

    let inventory_struct_for_subcommands = inventory_generator.inventory();
    let submit_command_to_inventory = inventory_generator.submit();

    let initialization_function_for_command = command_generator.initialization_function();
    let resolve_function_for_command = command_generator.resolve_function();

    let output = quote! {
        #inventory_struct_for_subcommands

        #input_function

        #initialization_function_for_command

        #resolve_function_for_command

        #submit_command_to_inventory
    };

    output.into()
}

/// Add a TUI application to a Clawless project
///
/// This macro attribute registers a function as a TUI application in a Clawless project. Unlike
/// `#[command]`, which creates a stateless CLI command rendered through a push-based presenter,
/// `#[application]` creates a stateful TUI application that queries a pull-based projection.
///
/// Application functions must accept exactly three parameters:
/// 1. An `args` parameter: a `clap::Args` struct with the application's arguments
/// 2. A `context` parameter: the [`Context`] for emitting events and cooperative shutdown
/// 3. A `projection` parameter: the [`Projection`] for querying accumulated state
///
/// # Attributes
///
/// - `alias = "name"` - Add a visible alias for the application. Can be repeated.
/// - `require_subcommand` - Require a subcommand; show help if invoked without one.
///
/// # Examples
///
/// ```rust,ignore
/// use clawless::prelude::*;
/// use clawless::tui::projection::Projection;
///
/// #[derive(Debug, Args)]
/// pub struct DashboardArgs {
///     #[arg(short, long, default_value = "3000")]
///     port: u16,
/// }
///
/// /// Interactive project dashboard
/// #[application]
/// pub async fn dashboard(
///     args: DashboardArgs,
///     context: Context,
///     projection: Projection,
/// ) -> CommandResult {
///     Ok(())
/// }
/// ```
///
/// [`Context`]: clawless::context::Context
/// [`Projection`]: clawless::tui::projection::Projection
#[proc_macro_attribute]
pub fn application(attrs: TokenStream, input: TokenStream) -> TokenStream {
    let input_function = parse_macro_input!(input as ItemFn);

    let application_generator =
        match ApplicationGenerator::new(attrs.into(), input_function.clone()) {
            Ok(generator) => generator,
            Err(e) => return e.into_compile_error().into(),
        };
    let inventory_generator = InventoryGenerator::new(&application_generator);

    let inventory_struct_for_subcommands = inventory_generator.inventory();
    let submit_application_to_inventory = inventory_generator.submit();

    let initialization_function = application_generator.initialization_function();
    let resolve_function = application_generator.resolve_function();

    let output = quote! {
        #inventory_struct_for_subcommands

        #input_function

        #initialization_function

        #resolve_function

        #submit_application_to_inventory
    };

    output.into()
}
