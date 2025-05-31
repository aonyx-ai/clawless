use std::fs::{read_dir, write};
use std::path::Path;

use convert_case::{Case, Casing};
use proc_macro2::TokenStream;
use quote::{format_ident, quote};

fn main() {
    let crate_root = std::env::var("CARGO_MANIFEST_DIR")
        .expect("failed to read environment variable CARGO_MANIFEST_DIR");
    let crate_root_path = Path::new(&crate_root);

    let commands_module = crate_root_path.join("src").join("commands");

    // Collect the commands by scanning the commands directory
    let commands = collect_commands(&commands_module);

    // Generate the enum
    let output = generate_commands_enum(&commands);

    // Write to a generated file in Cargo's output directory
    let out_dir = std::env::var("OUT_DIR").expect("failed to read environment variable OUT_DIR");
    let out_dir_path = Path::new(&out_dir).join("commands.rs");
    write(out_dir_path, output).unwrap();
}

fn collect_commands(dir: &Path) -> Vec<String> {
    let mut commands = Vec::new();

    for entry in read_dir(dir).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();

        if path.is_file() && path.extension().is_some_and(|ext| ext == "rs") {
            // Skip mod.rs
            if path.file_stem().unwrap() == "mod" {
                continue;
            }

            // Add the command name
            commands.push(path.file_stem().unwrap().to_string_lossy().to_string());
        }
    }

    commands
}

fn generate_commands_enum(commands: &[String]) -> String {
    let variants = commands.iter().map(|cmd| {
        let pascal_case = cmd.to_case(Case::Pascal);

        let module: TokenStream = format!("crate::commands::{cmd}").parse().unwrap();
        let variant = format_ident!("{}", pascal_case);
        let args = format_ident!("{}Args", pascal_case);

        quote! {
            #variant(#module::#args)
        }
    });

    let tokens = quote! {
        #[derive(Debug, clap::Subcommand)]
        pub enum Commands {
            #(#variants,)*
        }
    };

    tokens.to_string()
}
