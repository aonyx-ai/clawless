# Dispatch

Dispatch is the mechanism that routes parsed command-line arguments to the
correct subcommand leaf and identifies what kind of lifecycle it needs. The
process has two phases: resolution walks the subcommand tree to find the leaf,
and execution sets up the appropriate runtime and runs it.

## Resolved leaf

A resolved leaf is the result of the resolution phase. It carries the parsed
argument matches for the leaf and a function pointer to execute it, along with
a type tag that tells the execution phase which runner to use.

r[dispatch.leaf.command]
A resolved leaf MUST support a command variant for stateless CLI commands that
are rendered through a push-based presenter.

r[dispatch.leaf.application]
A resolved leaf MUST support an application variant for stateful TUI
applications that query a pull-based projection.

r[dispatch.leaf.matches]
Each variant MUST carry the parsed argument matches for the leaf.

r[dispatch.leaf.exec]
Each variant MUST carry a function pointer that executes the leaf with the
appropriate arguments for its lifecycle.

## Resolution

r[dispatch.resolve.sync]
Resolution MUST be synchronous and side-effect-free. It navigates the parsed
arguments and returns a resolved leaf without creating runtimes, channels, or
other resources.

r[dispatch.resolve.uniform]
Every node in the subcommand tree MUST use the same function signature for
resolution: accepting parsed argument matches and returning a resolved leaf.

r[dispatch.resolve.delegate]
Intermediate nodes MUST delegate resolution to their children by matching on
the subcommand name and calling the child's resolve function.

## Execution

Execution sets up the lifecycle that a leaf needs and then calls the leaf. The
`#[command]` and `#[application]` macros generate a function for that call, but
a caller that builds its command tree at run time has no function to name: the
leaf it resolved is a value that it owns. A runner therefore states what it
calls, not how that callable came to exist.

r[dispatch.exec.callable]
A runner MUST accept any callable that executes the leaf, and MUST NOT require
a function pointer.

r[dispatch.exec.command-runner]
When the resolved leaf is a command, the execution phase MUST delegate to a
runner that creates the event channel, context, presenter, and async runtime.

r[dispatch.exec.application-runner]
When the resolved leaf is an application, the execution phase MUST delegate to
a runner that creates the event channel, context, projection, and async
runtime.

## Thread safety

A resolved leaf is passed from the synchronous resolution phase to the
execution phase. It must be safe to move between threads.

r[dispatch.safety.send]
`ResolvedLeaf` MUST implement `Send`.

r[dispatch.safety.sync]
`ResolvedLeaf` MUST implement `Sync`.

r[dispatch.safety.unpin]
`ResolvedLeaf` MUST implement `Unpin`.
