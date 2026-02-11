# Signal handling

- **Project**: [P001-cancellation][project]
- **Dependencies**: [F001-cancellation-token][cancellation-token]
- **Breaking changes**: none

## Summary

Infrastructure that maps OS signals (SIGINT, SIGTERM) to a [Cancellation] token.
This is adapter-level plumbing that bridges the operating system and the domain.
It is not part of the public API; it is consumed only by the `main!()` macro
expansion.

## Motivation

A [Cancellation][cancellation] token is inert until something signals it. In a
CLI application, the natural signal source is the operating system: the user
presses Ctrl+C (SIGINT) or a process manager sends SIGTERM. This feature
provides the mapping from those OS signals to the domain's cancellation token,
completing the "infrastructure" half of the domain/infrastructure split defined
in the [architecture][architecture].

Without this, each application author would need to register signal handlers
and wire them to the token manually, defeating the purpose of framework-level
cancellation support.

## Domain concepts

None. Signal handling is infrastructure, not domain logic. Per the
[architecture][architecture]:

> The mapping from OS signals to cancellation tokens is infrastructure.

This feature lives entirely outside the domain core. It is an adapter that
produces a domain value (the signaled `Cancellation` token) in response to an
external event (an OS signal).

## Design rationale

### Hexagonal boundary

The [architecture][architecture] draws a clear line: `Cancellation` is a domain
value object; signal handling is infrastructure. This feature respects that
boundary. The signal module knows about `Cancellation` (it signals the token),
but the domain never knows about signals (commands observe the token, not the
signal source).

### Double Ctrl+C pattern

The double Ctrl+C pattern is the standard UX convention for CLI applications:

1. **First Ctrl+C**: graceful shutdown. The cancellation token is signaled,
   giving in-flight work a chance to complete, flush buffers, and clean up.
2. **Second Ctrl+C**: immediate exit. The user has decided that graceful
   shutdown is taking too long. The process exits immediately with code 130
   (128 + SIGINT signal number 2).

This matches the behavior of tools like `cargo`, `npm`, Docker, and most
well-behaved CLI applications. Users intuitively understand that one Ctrl+C
means "please stop" and two means "stop now."

### SIGTERM behavior

