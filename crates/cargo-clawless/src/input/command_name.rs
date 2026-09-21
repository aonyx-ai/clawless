use std::path::{Path, PathBuf};

use anyhow::{Result, bail};
use convert_case::{Case, Casing};
use getset::Getters;

/// Separator that divides a nested command name into its module path and leaf name
const COMMAND_SEPARATOR: &str = "/";

/// Represents a parsed command name with optional parent modules
///
/// Command names can be simple (e.g., "greet") or nested using slash notation
/// (e.g., "db/migrate"). The struct separates the command name from its parent
/// module hierarchy.
///
/// A user writes a command name as the command line shows it, with hyphens between the words.
/// The generator writes Rust, where the same name has underscores between the words. Each
/// segment is therefore stored in snake case, which is the form that the module declaration,
/// the file name, and the function signature all need.
///
/// [`CommandName::try_from`] is the only constructor. It rejects each name for which the
/// generator could not write code that compiles.
///
/// # Examples
///
/// ```rust,ignore
/// use crate::input::CommandName;
///
/// // Simple command: "greet"
/// let simple = CommandName::try_from(&"greet".to_string())?;
///
/// // Nested command: "db/migrate"
/// let nested = CommandName::try_from(&"db/migrate".to_string())?;
/// ```
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Getters)]
pub struct CommandName {
    /// The name of the command in snake case, without its parent modules
    #[getset(get = "pub")]
    name: String,

    /// Parent module hierarchy in snake case, ordered from outermost to innermost
    #[getset(get = "pub")]
    parent_modules: Vec<String>,
}

impl CommandName {
    /// Returns the filename for this command (e.g., "greet.rs")
    pub fn filename(&self) -> String {
        format!("{}.rs", self.name)
    }

    /// Constructs the full file path for this command from the project root
    ///
    /// # Examples
    ///
    /// For a command "db/migrate" in project "/my-app":
    /// - Returns `/my-app/src/commands/db/migrate.rs`
    pub fn path_from_project_root(&self, project_root: &Path) -> PathBuf {
        let mut path = project_root.join("src").join("commands");

        for module in &self.parent_modules {
            path = path.join(module);
        }

        path.join(self.filename())
    }
}

impl TryFrom<&String> for CommandName {
    type Error = anyhow::Error;

    fn try_from(value: &String) -> Result<Self> {
        let (parents, name) = match value.rsplit_once(COMMAND_SEPARATOR) {
            Some((parents, name)) => (parents.split(COMMAND_SEPARATOR).collect(), name),
            None => (Vec::new(), value.as_str()),
        };

        let name = parse_segment(name)?;

        let parent_modules = parents
            .into_iter()
            .map(parse_segment)
            .collect::<Result<Vec<String>>>()?;

        Ok(Self {
            name,
            parent_modules,
        })
    }
}

/// Returns one segment of a command name in the snake case form that Rust needs
///
/// The segment arrives in the spelling that the user typed, which can be kebab case, snake
/// case, or camel case. The conversion to snake case therefore accepts each of them.
///
/// # Errors
///
/// Returns an error if the segment does not become a name that a Rust function can carry and
/// that the `#[command]` macro accepts. The generator writes the segment into a module
/// declaration, a file name, and a function signature, and each of the three needs a name that
/// starts with a letter, that ends with a letter or a digit, and that has one hyphen between two
/// words.
fn parse_segment(segment: &str) -> Result<String> {
    let name = segment.to_case(Case::Snake);

    let Some(first) = name.chars().next() else {
        bail!("the command name must not be empty");
    };

    if !first.is_ascii_lowercase() {
        bail!("the command name `{segment}` must start with a letter");
    }

    if name.ends_with('_') {
        bail!("the command name `{segment}` must end with a letter or a digit");
    }

    if name.contains("__") {
        bail!("the command name `{segment}` must have one hyphen between two words");
    }

    if !name.chars().all(is_name_character) {
        bail!("the command name `{segment}` must have only letters, digits, and hyphens");
    }

    Ok(name)
}

/// Returns whether a character can appear in the snake case form of a command name
fn is_name_character(character: char) -> bool {
    character.is_ascii_lowercase() || character.is_ascii_digit() || character == '_'
}

