use std::fmt;
use std::path::PathBuf;

use clawless::prelude::*;

/// Arguments for the `run` command
///
/// Takes the program to run and the arguments to pass to it. No shell reads the command, so an
/// argument that holds a space stays one argument and nothing expands a character such as `*`.
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Args)]
pub struct RunArgs {
    /// Program to run
    program: PathBuf,

    /// Arguments to pass to the program
    #[arg(allow_hyphen_values = true, trailing_var_arg = true)]
    arguments: Vec<String>,
}

/// How much the program wrote to each of its streams
///
/// In text mode this displays as a sentence. In JSON mode it serializes as an object, which is
/// how a command reports a measurement that another tool reads.
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Serialize)]
struct Captured {
    /// Bytes that the program wrote to its standard output
    stdout: usize,

    /// Bytes that the program wrote to its standard error
    stderr: usize,
}

impl fmt::Display for Captured {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} bytes on standard output, {} bytes on standard error",
            self.stdout, self.stderr
        )
    }
}

/// Run another program and report what it produced
///
/// Clawless sends every line that the program writes into the event system as the line arrives,
/// so `--verbose` shows a long-running program while it runs instead of after it ended. The same
/// output also reaches the command in the result, which is what this command measures.
///
/// Press Ctrl+C while the program runs to see cooperative shutdown kill it.
///
/// ```shell
/// process --verbose run echo hello
/// ```
#[command]
// A command's doc comment is its `--help` text, so an `# Errors` section would render as a
// raw Markdown heading in the terminal rather than documenting an API.
#[allow(clippy::missing_errors_doc)]
pub async fn run(args: RunArgs, context: Context) -> CommandResult {
    let RunArgs { program, arguments } = args;

    let invocation = Invocation::new(program).args(arguments);

    message!("running {invocation}");

    let execution = context
        .process()
        .run(invocation)
        .await
        .context("run the program")?
        .require_success()
        .context("check what the program reported")?;

    artifact!(Captured {
        stdout: execution.stdout().get().len(),
        stderr: execution.stderr().get().len(),
    });

    Ok(())
}
