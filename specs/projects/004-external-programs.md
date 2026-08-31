# External programs

## Summary

A first-class interface for running external programs from a Clawless command.
The interface streams the output of a program through the [event system][event]
as the program writes it, so that a [presenter][presenter-rendering] can show a
long-running tool while it runs, and a [projection][projection] can keep the
last few lines of it on screen.

## Motivation

Most command-line applications drive other programs. A release tool calls
`git`, a scaffolding tool calls `cargo`, and a deployment tool calls the client
of its provider. Today a Clawless command reaches for `std::process::Command`
or `tokio::process::Command` and then has to solve the same four problems every
time:

- **Pipes that fill.** A program that writes more than a pipe holds stops until
  a reader empties it. A command that waits for the end before it reads
  deadlocks on a talkative program.
- **Output that arrives too late.** A command that collects the output and
  prints it afterwards shows nothing for two minutes and then everything at
  once.
- **Programs that outlive the command.** A command that is cancelled leaves its
  program running unless it kills the program itself.
- **Errors that say nothing.** "exit status 1" does not tell a user which
  command failed or what it said.

The event system solves the second problem for a command's own output already.
This project connects a running program to it: the lines of the program become
events, and the presenter decides what to do with them. The command's code is
the same whether it runs under a stateless CLI or a full-screen TUI.

## Feature specs

Each feature spec maps to one PR.

| #    | Spec                              | Depends on | Summary                                   |
| ---- | --------------------------------- | ---------- | ----------------------------------------- |
| F012 | [F012-run-programs][run-programs] |            | `Process`, process events, live streaming |

## Out of scope

The first feature runs a program and reports it. These extensions are
deliberately left for later, when a real application asks for them:

- Setting or clearing environment variables for a program.
- Writing to the standard input of a program.
- Running a pipeline of programs.
- A presenter that shows the output of a program at default verbosity.

[event]: ../event.md
[presenter-rendering]: ../features/009-presenter-rendering.md
[projection]: ../projection.md
[run-programs]: ../features/012-run-programs.md
