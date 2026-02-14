# Command output

- **Project**: [P002-output][project]
- **Dependencies**: [F004-output-types][output-types]
- **Breaking changes**: none (additive)

## Summary

Integrate [Output][output-types] into the command lifecycle by embedding it in
[Context] and injecting `--quiet`, `--verbose`, and `--json` global flags via
the `main!()` macro. This is the feature that makes the output system usable by
application authors: after this, every `#[command]` function automatically
receives a `Context` with an `Output` instance already configured from
command-line flags.

## Motivation

[F004][output-types] provides the building blocks: `Output`, `Verbosity`, and
`OutputMode`. But without framework integration, application authors would need
to manually construct `Output`, parse flags, and thread the instance through
their command dispatch. That is exactly the boilerplate the framework should
eliminate.

After this feature, commands replace `println!` with `context.output().print()`
and gain `--quiet`, `--verbose`, and `--json` support with zero setup. The flags
are injected by the macro, parsed before command dispatch, and the resulting
`Output` is embedded in `Context`.

## Domain concepts

### Output embedded in Context

Same reasoning as [Cancellation][cancellation-integration]: every command might
want output, so embedding avoids boilerplate. `Context` evolves from "read-only
environment" to "everything the runtime provides to commands" — CWD
(environment), Cancellation (signal), Output (output channel).

Commands access output via `context.output()`:

```rust
#[command]
pub async fn greet(args: GreetArgs, context: Context) -> CommandResult {
    context.output().print(format!("Hello, {}!", args.name));
    Ok(())
}
```

### Flag semantics

Three global flags control Output configuration:

| Flag        | Short | Effect                    |
| ----------- | ----- | ------------------------- |
| `--quiet`   | `-q`  | Sets `Verbosity::Quiet`   |
| `--verbose` | `-v`  | Sets `Verbosity::Verbose` |
| `--json`    |       | Sets `OutputMode::Json`   |

`--quiet` and `--verbose` are mutually exclusive (Clap conflict). `--json` is
orthogonal and may be combined with either:

| Combination        | Messages                   | Results          |
| ------------------ | -------------------------- | ---------------- |
| (default)          | stdout                     | stdout (Display) |
| `--verbose`        | stdout (including verbose) | stdout (Display) |
| `--quiet`          | suppressed                 | stdout (Display) |
| `--json`           | stderr                     | stdout (JSON)    |
| `--verbose --json` | stderr (including verbose) | stdout (JSON)    |
| `--quiet --json`   | suppressed                 | stdout (JSON)    |

All flags are `global(true)` so they work at any position in the command tree:
`mycli --verbose greet Alice` and `mycli greet --verbose Alice` are equivalent.

## Design rationale

### Flags built inline by the macro

The `main!()` macro constructs the `--quiet`, `--verbose`, and `--json` Clap
args directly in the generated code, the same way it already constructs the
runtime and cancellation token. This keeps all macro-generated setup in one
place and avoids hidden library functions that exist only for macro wiring.

### Alternatives considered

| Alternative                     | Why rejected                                                                                                                                    |
| ------------------------------- | ----------------------------------------------------------------------------------------------------------------------------------------------- |
| Flags in the args struct        | Would require every application to manually add output flags to their root args. Defeats the purpose of framework-level output control.         |
| Separate pre-parse step         | Adds complexity; Clap already supports global args that propagate through the command tree.                                                     |
| Environment variables only      | Less discoverable; flags are the standard CLI UX for output control.                                                                            |
| Hidden library helper functions | Adds indirection between the macro and the flag definitions. Since the flags are simple and unlikely to change, inline construction is clearer. |

### Root token ownership (parallel to Cancellation)

The `main!()` macro constructs `Output` with the parsed verbosity and mode,
just as it creates the root `Cancellation` token. Since the [Application] is
currently implicit (represented by `main!()`), Output is created in the
generated `main` function and passed to `Context`.

## Prerequisite: bon migration

This feature depends on a separate PR (not part of the output project) that
migrates `Context` from `typed-builder` to `bon`. The migration simplifies
`Context` construction and provides a cleaner pattern for adding the `Output`
field. The prerequisite PR must land before this feature is implemented.

## Functional requirements

1. `Context` gains an `output` field of type `Output`, accessible via
   `context.output()`.
2. `Context::try_new()` accepts configuration to construct `Output` (the exact
   signature depends on the bon migration).
3. `Context::builder()` allows setting `Output` explicitly (for tests).
   If not set, the builder provides a sensible default (`Verbosity::Default`,
   `OutputMode::Text`).
4. The `main!()` macro:
   a. Attaches `--quiet`, `--verbose`, and `--json` as global Clap args to the
   root command.
   b. Parses the flags from `ArgMatches` after command resolution.
   c. Constructs `Output` with the resolved `Verbosity` and `OutputMode`.
   d. Passes `Output` (or its configuration) to `Context::try_new()`.
5. `--quiet` and `--verbose` are mutually exclusive. Clap reports a conflict
   error if both are provided.
6. `--json` may be combined with any verbosity level.
7. All existing commands continue to compile and function correctly after
   updating their output calls.

## Non-functional requirements

1. **Compile-time validation**: the `#[command]` macro continues to validate
   two-parameter signatures. No changes to parameter validation are needed.
2. **Zero overhead for commands that ignore output**: a command that never calls
   `context.output()` incurs no runtime cost beyond the `Output` instance being
   present in `Context`.
3. **Help text**: the injected flags appear in `--help` output for every command
   (via `global(true)`), with clear descriptions.

## API surface

### Command usage

