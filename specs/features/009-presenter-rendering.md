# Presenter rendering

- **Project**: [P003-presenter][project]
- **Dependencies**: [F008-presenter][presenter]
- **Breaking changes**: none (modifies TerminalPresenter internals)

## Summary

Extend `TerminalPresenter` to consume events from the event channel and render
them to the terminal. The presenter spawns a render task that reads events from
the `EventReceiver` and writes to stdout/stderr, applying verbosity filtering
and mode-based routing. This is where all rendering logic lives — the Presenter
owns the full presentation decision.

## Motivation

[F008][presenter] established the Presenter trait and `TerminalPresenter` as a
pass-through execution wrapper. With the event types ([F006][event-types]) and
channel ([F007][event-channel]) in place, the presenter can now consume events
and render them. This feature moves the rendering responsibility from Output to
the Presenter, which is where the [architecture] says it belongs.

After this feature, `TerminalPresenter` is a complete stateless presenter: it
receives events and renders them in real time. The remaining features
([F010][presenter-macros], [F011][output-events]) wire this into the macro-
generated main function and modify Output to emit events.

## Domain concepts

### Render task

The presenter spawns a Tokio task that runs concurrently with the command. This
task reads events from the `EventReceiver` in a loop and writes them to
stdout/stderr based on the presenter's `Verbosity` and `OutputMode`
configuration.

The render task runs until the channel closes (all senders dropped), which
happens when the command completes and its `Output` (holding the `EventSender`)
is dropped. The presenter awaits both the command future and the render task,
returning the command's result.

### Verbosity filtering

The Presenter applies verbosity filtering when rendering events. Output emits
all events unconditionally — it is the command's voice, not the rendering
engine. The event variant encodes the semantic level, and the Presenter decides
what to render:

| Event      | Quiet   | Default | Verbose |
| ---------- | ------- | ------- | ------- |
| `Message`  | no-op   | renders | renders |
| `Detail`   | no-op   | no-op   | renders |
| `Artifact` | renders | renders | renders |

This matches Output's current behavior but moves the decision to the Presenter,
where it belongs.

### Rendering rules

The `OutputMode` determines where and how each event is rendered:

| Event      | Text mode            | JSON mode                     |
| ---------- | -------------------- | ----------------------------- |
| `Message`  | stdout via `Display` | stderr via `Display`          |
| `Detail`   | stdout via `Display` | stderr via `Display`          |
| `Artifact` | stdout via `Display` | stdout via `Serialize` (JSON) |

In text mode, everything goes to stdout. In JSON mode, messages redirect to
stderr so that stdout is reserved for machine-readable data — the same
convention used by `gh`, `kubectl`, and `jq`. Artifacts are rendered via
`Display` in text mode and via `Serialize` in JSON mode, using the trait object
carried on the event.

## Functional requirements

1. `TerminalPresenter::present` spawns a render task that consumes events from
   the `EventReceiver`.
2. The render task applies verbosity filtering: Message events are suppressed
   in Quiet mode, Detail events are only rendered in Verbose mode, Artifact
   events are always rendered.
3. The render task writes each event to the appropriate stream (stdout or
   stderr) based on `OutputMode`.
4. `Event::Message` renders the text payload followed by a newline.
5. `Event::Detail` renders the text payload followed by a newline.
6. `Event::Artifact` renders the `Artifact` trait object via `Display` (text
   mode) or `Serialize` (JSON mode) followed by a newline.
7. The render task terminates when the channel closes.
8. `present` awaits both the command future and the render task, returning the
   command's result.
9. If the render task completes before the command (because the channel was
   closed early), the command continues to run.
10. If the command completes before the render task, the render task drains any
    remaining events before `present` returns.

## Non-functional requirements

1. **No output reordering**: events are rendered in the order they are
   received from the channel. The channel preserves FIFO order.
2. **Graceful shutdown**: the render task drains all buffered events before
   terminating, ensuring no output is lost.

## API surface

No new public API. This feature modifies the internal behavior of
`TerminalPresenter::present`, which was introduced in [F008][presenter].

## File changes

### Modified files

| File                                        | Change                           |
| ------------------------------------------- | -------------------------------- |
| `crates/clawless/src/terminal_presenter.rs` | Add rendering logic to `present` |

## Edge cases

| Case                               | Expected behavior                                       |
| ---------------------------------- | ------------------------------------------------------- |
| No events emitted                  | Render task waits, channel closes, nothing rendered     |
| Artifact in JSON mode              | Serializes the `Artifact` trait object via `Serialize`  |
| Message in Quiet mode              | Suppressed by verbosity filtering; no output            |
| Detail in Default mode             | Suppressed by verbosity filtering; no output            |
| Rapid event emission               | Channel buffers events; render task catches up          |
| Command error after events emitted | Events already rendered are preserved; error propagated |
| Write failure (broken pipe)        | Matches `println!` behavior (process terminates)        |

## Out of scope

- Colored or styled output
- Spinners or progress bars
- Grouped output by task
- Output buffering strategies beyond the channel buffer
- Test-oriented rendering (assertions on rendered output)

## Open questions

None. All design decisions for this feature have been resolved.

[architecture]: ../architecture.md
[event-channel]: 007-event-channel.md
[event-types]: 006-event-types.md
[output]: ../../crates/clawless/src/output.rs
[output-events]: 011-output-events.md
[presenter]: 008-presenter.md
[presenter-macros]: 010-presenter-macros.md
[project]: ../projects/003-presenter.md
