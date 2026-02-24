# Output events

- **Project**: [P003-presenter][project]
- **Dependencies**: [F010-presenter-macros][presenter-macros]
- **Breaking changes**: Output methods become async; macros hide this from
  command authors

## Summary

Switch [Output]'s `message()`, `detail()`, and `artifact()` methods from
writing directly to stdout/stderr to sending [events][event-types] through the
[event channel][event-channel]. The methods become async to use the channel's
async `send`, and the output macros include `.await` in their expansion so
command authors see no change. This closes the event-driven loop: commands use
Output macros, Output emits events, events travel through the channel, and the
[TerminalPresenter] renders them to the terminal.

## Motivation

This is the final feature in the Presenter project. All infrastructure is in
place — event types (F006), event channel (F007), Presenter trait (F008),
TerminalPresenter rendering (F009), and macro wiring (F010). The only remaining
step is switching Output's internal implementation from direct writes to channel
sends.

After this feature, the data flow is fully event-driven:

```text
command → message!() → Output::message().await → EventSender → channel → EventReceiver → TerminalPresenter → stdout
```

The command-facing API is unchanged: `message!`, `detail!`, `artifact!`
continue to work exactly as before. The `--quiet`, `--verbose`, and `--json`
flags produce identical behavior. The only difference is internal: Output sends
events instead of writing directly, and the Presenter renders them.

## Domain concepts

### Output as event producer

Output's role shifts from "writer" to "event producer." Instead of holding
`Writer` targets (Stdout, Stderr, Buffer), Output holds an `EventSender` and
constructs `Event` values from its method arguments.

Output emits all events unconditionally — it does not filter based on
verbosity. The event variant encodes the semantic level (Message, Detail,
Artifact), and the [Presenter][presenter-rendering] decides what to render
based on its `Verbosity` setting. This keeps commands and Output free from
presentation concerns.

### Async methods

Output's `message()`, `detail()`, and `artifact()` methods become `async fn`.
This allows them to use the channel's async `send()` method, which provides
natural back-pressure without risking deadlocks.

Calling `tokio::mpsc::Sender::blocking_send` from within a Tokio runtime
thread can block the executor and deadlock under back-pressure if the receiver
needs the same runtime resources to make progress. Async `send().await` avoids
this by yielding the task when the channel is full, allowing the runtime to
continue driving the receiver.

The output macros (`message!`, `detail!`, `artifact!`) include `.await` in
their expansion, making the async nature transparent to command authors.
Command functions are already `async fn`, so the `.await` is always valid.

### Macro changes

The `message!` and `detail!` macros switch from `format_args!()` to
`format!()` in their expansion. `format_args!()` produces an
`std::fmt::Arguments<'_>` which borrows from temporary values and is not
`Send`. Since the future returned by an async method must be `Send` (it runs
on a multi-threaded Tokio runtime), the macro must produce a `String` instead.
`format!()` returns a `String` which is `Send` and `'static`.

```rust
// Before (sync):
//   message!("hello {}", name)
//   expands to: context.output().message(format_args!("hello {}", name))

// After (async):
//   message!("hello {}", name)
//   expands to: context.output().message(format!("hello {}", name)).await
```

The `artifact!` macro already takes an expression (not `format_args!`), so it
only needs the `.await` addition.

### Artifact trait objects

`Output::artifact<T: Display + Serialize>(&self, value: &T)` boxes the value
as a `Box<dyn Artifact>` and wraps it in `Event::Artifact`. The Presenter
receives the trait object and renders it via `Display` (text mode) or
`Serialize` (JSON mode). Serialization happens at render time, not emission
time.

### Test compatibility

Tests that use `Output::new_test()` (with buffer writers) must continue to
work. When Output has no `EventSender` (the test path), it falls back to
direct writes via the existing `Writer` mechanism. When Output has an
`EventSender` (the production path), it sends events through the channel.
Tests using Output methods directly will need to run in an async context
(`#[tokio::test]`).

