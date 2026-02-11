# Cancellation

## Summary

Token-based cooperative shutdown for Clawless. This project introduces a
[Cancellation] domain value object, signal-handling infrastructure, and macro
integration so that CLI applications built with Clawless can shut down
gracefully on Ctrl+C without each application author writing their own signal
handling.

## Motivation

Every non-trivial CLI application needs to handle Ctrl+C. Without framework
support, each application author must independently solve signal registration,
token propagation, and graceful teardown. The solutions vary in quality and
correctness, and the resulting code is boilerplate that obscures application
logic.

Clawless's [architecture] already defines Cancellation as a domain value object
and places signal handling in the infrastructure layer. This project makes that
definition concrete across three incremental PRs.

## Feature specs

Each feature spec maps to one PR. They must be implemented in order.

| #    | Spec                                              | Depends on | Summary                                        |
| ---- | ------------------------------------------------- | ---------- | ---------------------------------------------- |
| F001 | [F001-cancellation-token][cancellation-token]     |            | `Cancellation` domain value object             |
| F002 | [F002-signal-handling][signal-handling]           | F001       | OS signal to cancellation token mapping        |
| F003 | [F003-command-cancellation][command-cancellation] | F001, F002 | Macro integration and command signature change |

F001 is a pure addition with no downstream impact. F002 is also a pure addition
but depends on the `Cancellation` type from F001. F003 introduces a breaking
change to the command signature and updates all downstream code (macros,
examples, CLI commands, tests).

## Out of scope

The following are intentionally excluded from this project. They may become
future projects once the cancellation foundation is in place.

- **Task integration**: Tasks observing cancellation tokens is deferred until
  the Task entity is implemented.
- **Shutdown timeout**: Application-level timeout that force-exits if graceful
  shutdown takes too long.
- **Cleanup hooks**: Registering teardown callbacks that run during
  cancellation.
- **Cancellation reasons**: Enriching the token with why cancellation occurred
  (signal type, user action, timeout).

## Open questions

None at the project level. Open questions are pushed to the individual feature
specs where they can be resolved with full context.

[architecture]: ../architecture.md
[cancellation]: ../architecture.md#cancellation
[cancellation-token]: ../features/001-cancellation-token.md
[signal-handling]: ../features/002-signal-handling.md
[command-cancellation]: ../features/003-command-cancellation.md
