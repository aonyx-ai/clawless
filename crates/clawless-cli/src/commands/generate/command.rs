use std::fs::{create_dir_all, read_to_string, write};
use std::path::{Path, PathBuf};

use anyhow::{Result, anyhow};
use clawless::prelude::*;
use convert_case::{Case, Casing};
use indoc::indoc;

use crate::input::CommandName;

/// Arguments for the `generate command` command
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Default, Args)]
pub struct GenerateCommandArgs {
    /// Name of the command to generate.
    ///
    /// Use slash notation for nested commands (e.g., "db/migrate").
    name: String,
}

/// Generate a new command in a Clawless project
///
/// This command creates a new command file with boilerplate code and automatically
/// adds the necessary `mod` statement to the parent module. It supports nested
/// command hierarchies using slash notation.
///
/// The command must be run from within a Clawless project directory (or any of
/// its subdirectories). The project is identified by the presence of a `main.rs`
/// file containing the `clawless::main!` macro.
///
/// # Examples
///
/// Generate a simple command:
/// ```shell
/// clawless generate command deploy
/// ```
///
/// Generate a nested command:
/// ```shell
/// clawless generate command db/migrate
/// ```
#[command(alias = "c")]
pub async fn command(args: GenerateCommandArgs, context: Context) -> CommandResult {
    // Check is command is running inside a Clawless project
    let project = find_clawless_project(context.current_working_directory())?;

    // Parse command name to support nested commands (e.g. `clawless generate command generate/context`)
    let command_name = CommandName::try_from(&args.name)?;

    // Create `src/commands/<parent commands>/` directories if they do not exist
    create_parent_directory(&project, &command_name)?;

    // Create `src/commands/<parent commands>/<command>.rs` file with boilerplate code
    create_command_file(&project, &command_name)?;

    // Add mod <command> to parent module
    insert_mod_statement(&project, &command_name)?;

    // Print a success message to the user

    Ok(())
}

fn find_clawless_project(current_working_directory: &CurrentWorkingDirectory) -> Result<PathBuf> {
    let main_rs_path = find_main_rs(current_working_directory)
        .ok_or_else(|| anyhow!("failed to find a main.rs file in the current directory or any of its parent directories"))?;

    check_main_rs(&main_rs_path)?;

    let project_path = main_rs_path
        .parent() // src
        .and_then(Path::parent) // crate root
        .ok_or_else(|| anyhow!("failed to determine the project directory from main.rs path"))?
        .to_path_buf();

    Ok(project_path)
}

fn find_main_rs(current_working_directory: &CurrentWorkingDirectory) -> Option<PathBuf> {
    let mut dir = current_working_directory.get().to_path_buf();

    loop {
        // Check for src/main.rs first, then main.rs in the current directory
        let src_main_rs = dir.join("src").join("main.rs");
        if src_main_rs.exists() {
            return Some(src_main_rs);
        }

        let main_rs = dir.join("main.rs");
        if main_rs.exists() {
            return Some(main_rs);
        }

        // Move up to parent directory, or return None if at filesystem root
        dir = dir.parent()?.to_path_buf();
    }
}

fn check_main_rs(path: &Path) -> Result<()> {
    let content =
        read_to_string(path).context(format!("failed to read main.rs at {}", path.display()))?;

    if !content.contains("clawless::main!") {
        anyhow::bail!(
            "the main.rs file at '{}' does not contain the 'clawless::main!' macro, indicating that this is not a Clawless project",
            path.display()
        );
    }

    Ok(())
}

fn create_parent_directory(project: &Path, command_name: &CommandName) -> Result<()> {
    let mut dir_path = project.join("src").join("commands");

    for module in command_name.parent_modules() {
        dir_path = dir_path.join(module.to_case(Case::Snake));
    }

    create_dir_all(&dir_path).context(format!(
        "failed to create parent directories for command at {}",
        dir_path.display()
    ))?;

    Ok(())
}

fn create_command_file(project_path: &Path, command_name: &CommandName) -> Result<()> {
    let struct_prefix = command_name.name().to_case(Case::Pascal);

    let boilerplate = format!(
        indoc! {
            r#"use clawless::prelude::*;

            #[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Args)]
            pub struct {}Args {{
                // Define command arguments here
            }}

            #[command]
            pub async fn {}(args: {}Args, context: Context) -> CommandResult {{
                // Command implementation goes here
                Ok(())
            }}
            "#
        },
        struct_prefix,
        command_name.name(),
        struct_prefix
    );

    let command_file_path = command_name.path_from_project_root(project_path);

    write(&command_file_path, boilerplate).context(format!(
        "failed to create file for new command at {}",
        command_file_path.display()
    ))?;

    Ok(())
}

