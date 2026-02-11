# Cancellation token

- **Project**: [P001-cancellation][project]
- **Dependencies**: none
- **Breaking changes**: none

## Summary

Introduce `Cancellation`, a domain value object that represents a cooperative
shutdown signal. Commands and (eventually) Tasks observe this token to know when
to stop work. The token forms a tree: cancelling a parent cancels all children,
but cancelling a child leaves the parent and siblings unaffected.

## Motivation

The [architecture] defines [Cancellation] as a value object in the domain
core: "Token-based shutdown signal; Tasks observe, framework manages." This
feature implements the token itself, independent of what produces it (signals)
or what consumes it (commands, tasks). Separating the token from its producers
and consumers keeps the domain layer free of infrastructure concerns and makes
each piece independently testable.

## Domain concepts

### Cancellation (value object)

`Cancellation` is a value object per the [architecture]. It has no meaningful
identity: two tokens created at the same point in a tree are interchangeable in
the domain model. What matters is the token's state (signaled or not) and its
position in the tree (root or child).

Cancellation tokens form a tree:

- The [Application] owns the **root token**.
- A [Command] or (future) [Task] may create a **child token** scoped to a unit
  of work.
- Cancelling a child stops that unit without affecting the parent or siblings.
- Cancelling the root triggers shutdown of all outstanding work.

### Relationship to other domain concepts

- **Application**: owns the root `Cancellation` token.
- **Command**: receives the token; may create children for scoped work.
- **Task** (future): will observe the token for graceful shutdown.
- **Context**: remains read-only environment description. `Cancellation` is
  operational, not environmental, and is therefore a separate parameter.

## Design rationale

### Newtype over `CancellationToken`

`Cancellation` is a newtype wrapping
[`tokio_util::sync::CancellationToken`][tokio-ct].

**Why not re-export `CancellationToken` directly?**

- **Curated API surface**: `CancellationToken` exposes methods like
  `drop_guard()`, `run_until_cancelled()`, and `child_token()` that do not match
  Clawless's domain vocabulary. A newtype lets us expose exactly the operations
  that make sense in the domain (`child()`, `cancel()`, `is_cancelled()`,
  `cancelled()`).
- **Decoupled dependency**: downstream crates depend on
  `clawless::Cancellation`, not on `tokio-util` directly. If the underlying
  implementation changes, only the newtype wrapper needs updating.
