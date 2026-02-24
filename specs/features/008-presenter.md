# Presenter

- **Project**: [P003-presenter][project]
- **Dependencies**: [F007-event-channel][event-channel]
- **Breaking changes**: none (additive)

## Summary

Introduce the `Presenter` trait and `TerminalPresenter` — the port and adapter
that control how command output reaches the user. The `Presenter` trait defines
a single `present` method that wraps command execution. `TerminalPresenter` is
the first concrete adapter, implementing the simplest possible execution
wrapper. At this stage, `TerminalPresenter` executes the command but does not
yet consume events — that behavior is added in [F009][presenter-rendering].

## Motivation

The [architecture] defines the [Presenter] as the output port that consumes
events or queries the [Surface] to render output. Today, rendering is
implicit — [Output] writes directly to stdout/stderr. To evolve toward
swappable presenters (terminal, CI, ratatui, test), the framework needs a trait
that defines the presenter contract and at least one concrete implementation.

This feature establishes the execution wrapping pattern: the Presenter's
`present` method takes a command future, runs it, and returns the result. This
gives the Presenter control over the surrounding lifecycle — it can set up
rendering before the command runs and tear it down after. At this stage,
`TerminalPresenter` is a pass-through that executes the command directly. The
next feature ([F009][presenter-rendering]) adds event consumption.

## Domain concepts

### Presenter (port)

The `Presenter` trait is the output port in the hexagonal architecture. It
defines how the framework delegates rendering to an adapter. The trait has a
single method, `present`, which wraps command execution.

The `present` method accepts a future (the command) and an `EventReceiver`.
The Presenter runs the command, optionally consumes events from the receiver,
and returns the command's result. This signature gives the Presenter everything
it needs to control the output lifecycle.

### TerminalPresenter (adapter)

`TerminalPresenter` is the concrete adapter for terminal output. It is a
stateless presenter that will (in F009) render each event as it arrives,
writing to stdout/stderr. It holds `Verbosity` and `OutputMode` configuration
to determine how events are rendered.

At this stage (F008), `TerminalPresenter` executes the command and returns its
result without consuming events.

## Functional requirements

1. `Presenter` is an async trait with a single method: `present`.
2. `present` accepts a command future (`impl Future<Output = CommandResult>`)
   and an `EventReceiver`, and returns `CommandResult`.
3. `TerminalPresenter` implements `Presenter`.
4. `TerminalPresenter` is constructed with `Verbosity` and `OutputMode`.
5. At this stage, `TerminalPresenter::present` executes the command future and
   returns its result. Event consumption is added in F009.
6. `Presenter` trait is defined in `crates/clawless/src/presenter.rs`.
7. `TerminalPresenter` is defined in
   `crates/clawless/src/terminal_presenter.rs`.

## Non-functional requirements

1. **Object safety**: the `Presenter` trait should support dynamic dispatch
   for future per-command presenter selection.
2. **Thread safety**: `TerminalPresenter` must be `Send + Sync`.

## API surface

### Presenter trait

```rust
/// Output port for rendering command output
///
/// A `Presenter` wraps command execution and controls the output lifecycle.
/// The framework calls `present` with the command future and an event channel
/// receiver. The presenter runs the command and renders output from the event
/// stream.
pub trait Presenter: Send + Sync {
    /// Presents the output of a command
    ///
    /// Runs the given command future, optionally consuming events from the
    /// receiver, and returns the command's result.
    fn present(
        &self,
        command: Pin<Box<dyn Future<Output = CommandResult> + Send>>,
        receiver: EventReceiver,
    ) -> impl Future<Output = CommandResult> + Send;
}
```

### TerminalPresenter

```rust
/// Terminal presenter adapter
///
/// Renders command output to the terminal. In text mode, all output goes to
/// stdout. In JSON mode, messages go to stderr and artifacts are serialized
/// as JSON to stdout.
///
/// This is a stateless presenter: it processes each event as it arrives
/// without accumulating state.
#[derive(Clone, Debug)]
pub struct TerminalPresenter {
    verbosity: Verbosity,
    mode: OutputMode,
}

impl TerminalPresenter {
    /// Creates a new terminal presenter
    pub fn new(verbosity: Verbosity, mode: OutputMode) -> Self;
}

impl Presenter for TerminalPresenter {
    async fn present(
        &self,
        command: Pin<Box<dyn Future<Output = CommandResult> + Send>>,
        receiver: EventReceiver,
    ) -> CommandResult;
}
```

## File changes

### New files

| File                                        | Contents            |
| ------------------------------------------- | ------------------- |
| `crates/clawless/src/presenter.rs`          | `Presenter` trait   |
| `crates/clawless/src/terminal_presenter.rs` | `TerminalPresenter` |

### Modified files

| File                         | Change                                                     |
| ---------------------------- | ---------------------------------------------------------- |
| `crates/clawless/src/lib.rs` | Add `pub mod presenter;` and `pub mod terminal_presenter;` |

## Edge cases

| Case                                | Expected behavior                                               |
| ----------------------------------- | --------------------------------------------------------------- |
| Command returns `Ok(())`            | `present` returns `Ok(())`                                      |
| Command returns `Err`               | `present` returns the same error                                |
| Command panics                      | Panic propagates through the future                             |
| Events sent but not consumed (F008) | Events are buffered in the channel; dropped when receiver drops |

## Out of scope

- Event consumption (see [F009][presenter-rendering])
- Macro integration (see [F010][presenter-macros])
- Per-command presenter selection
- Stateful presenters (ratatui, test)
- Surface querying

## Open questions

None. All design decisions for this feature have been resolved.

[architecture]: ../architecture.md
[event-channel]: 007-event-channel.md
[presenter]: ../architecture.md#presenter-output-port
[presenter-macros]: 010-presenter-macros.md
[presenter-rendering]: 009-presenter-rendering.md
[project]: ../projects/003-presenter.md
[surface]: ../architecture.md#surface