```rust
use clawless::prelude::*;
use serde::Serialize;

#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Args)]
pub struct GreetArgs {
    #[arg(default_value = "World")]
    name: String,
}

#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Serialize)]
pub struct GreetResult {
    greeting: String,
}

impl std::fmt::Display for GreetResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.greeting)
    }
}

/// Greet the user
#[command]
pub async fn greet(args: GreetArgs, context: Context) -> CommandResult {
    let greeting = format!("Hello, {}!", args.name);
    context.output().verbose("computing greeting");
    context.output().result(&GreetResult { greeting });
    Ok(())
}
```

### Context changes

```rust
#[derive(Clone, Debug)]
pub struct Context {
    current_working_directory: CurrentWorkingDirectory,
    cancellation: Cancellation,
    output: Output,  // new field
}

impl Context {
    pub fn output(&self) -> &Output { ... }
}
```

### Macro changes

#### `main!()`

```rust
// Generated code (conceptual)
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cancellation = clawless::cancellation::Cancellation::new();

    let rt = clawless::tokio::runtime::Runtime::new()?;
    rt.block_on(async {
        clawless::tokio::spawn(
            clawless::signal::wait_for_shutdown(cancellation.clone())
        );

        let mut app = commands::clawless_init();
        app = app
            .arg(clawless::clap::Arg::new("quiet")
                .long("quiet").short('q').global(true)
                .action(clawless::clap::ArgAction::SetTrue)
                .conflicts_with("verbose")
                .help("Suppress informational messages"))
            .arg(clawless::clap::Arg::new("verbose")
                .long("verbose").short('v').global(true)
                .action(clawless::clap::ArgAction::SetTrue)
                .help("Show verbose output"))
            .arg(clawless::clap::Arg::new("json")
                .long("json").global(true)
                .action(clawless::clap::ArgAction::SetTrue)
                .help("Output results as JSON"));

        let matches = app.get_matches();

        let verbosity = if matches.get_flag("quiet") {
            clawless::output::Verbosity::Quiet
        } else if matches.get_flag("verbose") {
            clawless::output::Verbosity::Verbose
        } else {
            clawless::output::Verbosity::Default
        };
        let mode = if matches.get_flag("json") {
            clawless::output::OutputMode::Json
        } else {
            clawless::output::OutputMode::Text
        };
        let output = clawless::output::Output::new(verbosity, mode);
        let context = clawless::context::Context::try_new(cancellation, output)?;

        commands::clawless_exec(matches, context).await
    })?;

    Ok(())
}
```

## File changes

### Modified files

| File                                | Change                                                               |
| ----------------------------------- | -------------------------------------------------------------------- |
| `crates/clawless/src/context.rs`    | Add `Output` field, update `try_new()`, add `output()` accessor      |
| `crates/clawless/src/lib.rs`        | Update prelude if needed                                             |
| `crates/clawless-derive/src/lib.rs` | Update `main!()` to inject flags, parse them, and construct `Output` |

### Example updates

| File                                         | Change                                           |
| -------------------------------------------- | ------------------------------------------------ |
| `examples/hello-world/src/commands/greet.rs` | Replace `println!` with `context.output().print` |
| `examples/cancellation/src/commands/wait.rs` | Replace `println!` with `context.output().print` |

### Scaffolding template updates

| File                                                   | Change                                                                                                             |
| ------------------------------------------------------ | ------------------------------------------------------------------------------------------------------------------ |
| `crates/clawless-cli/src/commands/new.rs`              | Update template strings: replace `println!` with `context.output().print()` in generated code                      |
| `crates/clawless-cli/src/commands/generate/command.rs` | Update template: generated commands use `context` (not `_context`) since the template now references it for output |

### Test and fixture updates

| File                                               | Change                                                                                   |
| -------------------------------------------------- | ---------------------------------------------------------------------------------------- |
| `crates/clawless-cli/tests/commands/new.out/`      | Updated expected scaffold output to use `context.output().print()` instead of `println!` |
| `crates/clawless-cli/tests/commands/generate.out/` | Updated expected generated command output                                                |
| `crates/clawless-cli/tests/commands/generate.in/`  | Updated input fixture if greet.rs changes                                                |

Help text for all commands will change (new global flags appear in `--help`),
so any trycmd fixtures that assert on help output will need updating.

## Edge cases

| Case                                | Expected behavior                                                                                                |
| ----------------------------------- | ---------------------------------------------------------------------------------------------------------------- |
| `--quiet --verbose` together        | Clap reports a conflict error before command executes                                                            |
| `--json` without `result()` call    | No JSON output on stdout; messages go to stderr as usual                                                         |
| `--quiet` with `result()` call      | Result is printed normally; only messages are suppressed                                                         |
| `--json --quiet` combined           | Messages suppressed entirely; JSON result on stdout                                                              |
| Flag after subcommand               | Works because flags are `global(true)`: `mycli greet --verbose` is equivalent to `mycli --verbose greet`         |
| Command that never calls `output()` | Output instance exists in Context but is unused; no runtime cost                                                 |
| Existing commands with `println!`   | Still compile and work; `println!` bypasses Output. Migration to `output.print()` is recommended but not forced. |

## Out of scope

- Output types and methods (see [F004-output-types][output-types])
- Progress reporting integration
- Colored or styled output
- Custom formatter adapters
- `--no-color` or `--color` flags (future enhancement)
- `CLAWLESS_QUIET` or `CLAWLESS_JSON` environment variable equivalents

## Open questions

None. All design decisions for this feature have been resolved.

[application]: ../architecture.md#application
[architecture]: ../architecture.md
[cancellation-integration]: 003-command-cancellation.md
[context]: ../architecture.md#context
[output-types]: 004-output-types.md
[project]: ../projects/002-output.md
