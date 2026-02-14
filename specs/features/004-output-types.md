# Output types

- **Project**: [P002-output][project]
- **Dependencies**: none
- **Breaking changes**: none

## Summary

Introduce `Output`, `Verbosity`, and `OutputMode` — the domain types that give
Clawless commands a framework-controlled way to produce output. `Output`
replaces direct `println!` with methods whose behavior varies by verbosity level
and output mode, supporting both human-readable text and machine-readable JSON.

## Motivation

The [architecture] defines [Artifact] and [Progress] as domain concepts that
flow through the [Formatter] port. A simple streaming formatter could be built
today, but the nested formatters that render parallel [Task] output in groups
depend on the Task entity, which does not exist yet. Regardless, commands
already need a way to produce output that the framework can control.

This feature provides the minimal output abstraction: an `Output` struct with
three methods (`print`, `verbose`, `result`) whose behavior varies by
`Verbosity` and `OutputMode`. The types are pure domain objects with no
framework integration; they can be constructed and tested independently. The
integration with [Context] and macros is deferred to
[F005-command-output][command-output].

## Domain concepts

### Verbosity (value object)

`Verbosity` is a value object representing the level of output detail a user
has requested. Three levels:

| Variant   | Meaning                                                       |
| --------- | ------------------------------------------------------------- |
| `Quiet`   | Suppress informational messages; show only results and errors |
| `Default` | Show normal messages and results                              |
| `Verbose` | Show everything including additional detail                   |

`Verbosity` is orthogonal to `OutputMode`. A user can request `--verbose --json`
(verbose messages to stderr, JSON results to stdout) or `--quiet --json`
(suppress messages entirely, JSON results to stdout).

### OutputMode (value object)

`OutputMode` determines the format and destination of output:

| Variant | Messages (`print`, `verbose`) | Results (`result`)                       |
| ------- | ----------------------------- | ---------------------------------------- |
| `Text`  | stdout, formatted via Display | stdout, formatted via Display            |
| `Json`  | stderr, formatted via Display | stdout, serialized as JSON via Serialize |