fn insert_mod_statement(project: &Path, command_name: &CommandName) -> Result<()> {
    // Find the parent module file where the mod statement should be inserted
    let parent = find_parent_module(project, command_name)?;

    let mod_statement = format!("mod {};", command_name.name());

    // Read current content and ensure we don't duplicate the mod statement
    let mut content = read_to_string(&parent)?;
    if content.contains(&mod_statement) {
        return Ok(());
    }

    // Read the file into memory so that we can iterate over it
    let mut lines: Vec<&str> = content.lines().collect();

    // Find insertion point: after last mod, or last use, or first non-doc-comment line, or at start
    let index =
        lines
            .iter()
            .enumerate()
            .rev()
            .find_map(|(index, line)| line.trim_start().starts_with("mod ").then_some(index))
            .or_else(|| {
                lines.iter().enumerate().rev().find_map(|(index, line)| {
                    line.trim_start().starts_with("use ").then_some(index)
                })
            })
            .or_else(|| {
                lines.iter().enumerate().find_map(|(index, line)| {
                    (!line.trim_start().starts_with("//!")).then_some(index)
                })
            })
            .unwrap_or(0);

    // Insert the mod statement after the found index
    lines.insert(index + 1, &mod_statement);
    content = lines.join("\n");

    // Insert a trailing newline
    content.push('\n');

    // Write the updated content back to the parent module file
    write(&parent, content)?;

    Ok(())
}

fn find_parent_module(project: &Path, command_name: &CommandName) -> Result<PathBuf> {
    let parent_modules = command_name.parent_modules();

    // The parent module is either "commands" (if no nesting) or the last element in parent_modules
    let commands_default = "commands".to_string();
    let parent_name = parent_modules.last().unwrap_or(&commands_default);

    // Determine the path to the parent module file (either as a file or mod.rs)
    let mut base = project.join("src").join("commands");

    // Navigate down to the parent's directory, excluding the parent itself
    if parent_modules.len() > 1 {
        for module in &parent_modules[..parent_modules.len() - 1] {
            base = base.join(module.to_case(Case::Snake));
        }
    } else if parent_modules.is_empty() {
        // Parent is src/commands itself
        base = project.join("src");
    }

    let candidate_file = base.join(format!("{}.rs", parent_name.to_case(Case::Snake)));
    let candidate_dir_mod = base.join(parent_name.to_case(Case::Snake)).join("mod.rs");

    if candidate_file.exists() {
        Ok(candidate_file)
    } else if candidate_dir_mod.exists() {
        Ok(candidate_dir_mod)
    } else {
        Err(anyhow!(
            "parent module `{}` does not exist under `src/commands`; refusing to create it",
            parent_name
        ))
    }
}

#[cfg(test)]
mod tests {
    use std::fs::{create_dir, create_dir_all, write};

    use tempfile::TempDir;

    use super::*;

    #[test]
    fn check_clawless_project_finds_main_in_src_directory() {
        let cwd = TempDir::new().unwrap();

        let src_directory = cwd.path().join("src");
        create_dir(&src_directory).unwrap();

        let main_rs_path = src_directory.join("main.rs");
        write(&main_rs_path, "clawless::main!();").unwrap();

        let check = find_clawless_project(&cwd.path().into()).unwrap();

        assert_eq!(cwd.path(), &check);
    }

    #[test]
    fn check_clawless_project_finds_main_in_directory() {
        let cwd = TempDir::new().unwrap();

        let src_directory = cwd.path().join("src");
        create_dir(&src_directory).unwrap();

        let main_rs_path = src_directory.join("main.rs");
        write(&main_rs_path, "clawless::main!();").unwrap();

        let check = find_clawless_project(&src_directory.as_path().into()).unwrap();

        assert_eq!(cwd.path(), &check);
    }

    #[test]
    fn check_clawless_project_finds_main_in_parent_directory() {
        let cwd = TempDir::new().unwrap();

        let src_directory = cwd.path().join("src");

        // Create a subdirectory in which to call `check_clawless_project`
        let sub_dir = src_directory.join("subdir");
        create_dir_all(&sub_dir).unwrap();

        let main_rs_path = src_directory.join("main.rs");
        write(&main_rs_path, "clawless::main!();").unwrap();

        let check = find_clawless_project(&sub_dir.as_path().into()).unwrap();

        assert_eq!(cwd.path(), &check);
    }

    #[test]
    fn check_clawless_project_fails_without_main_rs() {
        let cwd = TempDir::new().unwrap();

        let check = find_clawless_project(&cwd.path().into());

        assert!(check.is_err());
    }

    #[test]
    fn check_clawless_project_fails_without_clawless_macro() {
        let cwd = TempDir::new().unwrap();

        let main_rs_path = cwd.path().join("main.rs");
        write(&main_rs_path, "fn main() {}").unwrap();

        let check = find_clawless_project(&cwd.path().into());

        assert!(check.is_err());
    }

