# Event

Events are the structured output primitives of a command-line application.
Commands produce events to communicate results, and a presenter consumes them
to render output. This separation decouples what a command says from how it is
displayed.

## Output

Commands produce three kinds of output, each with different semantics. The
presenter uses these semantics to decide what to show based on verbosity and
output mode.

r[event.output.message]
Commands MUST be able to produce informational text as events.

r[event.output.detail]
Commands MUST be able to produce supplementary text as events. Supplementary
text carries lower-priority information that a presenter MAY suppress at
default verbosity.

r[event.output.artifact]
Commands MUST be able to produce structured data as their primary output.
Artifacts are the main deliverable of a command — the data the user asked for.

## Artifact

An artifact is the primary output of a command. It must support multiple
rendering strategies so that the same command can produce human-readable text
or machine-readable structured output without changing its implementation.

r[event.artifact.text]
An artifact MUST be renderable as human-readable text.

r[event.artifact.structured]
An artifact MUST be serializable to structured formats such as JSON.

r[event.artifact.zero-cost]
Command authors MUST NOT need to manually implement the artifact contract.
Deriving standard traits (Display, Serialize, Debug) MUST be sufficient.

## Transport

Events travel from producers (commands) to a consumer (the presenter) through
an asynchronous channel. The channel provides ordering, back-pressure, and
lifecycle management.

r[event.transport.async]
Sending and receiving events MUST be asynchronous operations.

r[event.transport.multi-producer]
Multiple producers MUST be able to send events concurrently into the same
channel.

r[event.transport.single-consumer]
Exactly one consumer MUST receive events from a channel.

r[event.transport.ordered]
Events from a single producer MUST arrive at the consumer in the order they
were sent.

r[event.transport.backpressure]
When the consumer falls behind, producers MUST slow down rather than buffering
without bound.

r[event.transport.completion]
The consumer MUST be notified when all producers are done and no more events
will arrive.

r[event.transport.drain]
Buffered events MUST be delivered to the consumer before signaling completion.

r[event.transport.error]
Sending an event to a closed channel MUST return an error that carries the
unsent event.

## Thread safety

Events travel across async task boundaries in a concurrent runtime. The system
must be safe to use in this context.

r[event.safety.event-send]
Events MUST be sendable across thread boundaries.

r[event.safety.producer-clone]
Producers MUST be cheaply clonable to share the channel across tasks.

r[event.safety.producer-concurrent]
Producers MUST be safe for concurrent use from multiple threads.
