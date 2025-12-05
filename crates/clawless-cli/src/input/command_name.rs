use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use convert_case::{Case, Casing};
use getset::Getters;
use typed_builder::TypedBuilder;

const COMMAND_SEPARATOR: &str = "/";

/// Represents a parsed command name with optional parent modules
///
/// Command names can be simple (e.g., "greet") or nested using slash notation
/// (e.g., "db/migrate"). The struct separates the command name from its parent
/// module hierarchy.
///
/// # Examples
///
/// ```
/// # use clawless_cli::input::CommandName;
/// // Simple command: "greet"
/// let simple = CommandName::builder()
///     .name("greet")
///     .build();
///
/// // Nested command: "db/migrate"
/// let nested = CommandName::builder()
///     .name("migrate")
///     .parent_modules(vec!["db".to_string()])
///     .build();
/// ```
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Getters, TypedBuilder)]
pub struct CommandName {
    /// The name of the command (without parent modules)
    #[builder(setter(into))]
    #[getset(get = "pub")]
    name: String,

    /// Parent module hierarchy, ordered from outermost to innermost
    #[builder(default, setter(into))]
    #[getset(get = "pub")]
    parent_modules: Vec<String>,
}

impl CommandName {
    /// Returns the filename for this command (e.g., "greet.rs")
    pub fn filename(&self) -> String {
        format!("{}.rs", self.name.to_case(Case::Snake))
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
            path = path.join(module.to_case(Case::Snake));
        }

        path.join(self.filename())
    }
}

impl TryFrom<&String> for CommandName {
    type Error = anyhow::Error;

    fn try_from(value: &String) -> Result<Self> {
        let lowercase_value = value.to_case(Case::Lower);
        let command_parts: Vec<&str> = lowercase_value.split(COMMAND_SEPARATOR).collect();

        let parent_modules = command_parts[..command_parts.len() - 1]
            .iter()
            .map(|part| part.to_string())
            .collect();

        let name = command_parts
            .last()
            .context("the command name must not be empty")?
            .to_string();

        Ok(Self {
            name,
            parent_modules,
        })
    }
}
