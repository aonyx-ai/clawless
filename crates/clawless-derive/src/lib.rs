#![cfg_attr(not(doctest),doc = include_str!("../README.md"))]
#![warn(missing_docs)]

use proc_macro::TokenStream;
use quote::quote;
use syn::{ItemFn, parse_macro_input};

use crate::command::CommandGenerator;
use crate::inventory::InventoryGenerator;

mod command;
mod inventory;

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
        async fn clawless(_args: ClawlessEntryPoint, _context: clawless::context::Context) -> clawless::CommandResult {
            Ok(())
        }
    };
    output.into()
}

/// Initialize and run a Clawless application
///
/// This macro generates the `main` function for a Clawless application.
/// It should be called in `src/main.rs` after declaring the `commands` module.
///
/// # Example
///
/// ```rust,ignore
/// // src/main.rs
/// mod commands;
///
/// clawless::main!();
/// ```
#[proc_macro]
pub fn main(_input: TokenStream) -> TokenStream {
    let output = quote! {
        fn main() -> Result<(), Box<dyn std::error::Error>> {
            let context = clawless::context::Context::try_new()?;

            let rt = clawless::tokio::runtime::Runtime::new()?;
            rt.block_on(async {
                let app = commands::clawless_init();
                commands::clawless_exec(app.get_matches(), context.clone()).await
            })?;

            Ok(())
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
///     println!("Hello, {}!", args.name);
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

    let command_generator = CommandGenerator::new(attrs.into(), input_function.clone());
    let inventory_generator = InventoryGenerator::new(&command_generator);

    let inventory_struct_for_subcommands = inventory_generator.inventory();
    let submit_command_to_inventory = inventory_generator.submit_command();

    let initialization_function_for_command = command_generator.initialization_function();
    let wrapper_function_for_command = command_generator.wrapper_function();

    let output = quote! {
        #inventory_struct_for_subcommands

        #input_function

        #initialization_function_for_command

        #wrapper_function_for_command

        #submit_command_to_inventory
    };

    output.into()
}
