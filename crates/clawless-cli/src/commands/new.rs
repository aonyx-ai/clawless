use std::fs::{create_dir_all, write};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use clawless::prelude::*;
use indoc::indoc;
use typed_fields::name;

name!(CrateName);

/// Arguments for the `new` command
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Args)]
pub struct NewArgs {
    /// Name of the new Clawless project to create
    name: CrateName,
}

/// Create a new Clawless project with a complete setup
///
/// This command creates a new binary crate using `cargo new`, adds the Clawless
/// framework as a dependency, and sets up the project structure with example code
/// to get you started quickly.
///
/// The generated project includes:
/// - A `main.rs` file with the Clawless entry point
/// - A `commands.rs` module for organizing commands
/// - A sample `greet` command demonstrating the framework
///
/// # Examples
///
/// ```shell
/// clawless new my-app
/// ```
#[command(alias = "n")]
pub async fn new(args: NewArgs, context: Context) -> CommandResult {
    // Call `cargo new` to create a new binary crate
    let crate_path = create_binary_crate(&context, &args.name)?;

    // Call `cargo add` to add Clawless as a dependency to the new crate
    add_clawless_dependency(&crate_path)?;

    // Update the main.rs file to use clawless
    overwrite_main_rs(&crate_path)?;

    // Create src/commands.rs and initialize clawless
    create_commands_rs(&crate_path)?;

    // Create src/commands/greet.rs with the greeting example
    create_greeting_command(&crate_path)?;

    Ok(())
}

fn create_binary_crate(context: &Context, crate_name: &CrateName) -> Result<PathBuf, Error> {
    let mut cargo_new_exec = Command::new("cargo");

    // Add the arguments to create a new binary crate
    cargo_new_exec
        .current_dir(context.current_working_directory().get())
        .arg("new")
        .arg("--bin")
        .arg(crate_name.get());

    // TODO: If the command fails, we should capture the output and return it as part of the error
    cargo_new_exec.stdout(Stdio::null()).stderr(Stdio::null());

    // Run `cargo new` and check that it succeeded
    if !cargo_new_exec
        .status()
        .context("failed to run `cargo new`")?
        .success()
    {
        anyhow::bail!("failed to create new crate with `cargo new`");
    }

    let crate_path = context
        .current_working_directory()
        .get()
        .join(crate_name.get());

    Ok(crate_path)
}

fn add_clawless_dependency(crate_path: &Path) -> Result<(), Error> {
    let mut cargo_add_exec = Command::new("cargo");

    // Add the arguments to add a new dependency
    cargo_add_exec
        .current_dir(crate_path)
        .arg("add")
        .arg("clawless");

    // TODO: If the command fails, we should capture the output and return it as part of the error
    cargo_add_exec.stdout(Stdio::null()).stderr(Stdio::null());

    // Run `cargo add` and check that it succeeded
    if !cargo_add_exec
        .status()
        .context("failed to run `cargo add`")?
        .success()
    {
        anyhow::bail!("failed to add clawless as a dependency with `cargo add`");
    }

    Ok(())
}

