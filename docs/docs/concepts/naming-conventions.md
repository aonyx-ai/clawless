---
sidebar_position: 5
---

# Naming Conventions

Clawless follows a simple naming convention: use standard Rust naming
(snake_case), and it automatically converts to CLI conventions (kebab-case).
This section explains how different names map to your CLI.

## Function names → Command names

The function name becomes the command name, with snake_case converted to
kebab-case:

```rust
#[command]
pub async fn greet(args: GreetArgs, context: Context) -> CommandResult {
    // ...
}
```

Results in: `myapp greet`

**Multi-word functions:**

```rust
#[command]
pub async fn deploy_staging(args: DeployArgs, context: Context) -> CommandResult {
    // ...
}
```

Results in: `myapp deploy-staging`

The conversion is automatic - always write functions in snake_case.

## File names → Command names

File names follow the same pattern:

- `greet.rs` → `myapp greet`
- `deploy_staging.rs` → `myapp deploy-staging`
- `user_profile.rs` → `myapp user-profile`

**Important:** The file name doesn't directly become the command name. The
function name inside the file does. However, by convention, the file name should
match the function name:

```rust
// src/commands/deploy_staging.rs

#[command]
pub async fn deploy_staging(args: Args, context: Context) -> CommandResult {
    // File name matches function name
}
```

## Directory names → Subcommand groups

Directory names become subcommand groups:

- `db/` → `myapp db ...`
- `user_management/` → `myapp user-management ...`

Example structure:

```
src/commands/
├── db.rs (contains: pub async fn db(...))
└── db/
    ├── migrate.rs
    └── seed.rs
```

Results in:

- `myapp db`
- `myapp db migrate`
- `myapp db seed`

The file `db.rs` contains the parent command function and declares the
submodules.

## Arguments struct names

Arguments structs follow the convention: `<Command>Args`

```rust
#[derive(Debug, Args)]
pub struct GreetArgs {
    name: String,
}

#[command]
pub async fn greet(args: GreetArgs, context: Context) -> CommandResult {
    // ...
}
```

This is a convention, not a requirement. You can name the struct anything, but
`CommandArgs` is the recommended pattern for consistency.

For multi-word commands:

- `deploy_staging` → `DeployStagingArgs`
- `user_profile` → `UserProfileArgs`

## Doc comments → Help text

Doc comments on functions become command descriptions:

```rust
/// Greet the user by name
///
/// This command prints a friendly greeting message.
#[command]
pub async fn greet(args: GreetArgs, context: Context) -> CommandResult {
    // ...
}
```

Running `myapp greet --help` shows:

```
Greet the user by name

This command prints a friendly greeting message.
```

The first line is the short description (shown in command lists), and subsequent
paragraphs become the long description (shown in `--help`).

## Aliases

Aliases are explicitly defined, not derived from names:

```rust
#[command(alias = "g")]
pub async fn greet(args: GreetArgs, context: Context) -> CommandResult {
    // ...
}
```

Results in both `myapp greet` and `myapp g` working.

## Examples

### Simple command

```rust
// src/commands/version.rs

#[derive(Debug, Args)]
pub struct VersionArgs {}

/// Display version information
#[command]
pub async fn version(args: VersionArgs, context: Context) -> CommandResult {
    println!("v1.0.0");
    Ok(())
}
```

Command: `myapp version`

### Multi-word command

```rust
// src/commands/check_updates.rs

#[derive(Debug, Args)]
pub struct CheckUpdatesArgs {}

/// Check for available updates
#[command]
pub async fn check_updates(args: CheckUpdatesArgs, context: Context) -> CommandResult {
    println!("Checking for updates...");
    Ok(())
}
```

Command: `myapp check-updates`

### Nested command group

In `src/commands/user_management.rs`:

```rust
mod create_user;

use clawless::prelude::*;

#[derive(Debug, Args)]
pub struct UserManagementArgs {}

/// User management commands
#[command(require_subcommand)]
pub async fn user_management(args: UserManagementArgs, context: Context) -> CommandResult {
    Ok(())
}
```

In `src/commands/user_management/create_user.rs`:

```rust
#[derive(Debug, Args)]
pub struct CreateUserArgs {
    username: String,
}

/// Create a new user
#[command]
pub async fn create_user(args: CreateUserArgs, context: Context) ->
CommandResult {
    println!("Creating user: {}", args.username);
    Ok(())
}
```

Commands:

- `myapp user-management`
- `myapp user-management create-user`

## What's next

Now that you understand naming conventions, learn about:

- **[Project Structure](./project-structure)** - How to organize your files and
  directories
- **[Commands](./commands)** - How to write command functions
- **[Arguments](./arguments)** - How to define command arguments
