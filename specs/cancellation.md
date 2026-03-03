# Cancellation

Cancellation is a cooperative shutdown primitive. Commands observe a
cancellation token to know when to stop work. Tokens form a tree: cancelling a
parent cancels all children, but cancelling a child leaves the parent and
siblings unaffected.

## Token

r[cancel.token.new]
`Cancellation::new()` MUST create an unsignaled root token.

r[cancel.token.default]
`Cancellation::default()` MUST be equivalent to `Cancellation::new()`.

r[cancel.token.clone]
Cloning a `Cancellation` MUST produce a handle to the same underlying token, not
an independent copy.

## Tree

r[cancel.tree.child]
`child()` MUST create a new token linked to the parent.

r[cancel.tree.parent-to-child]
Cancelling a parent MUST cancel all of its descendants.

r[cancel.tree.child-to-parent]
Cancelling a child MUST NOT affect the parent or siblings.

r[cancel.tree.depth]
Child tokens MAY be nested to arbitrary depth. Cancellation MUST propagate
through the full chain.

r[cancel.tree.outlive]
A child token MUST remain functional after its parent is dropped. If the parent
was cancelled before being dropped, the child MUST still report as cancelled.

## Signaling

r[cancel.signal.cancel]
`cancel()` MUST signal the token so that `is_cancelled()` returns `true`.

r[cancel.signal.idempotent]
Calling `cancel()` on an already-cancelled token MUST be a no-op.

r[cancel.signal.check]
`is_cancelled()` MUST be a synchronous, non-blocking check.

r[cancel.signal.await]
`cancelled()` MUST return a future that completes when the token is signaled.

r[cancel.signal.await-already]
If the token is already cancelled, the future returned by `cancelled()` MUST
complete immediately.

## Thread safety

r[cancel.safety.send]
`Cancellation` MUST implement `Send`.

r[cancel.safety.sync]
`Cancellation` MUST implement `Sync`.

r[cancel.safety.unpin]
`Cancellation` MUST implement `Unpin`.

## OS signal adapter

The signal adapter maps OS signals to the application's root cancellation token.
This is infrastructure, not domain logic — the domain only sees tokens.

r[cancel.os.first]
The first SIGINT (or SIGTERM on Unix) MUST cancel the application's root
cancellation token.

r[cancel.os.second]
The second SIGINT MUST exit the process immediately with code 130.

r[cancel.os.unix]
On Unix, both SIGINT and SIGTERM MUST be handled. The first of either signal
triggers cancellation.

r[cancel.os.eager]
On Unix, signal handlers MUST be registered eagerly (at call time), not when the
returned future is first polled.

## Context integration

r[cancel.context.field]
`Context` MUST carry a `Cancellation` token accessible to commands.

r[cancel.context.default]
When no cancellation token is provided to the Context builder, the default MUST
be an unsignaled token.

r[cancel.context.injectable]
The Context builder MUST accept an optional `Cancellation` token, allowing tests
and callers to inject a pre-configured token.
