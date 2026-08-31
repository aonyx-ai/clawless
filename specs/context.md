# Context

A context is the runtime environment passed to commands and applications. It
provides access to shared resources: the output channel for emitting events,
a cancellation token for cooperative shutdown, the current working directory,
and the interface that runs external programs.

## Construction

r[context.new]
A context MUST be constructable via a builder pattern.

r[context.new.error]
If environment detection fails during construction, the builder MUST return a
structured error carrying the underlying cause as its source.

## Fields

r[context.field.cwd]
A context MUST provide access to a `CurrentWorkingDirectory` value.

r[context.field.output]
A context MUST provide access to the `Output` for emitting events.

r[context.field.cancellation]
A context MUST provide access to a `Cancellation` token.

See also [cancel.context.field], [cancel.context.default], and
[cancel.context.injectable] in the [cancellation spec][cancellation].

## Capabilities

r[context.process]
A context MUST provide an interface that runs external programs, wired to the
output and the cancellation token of that context. The [process
specification][process] defines what such a run does.

## Thread safety

A context is shared across async tasks. It must be safe to clone and send
between threads.

r[context.safety.send]
`Context` MUST implement `Send`.

r[context.safety.sync]
`Context` MUST implement `Sync`.

r[context.safety.unpin]
`Context` MUST implement `Unpin`.

[cancellation]: cancellation.md
[cancel.context.field]: cancellation.md#context-integration
[cancel.context.default]: cancellation.md#context-integration
[cancel.context.injectable]: cancellation.md#context-integration
[process]: process.md
