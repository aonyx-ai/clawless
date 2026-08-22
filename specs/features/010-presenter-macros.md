# Presenter macros

- **Project**: [P003-presenter][project]
- **Dependencies**: [F009-presenter-rendering][presenter-rendering]
- **Breaking changes**: none (modifies generated `main` function internals)

## Summary

Update the `main!()` macro to create a `TerminalPresenter`, create the event
channel, and wrap command execution with `presenter.present(...)`. The channel's
sender is stored in [Output] (via [Context]), and the receiver is passed to the
presenter. After this feature, the macro-generated main function establishes the
full event-driven pipeline, but Output still writes directly — the final
switchover happens in [F011][output-events].

## Motivation

[F008][presenter] and [F009][presenter-rendering] established the Presenter
trait and TerminalPresenter with event consumption. The presenter infrastructure
exists but is not wired into the application lifecycle. This feature connects
the pieces: the macro creates the channel, gives the sender to Output, gives
the receiver to the Presenter, and wraps the command execution.

After this feature, the infrastructure is in place for Output to emit events.
Output continues to write directly to stdout/stderr (the event sender is
available but not yet used), preserving existing behavior until F011 completes
the switchover.

## Domain concepts

### Macro-generated lifecycle

The `main!()` macro generates the application entry point. Currently it:

1. Creates a `Cancellation` token
2. Augments the command with output flags
3. Parses arguments
4. Creates `Output` from flags
5. Creates `Context` with cancellation and output
6. Spawns signal handling
7. Executes the command

After this feature, it additionally:

1. Creates an event channel (sender + receiver)
2. Stores the sender in `Output` (making it available via Context)
3. Creates a `TerminalPresenter` with the parsed verbosity and mode
4. Wraps command execution with `presenter.present(command, receiver)`

### Output with EventSender

Output gains an `EventSender` field. At this stage, Output continues to write
directly — it does not use the sender yet. The sender is stored so that F011
can switch Output's methods from direct writes to channel sends without
changing the macro or Context.

## Functional requirements

1. The `main!()` macro creates an event channel via `event_channel()`.
2. The macro passes the `EventSender` to `Output` (via a new constructor or
   builder method).
3. The macro creates a `TerminalPresenter` with the parsed `Verbosity` and
   `OutputMode`.
4. The macro wraps the command execution future with
   `presenter.present(command_future, receiver)`.
5. The macro-generated code compiles and runs correctly.
6. All existing examples produce identical output (Output still writes
   directly).
7. All existing tests pass.

## Non-functional requirements

1. **Backwards compatibility**: the generated main function must produce
   identical behavior to the current version. No observable output changes.
2. **Compile-time safety**: the generated code must compile without warnings.

## API surface

### Output changes

```rust
impl Output {
    /// Creates a new Output with an event sender
    ///
    /// The sender is stored for future use by the event-driven output path.
    /// Until that path is activated, Output continues to write directly.
    pub fn new_with_sender(
        verbosity: Verbosity,
        mode: OutputMode,
        sender: EventSender,
    ) -> Self;
}
```

### Macro expansion (conceptual)

```rust
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cancellation = clawless::cancellation::Cancellation::new();

    let app = clawless::output::Output::augment_command(commands::clawless_init());
    let matches = app.get_matches();

    let (sender, receiver) = clawless::event_channel::event_channel();
    let output = clawless::output::Output::from_arg_matches_with_sender(
        &matches, sender,
    );
    let presenter = clawless::terminal_presenter::TerminalPresenter::new(
        output.verbosity(), output.mode(),
    );

    let context = clawless::context::Context::builder()
        .cancellation(cancellation.clone())
        .output(output)
        .build()?;

    let rt = clawless::tokio::runtime::Runtime::new()?;
    rt.block_on(async {
        clawless::tokio::spawn(
            clawless::signal::wait_for_shutdown(cancellation)
        );

        presenter.present(
            Box::pin(commands::clawless_exec(matches, context)),
            receiver,
        ).await
    })?;

    Ok(())
}
```

## File changes

### Modified files

| File                                | Change                                                                                  |
| ----------------------------------- | --------------------------------------------------------------------------------------- |
| `crates/clawless-derive/src/lib.rs` | Update `main!()` to create channel, presenter, and wrap execution                       |
| `crates/clawless/src/output.rs`     | Add `new_with_sender` or `from_arg_matches_with_sender` method; add `EventSender` field |

## Edge cases

| Case                              | Expected behavior                                                       |
| --------------------------------- | ----------------------------------------------------------------------- |
| Output constructed without sender | Default constructor still works (for tests and backwards compatibility) |
| Sender stored but never used      | No effect; sender is dropped with Output when command completes         |
| Channel capacity reached          | Not possible at this stage since Output doesn't send events yet         |

## Out of scope

- Output using the sender to emit events (see [F011][output-events])
- Per-command presenter selection
- Presenter configuration beyond verbosity and mode
- Custom channel capacity

## Open questions

None. All design decisions for this feature have been resolved.

[output]: ../../crates/clawless/src/output.rs
[output-events]: 011-output-events.md
[presenter]: 008-presenter.md
[presenter-rendering]: 009-presenter-rendering.md
[project]: ../projects/003-presenter.md