## Functional requirements

1. `Output::message()` becomes `async fn` and sends `Event::Message` through
   the `EventSender` unconditionally.
2. `Output::detail()` becomes `async fn` and sends `Event::Detail` through
   the `EventSender` unconditionally.
3. `Output::artifact()` becomes `async fn`, boxes the value as
   `Box<dyn Artifact>`, and sends `Event::Artifact` through the `EventSender`.
4. Output does not apply verbosity filtering. All events are emitted.
5. When Output has no `EventSender` (tests, default construction), methods
   fall back to direct writes via the existing `Writer` mechanism.
6. The `message!` and `detail!` macros switch from `format_args!()` to
   `format!()` and append `.await`.
7. The `artifact!` macro appends `.await`.
8. All existing tests pass (updated to async where necessary).
9. All examples produce identical output.
10. `--quiet`, `--verbose`, and `--json` flags work as before.
11. `just pre-commit` passes.

## Non-functional requirements

1. **Transparent to command authors**: the macros hide the async nature.
   `message!("hello")` works exactly as before in async command functions.
2. **No output reordering**: events are sent in the order they are produced.
   The channel preserves FIFO order. The Presenter renders in order.
3. **No deadlock risk**: async `send().await` yields the task when the channel
   is full, allowing the runtime to drive the receiver.
4. **Graceful draining**: when the command completes, Output is dropped, which
   drops the EventSender, which closes the channel. The Presenter's render task
   drains any remaining events before `present` returns.

## API surface

### Output method changes

```rust
impl Output {
    /// Sends a message event
    pub async fn message(&self, message: impl Display + Send);

    /// Sends a detail event
    pub async fn detail(&self, message: impl Display + Send);

    /// Sends an artifact event
    pub async fn artifact<T: Display + Serialize + Debug + Send + Sync + 'static>(
        &self,
        value: &T,
    );
}
```

### Macro expansion changes

| Macro       | Before                                        | After                                          |
| ----------- | --------------------------------------------- | ---------------------------------------------- |
| `message!`  | `context.output().message(format_args!(...))` | `context.output().message(format!(...)).await` |
| `detail!`   | `context.output().detail(format_args!(...))`  | `context.output().detail(format!(...)).await`  |
| `artifact!` | `context.output().artifact(&(...))`           | `context.output().artifact(&(...)).await`      |

## File changes

### Modified files

| File                                | Change                                                                        |
| ----------------------------------- | ----------------------------------------------------------------------------- |
| `crates/clawless/src/output.rs`     | Make methods async; send events when sender is available                      |
| `crates/clawless-derive/src/lib.rs` | Update macros: `format!` + `.await` for message/detail; `.await` for artifact |

## Edge cases

| Case                                         | Expected behavior                                                        |
| -------------------------------------------- | ------------------------------------------------------------------------ |
| Output without sender (test construction)    | Falls back to direct Writer writes, same as before                       |
| Output with sender, receiver already dropped | `send()` returns error; ignored (fire-and-forget semantics)              |
| Artifact serialization failure               | Does not occur at emission time; happens at render time in the Presenter |
| Channel full                                 | `send().await` yields the task until space is available (back-pressure)  |
| Empty message string                         | Event is emitted with empty payload; presenter renders empty line        |
| Multiple artifact calls                      | Each emits a separate event; all rendered in order                       |

## Out of scope

- Removing the `Writer` abstraction (kept for test compatibility)
- Progress or diagnostic events
- Structured logging or tracing integration

## Open questions

None. All design decisions for this feature have been resolved.

[architecture]: ../architecture.md
[event-channel]: 007-event-channel.md
[event-types]: 006-event-types.md
[output]: ../../crates/clawless/src/output.rs
[presenter-macros]: 010-presenter-macros.md
[presenter-rendering]: 009-presenter-rendering.md
[project]: ../projects/003-presenter.md