In text mode, everything goes to stdout — matching the behavior of `println!`
that commands use today. In JSON mode, stdout is reserved for machine-readable
data, so messages redirect to stderr. This matches the convention described in
issue [#153] and used by tools like `gh`, `kubectl`, and `jq`.

### Output (value object)

`Output` is the command-level interface for producing output. It holds a
`Verbosity`, an `OutputMode`, and writer targets. Commands call its methods;
Output decides whether and where to write based on its configuration.

`Output` is a value object in the domain. It has no meaningful identity: two
Output instances with the same configuration are interchangeable.

### Behavior matrix

`Verbosity` controls **whether** a method produces output. `OutputMode` controls
**where** and **how**.

| Method      | Quiet  | Default | Verbose |
| ----------- | ------ | ------- | ------- |
| `print()`   | no-op  | writes  | writes  |
| `verbose()` | no-op  | no-op   | writes  |
| `result()`  | writes | writes  | writes  |

`result()` always produces output regardless of verbosity. It is the primary
output of a command — the data that scripts and users are asking for.

### Relationship to other domain concepts

- **[Context]**: will hold `Output` after [F005][command-output]. Commands
  access output via `context.output()`.
- **[Artifact]** (architecture): `result()` is the first-generation equivalent
  of artifacts, which may eventually be streamed or batched.
- **[Formatter]** (architecture): Output is itself a simple formatter. The name
  "Output" is used because the architecture reserves "Formatter" for the full
  port with nested child instances and swappable adapters.

## Design rationale

### Output, not Formatter

Output is itself a simple formatter, but we call it "Output" because the
architecture reserves "Formatter" for the full port with nested child instances,
structured [Progress]/[Artifact]/[Diagnostic] rendering, and swappable adapters.
Output is the command-level API (`context.output()`) that will eventually be
backed by a Formatter adapter. The name can evolve with the abstraction.

### Verbosity as enum, not bools

Per the project's [coding conventions][conventions], enums with meaningful
variants are preferred over boolean parameters. `Verbosity` with three variants
is clearer than `quiet: bool, verbose: bool` and eliminates the invalid state
where both are `true`.

### Message destination depends on mode

In text mode, messages go to stdout — matching `println!` behavior so that
simple commands work as expected with no changes. In JSON mode, stdout is
reserved for machine-readable data, so messages redirect to stderr. This is a
standard convention for CLI tools that support structured output.

### `result()` accepts Display + Serialize

Command authors implement both traits on their result types. In text mode,
`result()` renders via `Display`, giving authors full control over text
formatting. In JSON mode, `result()` serializes via `Serialize`, giving
structured output for free. This dual-trait approach avoids separate methods
for text and JSON output.

### Derives

**Verbosity** derives `Copy`, `Clone`, `Eq`, `PartialEq`, `Ord`, `PartialOrd`,
`Hash`, `Debug`, `Default`:

- Full set of standard derives: it is a simple enum with no data.
- `Copy`: small, stack-allocated value.
- `Default`: `Verbosity::Default` is the natural default.

**OutputMode** derives `Copy`, `Clone`, `Eq`, `PartialEq`, `Ord`, `PartialOrd`,
`Hash`, `Debug`, `Default`:

- Same reasoning as Verbosity.
- `Default`: `OutputMode::Text` is the natural default.

**Output** derives `Clone` and `Debug`:

- `Clone`: required because [Context] is Clone and will hold Output.
- `Debug`: required for diagnostics and test output.
- **Not** `Eq`, `PartialEq`, etc.: Output holds writer targets where equality
  comparison is not meaningful, similar to how [Cancellation] omits these
  derives.
- **Not** `Default`: constructing Output requires choosing writers. A
  convenience constructor provides the production default (stdout/stderr), but
  the `Default` trait is not implemented because "default Output" is ambiguous
  without knowing the intended mode and verbosity.

### Methods take `&self`

`print()`, `verbose()`, and `result()` take `&self` despite writing internally.
Output uses interior mutability for its writer targets, matching how
[Cancellation] uses `&self` for `cancel()`. This is necessary because Output
will live in [Context], which is passed to commands and may be cloned.

### Methods append newlines

All three methods append a newline after writing. `print()` and `verbose()`
behave like `writeln!`, not `write!`. `result()` appends a newline in both
modes: in text mode for consistency, in JSON mode to produce newline-delimited
JSON (one JSON object per line).

## Functional requirements

1. `Verbosity` has three variants: `Quiet`, `Default`, `Verbose`.
2. `OutputMode` has two variants: `Text`, `Json`.
3. `Output` is constructed with a `Verbosity`, an `OutputMode`, and writer
   targets for messages and results.
4. `Output::new(verbosity, mode)` creates an Output with stdout and stderr as
   writer targets: in text mode, messages go to stdout; in JSON mode, messages
   go to stderr. Results always go to stdout.
5. `Output` provides a test-oriented constructor (or builder) that accepts
   custom writer targets, enabling assertions on output content without I/O
   side effects.
6. `Output::print(message)` writes the message followed by a newline to the
   message writer. It is a no-op if verbosity is `Quiet`.
7. `Output::verbose(message)` writes the message followed by a newline to the
   message writer. It is a no-op unless verbosity is `Verbose`.
8. `Output::result(value)` writes the value to the result writer:
   - In text mode: formats via `Display`, appends a newline.
   - In JSON mode: serializes via `serde_json` to compact JSON, appends a
     newline.
   - `result()` produces output regardless of verbosity.
9. `Verbosity` and `OutputMode` are re-exported from the prelude.
10. `Output` is re-exported from the prelude.
11. `serde::Serialize` is re-exported from the prelude so command authors can
    derive it without adding `serde` as a direct dependency.

## Non-functional requirements

1. **Thread safety**: `Output` must be `Send + Sync + Unpin`.
2. **Testability**: Output can be constructed with in-memory buffer writers for
   assertion-based testing without I/O side effects.
3. **No framework coupling**: Output types can be used and tested independently
   of Context, macros, or any other framework component.

## API surface

### Verbosity

````rust
/// Level of output detail requested by the user
///
/// `Verbosity` controls whether `Output` methods produce output. It is
/// orthogonal to [`OutputMode`], which controls format and destination.
///
/// # Examples
///
/// ```
/// use clawless::prelude::*;
///
/// let verbosity = Verbosity::default();
/// assert_eq!(verbosity, Verbosity::Default);
/// ```
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Default)]
pub enum Verbosity {
    Quiet,
    #[default]
    Default,
    Verbose,
}
````

### OutputMode

````rust
/// Output format and destination strategy
///
/// `OutputMode` controls where messages and results are written and how
/// results are formatted. It is orthogonal to [`Verbosity`], which controls
/// whether output is produced at all.
///
/// # Examples
///
/// ```
/// use clawless::prelude::*;
///
/// let mode = OutputMode::default();
/// assert_eq!(mode, OutputMode::Text);
/// ```
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Default)]
pub enum OutputMode {
    #[default]
    Text,
    Json,
}
````

### Output

````rust
/// Framework-controlled output for commands
///
/// `Output` replaces direct `println!` usage in commands. It provides three
/// methods — `print`, `verbose`, and `result` — whose behavior varies by
/// [`Verbosity`] and [`OutputMode`].
///
/// In text mode, all output goes to stdout. In JSON mode, messages go to
/// stderr and results are serialized as JSON to stdout.
///
/// # Examples
///
/// ```
/// use clawless::prelude::*;
///
/// let output = Output::new(Verbosity::Default, OutputMode::Text);
/// output.print("processing files");
/// output.verbose("scanning directory: /home/user/project");
/// ```
#[derive(Clone, Debug)]
pub struct Output {
    /* verbosity, mode, writer targets */
}
````

### Methods

