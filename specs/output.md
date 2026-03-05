# Output

Output is the interface commands use to produce events. It wraps an event
channel sender and provides methods for each event kind, decoupling commands
from the rendering strategy.

## Sending events

Commands produce three kinds of output through Output. Each method sends an
event into the channel for the presenter to consume.

r[output.send.message]
Output MUST be able to send informational text as an event.

r[output.send.detail]
Output MUST be able to send supplementary text as an event.

r[output.send.artifact]
Output MUST be able to send structured data as an event.

r[output.send.async]
Sending output MUST be an asynchronous operation.

r[output.send.error]
Sending to a closed channel MUST return an error.

## Thread safety

Output is shared across async tasks. The system must be safe to use in this
context.

r[output.safety.clone]
Output MUST be cheaply clonable to share across tasks.

r[output.safety.send]
Output MUST be sendable across thread boundaries.

r[output.safety.concurrent]
Output MUST be safe for concurrent use from multiple threads.
