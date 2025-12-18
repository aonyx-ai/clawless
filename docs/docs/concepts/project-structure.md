---
sidebar_position: 4
---

# Project Structure

In Clawless, your file structure directly maps to your command structure. This convention makes your CLI's organization immediately clear from the codebase layout.

## The basic structure

Every Clawless project follows this structure:

```
myapp/
├── src/
│   ├── main.rs          # Application entry point
│   ├── commands.rs      # Commands module setup
│   └── commands/        # Commands directory
│       ├── greet.rs     # Top-level command: myapp greet
│       └── deploy.rs    # Top-level command: myapp deploy
└── Cargo.toml
```

**Key files:**

- **`src/main.rs`** - Contains `mod commands;` and `clawless::main!()`
- **`src/commands.rs`** - Declares command modules and calls `clawless::commands!()`
- **`src/commands/*.rs`** - Individual command files with `#[command]` functions

## File structure = Command structure

The convention is simple: **module hierarchy becomes subcommand hierarchy**.

### Single-level commands

```
src/
├── main.rs
├── commands.rs
└── commands/
    ├── greet.rs
    ├── deploy.rs
    └── version.rs
```

Results in:

```bash
myapp greet
myapp deploy
myapp version
```

Each file contains one command function marked with `#[command]`.

### Nested commands (subcommands)

```
src/
├── main.rs
├── commands.rs
└── commands/
    ├── greet.rs
    ├── db.rs
    └── db/
        ├── migrate.rs
        ├── seed.rs
        └── reset.rs
```

Results in:

```bash
myapp greet
myapp db migrate
myapp db seed
myapp db reset
```

The `db/` directory becomes a command group. Commands inside it become subcommands of `db`.

### Multi-level nesting

```
src/
├── main.rs
├── commands.rs
└── commands/
    ├── config.rs
    ├── config/
    │   ├── get.rs
    │   ├── set.rs
    │   ├── auth.rs
    │   └── auth/
    │       ├── login.rs
    │       └── logout.rs
    ├── deploy.rs
    └── deploy/
        ├── staging.rs
        └── production.rs
```

Results in:

```bash
myapp config get
myapp config set
myapp config auth login
myapp config auth logout
myapp deploy staging
myapp deploy production
```

Nesting can go as deep as needed, though more than 2-3 levels is rare in practice.

## Command group functions

When you have a nested directory like `db/`, you must create a command function in `db.rs`:

```rust
// src/commands/db.rs
mod migrate;
mod seed;

use clawless::prelude::*;

#[derive(Debug, Args)]
pub struct DbArgs {}

/// Database management commands
#[command]
pub async fn db(_args: DbArgs, context: Context) -> CommandResult {
    // This executes when user runs: myapp db
    println!("Use a subcommand: migrate, seed");
    Ok(())
}
```

Optionally, add `require_subcommand` to skip executing the function and show help instead:

```rust
/// Database management commands
#[command(require_subcommand)]
pub async fn db(_args: DbArgs, context: Context) -> CommandResult {
    // This won't execute - require_subcommand shows help instead
    Ok(())
}
```

With `require_subcommand`, running `myapp db` shows help for the db subcommands. Without it, the function executes.

## Organizing large CLIs

As your CLI grows, organize related commands into logical groups:

```
src/
├── main.rs
├── commands.rs
└── commands/
    ├── config.rs        # Configuration commands (parent)
    ├── config/
    │   ├── get.rs
    │   ├── set.rs
    │   └── list.rs
    ├── deploy.rs        # Deployment commands (parent)
    ├── deploy/
    │   ├── staging.rs
    │   └── production.rs
    ├── db.rs            # Database commands (parent)
    ├── db/
    │   ├── migrate.rs
    │   ├── seed.rs
    │   └── rollback.rs
    ├── user.rs          # User management (parent)
    └── user/
        ├── create.rs
        ├── delete.rs
        └── list.rs
```

This structure gives you:

```bash
myapp config get
myapp config set
myapp deploy staging
myapp deploy production
myapp db migrate
myapp user create
```

## Naming

Follow Rust's standard naming conventions (snake_case for files, directories, and functions). Clawless automatically converts snake_case to kebab-case for CLI commands:

- `deploy_staging.rs` → `myapp deploy-staging`
- `user_management.rs` → `myapp user-management`
- `fn deploy_staging()` → `myapp deploy-staging`

See [Naming Conventions](./naming-conventions) for details.

## Examples

### Simple CLI

```
src/
├── main.rs
├── commands.rs
└── commands/
    ├── init.rs
    ├── build.rs
    └── test.rs
```

Commands: `myapp init`, `myapp build`, `myapp test`

### Medium CLI with grouping

```
src/
├── main.rs
├── commands.rs
└── commands/
    ├── init.rs
    ├── config.rs
    ├── config/
    │   ├── get.rs
    │   └── set.rs
    ├── plugin.rs
    └── plugin/
        ├── install.rs
        └── remove.rs
```

Commands: `myapp init`, `myapp config get`, `myapp config set`, `myapp plugin install`, `myapp plugin remove`

### Large CLI with deep nesting

```
src/
├── main.rs
├── commands.rs
└── commands/
    ├── project.rs
    ├── project/
    │   ├── create.rs
    │   ├── delete.rs
    │   ├── settings.rs
    │   └── settings/
    │       ├── show.rs
    │       └── update.rs
    ├── deployment.rs
    ├── deployment/
    │   ├── list.rs
    │   ├── create.rs
    │   ├── logs.rs
    │   └── logs/
    │       ├── view.rs
    │       └── download.rs
    ├── user.rs
    └── user/
        ├── login.rs
        └── logout.rs
```

Commands: `myapp project create`, `myapp project settings show`, `myapp deployment logs view`, etc.

## What's next

Now that you understand project structure, learn about:

- **[Naming Conventions](./naming-conventions)** - How file and function names become command names
- **[Commands](./commands)** - How to write command functions
- **[Macros](./macros)** - How the structure is wired together
