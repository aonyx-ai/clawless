# Event types

- **Project**: [P003-presenter][project]
- **Dependencies**: none
- **Breaking changes**: none (additive)

## Summary

Introduce the `Event` enum and `Artifact` trait — the structured messages that
commands produce and [Presenter] adapters consume. This feature defines the
three event variants that mirror [Output]'s current methods: Message, Detail,
and Artifact. The Artifact variant carries a trait object (`Box<dyn Artifact>`)
so that the Presenter receives the original value and can choose how to render
it — via `Display` for text mode or `Serialize` for JSON mode.

## Motivation

The [architecture] defines [events][event] as structured
messages emitted by tasks. Today, [Output] writes directly to stdout/stderr,
coupling production and rendering into a single step. To decouple them, we need
a concrete event type that can travel from the producer (Output) to the consumer
(Presenter) through an async channel.

This feature introduces the minimal event type — just the three variants that
Output already supports. Future features will extend `Event` with Progress,
Diagnostic, and lifecycle variants as the domain model evolves.

## Domain concepts

### Event (value object)

`Event` is a value object representing a single piece of output that a command
has produced. It carries enough information for a [Presenter] adapter to render
it without needing access to the original Output or command context.

Three variants, mirroring Output's current methods:

| Variant    | Payload             | Produced by          |
| ---------- | ------------------- | -------------------- |
| `Message`  | `String`            | `Output::message()`  |
| `Detail`   | `String`            | `Output::detail()`   |
| `Artifact` | `Box<dyn Artifact>` | `Output::artifact()` |

**Message** and **Detail** carry a `String` — the formatted text that the
command produced.

**Artifact** carries a trait object that implements both `Display` and
`Serialize`. The Presenter receives the actual value and chooses the rendering
strategy based on its `OutputMode`: `Display` for text, `Serialize` for JSON.
This avoids pre-rendering data that may never be used and keeps the rendering
decision entirely in the Presenter.

### Artifact (trait)

The `Artifact` trait combines `Display`, `erased_serde::Serialize`, `Debug`,
`Send`, and `Sync`. It enables artifact values to travel through the event
channel as trait objects while remaining renderable by the Presenter in both
text and JSON modes.

A blanket implementation covers any type that satisfies `Display + Serialize +
Debug + Send + Sync + 'static`, so command authors just derive the usual traits
and pass their values to `Output::artifact()` — no manual `Artifact` impl
needed.

`Serialize` is not object-safe in standard serde. The [`erased-serde`] crate
provides `erased_serde::Serialize`, an object-safe equivalent that enables
serialization through trait objects.

### Design decisions

**Trait object over pre-rendering**: the alternative is to capture both the
`Display` and `Serialize` representations at emission time and store them as
`(String, serde_json::Value)`. The trait object approach is preferred because
it avoids duplicating data and deferring serialization until the Presenter
actually needs it. The Presenter calls only the trait method it needs for its
output mode.

**Verbosity filtering in the Presenter**: Output emits all events regardless of
verbosity. The event variant encodes the semantic level — Message means
"informational," Detail means "supplementary," Artifact means "primary output."
The Presenter applies its `Verbosity` setting to decide which variants to
render. This keeps commands and Output free from presentation concerns.

**No `Clone` on `Event`**: trait objects are not `Clone` without a `clone_box`
pattern that adds complexity. Events are produced once and consumed once through
the channel, so `Clone` is not needed for the core data path. This decision may
be revisited if a concrete use case for cloning events emerges.

**No source identity yet**: the architecture defines events as carrying the
identity of the task that emitted them. Since the Task entity does not exist
yet, events carry no source identity. This will be added when the Task feature
is implemented.

## Functional requirements

1. `Event` is an enum with three variants: `Message`, `Detail`, `Artifact`.
2. `Event::Message` carries a `String` payload.
3. `Event::Detail` carries a `String` payload.
4. `Event::Artifact` carries a `Box<dyn Artifact>`.
5. `Artifact` is a trait combining `Display + erased_serde::Serialize + Debug + Send + Sync`.
6. A blanket impl covers any `T: Display + Serialize + Debug + Send + Sync + 'static`.
7. `Event` implements `Debug` (manually, delegating to the `Debug` bound on `Artifact` for the trait object variant).
8. `Event` does not implement `Clone`.
9. `Event` is `Send + Sync`.
10. Types are defined in `crates/clawless/src/event.rs`.
11. The module is re-exported from the crate root (as `pub mod event`).

## Non-functional requirements

