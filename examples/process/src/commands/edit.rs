//! Opens a file in the editor of the user

use std::fmt;
use std::path::PathBuf;

use clawless::prelude::*;

/// The shell command that starts the editor of the user on the file in its first argument
///
/// `$EDITOR` can hold arguments of its own, such as `code --wait`, so the shell splits it into
/// words. The file reaches the editor as one argument, whatever characters its name holds.
const EDITOR: &str = "${EDITOR:-vi} \"$@\"";

/// Arguments for the `edit` command
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Args)]
pub struct EditArgs {
    /// File to open in the editor
    file: PathBuf,
}

/// How the editor ended
///
/// In text mode this displays as a sentence. In JSON mode it serializes as an object.
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Serialize)]
struct Closed {
    /// The exit code of the editor, which a signal that ended the editor leaves out
    code: Option<i32>,
}

impl fmt::Display for Closed {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.code {
            Some(code) => write!(f, "the editor exited with code {code}"),
            None => write!(f, "the editor ended without an exit code"),
        }
    }
}

/// Opens a file in the editor of the user
///
/// The editor is the program that `$EDITOR` names, or `vi` when it names none. It takes over the
/// terminal until it ends. The editor draws on the standard error, so the standard output carries
/// only the result of this command. The command fails at once without a terminal.
///
/// ```shell
/// process edit notes.md
/// ```
#[command]
#[allow(
    clippy::missing_errors_doc,
    reason = "the doc comment of a command is its help text, where an `# Errors` section would render as a raw heading"
)]
pub async fn edit(args: EditArgs, context: Context) -> CommandResult {
    let EditArgs { file } = args;

    message!("opening {}", file.display());

    let status = context
        .process()
        .run_interactive(
            Invocation::new("sh")
                .arg("-c")
                .arg(EDITOR)
                .arg("sh")
                .arg(file),
        )
        .await
        .context("run the editor")?;

    artifact!(Closed {
        code: status.code(),
    });

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn trait_send_closed() {
        fn assert_send<T: Send>() {}
        assert_send::<Closed>();
    }

    #[test]
    fn trait_send_edit_args() {
        fn assert_send<T: Send>() {}
        assert_send::<EditArgs>();
    }

    #[test]
    fn trait_sync_closed() {
        fn assert_sync<T: Sync>() {}
        assert_sync::<Closed>();
    }

    #[test]
    fn trait_sync_edit_args() {
        fn assert_sync<T: Sync>() {}
        assert_sync::<EditArgs>();
    }

    #[test]
    fn trait_unpin_closed() {
        fn assert_unpin<T: Unpin>() {}
        assert_unpin::<Closed>();
    }

    #[test]
    fn trait_unpin_edit_args() {
        fn assert_unpin<T: Unpin>() {}
        assert_unpin::<EditArgs>();
    }
}
