# Command cancellation

- **Project**: [P001-cancellation][project]
- **Dependencies**:
  - [F001-cancellation-token][cancellation-token]
  - [F002-signal-handling][signal-handling]
- **Breaking changes**: yes (pre-1.0)

## Summary

Integrate [Cancellation] into the command lifecycle by embedding it in
[Context]. The `main!()` macro creates a root token, spawns the signal
handler, and passes the token through `Context` to every command. This is the
feature that makes cancellation usable by application authors.

## Motivation

Features [F001][cancellation-token] and [F002][signal-handling] provide the
building blocks: a token type and a signal-to-token bridge. But without macro
integration, application authors would need to manually create tokens, spawn
signal handlers, and thread the token through their command dispatch. That is
exactly the boilerplate the framework should eliminate.

After this feature, every `#[command]` function automatically receives a
`Context` that carries a `Cancellation` token already wired to OS signals.
Commands can immediately use it for cooperative shutdown with zero setup.

## Domain concepts

### Cancellation embedded in Context

Originally (see "alternatives considered"), cancellation was a separate third
parameter. In practice, this forced every command to declare
`_cancellation: Cancellation` even when unused. Embedding `Cancellation` in
`Context` simplifies the command API to two parameters (`args, context`) while
keeping cancellation fully accessible via `context.cancellation()`.

This means `Context` is no longer purely "read-only environment description" in
the strictest sense: it now also carries the cancellation signal. However,
`Context` already serves as the single runtime-provided value that commands
receive, so housing the cancellation token there is the pragmatic choice that
minimizes API surface and boilerplate.

As a consequence, `Context` loses `Eq`, `PartialEq`, `Ord`, `PartialOrd`, and
`Hash` derives because `CancellationToken` does not implement them. No existing
code depends on `Context` equality.

## Design rationale

### Two-parameter command signature

Commands accept `(args, context)`. Cancellation is accessed via
`context.cancellation()`. This keeps the command signature clean and avoids
forcing every command to declare an unused parameter.

### Alternatives considered

| Alternative                       | Why rejected                                                  |
| --------------------------------- | ------------------------------------------------------------- |
| Third parameter                   | Forces `_cancellation: Cancellation` boilerplate on every     |
|                                   | command, even those that never use it. Originally implemented |
|                                   | but reversed in favor of embedding in Context.                |
| Trait-based injection             | Over-engineered for a single value; adds indirection without  |
|                                   | clear benefit.                                                |
| Optional parameter (detect arity) | Macro complexity increases significantly; implicit behavior   |
|                                   | is harder to understand than explicit parameters.             |
| Global / thread-local             | Violates explicit dependency passing; untestable.             |

### Root token ownership

The `main!()` macro creates the root `Cancellation` token. This
matches the [architecture]: "The Application owns the root token." Since the
Application is currently implicit (represented by `main!()`), the root token is
created in the generated `main` function and passed to `Context::try_new()`.

## Functional requirements

1. The `#[command]` macro accepts functions with exactly two parameters:
   args struct and `Context`.
2. The `commands!()` macro generates a root command that accepts `Context`.
3. The `main!()` macro:
   a. Creates a root `Cancellation` token.
   b. Passes the token to `Context::try_new(cancellation.clone())`.
   c. Spawns `clawless::signal::wait_for_shutdown` as a background task with
   the root token.
   d. Passes `context` to `commands::clawless_exec`.
4. The `ClawlessSubcommands` inventory struct's function pointer includes
   `Context` (which carries `Cancellation`) in its signature.
5. All existing commands continue to compile and function correctly after
   updating their signatures.

## Non-functional requirements

1. **Compile-time validation**: the `#[command]` macro rejects functions that do
   not have exactly two parameters, with a clear error message.
2. **Zero overhead for commands that ignore cancellation**: a command that
   never calls `context.cancellation()` incurs no runtime cost beyond the
   token being present in `Context` (which is a reference count increment on
   clone).

