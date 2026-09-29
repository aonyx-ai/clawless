---
sidebar_position: 8
---

# Exit Codes

A command can choose the exit code of the process and the last text that the
user reads.

## The Default Exit

Most commands return `CommandResult`. The exit code then follows from the
result:

- `Ok(())` ends the process with the exit code 0.
- An error ends the process with the exit code 1. Clawless writes `Error:` and
  the error to the standard error.

See [Commands][commands] for how to return an error.

## Choosing the Exit

A script or a CI job reads the exit code of a program to decide what to do
next. A command that must tell the caller more than "it worked" or "it failed"
returns `CommandResult<Exit>`:

```rust
use clawless::prelude::*;

/// Check that a version is a stable release
#[command]
pub async fn check(args: CheckArgs, context: Context) -> CommandResult<Exit> {
    message!("Checking version {}.", args.version);

    if args.version.starts_with("0.") {
        let text = format!("error: {} is not a stable release\n", args.version);

        return Ok(Exit::builder().code(ExitCode::from(3)).text(text).build());
    }

    Ok(Exit::from(()))
}
```

[`Exit`][exit] holds the exit code and an optional text. The text is optional,
so a command can end with a code after the messages that it sent.

`Exit::from(())` ends the process with the exit code 0 and no text, the same as
`Ok(())` in a command that returns `CommandResult`. A command that chooses only
the code returns `Exit::from(ExitCode::from(3))`, or returns
`CommandResult<ExitCode>` instead.

An error still ends the process with the exit code 1, so `?` works the same way
as in a command that returns `CommandResult`. A command that needs another code
for a failure turns the error into an `Exit` itself, and returns the `Exit` in
`Ok`:

```rust
let config = match read_config(&args.path) {
    Ok(config) => config,
    Err(error) => {
        let text = format!("error: {error}\n");

        return Ok(Exit::builder().code(ExitCode::from(4)).text(text).build());
    }
};
```

An exit code other than 0 is not always a failure. In the `check` command, the
version `0.7.0` is a valid input, and the exit code 3 is the answer of the
check. The command therefore returns the `Exit` in `Ok`.

## How Clawless Writes the Text

Clawless does not render the text. The command renders the text, for example
as a diagnostic, and Clawless writes it:

- Clawless writes the text once, exactly as it is. It adds no prefix and no
  line break, so end the text with a line break.
- Clawless writes the text to the standard error in every output mode. With
  `--json`, the standard output therefore keeps only JSON.
- The verbosity does not apply to the text, so `--quiet` does not hide it.
- Clawless writes the text after all the output that the command sent. The
  process exits after the text.

Run the example above with the version `0.7.0`:

```text
$ myapp check 0.7.0
Checking version 0.7.0.
error: 0.7.0 is not a stable release
$ echo $?
3
```

## Exit Codes That Clawless Uses

Clawless uses these exit codes itself. Clawless reserves none of them, so a
command can use them too. If a command uses one of them, the caller cannot tell
which one of the two ended the process.

| Code | Meaning                                                                   |
| ---- | ------------------------------------------------------------------------- |
| 0    | The command returned `Ok(())`                                             |
| 1    | The command returned an error                                             |
| 2    | The command line has an error, which clap reports                         |
| 101  | The command or Clawless panicked, for example on a closed standard output |
| 130  | The user pressed Ctrl+C a second time, see [Cancellation][ctl]            |

## Applications

An `#[application]` returns `CommandResult` or `CommandResult<Exit>` in the same
way as a command. Clawless writes the text after the application has returned.
Restore the terminal before the application returns, so that the user can read
the text.

## What's Next

- **[Commands][commands]** - Errors and the `?` operator
- **[Output][output]** - Messages, details, and artifacts
- **[Cancellation][ctl]** - What happens when the user presses Ctrl+C

The [`exit-code` example][example] has a command that ends with the exit code
0, 1, or 3, and a test for each one.

[commands]: ./commands
[ctl]: ./cancellation
[example]: https://github.com/aonyx-ai/clawless/tree/main/examples/exit-code
[exit]: https://docs.rs/clawless/latest/clawless/exit/struct.Exit.html
[output]: ./output
