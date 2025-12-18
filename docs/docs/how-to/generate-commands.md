---
sidebar_position: 0
---

# Generate Commands

Use the Clawless CLI to quickly generate new commands without manually creating
files and updating module declarations.

## Prerequisites

Install the Clawless CLI if you haven't already:

```bash
cargo install clawless-cli
```

## Generate a top-level command

To create a new command at the top level of your CLI:

```bash
clawless generate command <name>
```

For example, to add a `version` command:

```bash
clawless generate command version
```

This creates `src/commands/version.rs` with boilerplate code:

```rust
use clawless::prelude::*;

#[derive(Debug, Args)]
pub struct VersionArgs {}

#[command]
pub async fn version(args: VersionArgs, context: Context) -> CommandResult {
    todo!()
}
```

It also updates `src/commands.rs` to include the module declaration:

```rust
mod greet;
mod version;  // Added automatically

clawless::commands!();
```

Now you can implement the command:

```rust
use clawless::prelude::*;

#[derive(Debug, Args)]
pub struct VersionArgs {}

/// Display version information
#[command]
pub async fn version(_args: VersionArgs, _context: Context) -> CommandResult {
    println!("myapp v{}", env!("CARGO_PKG_VERSION"));
    Ok(())
}
```

Run it:

```bash
cargo run -- version
```

## Generate a nested command

To create a command within a command group, first create the parent command,
then the nested command.

### Step 1: Create the parent command

```bash
clawless generate command db
```

This creates `src/commands/db.rs`:

```rust
use clawless::prelude::*;

#[derive(Debug, Args)]
pub struct DbArgs {}

#[command]
pub async fn db(args: DbArgs, context: Context) -> CommandResult {
    todo!()
}
```

### Step 2: Create the nested command

Now you can create a subcommand using a path with `/`:

```bash
clawless generate command db/migrate
```

This creates `src/commands/db/migrate.rs` and updates the parent module.

The parent command `src/commands/db.rs`:

```rust
mod migrate;

use clawless::prelude::*;

#[derive(Debug, Args)]
pub struct DbArgs {}

#[command]
pub async fn db(args: DbArgs, context: Context) -> CommandResult {
    todo!()
}
```

The subcommand `src/commands/db/migrate.rs`:

```rust
use clawless::prelude::*;

#[derive(Debug, Args)]
pub struct MigrateArgs {}

#[command]
pub async fn migrate(args: MigrateArgs, context: Context) -> CommandResult {
    todo!()
}
```

And `src/commands.rs` is updated:

```rust
mod greet;
mod version;
mod db;  // Added automatically

clawless::commands!();
```

You can now run:

```bash
cargo run -- db migrate
```

## Generate multiple nested commands

Generate additional commands in the same group:

```bash
clawless generate command db/seed
clawless generate command db/reset
```

These are automatically added as submodules in `src/commands/db.rs`:

```rust
mod migrate;
mod seed;
mod reset;

use clawless::prelude::*;

#[derive(Debug, Args)]
pub struct DbArgs {}

#[command]
pub async fn db(args: DbArgs, context: Context) -> CommandResult {
    todo!()
}
```

## Multi-level nesting

You can nest commands as deeply as needed by creating each level of the
hierarchy:

```bash
# Create parent commands first
clawless generate command config
clawless generate command config/auth

# Now create the nested commands
clawless generate command config/auth/login
clawless generate command config/auth/logout
```

This creates:

- `src/commands/config.rs` - Top-level parent
- `src/commands/config/auth.rs` - Second-level parent
- `src/commands/config/auth/login.rs` - Leaf command
- `src/commands/config/auth/logout.rs` - Leaf command

Commands available:

```bash
cargo run -- config auth login
cargo run -- config auth logout
```

## Naming conventions

The generator follows Rust naming conventions:

- Use `snake_case` for module and function names
- The generator converts to kebab-case for CLI commands

Example:

```bash
clawless generate command deploy_staging
```

Creates `src/commands/deploy_staging.rs` with a `deploy_staging()` function that
becomes the `deploy-staging` CLI command:

```bash
cargo run -- deploy-staging
```

## What gets generated

For each command, the generator creates:

1. **A command file** with:
   - Imports from `clawless::prelude::*`
   - An Args struct with `#[derive(Debug, Args)]`
   - A command function with `#[command]` and `todo!()` placeholder

2. **Module declarations** in the appropriate parent file

3. **Directory structure** matching the command hierarchy

You then fill in:

- Doc comments for help text
- Arguments in the Args struct
- Implementation in the command function

## Cleaning up boilerplate

The generated code includes these derives by default:

```rust
#[derive(Debug, Args)]
```

You can add more derives if needed (Clone, PartialEq, etc.) or remove them if
they're not necessary for your use case.

## See also

- [Project Structure](/concepts/project-structure) - Understanding the file
  hierarchy
- [Naming Conventions](/concepts/naming-conventions) - How names become commands
- [Commands](/concepts/commands) - Command function details
