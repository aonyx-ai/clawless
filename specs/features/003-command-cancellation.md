# Command cancellation

- **Project**: [P001-cancellation][project]
- **Dependencies**:
  - [F001-cancellation-token][cancellation-token]
  - [F002-signal-handling][signal-handling]
- **Breaking changes**: yes (pre-1.0)

## Summary

Integrate [Cancellation] into the command lifecycle by changing the command
signature from `(Args, Context)` to `(Args, Context, Cancellation)`.
The `main!()` macro creates a root token, spawns the signal handler, and passes
the token through to every command. This is the feature that makes cancellation
usable by application authors.

## Motivation

Features [F001][cancellation-token] and [F002][signal-handling] provide the
building blocks: a token type and a signal-to-token bridge. But without macro
integration, application authors would need to manually create tokens, spawn
signal handlers, and thread the token through their command dispatch. That is
exactly the boilerplate the framework should eliminate.

After this feature, every `#[command]` function automatically receives a
`Cancellation` token that is already wired to OS signals. Commands can
immediately use it for cooperative shutdown with zero setup.

## Domain concepts

### Cancellation as a third parameter

The [architecture] defines [Context] as "read-only environment description: CWD,
config, env vars, services, terminal capabilities." Cancellation is operational,
not environmental. It represents an active shutdown signal, not a description of
the execution environment.

Making `Cancellation` a third parameter rather than embedding it in `Context`
keeps each concept focused:

- **Context**: what the world looks like (read-only, immutable).
- **Cancellation**: whether to stop working (mutable signal, may be triggered at
  any time).

This separation also avoids making `Context` non-`Eq` (since `Cancellation` does
not implement `Eq`).

## Design rationale

### Breaking signature change

Changing the command signature from two parameters to three is a breaking
change. Every existing `#[command]` function must be updated. This is acceptable
because:

- Clawless is pre-1.0. Breaking changes are expected and documented.
- The change is mechanical: add `_cancellation: Cancellation` (or
  `cancellation: Cancellation` if the command uses it) to every command
  function.
- The compiler catches every missed update at build time; there is no risk of
  silent breakage.

### Alternatives considered

| Alternative                       | Why rejected                                                   |
| --------------------------------- | -------------------------------------------------------------- |
| Embed in `Context`                | Context is read-only environment; cancellation is operational. |
|                                   | Would make `Context` non-`Eq`.                                 |
| Trait-based injection             | Over-engineered for a single value; adds indirection without   |
|                                   | clear benefit.                                                 |
| Optional parameter (detect arity) | Macro complexity increases significantly; implicit behavior    |
|                                   | is harder to understand than explicit parameters.              |
| Global / thread-local             | Violates explicit dependency passing; untestable.              |

### Root token ownership

The `main!()` macro creates the root `Cancellation` token. This
matches the [architecture]: "The Application owns the root token." Since the
Application is currently implicit (represented by `main!()`), the root token is
created in the generated `main` function.

## Functional requirements

1. The `#[command]` macro accepts functions with exactly three parameters:
   args struct, `Context`, and `Cancellation`.
2. The `commands!()` macro generates a root command that accepts `Cancellation`.
3. The `main!()` macro:
   a. Creates a root `Cancellation` token.
   b. Spawns `clawless::signal::wait_for_shutdown` as a background task with a
   clone of the root token.
   c. Passes the root token to `commands::clawless_exec`.
4. The `ClawlessSubcommands` inventory struct's function pointer includes
   `Cancellation` in its signature.
5. All existing commands continue to compile and function correctly after
   updating their signatures.

## Non-functional requirements

1. **Compile-time validation**: the `#[command]` macro rejects functions that do
   not have exactly three parameters, with a clear error message.
2. **Zero overhead for commands that ignore cancellation**: a command that
   accepts `_cancellation: Cancellation` but never uses it incurs no runtime
   cost beyond cloning the token (which is a reference count increment).

## API surface

### Command signature (changed)

```rust
// Before
#[command]
pub async fn greet(args: GreetArgs, _context: Context) -> CommandResult {
    println!("Hello, {}!", args.name);
    Ok(())
}

// After
#[command]
pub async fn greet(
    args: GreetArgs,
    _context: Context,
    _cancellation: Cancellation,
) -> CommandResult {
    println!("Hello, {}!", args.name);
    Ok(())
}
```

