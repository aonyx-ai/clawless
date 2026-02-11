# Architecture

## Introduction

Clawless uses [domain-driven design][ddd] to define the vocabulary of a
command-line application framework. Every concept in the system has a precise
name and a clear role. This **ubiquitous language** ensures that humans, LLMs,
and the code itself all use the same terms with the same meanings.

The architecture follows a [hexagonal][hexagonal] (ports and adapters) pattern:

- **Domain core**: the stable concepts that define what a CLI application _is_.
  These do not depend on any particular runtime, terminal, or I/O mechanism.
- **Ports**: interfaces where behavior varies by environment. The domain
  declares _what_ it needs; ports define the contract.
- **Adapters**: concrete implementations that plug into ports. Swappable per
  environment (interactive terminal, CI, testing, scripting).

This separation keeps the domain testable and portable. A command's logic does
not know whether it is running in a color terminal, a CI pipeline, or a test
harness.

[ddd]: https://en.wikipedia.org/wiki/Domain-driven_design
[hexagonal]: https://en.wikipedia.org/wiki/Hexagonal_architecture_(software)

## Architecture overview

```text
                         ┌──────────────────────────────────┐
                         │         Domain core              │
                         │                                  │
                         │  Application  Command  Argument  │
                         │  Context      Task     Hook      │
                         │                                  │
                         │  Progress   Artifact   Diagnostic│
                         │  Outcome    Cancellation         │
                         │                                  │
                         └──────┬──────────────┬────────────┘
                                │              │
                      ┌─────────▼──┐     ┌─────▼──────────┐
                      │  Formatter │     │    Prompt      │
                      │   (port)   │     │    (port)      │
                      └─────┬──────┘     └──────┬─────────┘
                            │                   │
              ┌─────────────┼───────┐     ┌─────┼──────────┐
              ▼             ▼       ▼     ▼     ▼          ▼
         ┌─────────┐  ┌─────────┐ ┌────┐ ┌───┐ ┌───┐  ┌──────┐
         │Terminal │  │  CI /   │ │JSON│ │TTY│ │Env│  │ Test │
         │(colors, │  │  plain  │ │    │ │   │ │var│  │      │
         │ layout) │  │         │ │    │ │   │ │   │  │      │
         └─────────┘  └─────────┘ └────┘ └───┘ └───┘  └──────┘
           adapter      adapter  adapter adapter adapter adapter
```

Arrows flow outward from the domain through ports to adapters. The domain never
depends on a specific adapter. Adapters depend on the port interface.

## Domain model

Clawless is built around 13 concepts organized into three layers.

### Core entities

Identity matters. These have lifecycles.

| Entity      | Role                                                                         |
| ----------- | ---------------------------------------------------------------------------- |
| Application | Aggregate root — owns the command tree, hooks, and context factory           |
| Command     | A node in the command tree; accepts arguments, produces an outcome           |
| Argument    | Declarative input parsed from argv before execution                          |
| Context     | Injected environment: CWD, config, env vars, services, terminal capabilities |
| Task        | Unit of work within a command; opt-in, parallelizable, cancellable           |
| Hook        | Cross-cutting lifecycle behavior (ordered pipeline)                          |

### Value objects

Identity does not matter. These are data, state, or tokens.

| Value object | Role                                                                         |
| ------------ | ---------------------------------------------------------------------------- |
| Progress     | Ephemeral status data emitted by Tasks (percentage, message, step)           |
| Artifact     | Structured result data produced by Tasks; may be streamed or batched         |
| Diagnostic   | Rich error/warning info: message, cause chain, context, suggestion, severity |
| Outcome      | Final result of command execution; maps to exit code                         |
| Cancellation | Token-based shutdown signal; Tasks observe, framework manages                |

Cancellation is a value object (a token) in the domain. The mapping from OS
signals to cancellation tokens is infrastructure.

### Ports

Interfaces with swappable adapters. The domain declares intent; the adapter
decides how to fulfill it.

| Port      | Direction | Role                                                         |
| --------- | --------- | ------------------------------------------------------------ |
| Formatter | Output    | Renders Progress, Artifacts, and Diagnostics for the user    |
| Prompt    | Input     | Resolves runtime input needs; behavior varies by environment |

## Entity definitions

### Application

The aggregate root. Configures the CLI program.

- **Metadata**: name, version, description, author
- **Commands**: the root of the command tree
- **Hooks**: ordered lifecycle pipeline
- **Cancellation**: signal handling, shutdown timeout/cleanup
- **Context factory**: how to build the execution context
- **Formatter**: the selected Formatter adapter

### Command

A node in the command tree.

- **Identity**: name, aliases
- **Description**: short + long (from doc comments)
- **Arguments**: the typed input this command accepts
- **Children**: subcommands (forming a tree)
- **Behavior**: async function body; optionally spawns Tasks. A command's
  behavior may be a long-running interactive loop that repeatedly spawns Tasks
  and solicits Prompts. The framework provides the primitives (Task, Prompt,
  Cancellation) that such a loop uses; the session lifecycle is managed by the
  command itself.

