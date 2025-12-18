---
sidebar_position: 1
---

# Add Command Aliases

Create short aliases for frequently used commands to improve the user experience
of your CLI.

## Add a single alias

Use the `alias` attribute on the `#[command]` macro:

```rust
use clawless::prelude::*;

#[derive(Debug, Args)]
pub struct GenerateArgs {
    /// What to generate
    item: String,
}

/// Generate code from templates
#[command(alias = "g")]
pub async fn generate(args: GenerateArgs, context: Context) -> CommandResult {
    println!("Generating {}...", args.item);
    Ok(())
}
```

Now users can run either:

```bash
myapp generate component
myapp g component
```

Both commands work identically. The alias appears in help text:

```bash
$ myapp --help
Commands:
  generate, g  Generate code from templates
```

## Common use cases for aliases

### Short forms for long commands

```rust
/// Initialize a new project
#[command(alias = "i")]
pub async fn initialize(args: InitArgs, context: Context) -> CommandResult {
    // ...
}
```

```bash
myapp initialize  # Full name
myapp i           # Short alias
```

### Single-letter shortcuts for frequent operations

```rust
/// Build the project
#[command(alias = "b")]
pub async fn build(args: BuildArgs, context: Context) -> CommandResult {
    // ...
}

/// Run the application
#[command(alias = "r")]
pub async fn run(args: RunArgs, context: Context) -> CommandResult {
    // ...
}

/// Test the project
#[command(alias = "t")]
pub async fn test(args: TestArgs, context: Context) -> CommandResult {
    // ...
}
```

### Familiar abbreviations from other tools

Match conventions from popular tools your users already know:

```rust
/// List available items
#[command(alias = "ls")]
pub async fn list(args: ListArgs, context: Context) -> CommandResult {
    // ...
}

/// Print working directory
#[command(alias = "pwd")]
pub async fn current_directory(args: DirArgs, context: Context) -> CommandResult {
    // ...
}
```

## Aliases with subcommands

Aliases work with nested commands too:

```rust
// src/commands/db.rs
/// Database management commands
#[command(alias = "d")]
pub async fn db(args: DbArgs, context: Context) -> CommandResult {
    // ...
}
```

Users can use the alias for the parent command:

```bash
myapp db migrate   # Full name
myapp d migrate    # Using alias
```

## Limitations

### One alias per command

Currently, you can only specify one alias per command:

```rust
// ✅ This works
#[command(alias = "g")]
pub async fn generate(args: Args, context: Context) -> CommandResult { ... }

// ❌ Multiple aliases not supported
#[command(alias = "g", alias = "gen")]  // Won't work
pub async fn generate(args: Args, context: Context) -> CommandResult { ... }
```

### Alias conflicts

Make sure aliases don't conflict with existing command names:

```rust
// ❌ Bad: alias conflicts with another command
#[command(alias = "build")]  // If you already have a 'build' command
pub async fn compile(args: Args, context: Context) -> CommandResult { ... }
```

The framework will catch this at runtime with an error.

## See also

- [Commands](/concepts/commands#macro-attributes) - All `#[command]` attributes
- [Require Subcommands](./require-subcommands) - Using `require_subcommand`
  attribute