### Macro changes

#### `#[command]`

- Validates three parameters (was two).
- Generated dispatch function passes `Cancellation` to the command body.

#### `commands!()`

- Generated root command function accepts `Cancellation`.

#### `main!()`

```rust
// Generated code (conceptual)
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let context = clawless::context::Context::try_new()?;
    let cancellation = clawless::cancellation::Cancellation::new();

    let rt = clawless::tokio::runtime::Runtime::new()?;
    rt.block_on(async {
        clawless::tokio::spawn(
            clawless::signal::wait_for_shutdown(cancellation.clone())
        );

        let app = commands::clawless_init();
        commands::clawless_exec(
            app.get_matches(),
            context.clone(),
            cancellation,
        ).await
    })?;

    Ok(())
}
```

#### `ClawlessSubcommands`

```rust
// Before
struct ClawlessSubcommands {
    name: &'static str,
    init: fn() -> clawless::clap::Command,
    func: fn(
        clawless::clap::ArgMatches,
        clawless::context::Context,
    ) -> Pin<Box<dyn Future<Output=clawless::CommandResult>>>,
}

// After
struct ClawlessSubcommands {
    name: &'static str,
    init: fn() -> clawless::clap::Command,
    func: fn(
        clawless::clap::ArgMatches,
        clawless::context::Context,
        clawless::cancellation::Cancellation,
    ) -> Pin<Box<dyn Future<Output=clawless::CommandResult>>>,
}
```

## File changes

### Modified files

| File                                                   | Change                                                |
| ------------------------------------------------------ | ----------------------------------------------------- |
| `crates/clawless-derive/src/lib.rs`                    | Update `commands!()` and `main!()` macro output       |
| `crates/clawless-derive/src/command.rs`                | Validate 3 params; update generated dispatch          |
| `crates/clawless-derive/src/inventory.rs`              | Add `Cancellation` to `ClawlessSubcommands` signature |
| `examples/hello-world/src/commands/greet.rs`           | Add `Cancellation` parameter                          |
| `crates/clawless-cli/src/commands/new.rs`              | Add `Cancellation` parameter                          |
| `crates/clawless-cli/src/commands/generate.rs`         | Add `Cancellation` parameter                          |
| `crates/clawless-cli/src/commands/generate/command.rs` | Add `Cancellation` parameter                          |

### Test and fixture updates

All test fixtures that exercise command signatures must be updated:

- **trycmd fixtures** (`crates/clawless-cli/tests/`): update expected output if
  command help text changes.
- **trybuild expectations** (`crates/clawless-derive/tests/`): update
  compile-fail tests that validate parameter count errors; update compile-pass
  tests to use three parameters.
- **Scaffolding templates**: the `new` and `generate command` commands contain
  inline template strings that emit source code for new commands. These
  templates
  must generate three-parameter signatures. The affected template strings are in
  `crates/clawless-cli/src/commands/new.rs` and
  `crates/clawless-cli/src/commands/generate/command.rs` (already listed in the
  modified files table above).

## Edge cases

| Case                                     | Expected behavior                                       |
| ---------------------------------------- | ------------------------------------------------------- |
| Command with wrong parameter count       | `#[command]` macro emits a clear compile error          |
| Container command (`require_subcommand`) | Receives and forwards `Cancellation` to child commands  |
| Nested subcommands                       | Each level passes the same `Cancellation` token through |
| Command creates child token              | Works via `cancellation.child()`; parent is unaffected  |
| Signal arrives before command starts     | Token is already cancelled when command receives it;    |
|                                          | command can check `cancellation.is_cancelled()` upfront |

## Out of scope

- Task integration (deferred until Task entity is implemented)
- Cancellation-aware progress reporting
- Automatic cancellation of spawned child processes
- Graceful shutdown timeout

## Open questions

None. The design decisions for this feature follow directly from the
decisions made in [F001-cancellation-token][cancellation-token] and
[F002-signal-handling][signal-handling].

[architecture]: ../architecture.md
[cancellation]: ../architecture.md#cancellation
[cancellation-token]: 001-cancellation-token.md
[context]: ../architecture.md#context
[project]: ../projects/001-cancellation.md
[signal-handling]: 002-signal-handling.md
