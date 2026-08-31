# Run programs

- **Project**: [P004-external-programs][project]
- **Dependencies**: [F011-output-events][output-events]
- **Breaking changes**: `Event` and `Entry` gain a variant, so an exhaustive
  match on either needs a new arm

## Summary

Add [`Process`][process-spec], the interface a command uses to run an external
program. A run starts the program, sends every line it writes into the [event
channel][event-channel] as the line arrives, kills the program when the command
is cancelled, and returns the exit status together with the whole output.

The mechanics of starting a program and reading its streams come from the
`kawauso-process` crate. This feature is the layer that connects that crate to
the event system and to the cancellation token of a command.

## Motivation

See the [project motivation][project]. In short, a command that drives another
program should get live output, no deadlock on a full pipe, cooperative
shutdown, and an error message that names the command — without writing any of
it itself.

## Domain concepts

### Invocation and execution

An `Invocation` is the description of a command: a program, its arguments, and
optionally a working directory. It is a value, so an application can build one,
write it to a log, and name it in an error without running anything.

An `Execution` is the result of one run: the exit status, what the program
wrote to each of its streams, and how long the run took. A status that is not a
success is data, not a failure — the check mode of a formatter exits non-zero
when it finds a file to format, and that is the answer the caller asked for.
`Execution::require_success` turns a status the caller cannot accept into an
error.

Both types come from `kawauso-process` and are re-exported, so an application
needs Clawless alone.

### Process as the wiring

`Process` holds an `Output` and a `Cancellation`. `Context::process()` builds
one from the context of a command, which is how a command reaches the
interface:

```rust
let execution = context
    .process()
    .run(Invocation::new("cargo").arg("build"))
    .await?
    .require_success()?;
```

The handle is separate from the context so that a background task, which has an
output and a cancellation token but no context, can run a program the same way.

### The events of a run

A run produces one `ProcessEvent::Started`, one `ProcessEvent::Line` per line
of output, and one `ProcessEvent::Finished`. A consumer therefore knows what is
running, what it has said so far, and when it stopped.

Every event carries a `RunId`. A command can run two programs at once, and their
lines then interleave in a single channel; the identity is what separates them
again. A `RunId` counts runs, which is why it is not the `ProcessId` the
operating system assigns: that value names a program only while it runs, and the
operating system reuses it afterwards. `Started` carries both, so that an
operator can find the program in a process list.

`Started` and `Finished` also carry the `Invocation`, so that a presenter which
keeps no state can name the program it reports on.

`Finished` carries an `Outcome`, which says which of the three ends it was: the
program exited with a status, cancellation stopped it, or the run produced no
result. Modelling the end as an outcome rather than a status is what makes the
guarantee "every run that reports its start also reports its end" possible, and
that guarantee is what lets a TUI remove a running program from its view.

### Cancellation ends the program

Both the read of the output and the wait for the end race the cancellation
token. A program that writes without end stops when the user asks for it, and
so does a program that closed its streams and kept running.

A cancelled program is asked to end and killed only if it does not answer within
the grace period, so a build tool gets the moment it needs to remove its lock
file. The grace period bounds the whole ending, so a program that ignores the
request costs that period and a program that answers costs what it takes to
answer.

This holds on both paths. Waiting for the end of a program leaves the handle
where it is, so a program that closed both of its streams and kept running is
asked to end like any other.

The cancellation branch is biased first, which means a token that is already
cancelled ends the run instead of reporting one more line.

### Rendering

The output of a program is supplementary: the presenter shows it at verbose
verbosity, in the same way it shows a detail. A command that wants a program to
be visible at the default verbosity says so itself with a message.

In text mode the presenter keeps the two streams apart, writing what the
program wrote to its standard error to the standard error of the application.
Redirecting one of the two streams therefore gives the same split as running
the program by hand. In JSON mode everything goes to standard error, because
standard output carries artifacts alone.

## Functional requirements

1. `Process::run` starts a program, streams its output as events, and returns
   an `Execution`.
2. The `Execution` carries the whole output of the program, whether or not a
   consumer read the events.
3. Cancellation ends the program and returns an error that names the command,
   while the program writes and after it stops writing. The program is asked to
   end before it is killed, and the grace period bounds the whole ending.
4. A run that reports its start also reports its end, on every path that can
   still reach the consumer and that the caller did not abandon.
5. `Context::process()` returns a `Process` wired to the output and the
   cancellation token of the context.
