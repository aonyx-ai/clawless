---
sidebar_position: 2
---

# Require Subcommands

Prevent a command from executing without a subcommand, automatically showing
help instead.

:::info[Future API Changes]
The `require_subcommand` attribute and the need for empty command group functions are temporary implementation details. A future release will likely introduce a dedicated command group macro to make this pattern cleaner and more explicit. The current API works but may change.
:::

## When to use this

Use `require_subcommand` when a command only serves as a grouping mechanism for
related subcommands:

```rust
use clawless::prelude::*;

#[derive(Debug, Args)]
pub struct DbArgs {}

/// Database management commands
#[command(require_subcommand)]
pub async fn db(args: DbArgs, context: Context) -> CommandResult {
    // This function body won't execute
    Ok(())
}
```

With `require_subcommand`, running just the parent command shows help:

```bash
$ myapp db
Database management commands

Usage: myapp db <COMMAND>

Commands:
  migrate  Run database migrations
  seed     Seed the database
  reset    Reset the database

Options:
  -h, --help  Print help
```

## Without require_subcommand

Without this attribute, the command function executes when run without a
subcommand:

```rust
/// Database management commands
#[command]  // No require_subcommand
pub async fn db(args: DbArgs, context: Context) -> CommandResult {
    println!("Use a subcommand: migrate, seed, reset");
    Ok(())
}
```

```bash
$ myapp db
Use a subcommand: migrate, seed, reset
```

This approach requires you to implement the function body yourself.

## Common use cases

### Pure command groups

Commands that only organize subcommands and have no independent functionality:

```rust
/// Configuration management
#[command(require_subcommand)]
pub async fn config(args: ConfigArgs, context: Context) -> CommandResult {
    Ok(())
}
```

### Commands with multiple operations

When a command has multiple distinct operations that should be explicitly
chosen:

```rust
/// Cache management operations
#[command(require_subcommand)]
pub async fn cache(args: CacheArgs, context: Context) -> CommandResult {
    Ok(())
}
```

With subcommands like `get`, `put`, and `clear`:

```bash
myapp cache get <key>
myapp cache put <key> <value>
myapp cache clear
```

This prevents accidentally running `myapp cache` without specifying which
operation to perform.

## The function body

When using `require_subcommand`, the function body never executes, so you can
leave it as:

```rust
#[command(require_subcommand)]
pub async fn db(args: DbArgs, context: Context) -> CommandResult {
    Ok(())
}
```

Some developers prefer to make this explicit with `unreachable!()`:

```rust
#[command(require_subcommand)]
pub async fn db(args: DbArgs, context: Context) -> CommandResult {
    unreachable!("This function should never execute due to require_subcommand")
}
```

However, `Ok(())` is the conventional approach and matches the generated code
from `clawless generate command`.

## See also

- [Project Structure](/concepts/project-structure#command-group-functions) - How
  command groups work
- [Add Command Aliases](./add-command-aliases) - Using the `alias` attribute
- [Commands](/concepts/commands#macro-attributes) - All `#[command]` attributes
