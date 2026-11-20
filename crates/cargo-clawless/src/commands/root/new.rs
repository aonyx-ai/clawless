//! Creates a new Clawless project

use std::fs::{create_dir_all, write};
use std::path::{Path, PathBuf};

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

/// Creates a new Clawless project with complete setup
///
/// This command creates a new binary crate, adds Clawless as a dependency,
/// and sets up the project structure with example code to start quickly.
///
/// The generated project includes:
/// - A `main.rs` file with the Clawless entry point
/// - A `commands.rs` module for organizing commands
/// - A sample `greet` command demonstrating the framework
///
/// # Examples
///
/// ```shell
/// cargo clawless new my-app
/// ```
#[command(alias = "n")]
#[allow(
    clippy::missing_errors_doc,
    reason = "a command's doc comment is its `--help` text, so an `# Errors` section would render as a raw Markdown heading in the terminal rather than documenting an API"
)]
pub async fn new(args: NewArgs, context: Context) -> CommandResult {
    let crate_path = create_binary_crate(&context, &args.name)
        .await
        .context("create binary crate")?;
    add_clawless_dependency(&context, &crate_path)
        .await
        .context("add Clawless dependency")?;
    overwrite_main_rs(&crate_path).context("overwrite main.rs")?;
    create_commands_rs(&crate_path).context("create commands.rs")?;
    create_greeting_command(&crate_path).context("create greeting command")?;

    Ok(())
}

/// Creates the binary crate
///
/// Cargo writes the manifest, not this tool.
/// The new project therefore always matches the current toolchain.
///
/// # Errors
///
/// Returns an error if `cargo new` cannot start, is cancelled, or fails.
async fn create_binary_crate(context: &Context, crate_name: &CrateName) -> Result<PathBuf, Error> {
    context
        .process()
        .run(
            Invocation::new("cargo")
                .arg("new")
                .arg("--bin")
                .arg(crate_name.get())
                .in_directory(context.current_working_directory().get()),
        )
        .await
        .context("run `cargo new`")?
        .require_success()
        .context("create the new crate with `cargo new`")?;

    let crate_path = context
        .current_working_directory()
        .get()
        .join(crate_name.get());

    Ok(crate_path)
}

/// Adds `clawless` to the new crate's dependencies
///
/// # Errors
///
/// Returns an error if `cargo add` cannot start, is cancelled, or fails.
async fn add_clawless_dependency(context: &Context, crate_path: &Path) -> Result<(), Error> {
    context
        .process()
        .run(
            Invocation::new("cargo")
                .arg("add")
                .arg("clawless")
                .in_directory(crate_path),
        )
        .await
        .context("run `cargo add`")?
        .require_success()
        .context("add clawless as a dependency with `cargo add`")?;

    Ok(())
}

/// Replaces the `main.rs` file from `cargo new` with a Clawless entry point
///
/// # Errors
///
/// Returns an error if the file cannot be written.
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

/// Writes the `commands` module that collects the commands of the project
///
/// # Errors
///
/// Returns an error if the file cannot be written.
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

/// Writes the first `greet` command, so that a new project runs immediately
///
/// # Errors
///
/// Returns an error if the commands directory or the command file cannot be created.
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
        pub async fn greet(args: GreetArgs, context: Context) -> CommandResult {
            message!("Hello, {}!", args.name);
            Ok(())
        }
    "#};

    write(&greet_rs_path, greet_rs_contents).context("failed to create greet.rs")?;

    Ok(())
}