SIGTERM has different semantics from SIGINT. SIGINT is user-initiated ("I want
to interrupt this"). SIGTERM is system-initiated ("this process should
terminate"). There are two reasonable approaches:

| Approach              | Behavior                                  | Rationale                                                       |
| --------------------- | ----------------------------------------- | --------------------------------------------------------------- |
| Graceful cancellation | Cancel the token, same as first SIGINT    | Forgiving for process managers that send SIGTERM before SIGKILL |
| Immediate exit        | `std::process::exit(143)` without cleanup | SIGTERM semantically means "terminate now"                      |

This is an [open question](#open-questions). Both approaches are defensible.

### Exit code conventions

Signal-induced exit codes follow the Unix convention of 128 + signal number:

| Signal  | Number | Exit code |
| ------- | ------ | --------- |
| SIGINT  | 2      | 130       |
| SIGTERM | 15     | 143       |

These codes allow parent processes and scripts to distinguish signal-induced
exits from normal failures.

### Hidden API

The signal module is `#[doc(hidden)]`. It is internal plumbing used by
`main!()`, not a user-facing API. Users interact with the
[Cancellation][cancellation] token; they never interact with the signal handler
directly. Hiding it:

- Keeps the public API surface small and focused on domain concepts.
- Allows the signal handling implementation to change without breaking
  downstream code.
- Avoids confusion about whether users should register their own signal
  handlers.

## Functional requirements

1. `wait_for_shutdown` accepts a `Cancellation` token and listens for OS
   signals.
2. On the first SIGINT, the token is cancelled (graceful shutdown begins).
3. On the second SIGINT, the process exits immediately with code 130.
4. SIGTERM handling is platform-specific (Unix only). Behavior is an
   [open question](#open-questions).
5. On platforms without Unix signals (Windows), only Ctrl+C is handled.
   Tokio abstracts the platform-specific mechanism (console control handler on
   Windows, SIGINT on Unix).

## Non-functional requirements

1. **No busy-waiting**: signal handling must be async, using Tokio's signal
   facilities.
2. **Thread safety**: the signal handler runs in a Tokio task and must be
   `Send`.
3. **Platform portability**: the module must compile and function correctly on
   Unix (Linux, macOS) and Windows. Platform-specific behavior is gated with
   `cfg` attributes.

## API surface

### New function (hidden)

```rust
/// Waits for shutdown signals and maps them to cancellation
///
/// This function is `#[doc(hidden)]` and not part of the public API.
/// It is called by the `main!()` macro expansion.
#[doc(hidden)]
pub async fn wait_for_shutdown(cancellation: Cancellation) {
    // First SIGINT → cancellation.cancel()
    // Second SIGINT → std::process::exit(130)
    // SIGTERM (Unix) → TBD (open question)
}
```

This function is designed to be spawned as a background Tokio task by
`main!()`.

### No prelude export

`wait_for_shutdown` is **not** added to the prelude. It is accessed via
`clawless::signal::wait_for_shutdown` only from generated macro code.

## Dependencies

### Modified workspace dependency

```toml
# Cargo.toml [workspace.dependencies]
tokio = { version = "1", features = ["macros", "rt-multi-thread", "signal"] }
```

The `signal` feature is added to the existing `tokio` dependency.

## File changes

### New files

| File                            | Contents                                        |
| ------------------------------- | ----------------------------------------------- |
| `crates/clawless/src/signal.rs` | `wait_for_shutdown` function and platform logic |

### Modified files

| File                         | Change                                             |
| ---------------------------- | -------------------------------------------------- |
| `Cargo.toml`                 | Add `signal` feature to tokio workspace dependency |
| `crates/clawless/src/lib.rs` | Add `#[doc(hidden)] pub mod signal;`               |

## Edge cases

| Case                                     | Expected behavior                                        |
| ---------------------------------------- | -------------------------------------------------------- |
| Rapid double Ctrl+C                      | Second signal exits immediately even if first handler    |
|                                          | has not finished processing                              |
| Signal during argument parsing           | Signal is not handled until `wait_for_shutdown` is       |
|                                          | spawned by `main!()`; default OS behavior applies before |
|                                          | that point                                               |
| SIGTERM on Windows                       | Not handled; only Ctrl+C (SIGINT equivalent) is          |
|                                          | supported on Windows                                     |
| Token already cancelled when signal      | `cancel()` is idempotent; the signal handler still       |
| arrives                                  | proceeds to exit on second SIGINT                        |
| Multiple concurrent signal handler tasks | Not supported; `main!()` spawns exactly one              |

## Out of scope

- Shutdown timeout (force-exiting after a deadline if graceful shutdown stalls)
- Cleanup hooks (registering callbacks that run during shutdown)
- Custom signal handling (SIGHUP, SIGUSR1, etc.)
- User-facing signal API (this module is `#[doc(hidden)]`)
- SIGTERM on Windows

## Open questions

### SIGTERM behavior

Should SIGTERM trigger graceful cancellation (same as first SIGINT) or immediate
exit (code 143)?

**Arguments for graceful cancellation**:

- Process managers like systemd and Docker send SIGTERM first, then SIGKILL
  after a timeout. Graceful cancellation gives the application a chance to clean
  up within that window.
- Consistent behavior: all external shutdown signals take the same path through
  the cancellation token.

**Arguments for immediate exit**:

- SIGTERM semantically means "terminate this process." The user did not press
  Ctrl+C; a system component decided the process should end.
- Simplicity: no need to track whether the first signal was SIGINT or SIGTERM.

**Recommendation**: graceful cancellation. Most CLI tools that handle both
signals treat SIGTERM as a graceful shutdown trigger, and the systemd/Docker
workflow is common enough to warrant supporting it.

[architecture]: ../architecture.md
[cancellation]: ../architecture.md#cancellation
[cancellation-token]: 001-cancellation-token.md
[project]: ../projects/001-cancellation.md