#[cfg(test)]
mod tests {
    // An assertion in a test panics by design. A `# Panics` section on every test
    // would repeat that and give the reader no information.
    #![allow(clippy::missing_panics_doc)]

    use super::*;

    #[test]
    fn command_name_try_from_with_digit_at_start_returns_error() {
        let value = "2fa".to_string();

        let error = CommandName::try_from(&value).expect_err("should fail");

        assert_eq!(
            "the command name `2fa` must start with a letter",
            error.to_string()
        );
    }

    #[test]
    fn command_name_try_from_with_empty_name_returns_error() {
        let value = String::new();

        let error = CommandName::try_from(&value).expect_err("should fail");

        assert_eq!("the command name must not be empty", error.to_string());
    }

    #[test]
    fn command_name_try_from_with_empty_parent_module_returns_error() {
        let value = "db//migrate".to_string();

        let error = CommandName::try_from(&value).expect_err("should fail");

        assert_eq!("the command name must not be empty", error.to_string());
    }

    #[test]
    fn command_name_try_from_with_hyphenated_name_returns_snake_case_name() {
        let value = "deploy-staging".to_string();

        let command_name = CommandName::try_from(&value).expect("should succeed");

        assert_eq!("deploy_staging", command_name.name());
    }

    #[test]
    fn command_name_try_from_with_leading_underscore_returns_error() {
        let value = "_deploy".to_string();

        let error = CommandName::try_from(&value).expect_err("should fail");

        assert_eq!(
            "the command name `_deploy` must start with a letter",
            error.to_string()
        );
    }

    #[test]
    fn command_name_try_from_with_mixed_case_name_returns_snake_case_name() {
        let value = "DeployStaging".to_string();

        let command_name = CommandName::try_from(&value).expect("should succeed");

        assert_eq!("deploy_staging", command_name.name());
    }

    #[test]
    fn command_name_try_from_with_simple_name_returns_no_parent_modules() {
        let value = "greet".to_string();

        let command_name = CommandName::try_from(&value).expect("should succeed");

        assert!(command_name.parent_modules().is_empty());
    }

    #[test]
    fn command_name_try_from_with_snake_case_name_returns_snake_case_name() {
        let value = "deploy_staging".to_string();

        let command_name = CommandName::try_from(&value).expect("should succeed");

        assert_eq!("deploy_staging", command_name.name());
    }

    #[test]
    fn command_name_try_from_with_trailing_underscore_returns_error() {
        let value = "deploy_".to_string();

        let error = CommandName::try_from(&value).expect_err("should fail");

        assert_eq!(
            "the command name `deploy_` must end with a letter or a digit",
            error.to_string()
        );
    }

    #[test]
    fn command_name_try_from_with_two_underscores_returns_error() {
        let value = "deploy__staging".to_string();

        let error = CommandName::try_from(&value).expect_err("should fail");

        assert_eq!(
            "the command name `deploy__staging` must have one hyphen between two words",
            error.to_string()
        );
    }

    #[test]
    fn filename_with_hyphenated_name_returns_snake_case_file_name() {
        let value = "deploy-staging".to_string();

        let command_name = CommandName::try_from(&value).expect("should succeed");

        assert_eq!("deploy_staging.rs", command_name.filename());
    }

    #[test]
    fn path_from_project_root_with_nested_name_returns_nested_path() {
        let value = "user-management/create-user".to_string();

        let command_name = CommandName::try_from(&value).expect("should succeed");

        assert_eq!(
            Path::new("/my-app/src/commands/user_management/create_user.rs"),
            command_name.path_from_project_root(Path::new("/my-app"))
        );
    }

    #[test]
    fn trait_send() {
        fn assert_send<T: Send>() {}
        assert_send::<CommandName>();
    }

    #[test]
    fn trait_sync() {
        fn assert_sync<T: Sync>() {}
        assert_sync::<CommandName>();
    }

    #[test]
    fn trait_unpin() {
        fn assert_unpin<T: Unpin>() {}
        assert_unpin::<CommandName>();
    }
}