## API surface

### Command signature

```rust
#[command]
pub async fn greet(args: GreetArgs, _context: Context) -> CommandResult {
    println!("Hello, {}!", args.name);
    Ok(())
}
```

### Accessing cancellation

```rust
#[command]
pub async fn wait(_args: WaitArgs, context: Context) -> CommandResult {
    println!("waiting");
    context.cancellation().cancelled().await;
    println!("cancelled");
    Ok(())
}
```

### Macro changes

#### `#[command]`

- Validates two parameters (args struct and context).
- Generated dispatch function passes `Context` to the command body.

#### `commands!()`

- Generated root command function accepts `Context`.

#### `main!()`

```rust
// Generated code (conceptual)
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cancellation = clawless::cancellation::Cancellation::new();
    let context = clawless::context::Context::try_new(cancellation.clone())?;

    let rt = clawless::tokio::runtime::Runtime::new()?;
    rt.block_on(async {
        clawless::tokio::spawn(
            clawless::signal::wait_for_shutdown(cancellation)
        );

        let app = commands::clawless_init();
        commands::clawless_exec(
            app.get_matches(),
            context,
        ).await
    })?;

    Ok(())
}
```

#### `ClawlessSubcommands`

```rust
struct ClawlessSubcommands {
    name: &'static str,
    init: fn() -> clawless::clap::Command,
    func: fn(
        clawless::clap::ArgMatches,
        clawless::context::Context,
    ) -> Pin<Box<dyn Future<Output=clawless::CommandResult>>>,
}
```

## File changes

### Modified files

| File                                                   | Change                                        |
| ------------------------------------------------------ | --------------------------------------------- |
| `crates/clawless/src/context.rs`                       | Add `Cancellation` field, update `try_new()`  |
| `crates/clawless-derive/src/lib.rs`                    | Update `commands!()` and `main!()` output     |
| `crates/clawless-derive/src/command.rs`                | Validate 2 params; update generated dispatch  |
| `crates/clawless-derive/src/inventory.rs`              | Remove `Cancellation` from function signature |
| `examples/hello-world/src/commands/greet.rs`           | Remove `Cancellation` parameter               |
| `examples/hello-world/src/commands/wait.rs`            | Access cancellation via `context`             |
| `crates/clawless-cli/src/commands/new.rs`              | Remove `Cancellation` parameter               |
| `crates/clawless-cli/src/commands/generate.rs`         | Remove `Cancellation` parameter               |
| `crates/clawless-cli/src/commands/generate/command.rs` | Remove `Cancellation` parameter               |

### Test and fixture updates

All test fixtures that exercise command signatures were updated:

- **trycmd fixtures** (`crates/clawless-cli/tests/`): updated to 2-param
  signatures.
- **trybuild expectations** (`crates/clawless-derive/tests/`): updated
  compile-fail tests for 2-param validation; removed the now-passing
  `fail-missing-cancellation` test; renamed
  `fail-missing-context-and-cancellation` to `fail-missing-context`.
- **Scaffolding templates**: the `new` and `generate command` commands contain
  inline template strings that emit source code for new commands, updated to
  generate two-parameter signatures.

## Edge cases

| Case                                     | Expected behavior                                       |
| ---------------------------------------- | ------------------------------------------------------- |
| Command with wrong parameter count       | `#[command]` macro emits a clear compile error          |
| Container command (`require_subcommand`) | Receives and forwards `Context` to child commands       |
| Nested subcommands                       | Each level passes the same `Context` (and thus the same |
|                                          | `Cancellation` token) through                           |
| Command creates child token              | Works via `context.cancellation().child()`; parent is   |
|                                          | unaffected                                              |
| Signal arrives before command starts     | Token is already cancelled when command receives it;    |
|                                          | command can check                                       |
|                                          | `context.cancellation().is_cancelled()` upfront         |

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