6. `TerminalPresenter` renders process events at verbose verbosity only, and
   keeps the two streams of the program apart in text mode.
7. `Projection` stores process events as entries and offers a filtered query
   for them.
8. `just pre-commit` passes.

## Non-functional requirements

1. **No deadlock on a full pipe**: both streams are read while the run waits.
2. **Cheap projection queries**: a process entry is behind an `Arc`, because a
   run produces one entry per line and a render frame clones what it reads.
3. **No shell**: nothing splits an argument at a space or expands a `*`, so an
   argument that holds a space stays one argument.
4. **Null standard input**: a program that asks for a password ends instead of
   waiting for an answer that no one will type.

## API surface

```rust
// clawless_core::process
pub struct Process { /* output, cancellation, grace_period */ }

impl Process {
    pub fn builder() -> ProcessBuilder;
    pub async fn run(&self, invocation: Invocation) -> Result<Execution, RunProcessError>;
}

pub enum RunProcessError {
    CancelledRun { invocation: Invocation },
    UnrunnableCommand { source: RunCommandError },
    UnreportableOutput { invocation: Invocation, source: SendError },
}

// clawless_core::event::process
pub enum ProcessEvent {
    Started { id: RunId, invocation: Invocation, process_id: Option<ProcessId> },
    Line { id: RunId, line: Line },
    Finished { id: RunId, invocation: Invocation, outcome: Outcome, duration: Duration },
}

pub enum Outcome {
    Cancelled,
    Exited(ExitStatus),
    Incomplete,
}

pub struct RunId(/* u64 */);

// clawless_core::context
impl Context {
    pub fn process(&self) -> Process;
}

// clawless_core::output
impl Output {
    pub async fn process_event(&self, event: ProcessEvent) -> Result<(), SendError>;
}

// clawless_tui::projection
impl Projection {
    pub fn processes(&self) -> Vec<Entry>;
}
```

## File changes

### New files

| File                                                | Purpose                    |
| --------------------------------------------------- | -------------------------- |
| `crates/clawless-core/src/process/mod.rs`           | `Process` and its run loop |
| `crates/clawless-core/src/process/error.rs`         | `RunProcessError`          |
| `crates/clawless-core/src/event/process/mod.rs`     | `ProcessEvent`             |
| `crates/clawless-core/src/event/process/run_id.rs`  | `RunId`                    |
| `crates/clawless-core/src/event/process/outcome.rs` | `Outcome`                  |
| `examples/process/`                                 | Example and its tests      |

### Modified files

| File                                            | Change                         |
| ----------------------------------------------- | ------------------------------ |
| `crates/clawless-core/src/event/mod.rs`         | `Event::Process` variant       |
| `crates/clawless-core/src/output.rs`            | `Output::process_event`        |
| `crates/clawless-core/src/context/mod.rs`       | `Context::process`             |
| `crates/clawless-cli/src/presenter/terminal.rs` | Render process events          |
| `crates/clawless-tui/src/projection/entry.rs`   | `Entry::Process` variant       |
| `crates/clawless-tui/src/projection/mod.rs`     | `Projection::processes`        |
| `crates/clawless/src/lib.rs`                    | Re-export the `process` module |

## Edge cases

| Case                                          | Expected behavior                                         |
| --------------------------------------------- | --------------------------------------------------------- |
| Program does not exist                        | `UnrunnableCommand`, no start event, no run               |
| Program exits non-zero                        | `Ok(Execution)`; `require_success` turns it into an error |
| Program writes more than a pipe holds         | Both streams are read while waiting, so the run finishes  |
| Program closes its streams and keeps running  | The wait races cancellation, so Ctrl+C still ends the run |
| Program writes a last line without a newline  | That line is reported                                     |
| Program writes bytes that are not valid UTF-8 | The line shows `U+FFFD`; the capture keeps the bytes      |
| Token already cancelled before the run        | Started, then Finished with `Cancelled`, then an error    |
| Consumer of the events is gone                | `UnreportableOutput`; the program is killed with the run  |
| Two programs run at once                      | Their lines interleave and the `ProcessId` separates them |

## Out of scope

See the [project][project]. Environment control, standard input, pipelines, and
default-verbosity rendering are all left for later.

## Open questions

None. All design decisions for this feature have been resolved.

[event-channel]: 007-event-channel.md
[output-events]: 011-output-events.md
[process-spec]: ../process.md
[project]: ../projects/004-external-programs.md