fn overwrite_main_rs(crate_path: &Path) -> Result<(), Error> {
    let main_rs_path = crate_path.join("src").join("main.rs");

    let main_rs_contents = indoc! {r#"
        mod commands;

        // Initialize and start the Clawless application
        //
        // This macro sets up the Clawless runtime, parses the command-line arguments, and then
        // calls the appropriate command function based on the user's input.
        clawless::main!();
    "#};

    write(&main_rs_path, main_rs_contents).context("failed to overwrite main.rs")?;

    Ok(())
}

fn create_commands_rs(crate_path: &Path) -> Result<(), Error> {
    let commands_rs_path = crate_path.join("src").join("commands.rs");

    let commands_rs_contents = indoc! {r#"
        mod greet;

        // Collect the commands of the application
        //
        // This macro collects all the command functions defined in this module and its sub-modules,
        // and registers them with the Clawless runtime so they can be invoked from the command line.
        clawless::commands!();
    "#};

    write(&commands_rs_path, commands_rs_contents).context("failed to create commands.rs")?;

    Ok(())
}

fn create_greeting_command(crate_path: &Path) -> Result<(), Error> {
    let commands_dir_path = crate_path.join("src").join("commands");
    create_dir_all(&commands_dir_path).context("failed to create directory for commands module")?;

    let greet_rs_path = commands_dir_path.join("greet.rs");

    let greet_rs_contents = indoc! {r#"
        use clawless::prelude::*;

        #[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Args)]
        pub struct GreetArgs {
            /// Name of the person to greet
            #[arg(default_value = "World")]
            name: String,
        }

        #[command]
        pub async fn greet(args: GreetArgs) -> CommandResult {
            // Print the greeting to the console
            println!("Hello, {}!", args.name);

            // Exit the CLI successfully
            Ok(())
        }
    "#};

    write(&greet_rs_path, greet_rs_contents).context("failed to create greet.rs")?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use std::fs::{create_dir_all, read_to_string};

    use tempfile::TempDir;

    use super::*;

    #[test]
    fn create_binary_crate_creates_directory() {
        let cwd = TempDir::new().unwrap();

        let context = Context::builder()
            .current_working_directory(cwd.path())
            .build();

        let crate_name = CrateName::new("my_crate");

        let crate_path = create_binary_crate(&context, &crate_name).unwrap();

        assert!(crate_path.exists());
    }

    #[test]
    fn create_binary_crate_fails_if_crate_already_exists() {
        let cwd = TempDir::new().unwrap();

        let context = Context::builder()
            .current_working_directory(cwd.path())
            .build();

        // Pre-create the directory to simulate an existing crate
        create_dir_all(cwd.path().join("crate-that-already-exists")).unwrap();

        let crate_name = CrateName::new("crate-that-already-exists");

        assert!(create_binary_crate(&context, &crate_name).is_err());
    }

    #[test]
    fn add_clawless_dependency_adds_dependency() {
        let cwd = TempDir::new().unwrap();

        let context = Context::builder()
            .current_working_directory(cwd.path())
            .build();

        let crate_path = create_binary_crate(&context, &CrateName::new("my_crate")).unwrap();

        add_clawless_dependency(&crate_path).unwrap();

        let cargo_toml_contents = read_to_string(crate_path.join("Cargo.toml")).unwrap();

        assert!(cargo_toml_contents.contains("clawless"));
    }

    #[test]
    fn overwrite_main_rs_updates_file() {
        let cwd = TempDir::new().unwrap();
        let main_rs_path = cwd.path().join("src").join("main.rs");

        // Create a dummy crate structure and main.rs file
        create_dir_all(cwd.path().join("src")).unwrap();
        write(&main_rs_path, "// dummy main.rs").unwrap();

        overwrite_main_rs(cwd.path()).unwrap();

        let main_rs_contents = read_to_string(main_rs_path).unwrap();

        assert!(main_rs_contents.contains("clawless::main!();"));
    }

    #[test]
    fn create_commands_rs_creates_file() {
        let cwd = TempDir::new().unwrap();
        let src_path = cwd.path().join("src");

        // Create the src directory
        create_dir_all(&src_path).unwrap();

        create_commands_rs(cwd.path()).unwrap();

        let commands_rs_contents = read_to_string(src_path.join("commands.rs")).unwrap();

        assert!(commands_rs_contents.contains("clawless::commands!();"));
    }

    #[test]
    fn create_greeting_command_creates_file() {
        let cwd = TempDir::new().unwrap();
        let commands_dir_path = cwd.path().join("src").join("commands");

        // Create the commands directory
        create_dir_all(&commands_dir_path).unwrap();

        create_greeting_command(cwd.path()).unwrap();

        let greet_rs_contents = read_to_string(commands_dir_path.join("greet.rs")).unwrap();

        assert!(greet_rs_contents.contains("pub async fn greet"));
    }
}
