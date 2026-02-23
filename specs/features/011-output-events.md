# Output events

- **Project**: [P003-presenter][project]
- **Dependencies**: [F010-presenter-macros][presenter-macros]
- **Breaking changes**: none (internal behavior change, public API unchanged)

## Summary

Switch [Output]'s `message()`, `detail()`, and `artifact()` methods from
writing directly to stdout/stderr to sending [events][event-types] through the
[event channel][event-channel]. This closes the event-driven loop: commands use
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
command → message!() → Output::message() → EventSender → channel → EventReceiver → TerminalPresenter → stdout
```

The public API is unchanged: `message!`, `detail!`, `artifact!` continue to
work exactly as before. The `--quiet`, `--verbose`, and `--json` flags produce
identical behavior. The only difference is internal: Output sends events instead
of writing directly, and the Presenter renders them.

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

## Functional requirements

1. `Output::message()` sends `Event::Message` through the `EventSender`
   unconditionally.
2. `Output::detail()` sends `Event::Detail` through the `EventSender`
   unconditionally.
3. `Output::artifact()` boxes the value as `Box<dyn Artifact>` and sends
   `Event::Artifact` through the `EventSender`.
4. Output does not apply verbosity filtering. All events are emitted.
5. When Output has no `EventSender` (tests, default construction), methods
   fall back to direct writes via the existing `Writer` mechanism.
6. The public API of Output is unchanged: `message()`, `detail()`, and
   `artifact()` have the same signatures and behavior.
7. All existing tests pass.
8. All examples produce identical output.
9. `--quiet`, `--verbose`, and `--json` flags work as before.
10. `just pre-commit` passes.

## Non-functional requirements

1. **Backwards compatibility**: the `message!`, `detail!`, `artifact!` macros
   continue to work without changes. Command authors see no difference.
2. **No output reordering**: events are sent in the order they are produced.
   The channel preserves FIFO order. The Presenter renders in order.
3. **Graceful draining**: when the command completes, Output is dropped, which
   drops the EventSender, which closes the channel. The Presenter's render task
   drains any remaining events before `present` returns.

## API surface

No new public API. This feature modifies the internal implementation of
`Output::message()`, `Output::detail()`, and `Output::artifact()`.

The `Output::new()` constructor continues to work as before (for tests and
backwards compatibility). `Output::new_with_sender()` (introduced in F010) is
the production path.

## File changes

### Modified files

| File                            | Change                                                 |
| ------------------------------- | ------------------------------------------------------ |
| `crates/clawless/src/output.rs` | Modify methods to send events when sender is available |

## Edge cases

| Case                                         | Expected behavior                                                        |
| -------------------------------------------- | ------------------------------------------------------------------------ |
| Output without sender (test construction)    | Falls back to direct Writer writes, same as before                       |
| Output with sender, receiver already dropped | `send()` error is ignored (fire-and-forget semantics)                    |
| Artifact serialization failure               | Does not occur at emission time; happens at render time in the Presenter |
| Rapid event emission filling channel buffer  | `send()` blocks until space available (back-pressure)                    |
| Empty message string                         | Event is emitted with empty payload; presenter renders empty line        |
| Multiple artifact calls                      | Each emits a separate event; all rendered in order                       |

## Out of scope

- Removing the `Writer` abstraction (kept for test compatibility)
- Changing the public API of Output
- Progress or diagnostic events
- Structured logging or tracing integration

## Open questions

### Blocking vs. fire-and-forget sends

Should `Output::message()` block (await) when the channel is full, or should
it use `try_send` and drop events that cannot be delivered?

**Recommendation**: use blocking sends. Output methods are called from async
command functions, but they are currently synchronous (`fn`, not `async fn`).
To send events, Output needs to use `tokio::mpsc::Sender::blocking_send` (from
a sync context within an async runtime). This preserves back-pressure and
ensures no events are lost. If this proves too restrictive, the methods can be
made async in a future release.

[architecture]: ../architecture.md
[event-channel]: 007-event-channel.md
[event-types]: 006-event-types.md
[output]: ../../crates/clawless/src/output.rs
[presenter-macros]: 010-presenter-macros.md
[presenter-rendering]: 009-presenter-rendering.md
[project]: ../projects/003-presenter.md
