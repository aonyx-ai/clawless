# Projection

A projection is a pull-based, queryable view of the event stream for stateful
applications. It consumes events from an event channel in the background and
provides read access to accumulated state at any time. TUI applications query
the projection on each render frame without interacting with the event system
directly.

## Construction

r[projection.new]
`Projection::new(receiver)` MUST accept an `EventReceiver` and return a
projection that is ready to query.

r[projection.new.drain]
Construction MUST start a background task that drains events from the receiver
into internal storage. The caller MUST NOT need to manually drive the drain.

## Entry

A projection entry is the user-facing representation of a single event. The
projection translates internal events into entries so that consumers never
interact with the event system directly.

r[projection.entry.message]
A projection MUST store message events as entries carrying the message text.

r[projection.entry.detail]
A projection MUST store detail events as entries carrying the detail text.

r[projection.entry.artifact]
A projection MUST store artifact events as entries carrying the artifact value.

r[projection.entry.order]
Entries MUST be stored in the order they were received from the event channel.

## Queries

A projection provides access to accumulated entries, both as a complete
ordered sequence and filtered by type.

r[projection.query.entries]
A projection MUST provide access to all accumulated entries in receive order.

r[projection.query.messages]
A projection MUST provide filtered access to message entries only.

r[projection.query.details]
A projection MUST provide filtered access to detail entries only.

r[projection.query.artifacts]
A projection MUST provide filtered access to artifact entries only.

## Lifecycle

r[projection.lifecycle.complete]
A projection MUST report whether the event stream has closed and all buffered
events have been drained.

## Thread safety

A projection is shared between the background drain task (which writes) and
the querying thread (which reads, typically a render loop). Both operations
must be safe to perform concurrently.

r[projection.safety.send]
`Projection` MUST implement `Send`.

r[projection.safety.sync]
`Projection` MUST implement `Sync`.

r[projection.safety.unpin]
`Projection` MUST implement `Unpin`.