| Method      | Signature                                                  | Description                       |
| ----------- | ---------------------------------------------------------- | --------------------------------- |
| `new()`     | `fn new(verbosity: Verbosity, mode: OutputMode) -> Output` | Creates Output with stdout/stderr |
| `print()`   | `fn print(&self, message: impl Display)`                   | Writes a message (default+)       |
| `verbose()` | `fn verbose(&self, message: impl Display)`                 | Writes a verbose message          |
| `result()`  | `fn result<T: Display + Serialize>(&self, value: &T)`      | Writes a result value             |
| `verbosity` | `fn verbosity(&self) -> Verbosity`                         | Returns the verbosity level       |
| `mode`      | `fn mode(&self) -> OutputMode`                             | Returns the output mode           |

### Prelude exports

- `Output`
- `Verbosity`
- `OutputMode`
- `serde::Serialize` (so command authors can derive it via the prelude)

## Dependencies

### New workspace dependencies

```toml
# Cargo.toml [workspace.dependencies]
serde = { version = "1", features = ["derive"] }
serde_json = "1"
```

### Crate dependencies

```toml
# crates/clawless/Cargo.toml [dependencies]
serde = { workspace = true }
serde_json = { workspace = true }
```

## File changes

### New files

| File                                        | Contents          |
| ------------------------------------------- | ----------------- |
| `crates/clawless/src/output.rs`             | Module root       |
| `crates/clawless/src/output/verbosity.rs`   | `Verbosity` enum  |
| `crates/clawless/src/output/output_mode.rs` | `OutputMode` enum |

### Modified files

| File                         | Change                                                  |
| ---------------------------- | ------------------------------------------------------- |
| `Cargo.toml`                 | Add `serde`, `serde_json` to `[workspace.dependencies]` |
| `crates/clawless/Cargo.toml` | Add `serde`, `serde_json` dependencies                  |
| `crates/clawless/src/lib.rs` | Add `pub mod output;` and prelude re-exports            |

## Edge cases

| Case                                         | Expected behavior                                                                         |
| -------------------------------------------- | ----------------------------------------------------------------------------------------- |
| `print()` in Quiet mode                      | No-op; nothing is written                                                                 |
| `verbose()` in Default mode                  | No-op; verbose messages are suppressed by default                                         |
| `result()` in Quiet mode                     | Writes normally; results are never suppressed                                             |
| Multiple `result()` calls                    | Each call writes one line (text) or one JSON object (JSON mode, newline-delimited)        |
| `result()` with empty Display                | Writes an empty line (text mode) or `""` (JSON mode)                                      |
| Write failure (broken pipe)                  | Behavior matches `println!` (process terminates)                                          |
| `result()` with type that fails to serialize | Should not occur for well-formed `Serialize` impls; see [open questions](#open-questions) |

## Out of scope

- Context integration (see [F005-command-output][command-output])
- Macro-injected `--quiet`, `--verbose`, `--json` flags
  (see [F005-command-output][command-output])
- Progress reporting (spinners, bars)
- Colored output or terminal-aware formatting
- Pretty-printed JSON (`--json` produces compact, single-line JSON)
- Streaming or batched output strategies

## Open questions

### Serialization failure in `result()`

If a type implements `Serialize` but serialization fails at runtime (e.g., a
map with non-string keys in JSON mode), what should `result()` do?

**Options**:

| Option                | Pros                                                        | Cons                                                                    |
| --------------------- | ----------------------------------------------------------- | ----------------------------------------------------------------------- |
| Panic                 | Simple; matches `println!` on write failure                 | Crashes the command; unhelpful error message                            |
| Return `Result<()>`   | Lets commands propagate with `?`; composable error handling | Adds ceremony to every `result()` call                                  |
| Write error to stderr | Graceful degradation; non-fatal                             | Silently loses the result; may confuse scripts expecting JSON on stdout |

**Recommendation**: panic. Serialization failures for types that implement
`Serialize` are programming errors (like `println!` format mismatches), not
runtime conditions that callers should handle. If this proves too harsh, the
method can be made fallible in a future release.

### Writer abstraction

How should the internal writer targets be represented for `Clone` + `Send` +
`Sync` compatibility?

**Options**:

| Option                                                            | Pros                                         | Cons                                                 |
| ----------------------------------------------------------------- | -------------------------------------------- | ---------------------------------------------------- |
| `Arc<Mutex<dyn Write + Send>>`                                    | Standard pattern; flexible                   | Mutex overhead; custom `Debug` impl needed           |
| Enum with known variants (Stdout, Stderr, Buffer)                 | No trait objects; simple `Clone` and `Debug` | Less flexible for testing; more variants to maintain |
| `Arc<Mutex<Vec<u8>>>` for tests, raw stdio handles for production | Optimized per use case                       | Two construction paths; more complex                 |

**Recommendation**: defer to implementation. The public API (`print`,
`verbose`, `result`) does not expose the writer abstraction. The choice is an
internal implementation detail that can be evaluated during the F004 PR.

[architecture]: ../architecture.md
[artifact]: ../architecture.md#artifact
[cancellation]: ../architecture.md#cancellation
[command-output]: 005-command-output.md
[context]: ../architecture.md#context
[conventions]: ../../CLAUDE.md#enums-over-bools
[diagnostic]: ../architecture.md#diagnostic
[formatter]: ../architecture.md#formatter-output-port
[progress]: ../architecture.md#progress
[project]: ../projects/002-output.md
[task]: ../architecture.md#task