- **Domain vocabulary**: the type name `Cancellation` matches the
  [architecture's ubiquitous language][cancellation], not Tokio's naming
  conventions.

### Derives

`Cancellation` derives `Clone`, `Debug`, and `Default`:

- **`Clone`**: tokens are shared by cloning, consistent with the inner
  `CancellationToken` which is `Clone` (reference-counted internally).
- **`Debug`**: required for diagnostics and test output.
- **`Default`**: creates an unsignaled root token, useful for testing and as a
  builder default.

`Cancellation` does **not** derive `Eq`, `PartialEq`, `Ord`, `PartialOrd`, or
`Hash`:

- **Identity comparison is not meaningful**: two `Cancellation` values wrapping
  the same underlying token are operationally identical, but the domain does not
  define what "equal" means for tokens. Are two independently-created unsignaled
  tokens equal? Are a parent and child equal? These questions have no useful
  answer, so equality is intentionally omitted.
- **The inner type does not implement them**: `CancellationToken` does not
  implement `Eq` or `Hash`, so deriving them is not possible without manual
  implementation, which would require choosing semantics that do not exist in
  the domain.

### Return type of `cancelled()`

The `cancelled()` method returns a future that completes when the token is
signaled. There are two options for the return type:

| Option                                   | Pros                                                        | Cons                                              |
| ---------------------------------------- | ----------------------------------------------------------- | ------------------------------------------------- |
| Concrete `WaitForCancellationFuture<'_>` | Named type in docs, can be stored in structs, no allocation | Leaks `tokio-util` type into public API           |
| Opaque `impl Future<Output = ()> + '_`   | Hides implementation detail, clean public API               | Cannot be named in type position, harder to store |

This is an [open question](#open-questions).

## Functional requirements

1. `Cancellation::new()` creates an unsignaled root token.
2. `Cancellation::child()` creates a child token linked to the parent.
   Cancelling the parent cancels the child. Cancelling the child does not
   cancel the parent.
3. `Cancellation::cancel()` signals the token. All children are also signaled.
   Calling `cancel()` on an already-cancelled token is a no-op (idempotent).
4. `Cancellation::is_cancelled()` returns `true` if the token has been signaled,
   `false` otherwise. This is a synchronous, non-blocking check.
5. `Cancellation::cancelled()` returns a future that completes when the token
   is signaled. If the token is already cancelled, the future completes
   immediately.

## Non-functional requirements

1. **Thread safety**: `Cancellation` must be `Send + Sync + Unpin`.
2. **No allocation on check**: `is_cancelled()` must not allocate.
3. **Platform-independent**: the token itself has no platform-specific behavior.

## API surface

### Newtype

````rust
/// Token-based shutdown signal
///
/// `Cancellation` is a cooperative shutdown primitive. Commands and Tasks
/// observe a cancellation token to know when to stop work. Tokens form a tree:
/// cancelling a parent cancels all children, but cancelling a child leaves the
/// parent and siblings unaffected.
///
/// # Examples
///
/// ```
/// use clawless::prelude::*;
///
/// let root = Cancellation::new();
/// let child = root.child();
///
/// assert!(!root.is_cancelled());
/// assert!(!child.is_cancelled());
///
/// root.cancel();
///
/// assert!(root.is_cancelled());
/// assert!(child.is_cancelled());
/// ```
#[derive(Clone, Debug, Default)]
pub struct Cancellation {
    /* tokio_util::sync::CancellationToken */
}
````

### Methods

| Method           | Signature                         | Description                            |
| ---------------- | --------------------------------- | -------------------------------------- |
| `new()`          | `fn new() -> Cancellation`        | Creates an unsignaled root token       |
| `child()`        | `fn child(&self) -> Cancellation` | Creates a scoped child token           |
| `cancel()`       | `fn cancel(&self)`                | Signals the token (idempotent)         |
| `is_cancelled()` | `fn is_cancelled(&self) -> bool`  | Synchronous cancellation check         |
| `cancelled()`    | `async` / returns future          | Async observation for use in `select!` |

### Prelude export

`Cancellation` is added to `clawless::prelude`.

## Dependencies

### New workspace dependency

```toml
# Cargo.toml [workspace.dependencies]
tokio-util = "0.7"
```

### Crate dependency

```toml
# crates/clawless/Cargo.toml [dependencies]
tokio-util = { workspace = true }
```

## File changes

### New files

| File                                  | Contents                      |
| ------------------------------------- | ----------------------------- |
| `crates/clawless/src/cancellation.rs` | `Cancellation` type and tests |

### Modified files

| File                         | Change                                            |
| ---------------------------- | ------------------------------------------------- |
| `Cargo.toml`                 | Add `tokio-util` to `[workspace.dependencies]`    |
| `crates/clawless/Cargo.toml` | Add `tokio-util` dependency                       |
| `crates/clawless/src/lib.rs` | Add `pub mod cancellation;` and prelude re-export |

## Edge cases

| Case                             | Expected behavior                                   |
| -------------------------------- | --------------------------------------------------- |
| Idempotent cancel                | Calling `cancel()` twice is a no-op the second time |
| Child outliving parent           | Child remains cancelled even if parent is dropped   |
| `cancelled()` on cancelled token | Future completes immediately                        |
| Deeply nested children           | Cancelling root propagates through entire tree      |
| `Default::default()`             | Equivalent to `Cancellation::new()` (unsignaled)    |

## Out of scope

- Signal handling (see [F002-signal-handling][signal-handling])
- Injection into commands (
  see [F003-command-cancellation][command-cancellation])
- Task integration (deferred until Task entity is implemented)
- Shutdown timeout and cleanup hooks
- Cancellation reasons or error enrichment

## Open questions

### Return type of `cancelled()`

Should `cancelled()` return the concrete `WaitForCancellationFuture<'_>` from
`tokio-util`, or an opaque `impl Future<Output = ()> + '_`?

The concrete type leaks the `tokio-util` dependency into the public API but
allows the future to be named and stored. The opaque type hides the
implementation but cannot be used in type position.

**Recommendation**: start with the concrete type for maximum flexibility, and
consider switching to an opaque type if `tokio-util` becomes a problematic
public dependency.

[application]: ../architecture.md#application
[architecture]: ../architecture.md
[cancellation]: ../architecture.md#cancellation
[command]: ../architecture.md#command
[command-cancellation]: 003-command-cancellation.md
[project]: ../projects/001-cancellation.md
[signal-handling]: 002-signal-handling.md
[task]: ../architecture.md#task
[tokio-ct]: https://docs.rs/tokio-util/latest/tokio_util/sync/struct.CancellationToken.html
