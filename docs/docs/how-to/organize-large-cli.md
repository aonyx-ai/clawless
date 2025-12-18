---
sidebar_position: 3
---

# Organize Large CLI Applications

Learn how to structure your Clawless project as it grows, separating CLI
interface from business logic.

## Commands are your interface

The `src/commands/` directory defines your CLI's interface - the commands users
can run. Keep command functions thin, using them to parse arguments and call
your application logic:

```rust
// src/commands/deploy.rs
use clawless::prelude::*;
use crate::deployment;  // Business logic in separate module

#[derive(Debug, Args)]
pub struct DeployArgs {
    environment: String,
}

/// Deploy to an environment
#[command]
pub async fn deploy(args: DeployArgs, context: Context) -> CommandResult {
    // Command function just handles CLI concerns
    deployment::run(&args.environment, context.current_working_directory()).await?;
    Ok(())
}
```

## Put business logic in separate modules

Create modules outside `src/commands/` for your actual application logic:

```
src/
├── main.rs
├── commands.rs
├── commands/
│   ├── deploy.rs       // CLI interface
│   ├── build.rs        // CLI interface
│   └── test.rs         // CLI interface
├── deployment/         // Business logic
│   ├── mod.rs
│   ├── validator.rs
│   └── runner.rs
├── builder/            // Business logic
│   ├── mod.rs
│   └── compiler.rs
└── testing/            // Business logic
    ├── mod.rs
    └── runner.rs
```

The command functions become thin wrappers:

```rust
// src/commands/build.rs
use crate::builder;

#[command]
pub async fn build(args: BuildArgs, context: Context) -> CommandResult {
    let config = builder::Config::from_args(&args)?;
    builder::compile(config).await?;
    Ok(())
}
```

This separation means:

- Business logic can be tested without CLI dependencies
- Logic can be reused across different commands
- Command structure stays clean and focused on interface

## Grouping commands

As you add commands, group related ones together using nested modules:

```
src/commands/
├── build.rs
├── test.rs
├── config.rs
└── config/
    ├── get.rs
    ├── set.rs
    └── list.rs
```

This creates the command structure:

```bash
myapp build
myapp test
myapp config get
myapp config set
myapp config list
```

Each command is still thin, calling into your business logic modules in `src/`:

```rust
// src/commands/config/get.rs
use crate::config_manager;  // Business logic

#[command]
pub async fn get(args: GetArgs, context: Context) -> CommandResult {
    let value = config_manager::get_value(&args.key)?;
    println!("{}", value);
    Ok(())
}
```

## Example structure for a large CLI

A deployment tool with multiple domains:

```
src/
├── main.rs
├── commands.rs
├── commands/              # CLI interface layer
│   ├── init.rs
│   ├── project.rs
│   ├── project/
│   │   ├── create.rs
│   │   └── delete.rs
│   ├── deploy.rs
│   └── deploy/
│       ├── staging.rs
│       └── production.rs
│
├── projects/             # Business logic
│   ├── mod.rs
│   ├── creator.rs
│   ├── deleter.rs
│   └── validator.rs
│
├── deployment/           # Business logic
│   ├── mod.rs
│   ├── pipeline.rs
│   ├── health_check.rs
│   └── rollback.rs
│
├── config/               # Business logic
│   ├── mod.rs
│   └── loader.rs
│
└── api/                  # Business logic
    ├── mod.rs
    └── client.rs
```

The commands just wire up the interface:

```rust
// src/commands/project/create.rs
use crate::projects;

#[command]
pub async fn create(args: CreateArgs, context: Context) -> CommandResult {
    projects::Creator::new(&args.name)
        .in_directory(context.current_working_directory())
        .create()
        .await?;
    Ok(())
}
```

## Shared code within command groups

If you need to share code between commands in the same group, create a
non-command module:

```
src/commands/
├── database.rs
└── database/
    ├── migrate.rs      # Command
    ├── seed.rs         # Command
    ├── reset.rs        # Command
    └── helpers.rs      # NOT a command - just shared utilities
```

In `database/helpers.rs`:

```rust
// This is NOT a command - it doesn't have #[command]
// Just regular Rust code shared by commands in this group

pub fn parse_connection_string(s: &str) -> Result<DbConfig, Error> {
    // Parsing logic used by multiple database commands
}
```

Use from commands:

```rust
// src/commands/database/migrate.rs
use super::helpers;  // Import from the same module

#[command]
pub async fn migrate(args: MigrateArgs, context: Context) -> CommandResult {
    let config = helpers::parse_connection_string(&args.connection)?;
    // ...
}
```

Note: Don't forget to declare the module in the parent:

```rust
// src/commands/database.rs
mod migrate;
mod seed;
mod reset;
mod helpers;  // Declare non-command modules too

// ... rest of database.rs
```

## When to create command groups

Create nested command structures when you have multiple related commands that
share a conceptual grouping. There's no hard rule - use groups when they make
the CLI more intuitive for your users.

The file structure flexibility is there when you need it. Start with flat
structure, add grouping when it improves clarity.

## See also

- [Project Structure](../concepts/project-structure) - How file structure maps
  to commands
- [Generate Commands](./generate-commands) - Creating commands and groups with
  the CLI
- [Require Subcommands](./require-subcommands) - Enforcing subcommand usage in
  groups