### Argument

Declarative, upfront input parsed from argv before execution begins.

Three species:

- **Positional**: ordered, unnamed (`git clone <url>`)
- **Flag**: boolean toggle (`--verbose`, `-v`)
- **Option**: named value (`--output file.txt`)

Properties: name, type, default, required, description, validation, conflicts.

Long-term goal: Clawless-owned abstraction replacing Clap, without writing a
custom parser.

### Context

Injected environment. Everything a command needs that isn't from argv.

Context is read-only. It describes the environment in which a command executes,
not the application's mutable state. Mutable shared state is an
application-level concern; the mechanism for managing it is intentionally
deferred until real usage patterns emerge.

- Working directory
- Configuration (hierarchical: global, project, env, flags)
- Environment variables
- Shared services (HTTP client, DB pool, etc.)
- Terminal capabilities (color, width, interactive, piped)

### Hook

Cross-cutting lifecycle behavior. Ordered pipeline.

Lifecycle points:

- `before_parse`: before argv is parsed
- `after_parse`: after arguments are resolved
- `before_execute`: before command runs
- `after_execute`: after command completes (success or failure)
- `on_error`: when a diagnostic is raised

Use cases: logging, auth, `--dry-run`, telemetry, retry.

Registration: attribute-based initially, builder pattern long-term.

### Task

A unit of work within a command. Opt-in: simple commands run inline without
creating Tasks explicitly.

- **Description**: what this task does (used by Formatter for Progress)
- **Behavior**: async work
- **Cancellation**: observes tokens
- **Children**: can spawn sub-tasks
- Emits: Progress, Artifacts, Diagnostics
- Can solicit: Prompts

### Progress

Ephemeral status data emitted by Tasks. Progress is a **domain value object**:
a command updates it ("60% done", "processing file X"). The Formatter port
decides how to render it (spinner, bar, step indicator).

- Spinners, progress bars, step indicators, counters
- Multi-task parallel progress display
- Ephemeral: replaced/cleared after task completes
- Consumed by the Formatter port for rendering

### Artifact

Structured result data produced by Tasks.

- Typed, serializable (JSON, YAML, table, plain text)
- Machine-readable (for piping, scripting)
- Composable (multiple tasks produce merged artifacts)
- Consumed by the Formatter port for rendering

A Task produces Artifacts over the course of its execution. They may arrive as
a **stream** — one at a time, as work progresses. The Formatter adapter decides
the rendering strategy: immediate (print each Artifact as it arrives) or
batched (collect and present at the end). Streaming is not a separate concept;
it is simply Artifacts produced over time.

### Diagnostic

Rich error/warning information raised by Tasks.

- **Message**: what went wrong
- **Cause chain**: underlying errors
- **Context**: what was happening ("while reading config.toml")
- **Suggestion**: what to do ("did you mean --output?")
- **Severity**: fatal, warning, hint
- **Code**: machine-readable identifier
- Consumed by the Formatter port for rendering

### Outcome

Final result of command execution.

- **Exit code**: 0 = success, non-zero = failure
- **Produced by**: Command after Tasks complete
- **Contains**: aggregated Artifacts, any Diagnostics
- Maps to process exit code

### Cancellation

Token-based shutdown signal. Cancellation has a clear domain/infrastructure
split:

- **Domain**: a cancellation token is a value object. Tasks observe it and
  perform graceful shutdown when signaled. Application defines shutdown behavior
  (timeout, cleanup).
- **Infrastructure**: OS signal handling (SIGINT, SIGTERM) creates cancellation
  tokens. This mapping lives outside the domain core.

Cancellation tokens form a tree. The Application owns the root token. A Command
or Task may create a child token scoped to a unit of work (e.g., one REPL turn).
Cancelling a child stops that unit without affecting the parent. Cancelling the
root triggers shutdown of all outstanding work.

Tasks do not know _why_ they were cancelled — only that the token was signaled.

## Port definitions

### Formatter (output port)

The Formatter controls all output. The domain emits structured data (Progress,
Artifacts, Diagnostics); the Formatter adapter decides how to present it.

Formatters form a tree that mirrors the Task tree. Each Task receives its own
Formatter instance, created by its parent's Formatter. A Task writes Progress,
Artifacts, and Diagnostics to its Formatter without knowing the rendering
strategy. The root Formatter (owned by the Application) coordinates rendering
across all children.

**Interface** (what the domain sees):

- Creates child Formatter instances (one per child Task)
- Receives Progress updates from the owning Task
- Receives Artifacts produced by the owning Task (possibly as a stream)
- Receives Diagnostics raised by the owning Task
- Renders Prompt interactions on behalf of the Prompt port

**Example adapters**:

| Adapter  | Behavior                                                   |
| -------- | ---------------------------------------------------------- |
| Terminal | Colors, layout, screen regions, redrawing; adapts to width |
| CI       | Plain text, no cursor control, sequential output           |
| JSON     | Machine-readable output structured by task                 |
| Test     | Captures output for assertions; no side effects            |

