---
sidebar_position: 5
---

# Cancellation

Cancellation is the mechanism Clawless uses for cooperative shutdown. When a
user presses Ctrl+C, the framework doesn't kill your command instantly. Instead,
it sets a cancellation token that your command can observe, giving in-flight work
a chance to finish cleanly.

## What is cancellation?

Cancellation is a cooperative protocol between the framework and your commands.
Rather than forcefully terminating a process, Clawless provides a
[`Cancellation`][cancellation-type] token through [`Context`][context]. Commands
observe this token and decide when and how to stop work.

This matters for commands that manage resources: open files, network
connections, database transactions, or temporary state. Cooperative cancellation
lets your command close connections, flush buffers, and clean up before exiting.

## How it works

The cancellation flow has three participants:

1. **The user** sends a signal (Ctrl+C or SIGTERM)
2. **The framework** catches the signal and cancels the root token
3. **Your command** observes the token and shuts down gracefully

```rust
use clawless::prelude::*;

/// Process items until cancelled
#[command]
pub async fn process(_args: ProcessArgs, context: Context) -> CommandResult {
    let cancellation = context.cancellation();

    loop {
        if cancellation.is_cancelled() {
            message!("shutting down");
            break;
        }

        // Do work...
    }

    Ok(())
}
```

The framework handles signal registration and token management automatically.
Your command only needs to check the token.

## The Cancellation type

[`Cancellation`][cancellation-type] exposes four methods:

### `cancelled()`

Returns a future that completes when the token is cancelled. Use this in async
code to wait for cancellation:

```rust
context.cancellation().cancelled().await;
```

The returned future works with `tokio::select!`, making it easy to race
cancellation against other async work.

### `is_cancelled()`

Returns `true` if the token has been cancelled. This is a synchronous,
non-blocking check suited for loops and CPU-bound work:

```rust
if context.cancellation().is_cancelled() {
    return Ok(());
}
```

### `child()`

Creates a child token linked to the parent. See
[parent-child tokens](#parent-child-tokens) below.

### `cancel()`

Cancels the token and all its children. Calling `cancel()` on an
already-cancelled token is a no-op:

```rust
let cancellation = Cancellation::new();
cancellation.cancel();
cancellation.cancel(); // idempotent, no effect
```

## Parent-child tokens

Tokens form a tree. Cancelling a parent cascades to all children, but
cancelling a child leaves the parent and siblings unaffected:

```rust
use clawless::prelude::*;

let root = Cancellation::new();
let child_a = root.child();
let child_b = root.child();

// Cancelling a child doesn't affect the parent or siblings
child_a.cancel();
assert!(child_a.is_cancelled());
assert!(!child_b.is_cancelled());
assert!(!root.is_cancelled());

// Cancelling the parent cascades to all children
root.cancel();
assert!(child_b.is_cancelled());
```

This is useful when your command spawns independent sub-tasks. Each sub-task
gets its own child token and can be cancelled individually without affecting the
others. When the parent is cancelled (for example, by Ctrl+C), all sub-tasks
are cancelled together.

Children can be nested to any depth. Cancellation always propagates downward
through the entire subtree.

## Double Ctrl+C

Clawless follows the standard CLI convention for signal handling:

- **First Ctrl+C** triggers graceful cancellation. The framework sets the
  cancellation token, and your command has a chance to clean up.
- **Second Ctrl+C** force-exits the process immediately with code 130
  (128 + SIGINT).

This matches the behavior of cargo, npm, Docker, and other CLI tools that users
already expect. On Unix, SIGTERM is also handled and triggers the same graceful
cancellation as the first Ctrl+C.

## Why cooperative?

Clawless uses cooperative cancellation rather than forceful termination for
several reasons:

**Clean resource management.** Commands that hold open files, network
connections, or database transactions can close them properly. This prevents
data corruption and resource leaks.

**Predictable behavior.** Commands always run their cleanup logic, whether they
complete normally or are cancelled. There's no distinction between "finished"
and "interrupted" at the resource management level.

**Explicit in the type system.** The `Cancellation` type makes cancellation a
first-class concept. Commands that need to handle cancellation declare it
through their use of `context.cancellation()`, making the intent visible in the
code.

**User control.** If graceful shutdown takes too long, the user can press
Ctrl+C again to force-exit. This provides an escape hatch without sacrificing
the default of clean shutdown.

## What's next

- **[Handle cancellation](../how-to/handle-cancellation)** - Practical patterns
  for responding to cancellation in your commands
- **[Context](./context)** - The broader Context system that provides
  cancellation to commands

[cancellation-type]: https://docs.rs/clawless/latest/clawless/cancellation/struct.Cancellation.html
[context]: ./context
