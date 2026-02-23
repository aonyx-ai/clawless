# Presenter

## Summary

Event-driven output rendering for Clawless commands. This project introduces the
[execution event][execution-event] infrastructure, the [Presenter] port, and a
concrete [Terminal adapter][terminal-adapter] that replaces [Output]'s direct
stdout/stderr writes with event-driven rendering while preserving the existing
command-facing API (`message!`, `detail!`, `artifact!`).

## Motivation

The [architecture] defines a rich output model where commands emit [execution
events][execution-event] that flow through an event stream to [Presenter]
adapters for rendering. Today, [`Output`][output] writes directly to
stdout/stderr — a pragmatic first step that predates the event model. This works
for the simplest case but prevents the framework from evolving toward richer
rendering strategies (grouped task output, TUIs, test adapters) because the
rendering decision is baked into `Output` rather than delegated to a swappable
adapter.

This project lays the foundation for the full event-driven architecture by
building the minimum viable infrastructure through a single stateless presenter:

| Architecture concept                 | This project's contribution              | Evolution path                          |
| ------------------------------------ | ---------------------------------------- | --------------------------------------- |
| [Execution event][execution-event]   | `Event` enum (Message, Detail, Artifact) | Progress, Diagnostic, lifecycle events  |
| Event stream                         | `tokio::mpsc` channel                    | Broadcast, persistence, replay          |
| [Presenter] port                     | `Presenter` trait                        | Per-command presenter selection         |
| [Terminal adapter][terminal-adapter] | `TerminalPresenter`                      | Colors, layout, spinners                |
| [Surface]                            | Not yet — stateless rendering only       | Materialized view for stateful adapters |

After this project, the data path changes from:

```text
command → Output → stdout/stderr
```

to:

```text
command → Output → event channel → TerminalPresenter → stdout/stderr
```

Commands continue to use `message!`, `detail!`, and `artifact!` unchanged.
Examples produce identical output. The new data path establishes the event bus
pattern that future features (progress reporting, parallel task output,
alternative presenters) will build on.

## Feature specs

Each feature spec maps to one PR. They must be implemented in order.

| #    | Spec                                            | Depends on | Summary                                               |
| ---- | ----------------------------------------------- | ---------- | ----------------------------------------------------- |
| F006 | [F006-event-types][event-types]                 |            | `Event` enum with Message, Detail, Artifact variants  |
| F007 | [F007-event-channel][event-channel]             | F006       | Async `tokio::mpsc` channel with domain wrapper types |
| F008 | [F008-presenter][presenter]                     | F007       | `Presenter` trait and `TerminalPresenter` impl        |
| F009 | [F009-presenter-rendering][presenter-rendering] | F008       | TerminalPresenter consumes events and renders         |
| F010 | [F010-presenter-macros][presenter-macros]       | F009       | `main!()` macro creates presenter and event channel   |
| F011 | [F011-output-events][output-events]             | F010       | Output emits events instead of writing directly       |

F006 and F007 are pure additions with no downstream impact. F008 and F009
introduce new types and behavior but do not modify existing code. F010 modifies
the `main!()` macro to wire the presenter and channel into the application
lifecycle. F011 modifies `Output` internals to emit events instead of writing
directly, closing the loop.

## Out of scope

The following are intentionally excluded from this project. They may become
future projects once the presenter foundation is in place.

- **Surface**: the queryable projection of execution state is deferred until
  stateful adapters (ratatui, test) need it.
- **Progress events**: spinners, progress bars, and step indicators.
- **Diagnostic events**: rich structured error/warning information.
- **Lifecycle events**: task started, task completed, task failed.
- **Per-command presenter selection**: all commands use the same
  `TerminalPresenter` for now.
- **Prompt port**: bidirectional input through the surface.
- **Task entity**: parallel task output and grouped rendering.
- **Broadcast channel**: multiple consumers of the event stream.
- **Event replay or persistence**: recording events for post-hoc debugging.

## Open questions

None at the project level. Open questions are pushed to the individual feature
specs where they can be resolved with full context.

[architecture]: ../architecture.md
[event-channel]: ../features/007-event-channel.md
[event-types]: ../features/006-event-types.md
[execution-event]: ../architecture.md#execution-event
[output]: ../../crates/clawless/src/output.rs
[output-events]: ../features/011-output-events.md
[presenter]: ../features/008-presenter.md
[presenter-macros]: ../features/010-presenter-macros.md
[presenter-rendering]: ../features/009-presenter-rendering.md
[surface]: ../architecture.md#surface
[terminal-adapter]: ../architecture.md#presenter-output-port
