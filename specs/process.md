# Process

Process is the interface that runs an external program from a command. Most
command-line applications drive other programs, and a run reports what the
program does through the event system while it happens, so that a presenter can
show progress instead of waiting for the program to end.

## Running a program

A run takes the description of a command, starts the program, and reports what
the program produced.

r[process.new]
Process MUST be constructible from an output and a cancellation token.

r[process.new.grace]
The time a program gets to end itself before Clawless kills it MUST be
configurable, and it MUST have a default.

r[process.run]
Process MUST be able to run an external program and return its result. The
result carries the exit status, the output of both streams, and the time that
the run took.

r[process.run.capture]
A run MUST report the whole output of the program in its result, whether or not
a consumer read the output while the program ran.

r[process.run.stream]
A run MUST report each line of output as an event before the program ends.

r[process.run.lifecycle]
A run that reports its start MUST also report its end, whatever ends it. A run
whose consumer stopped listening, and a run that the caller dropped instead of
cancelling, are the two exceptions, because neither leaves anyone to send the
event.

r[process.run.error]
A program that does not start, and a run that produces no result, MUST return an
error.

r[process.run.report.error]
A run whose events can no longer be delivered MUST return an error.

## Cancellation

A program that a command started outlives the command unless the command stops
it. A run therefore observes the cancellation token of its command.

r[process.run.cancel]
Cancellation MUST stop a run and end the program, whether the program is writing
output or has stopped writing.

r[process.run.cancel.grace]
Cancellation MUST ask the program to end before it kills it, and give it the
grace period to answer. A build tool that holds a lock file can then remove it,
which a kill gives it no time to do.

r[process.run.cancel.bound]
A cancelled run MUST end within the grace period, whatever the program does with
the request.

r[process.run.cancel.error]
A run that cancellation stopped MUST return an error that names the command.

## Errors

r[process.error]
Each way in which a run can fail MUST be a variant of its own, so that a command
can treat cancellation differently from a program that never ran.

## Events

A run reports itself as events, which is what lets a presenter render a program
while it runs.

r[process.event.lifecycle]
The events of a run MUST describe its start, each line of its output, and its
end.

r[process.event.started]
The start of a run MUST name the command.

r[process.event.started.identifier]
The start of a run MUST carry the identifier that the operating system gave the
program, when the operating system reports one. An operator who looks for the
program in a process list needs that value, and no other event carries it.

r[process.event.line]
A line of output MUST carry the stream that produced it and the text of the
line, without the characters that ended it.

r[process.event.finished]
The end of a run MUST name the command, say how the run ended, and report the
time that the run took.

r[process.event.display]
An event of a run MUST be renderable as one line of a transcript.

r[process.event.correlation]
Every event of a run MUST carry the identity of that run, so that a consumer can
separate two programs that run at the same time. That identity counts runs and
is not the identifier that the operating system gives a program, which names a
program only while it runs and which the operating system reuses afterwards.

r[process.event.correlation.unique]
No two runs of one application MUST share an identity.

r[process.event.correlation.accessor]
A consumer MUST be able to read the identity of an event without matching on the
kind of the event.

## Outcome

r[process.event.outcome]
The end of a run MUST say which of the ends it was.

r[process.event.outcome.exited]
A program that ran to its end MUST report its exit status. A status that is not
a success is a result and not a failure.

r[process.event.outcome.cancelled]
A program that cancellation stopped MUST report that it was cancelled.

r[process.event.outcome.incomplete]
A run that produced no result MUST report that it produced none.

## Rendering

r[process.render.verbosity]
A presenter MUST treat the output of an external program as supplementary, and
show it only at verbose verbosity.

r[process.render.streams]
A presenter that writes to a terminal MUST keep the two streams of the program
apart, so that what the program wrote to its standard error reaches the standard
error of the application.

## Thread safety

Runs happen in async tasks, and a command can start several of them at once.

r[process.safety.clone]
Process MUST be cheaply clonable to share across tasks.

r[process.safety.send]
Process MUST be sendable across thread boundaries.

r[process.safety.sync]
Process MUST be safe for concurrent use from multiple threads.
