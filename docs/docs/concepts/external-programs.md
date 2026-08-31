---
sidebar_position: 6
---

# External Programs

Most command-line applications drive other programs. Clawless gives your
commands a first-class way to run them: the output of the program streams
through the event system while it runs, cancellation kills it, and the result
carries everything the program wrote.

## What is a process?

A process is one run of an external program. You describe the run with an
[`Invocation`][invocation] and hand it to the [`Process`][process] interface
that [`Context`][context] provides:

```rust
use clawless::prelude::*;

/// Build the project
#[command]
pub async fn build(args: BuildArgs, context: Context) -> CommandResult {
    let execution = context
        .process()
        .run(Invocation::new("cargo").arg("build"))
        .await
        .context("build the project")?
        .require_success()
        .context("check what cargo reported")?;

    message!("cargo wrote {} bytes", execution.stdout().get().len());

    Ok(())
}
```

No shell reads the command. Nothing splits an argument at a space, removes a
quotation mark, or expands a character such as `*`, so an argument that holds a
space stays one argument. If you want a shell, name the shell as the program.

## Live output

Clawless sends every line the program writes into the event system as the line
arrives. Your command writes no code for this — running the program is enough:

```console
$ mycli --verbose build
$ cargo build
   Compiling mycli v0.1.0
    Finished `dev` profile in 3.21s
cargo build exited with code 0
```

Every event carries the `RunId` of its run, which is what separates two programs
your command runs at the same time, and the start of a run also carries the
`ProcessId` the operating system gave the program.

Because the lines are events, the [presenter][rendering] decides what to do
with them. A stateless CLI prints them as they arrive. A TUI keeps them in a
[projection][rendering] and shows the last few in a corner of the screen:

```rust
let tail: Vec<_> = projection
    .processes()
    .into_iter()
    .rev()
    .take(5)
    .collect();
```

The output of a program is supplementary, so it appears at `--verbose` and not
at the default verbosity. Your command decides what the user sees by default:
say `message!("building")` and the user gets one line instead of a wall of
compiler output.

In text mode, what the program wrote to its standard error goes to the standard
error of your application. Redirecting one of the two streams therefore gives
the same split that running the program by hand would give.

## Reading the result

The same output that streamed as events also reaches your command in the
[`Execution`][execution]:

```rust
let execution = context.process().run(invocation).await?;

let text = execution.stdout().to_string_lossy();
let took = execution.duration();
let code = execution.status().code();
let pid = execution.id();
```

You do not have to choose between showing the output and using it.

## Exit status is data

A program that exits with a non-zero code is not a failure of the run. The
check mode of a formatter exits non-zero when it finds a file to format, and
that is the answer you asked for. The status travels in the `Execution`, and
you decide what it means:

```rust
let execution = context.process().run(invocation).await?;

if execution.status().success() {
    message!("everything is formatted");
} else {
    message!("some files need formatting");
}
```

When your command cannot accept a failure, `require_success` turns the status
into an error that names the command, the status, and what the program wrote to
its standard error:

```console
Error: check what the program reported

Caused by:
    the command `false` ended with exit status: 1, and it wrote nothing to its standard error
```

## Cancellation ends the program

A run observes the [cancellation][cancellation] token of your command. When the
user presses Ctrl+C, Clawless ends the program and the run returns
`RunProcessError::CancelledRun`. You do not leave a build running behind you,
and you write no code to achieve that.

The program is asked to end first and killed only if it does not answer, so a
build tool gets the moment it needs to remove its lock file. That moment is the
grace period, five seconds by default. Build your own handle when a program needs
longer:

```rust
let process = Process::builder()
    .output(context.output().clone())
    .cancellation(context.cancellation().clone())
    .grace_period(Duration::from_secs(30))
    .build();
```

The grace period is a ceiling, not a cost. A program that answers the request
costs only what answering takes, so the period is paid in full only by a program
that ignores it.

Cancellation works both while the program is writing output and after it has
stopped writing, so a program that closes its streams and keeps running is asked
to end like any other.

## Errors

`Process::run` fails in three ways, and each is a variant of its own so you can
treat them differently:

| Variant              | Meaning                                               |
| -------------------- | ----------------------------------------------------- |
| `UnrunnableCommand`  | The program never started, or the run had no result   |
| `CancelledRun`       | Cancellation stopped the program                      |
| `UnreportableOutput` | The presenter stopped listening while the program ran |

A command that treats cancellation as an orderly end matches `CancelledRun` and
returns `Ok(())`.

## Why it works this way

**Pipes never fill.** A program that writes more than a pipe holds stops until
a reader empties it. Clawless reads both streams while it waits, so a talkative
program cannot deadlock your command.

**Output arrives on time.** A command that collects output and prints it
afterwards shows nothing for two minutes and then everything at once. Streaming
the lines as events means the user sees progress.

**The command does not know the renderer.** The same command code streams to a
terminal, into a TUI panel, or into a test harness. That is the same property
`message!` and `artifact!` have, extended to the programs your command runs.

**Standard input is null.** A program that asks for a password sees the end of
its input at once and ends, instead of hanging while nobody types an answer.

## What's next

- **[Rendering](./rendering)** - How events reach the terminal or a TUI
- **[Cancellation](./cancellation)** - The token that stops a run
- **[Output](./output)** - The other kinds of output a command produces

[cancellation]: ./cancellation
[context]: ./context
[execution]: https://docs.rs/clawless/latest/clawless/process/struct.Execution.html
[invocation]: https://docs.rs/clawless/latest/clawless/process/struct.Invocation.html
[process]: https://docs.rs/clawless/latest/clawless/process/struct.Process.html
[rendering]: ./rendering