#[cfg(test)]
mod tests {
    #![allow(
        clippy::missing_panics_doc,
        reason = "an assertion in a test panics by design. A `# Panics` section on every test would repeat that and give the reader no information"
    )]

    use std::fs::{create_dir_all, read_to_string};

    use clawless::event::event_channel;
    use clawless::output::Output;
    use tempfile::TempDir;

    use super::*;

    /// Returns an output whose events a task reads and discards
    ///
    /// A run reports every line that Cargo writes and fails once nobody listens.
    /// The reader discards these events since tests don't read them,
    /// so the reader stands in for the real presenter that keeps the channel open.
    fn test_output() -> Output {
        let (sender, mut receiver) = event_channel();

        tokio::spawn(async move { while receiver.recv().await.is_some() {} });

        Output::new(sender)
    }

    #[tokio::test]
    async fn add_clawless_dependency_adds_dependency() {
        let cwd = TempDir::new().unwrap();

        let context = Context::builder()
            .current_working_directory(cwd.path())
            .output(test_output())
            .build()
            .expect("failed to create context");

        let crate_path = create_binary_crate(&context, &CrateName::new("my_crate"))
            .await
            .unwrap();

        add_clawless_dependency(&context, &crate_path)
            .await
            .unwrap();

        let cargo_toml_contents = read_to_string(crate_path.join("Cargo.toml")).unwrap();

        assert!(cargo_toml_contents.contains("clawless"));
    }

    #[tokio::test]
    async fn create_binary_crate_creates_directory() {
        let cwd = TempDir::new().unwrap();

        let context = Context::builder()
            .current_working_directory(cwd.path())
            .output(test_output())
            .build()
            .expect("failed to create context");

        let crate_name = CrateName::new("my_crate");

        let crate_path = create_binary_crate(&context, &crate_name).await.unwrap();

        assert!(crate_path.exists());
    }

    #[tokio::test]
    async fn create_binary_crate_fails_if_crate_already_exists() {
        let cwd = TempDir::new().unwrap();

        let context = Context::builder()
            .current_working_directory(cwd.path())
            .output(test_output())
            .build()
            .expect("failed to create context");

        create_dir_all(cwd.path().join("crate-that-already-exists")).unwrap();

        let crate_name = CrateName::new("crate-that-already-exists");

        assert!(create_binary_crate(&context, &crate_name).await.is_err());
    }

    #[tokio::test]
    async fn create_binary_crate_reports_what_cargo_wrote() {
        let cwd = TempDir::new().unwrap();

        let context = Context::builder()
            .current_working_directory(cwd.path())
            .output(test_output())
            .build()
            .expect("failed to create context");

        create_dir_all(cwd.path().join("crate-that-already-exists")).unwrap();

        let crate_name = CrateName::new("crate-that-already-exists");

        let error = create_binary_crate(&context, &crate_name)
            .await
            .unwrap_err();

        assert!(format!("{error:#}").contains("already exists"));
    }

    #[test]
    fn create_commands_rs_creates_file() {
        let cwd = TempDir::new().unwrap();
        let src_path = cwd.path().join("src");

        create_dir_all(&src_path).unwrap();

        create_commands_rs(cwd.path()).unwrap();

        let commands_rs_contents = read_to_string(src_path.join("commands.rs")).unwrap();

        assert!(commands_rs_contents.contains("clawless::commands!();"));
    }

    #[test]
    fn create_greeting_command_creates_file() {
        let cwd = TempDir::new().unwrap();
        let commands_dir_path = cwd.path().join("src").join("commands");

        create_dir_all(&commands_dir_path).unwrap();

        create_greeting_command(cwd.path()).unwrap();

        let greet_rs_contents = read_to_string(commands_dir_path.join("greet.rs")).unwrap();

        assert!(
            greet_rs_contents.contains("pub async fn greet(args: GreetArgs, context: Context)")
        );
    }

    #[test]
    fn overwrite_main_rs_updates_file() {
        let cwd = TempDir::new().unwrap();
        let main_rs_path = cwd.path().join("src").join("main.rs");

        create_dir_all(cwd.path().join("src")).unwrap();
        write(&main_rs_path, "// dummy main.rs").unwrap();

        overwrite_main_rs(cwd.path()).unwrap();

        let main_rs_contents = read_to_string(main_rs_path).unwrap();

        assert!(main_rs_contents.contains("clawless::main!();"));
    }

    #[test]
    fn trait_send() {
        fn assert_send<T: Send>() {}
        assert_send::<NewArgs>();
    }

    #[test]
    fn trait_sync() {
        fn assert_sync<T: Sync>() {}
        assert_sync::<NewArgs>();
    }

    #[test]
    fn trait_unpin() {
        fn assert_unpin<T: Unpin>() {}
        assert_unpin::<NewArgs>();
    }
}