1. **Thread safety**: `Event` must be `Send + Sync` because events travel
   through async channels between tasks.
2. **No framework coupling**: `Event` can be constructed and tested
   independently of Output, Presenter, or any other framework component.
3. **Zero-cost for command authors**: the blanket impl means command authors
   derive their usual traits (`Display`, `Serialize`, `Debug`, `Clone`) and
   everything works. No manual `Artifact` impl needed.

## API surface

### Event

```rust
/// Structured output event produced by commands
///
/// An `Event` represents a single piece of output that a command has produced.
/// Events travel from the producer ([`Output`]) through an async channel to the
/// consumer ([`Presenter`]), decoupling production from rendering.
///
/// Three variants mirror [`Output`]'s methods:
///
/// - [`Message`] — informational text (shown at default verbosity and above).
/// - [`Detail`] — supplementary text (shown only at verbose verbosity).
/// - [`Artifact`] — the primary data a command produces, carried as a trait
///   object that the Presenter can render via [`Display`] or [`Serialize`].
///
/// The Presenter decides which events to render based on its [`Verbosity`]
/// setting. Output emits all events unconditionally.
///
/// [`Message`]: Event::Message
/// [`Detail`]: Event::Detail
/// [`Artifact`]: Event::Artifact
/// [`Display`]: std::fmt::Display
/// [`Output`]: crate::output::Output
/// [`Presenter`]: crate::presenter::Presenter
/// [`Serialize`]: serde::Serialize
/// [`Verbosity`]: crate::output::Verbosity
pub enum Event {
    /// Informational message
    Message(String),
    /// Supplementary detail
    Detail(String),
    /// Primary command output
    Artifact(Box<dyn Artifact>),
}
```

### Artifact trait

```rust
/// Trait for artifact values that can be rendered as text or JSON
///
/// `Artifact` combines [`Display`] (for text rendering), [`Serialize`] (for
/// JSON rendering), and [`Debug`] (for diagnostics). The Presenter uses the
/// appropriate trait based on its output mode.
///
/// Command authors do not implement this trait directly. A blanket
/// implementation covers any type that satisfies the required bounds.
///
/// [`Display`]: std::fmt::Display
/// [`Serialize`]: serde::Serialize
pub trait Artifact: Display + erased_serde::Serialize + Debug + Send + Sync {}

impl<T> Artifact for T
where
    T: Display + Serialize + Debug + Send + Sync + 'static,
{
}
```

## Dependencies

### New workspace dependencies

```toml
# Cargo.toml [workspace.dependencies]
erased-serde = "0.4"
```

### Crate dependencies

```toml
# crates/clawless/Cargo.toml [dependencies]
erased-serde = { workspace = true }
```

## File changes

### New files

| File                           | Contents                       |
| ------------------------------ | ------------------------------ |
| `crates/clawless/src/event.rs` | `Event` enum, `Artifact` trait |

### Modified files

| File                         | Change                                       |
| ---------------------------- | -------------------------------------------- |
| `Cargo.toml`                 | Add `erased-serde` to workspace dependencies |
| `crates/clawless/Cargo.toml` | Add `erased-serde` dependency                |
| `crates/clawless/src/lib.rs` | Add `pub mod event;`                         |

## Edge cases

| Case                               | Expected behavior                                            |
| ---------------------------------- | ------------------------------------------------------------ |
| Empty message string               | Valid; `Event::Message(String::new())` is a legitimate event |
| Artifact serialization failure     | Occurs at render time in the Presenter, not at emission time |
| Debug formatting of Artifact event | Delegates to the `Debug` bound on the trait object           |

## Out of scope

- Event channel (see [F007][event-channel])
- Presenter consumption of events (see [F009][presenter-rendering])
- Output emitting events (see [F011][output-events])
- `Clone` on `Event` (may be revisited in a future session)
- Progress, Diagnostic, or lifecycle event variants
- Source identity (task ID) on events
- Event timestamps
- Prelude re-export (Event is infrastructure, not command-facing)

## Open questions

None. All design decisions for this feature have been resolved.

[`erased-serde`]: https://docs.rs/erased-serde
[architecture]: ../architecture.md
[event-channel]: 007-event-channel.md
[event]: ../architecture.md#event
[output]: ../../crates/clawless/src/output.rs
[output-events]: 011-output-events.md
[presenter]: ../architecture.md#presenter-output-port
[presenter-rendering]: 009-presenter-rendering.md
[project]: ../projects/003-presenter.md
