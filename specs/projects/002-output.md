# Output

## Summary

Framework-controlled output for Clawless commands. This project replaces direct
`println!` usage with an Output type that provides verbosity control (`--quiet`,
`--verbose`), structured JSON output (`--json`), and a foundation for
the [Formatter] port described in the [architecture].

## Motivation

Commands currently produce output via `println!`, giving the framework no
control over verbosity, format, or destination. This means:

- Users cannot suppress noisy output.
- Users cannot enable additional detail.
- Script authors cannot consume machine-readable output.
- The framework cannot evolve toward richer formatting strategies.

The [architecture] envisions a rich output system where [Artifact], [Progress],
and [Diagnostic] flow through a [Formatter] port with swappable adapters. A
simple streaming formatter could be built today, but the nested formatters that
render parallel [Task] output in groups depend on the Task entity, which does
not exist yet. This project introduces the pragmatic first step: an Output type
on [Context] that replaces direct printing, with verbosity control and
structured JSON output.

Output is itself a simple formatter — one that prints line-by-line to the
terminal. The architecture envisions this evolving toward richer formatters
(grouped-by-task output, stateful TUIs), but the Output abstraction provides
the foundation that those formatters will build on.

| Architecture concept | Output project equivalent         | Evolution path                                    |
| -------------------- | --------------------------------- | ------------------------------------------------- |
| [Artifact]           | `output.result(value)`            | Streaming, batching                               |
| [Progress]           | `output.print()` / `verbose()`    | Structured status (spinners, bars)                |
| [Formatter] port     | Output struct with mode switching | Trait-based port when multiple strategies coexist |
| Terminal adapter     | Text mode (default)               | Richer rendering (colors, layout)                 |
| JSON adapter         | JSON mode (`--json`)              | Task-structured JSON output                       |

## Feature specs

Each feature spec maps to one PR. They must be implemented in order.

| #    | Spec                                  | Depends on | Summary                                           |
| ---- | ------------------------------------- | ---------- | ------------------------------------------------- |
| F004 | [F004-output-types][output-types]     |            | `Output`, `Verbosity`, `OutputMode` domain types  |
| F005 | [F005-command-output][command-output] | F004       | Context integration, macro flags, example updates |

F004 is a pure addition with no downstream impact. F005 is also additive — it
adds an `Output` field to [Context] and new global flags to the macro-generated
main function — but updates all examples, scaffolding templates, and test
fixtures to use the new output API.

### Prerequisite

F005 depends on a separate PR (not part of this project) that migrates Context
from `typed-builder` to `bon`. This simplifies Context construction and sets up
a cleaner pattern for adding the Output field. The migration must land before
F005 implementation begins.

## Out of scope

The following are intentionally excluded from this project. They may become
future projects once the output foundation is in place.

- **Progress reporting**: spinners, progress bars, and step indicators.
- **Nested Formatter port**: a trait-based port with child instances per Task
  for rendering parallel task output in groups. A simple streaming formatter
  is possible today, but the nested architecture is deferred until the Task
  tree exists.
- **Diagnostic restructuring**: `anyhow` is sufficient for error handling today.
- **Outcome and exit code control**: mapping command results to specific exit
  codes is a separate project.
- **CI adapter**: a plain-text, no-cursor-control adapter can be added later as
  a third output strategy.
- **External commands** (#154): running child processes is an orthogonal
  concern.
- **Observability and tracing** (#155): developer-facing logging is separate
  from user-facing output.

## Open questions

None at the project level. Open questions are pushed to the individual feature
specs where they can be resolved with full context.

[architecture]: ../architecture.md
[artifact]: ../architecture.md#artifact
[command-output]: ../features/005-command-output.md
[context]: ../architecture.md#context
[diagnostic]: ../architecture.md#diagnostic
[formatter]: ../architecture.md#formatter-output-port
[output-types]: ../features/004-output-types.md
[progress]: ../architecture.md#progress
[task]: ../architecture.md#task
