---
sidebar_position: 1
---

# Manual Setup

This guide shows how to set up Clawless manually without using the `clawless-cli`
scaffolding tool. This is useful if you want to add Clawless to an existing
project or prefer to understand the setup process.

## Prerequisites

You need Rust installed. Any recent stable version will work.

## Step 1: Create a new crate

If you're starting fresh, create a new binary crate:

```bash
cargo new my-cli
cd my-cli
```

## Step 2: Add Clawless as a dependency

```bash
cargo add clawless
```

Or add it manually to your `Cargo.toml`:

```toml
[dependencies]
clawless = "0.4"
```

## Step 3: Set up main.rs

Replace the contents of `src/main.rs` with:

```rust
mod commands;

clawless::main!();
```

The `main!()` macro generates the application entry point. It:

- Creates the Tokio async runtime
- Initializes the Context
- Parses command-line arguments
- Routes to the appropriate command

## Step 4: Create commands.rs

Create `src/commands.rs`:

```rust
mod greet;

clawless::commands!();
```

The `commands!()` macro sets up the root command and collects all subcommands
from the modules declared above it. Each `mod` statement adds a command to your
CLI.

## Step 5: Create your first command

Create the directory `src/commands/` and add `src/commands/greet.rs`:

```rust
use clawless::prelude::*;

#[derive(Debug, Args)]
pub struct GreetArgs {
    /// Name of the person to greet
    #[arg(default_value = "World")]
    name: String,
}

/// Greet someone by name
#[command]
pub async fn greet(args: GreetArgs, context: Context) -> CommandResult {
    println!("Hello, {}!", args.name);
    Ok(())
}
```

## Step 6: Run your CLI

```bash
cargo run -- greet
cargo run -- greet Rust
cargo run -- --help
```

## Final project structure

Your project should look like this:

```
my-cli/
├── Cargo.toml
└── src/
    ├── main.rs
    ├── commands.rs
    └── commands/
        └── greet.rs
```

## Adding more commands

To add a new command:

1. Create a new file in `src/commands/`, e.g., `src/commands/deploy.rs`
2. Add `mod deploy;` to `src/commands.rs` (before the `commands!()` macro)
3. Implement the command with `#[command]`

For nested commands like `myapp db migrate`:

1. Create `src/commands/db.rs` with a parent command
2. Create `src/commands/db/migrate.rs` with the subcommand
3. Add `mod migrate;` to `db.rs`
4. Add `mod db;` to `commands.rs`

See [Project Structure](./concepts/project-structure) for more details on
organizing commands.

## What's next

- **[Quick Start](./quick-start)** - See the scaffolding workflow
- **[Commands](./concepts/commands)** - Learn about command functions
- **[Arguments](./concepts/arguments)** - Define command inputs
