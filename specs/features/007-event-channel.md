# Event channel

- **Project**: [P003-presenter][project]
- **Dependencies**: [F006-event-types][event-types]
- **Breaking changes**: none (additive)

## Summary

Wrap `tokio::mpsc` with domain types — `EventSender` and `EventReceiver` — to
create a typed async channel for transporting [events][event-types] from
producers to consumers. This is the minimal event bus: a bounded, multi-producer
single-consumer channel that decouples [Output] from the [Presenter].

## Motivation

[F006][event-types] defines the `Event` type, but events need a transport
mechanism to travel from Output (the producer) to the Presenter (the consumer).
A raw `tokio::mpsc` channel would work, but wrapping it in domain types provides
type safety, encapsulates the channel capacity decision, and establishes a clear
vocabulary for the event bus pattern.

The [architecture] envisions an event stream that carries events from
tasks to the [Surface] and [Presenter] adapters. This feature builds the
simplest version: a single bounded `mpsc` channel. Future features may evolve
toward broadcast channels (multiple consumers) or more sophisticated routing,
but the domain types introduced here will remain the command-facing API.

## Domain concepts

### EventSender (value object)

A clonable handle for sending events into the channel. Output will hold an
`EventSender` and use it to emit events. Because `tokio::mpsc::Sender` is
already `Clone + Send + Sync`, `EventSender` inherits these properties.

### EventReceiver (value object)

A handle for receiving events from the channel. The Presenter will hold the
`EventReceiver` and consume events from it. `EventReceiver` is not `Clone` —
there is exactly one consumer of the channel.

### Channel creation

A factory function creates the paired sender and receiver with a fixed buffer
capacity. The capacity is an implementation detail that can be tuned later
without changing the public API.

## Functional requirements

1. `EventSender` wraps `tokio::mpsc::Sender<Event>` and provides a `send`
   method that accepts an `Event`.
2. `EventReceiver` wraps `tokio::mpsc::Receiver<Event>` and provides a `recv`
   method that returns `Option<Event>`.
3. `event_channel()` is a factory function that returns
   `(EventSender, EventReceiver)`.
4. The channel is bounded with a reasonable default capacity (e.g., 256).
5. `EventSender` is `Clone + Send + Sync`.
6. `EventReceiver` is `Send`.
7. When all senders are dropped, `recv()` returns `None` (channel closed).
8. Types are defined in `crates/clawless/src/event_channel.rs`.

## Non-functional requirements

1. **Thread safety**: `EventSender` must be `Clone + Send + Sync` because it
   will be stored in `Output`, which is in `Context`, which is `Clone + Send +
Sync`. `EventReceiver` must be `Send` because it will be moved to the
   Presenter's render task.
2. **Back-pressure**: the bounded channel provides natural back-pressure. If
   the consumer falls behind, the producer will wait. This is acceptable for
   the current use case where the single consumer (TerminalPresenter) renders
   events as fast as they arrive.

## API surface

### EventSender

```rust
/// Sender handle for the event channel
///
/// `EventSender` is a clonable handle that commands (via [`Output`]) use to
/// emit events into the channel. The paired [`EventReceiver`] consumes these
/// events for rendering.
///
/// [`Output`]: crate::output::Output
#[derive(Clone, Debug)]
pub struct EventSender { /* tokio::mpsc::Sender<Event> */ }

impl EventSender {
    /// Sends an event into the channel
    ///
    /// # Errors
    ///
    /// Returns an error if the receiver has been dropped.
    pub async fn send(&self, event: Event) -> Result<(), SendError>;
}
```

### EventReceiver

```rust
/// Receiver handle for the event channel
///
/// `EventReceiver` is the consuming end of the event channel. The [`Presenter`]
/// holds this handle and reads events for rendering.
///
/// [`Presenter`]: crate::presenter::Presenter
#[derive(Debug)]
pub struct EventReceiver { /* tokio::mpsc::Receiver<Event> */ }

impl EventReceiver {
    /// Receives the next event from the channel
    ///
    /// Returns `None` when all senders have been dropped and the channel is
    /// empty.
    pub async fn recv(&mut self) -> Option<Event>;
}
```

### Factory function

```rust
/// Creates a bounded event channel
///
/// Returns a paired sender and receiver. The sender is clonable; the receiver
/// is not. When all senders are dropped, the receiver's `recv` method returns
/// `None`.
pub fn event_channel() -> (EventSender, EventReceiver);
```

## File changes

### New files

| File                                   | Contents                                          |
| -------------------------------------- | ------------------------------------------------- |
| `crates/clawless/src/event_channel.rs` | `EventSender`, `EventReceiver`, `event_channel()` |

### Modified files

| File                         | Change                       |
| ---------------------------- | ---------------------------- |
| `crates/clawless/src/lib.rs` | Add `pub mod event_channel;` |

## Edge cases

| Case                                | Expected behavior                                       |
| ----------------------------------- | ------------------------------------------------------- |
| All senders dropped                 | `recv()` returns remaining buffered events, then `None` |
| Receiver dropped while senders live | `send()` returns an error                               |
| Channel at capacity                 | `send()` awaits until space is available                |
| Empty channel                       | `recv()` awaits until an event is available             |

## Out of scope

- Broadcast channels (multiple consumers)
- Unbounded channels
- Channel capacity configuration by users
- Event filtering at the channel level
- Prelude re-export (channel types are infrastructure, not command-facing)

## Open questions

None. All design decisions for this feature have been resolved.

[architecture]: ../architecture.md
[event-types]: 006-event-types.md
[output]: ../../crates/clawless/src/output.rs
[project]: ../projects/003-presenter.md
[surface]: ../architecture.md#surface
