use anyhow::Result;
use typed_fields::path;

path!(
    /// The working directory in which a command was called
    ///
    /// The current working directory is the directory from which Clawless commands are run. It is
    /// passed to commands via the `Context` struct, allowing commands to operate relative to this
    /// directory.
    ///
    /// Please note that the path is not updated if commands manually change into another directory.
    CurrentWorkingDirectory
);

impl CurrentWorkingDirectory {
    /// Create a `CurrentWorkingDirectory` from the environment
    ///
    /// This function retrieves the current working directory from the environment and creates a
    /// `CurrentWorkingDirectory` instance. If the current directory cannot be determined, an error
    /// is returned.
    pub fn try_from_env() -> Result<CurrentWorkingDirectory> {
        let cwd = std::env::current_dir()?;
        Ok(CurrentWorkingDirectory::new(cwd))
    }
}