### Prompt (input port)

Prompt is a port, not a domain entity. A command declares "I need this
information" — the Prompt adapter decides how to obtain it.

**Interface** (what the domain sees):

- **What**: a description of the information needed, plus optional structured
  metadata that adapters may use for rendering (e.g., a tool name, arguments,
  and risk level for an approval prompt)
- **Type**: text, confirmation, selection, password
- **Default**: optional fallback value
- **Validation**: constraints on acceptable answers

**Example adapters**:

| Adapter         | Behavior                                                                                                                           |
| --------------- | ---------------------------------------------------------------------------------------------------------------------------------- |
| Interactive     | Renders questions through the Formatter, collects user answers                                                                     |
| Non-interactive | Resolves programmatically (environment variables, defaults, policy-based auto-resolution); errors if required input is unavailable |
| Test            | Returns preconfigured answers for deterministic testing                                                                            |

The domain is not aware of _how_ the answer is obtained. A Prompt for a
database name might be answered by a terminal question, an environment
variable, or a test fixture — the command's logic is identical in all cases.

## Relationships

```text
Application
├── Commands (tree)
│   ├── Arguments (per command)
│   └── Behavior → optionally spawns Tasks
├── Hooks (lifecycle pipeline)
├── Cancellation (framework-level, propagated to Tasks)
├── Context (built once, injected into commands)
└── Formatter adapter (selected at startup)

                      ┌──── port boundary ────┐

Task emissions:       domain                  │  adapter
  Task → Progress  ─┐                         │
  Task → Artifact   ├──→ Formatter port ──────┼──→ Terminal / CI / JSON / Test
  Task → Diagnostic ┘                         │

Task interactions:    domain                  │  adapter
  Task → Prompt port ─────────────────────────┼──→ Interactive / Env / Test
         ◄── answer ──────────────────────────┤

Task lifecycle:
  Task → observes Cancellation token → graceful shutdown

Command → Tasks → Outcome
```

Arrows that cross the port boundary are the seams where adapters are swapped.

## Lifecycle

```text
 1. Application starts
 2. Context is built (CWD, config, env, terminal capabilities)
 3. Hooks::before_parse()
 4. Arguments parsed from argv
 5. Command resolved (routing through tree)
 6. Hooks::before_execute(command, args, context)
 7. Command executes
    - Optionally spawns Tasks (or runs inline for simple commands)
    - Tasks emit Progress → Formatter port → adapter renders
    - Tasks produce Artifacts → Formatter port → adapter renders
    - Tasks solicit Prompts → Prompt port → adapter resolves
    - Tasks observe Cancellation → graceful shutdown
    - Tasks may raise Diagnostics → Formatter port → adapter renders
 8. Command produces Outcome
 9. Hooks::after_execute(outcome)
10. Hooks::on_error(diagnostic)  [if applicable]
11. Application exits with Outcome's exit code
```

Step 7's emissions all flow through ports. The command does not interact with
the terminal, the CI system, or the test harness directly.

## Current state

| Entity       | Current implementation                 | Status                             |
| ------------ | -------------------------------------- | ---------------------------------- |
| Application  | `main!()` macro                        | Implicit, no first-class type      |
| Command      | `#[command]` async fn                  | Emergent from fn + attrs + module  |
| Argument     | Clap `#[derive(Args)]`                 | Fully delegated to Clap            |
| Context      | `Context` struct (CWD only)            | Exists but minimal                 |
| Prompt       | Not represented                        | Missing                            |
| Hook         | Not represented                        | Missing                            |
| Task         | Not represented                        | Missing; commands are monolithic   |
| Cancellation | Not represented                        | Missing                            |
| Progress     | Not represented                        | Missing                            |
| Artifact     | Not represented                        | Missing; commands print directly   |
| Diagnostic   | `anyhow::Result`                       | Exists but unstructured            |
| Outcome      | `CommandResult` = `anyhow::Result<()>` | Thin alias, no exit code control   |
| Formatter    | Not represented                        | Missing; commands own their output |

## Open questions

### Stdin placement

Piped data (`echo "input" | mycli process`) is fundamentally different from
interactive prompts. It does not fit under the Prompt port: piped data is a
byte stream available at startup, not a question-and-answer interaction during
execution.

Options:

- **Part of Context**: stdin as an input stream available in the execution
  environment, alongside CWD and config.
- **Separate Input port**: a dedicated port for reading streaming input, with
  adapters for pipes, files, and test fixtures.
- **Something else**: an approach not yet considered.

### Output ergonomics

Should simple commands still be able to call `println!`, or must all output
flow through the Formatter port?

A strict "everything through the Formatter" rule ensures consistent rendering,
but adds ceremony for commands that just want to print a line. A layered
approach — convenience helpers that route through the Formatter port
transparently — might offer both correctness and ergonomics.