    #[test]
    fn create_command_file_writes_boilerplate() {
        let cwd = TempDir::new().unwrap();
        create_dir_all(cwd.path().join("src").join("commands").join("parent")).unwrap();

        let command_name = CommandName::builder()
            .name("command".to_string())
            .parent_modules(vec!["parent".into()])
            .build();

        create_command_file(cwd.path(), &command_name).unwrap();

        let command_file_path = command_name.path_from_project_root(cwd.path());
        let content = read_to_string(command_file_path).unwrap();

        assert!(content.contains("pub struct CommandArgs"));
        assert!(content.contains("pub async fn command(args: CommandArgs, context: Context)"));
    }

    #[test]
    fn find_parent_module_locates_file_module() {
        let cwd = TempDir::new().unwrap();
        create_dir_all(cwd.path().join("src").join("commands")).unwrap();

        let commands_rs_path = cwd.path().join("src").join("commands.rs");
        write(&commands_rs_path, "").unwrap();

        let command_name = CommandName::builder()
            .name("test".to_string())
            .parent_modules(vec![])
            .build();

        let parent_module_path = find_parent_module(cwd.path(), &command_name).unwrap();

        assert_eq!(parent_module_path, commands_rs_path);
    }

    #[test]
    fn find_parent_module_locates_mod_rs_module() {
        let cwd = TempDir::new().unwrap();
        create_dir_all(cwd.path().join("src").join("commands")).unwrap();

        let commands_mod_rs_path = cwd.path().join("src").join("commands").join("mod.rs");
        write(&commands_mod_rs_path, "").unwrap();

        let command_name = CommandName::builder()
            .name("test".to_string())
            .parent_modules(vec![])
            .build();

        let parent_module_path = find_parent_module(cwd.path(), &command_name).unwrap();

        assert_eq!(parent_module_path, commands_mod_rs_path);
    }

    #[test]
    fn insert_mod_statement_inserts_after_use_in_commands_rs() {
        let cwd = TempDir::new().unwrap();
        create_dir_all(cwd.path().join("src")).unwrap();

        let commands_rs_path = cwd.path().join("src").join("commands.rs");
        // file with use statements but no mod statements
        write(
            &commands_rs_path,
            "use crate::foo;\nuse crate::bar;\n\nfn helper() {}\n",
        )
        .unwrap();

        let command_name = CommandName::builder()
            .name("mycmd".to_string())
            .parent_modules(vec![])
            .build();

        insert_mod_statement(cwd.path(), &command_name).unwrap();

        let content = read_to_string(&commands_rs_path).unwrap();
        assert!(content.contains("mod mycmd;"));

        // Ensure the inserted `mod` appears after at last `use` and before the function
        let idx_first_use = content.find("use crate::bar;").unwrap();
        let idx_mod = content.find("mod mycmd;").unwrap();
        let idx_fn = content.find("fn helper()").unwrap();
        assert!(idx_first_use < idx_mod && idx_mod < idx_fn);
    }

    #[test]
    fn insert_mod_statement_does_not_duplicate_existing_mod() {
        let cwd = TempDir::new().unwrap();
        create_dir_all(cwd.path().join("src")).unwrap();

        let commands_rs_path = cwd.path().join("src").join("commands.rs");
        write(&commands_rs_path, "mod existing;\nmod mycmd;\n").unwrap();

        let command_name = CommandName::builder()
            .name("mycmd".to_string())
            .parent_modules(vec![])
            .build();

        // Should be a no-op and not produce a second `mod mycmd;`
        insert_mod_statement(cwd.path(), &command_name).unwrap();

        let content = read_to_string(&commands_rs_path).unwrap();
        // There should be exactly one occurrence
        let occurrences = content.matches("mod mycmd;").count();
        assert_eq!(occurrences, 1);
    }

    #[test]
    fn insert_mod_statement_inserts_after_existing_use_and_mod() {
        let cwd = TempDir::new().unwrap();
        let base = cwd.path().join("src").join("commands").join("parent");
        create_dir_all(&base).unwrap();

        let parent_mod_rs = base.join("mod.rs");
        write(
            &parent_mod_rs,
            "//! module docs\nuse crate::common;\n\nmod another_command;\n\n// implementation\n",
        )
        .unwrap();

        let command_name = CommandName::builder()
            .name("command".to_string())
            .parent_modules(vec!["parent".into()])
            .build();

        insert_mod_statement(cwd.path(), &command_name).unwrap();

        let content = read_to_string(&parent_mod_rs).unwrap();
        assert!(content.contains("mod command;"));

        // Ensure the mod was inserted before the implementation comment and after at least one use
        let idx_existing_mod = content.find("mod another_command;").unwrap();
        let idx_mod = content.find("mod command;").unwrap();
        let idx_impl = content.find("// implementation").unwrap();
        assert!(idx_existing_mod < idx_mod && idx_mod < idx_impl);
    }
}
