---
sidebar_position: 0
---

# Commands

Commands are the core building blocks of a Clawless CLI. Each command is a Rust
function marked with the `#[command]` macro that automatically becomes a CLI
command.

## Anatomy of a command

Here's a complete command definition:

```rust
use clawless::prelude::*;

#[derive(Debug, Args)]
pub struct GreetArgs {
    /// The name to greet
    #[arg(default_value = "World")]
    name: String,
}

/// Greet the user
///
/// This command prints a greeting message to the console.
#[command]
pub async fn greet(args: GreetArgs, context: Context) -> CommandResult {
    println!("Hello, {}!", args.name);
    Ok(())
}
```

Every command has three key components:

### 1. Arguments struct

```rust
#[derive(Debug, Args)]
pub struct GreetArgs {
    /// The name to greet
    #[arg(default_value = "World")]
    name: String,
}
```

The arguments struct defines what inputs the command accepts. It must derive
`Args` from Clap and can use any of Clap's argument attributes.

See [Arguments](./arguments) for details.

### 2. Command function

```rust
#[command]
pub async fn greet(args: GreetArgs, context: Context) -> CommandResult {
    println!("Hello, {}!", args.name);
    Ok(())
}
```

The command function signature is strict to enable automatic code generation:

```rust
pub async fn command_name(args: ArgsStruct, context: Context) -> CommandResult
```

**Required elements:**

- **`pub`** - Commands must be public so they can be discovered
- **`async fn`** - Commands are async by default; Clawless manages the Tokio
  runtime
- **First parameter** - An arguments struct deriving `Args`
- **Second parameter** - `Context` for accessing framework features
- **Return type** - `CommandResult` (alias for `anyhow::Result<()>`)

**Parameter naming:**

You can name the parameters anything you want:

```rust
pub async fn deploy(opts: DeployArgs, ctx: Context) -> CommandResult
pub async fn migrate(args: MigrateArgs, context: Context) -> CommandResult  // _ if unused
```

**Parameter order:**

The arguments must come before context. This order cannot be changed.

### 3. Doc comments

```rust
/// Greet the user
///
/// This command prints a greeting message to the console.
```

Doc comments on the function become the command's help text. The first line is
the short description shown in command lists, and subsequent paragraphs become
the long description shown in `--help`.

## CommandResult and error handling

Commands return `CommandResult`, which is a type alias for `anyhow::Result<()>`:

```rust
pub type CommandResult = anyhow::Result<()>;
```

This means:

- Commands either succeed (return `Ok(())`) or fail (return `Err`)
- You can use `?` to propagate errors
- Any error type implementing `Into<anyhow::Error>` works
- Errors automatically display nice messages to users

Example:

```rust
use clawless::prelude::*;

#[command]
pub async fn read_file(args: ReadArgs, context: Context) -> CommandResult {
    let contents = std::fs::read_to_string(&args.path)?;  // ? propagates errors
    println!("{}", contents);
    Ok(())
}
```

If the file doesn't exist, users see:

```
Error: No such file or directory (os error 2)
```

### Adding context to errors

Use `ErrorContext` (re-exported from `anyhow::Context`) to add helpful context:

```rust
use clawless::prelude::*;

#[command]
pub async fn read_config(args: ReadArgs, context: Context) -> CommandResult {
    let contents = std::fs::read_to_string(&args.path)
        .context("Failed to read configuration file")?;

    let config: Config = toml::from_str(&contents)
        .context("Failed to parse TOML configuration")?;

    println!("{:?}", config);
    Ok(())
}
```

Now errors include context:

```
Error: Failed to parse TOML configuration

Caused by:
    expected '=' at line 5 column 10
```

## The #[command] macro

The `#[command]` macro does several things:

1. **Generates initialization code** - Creates a Clap `Command` with help text
   from doc comments
2. **Generates wrapper function** - Handles argument parsing and calls your
   function
3. **Registers with inventory** - Makes the command discoverable by the
   framework

You don't need to understand these internals to use Clawless, but it's helpful
to know the macro is generating glue code that wires everything together.

### Macro attributes

The `#[command]` macro accepts two optional attributes:

**`alias`** - Create a shorthand for the command:

```rust
/// Generate code from templates
#[command(alias = "g")]
pub async fn generate(args: GenerateArgs, context: Context) -> CommandResult {
    // Can be called as either 'generate' or 'g'
}
```

You can combine attributes:

```rust
#[command(require_subcommand, alias = "d")]
pub async fn db(args: DbArgs, context: Context) -> CommandResult {
    // Can be called as 'db' or 'd', and both require subcommands
}
```

**`require_subcommand`** - Prevent the command from executing without a
subcommand:

```rust
#[command(require_subcommand)]
pub async fn db(args: DbArgs, context: Context) -> CommandResult {
    // This function won't execute unless a subcommand is provided
    // Running just 'myapp db' shows help instead
}
```

This is useful for commands that only act as grouping mechanisms for
subcommands.

See [Add Command Aliases](../how-to/add-command-aliases)
and [Require Subcommands](../how-to/require-subcommands) for practical examples.

## Commands without arguments

If a command doesn't need arguments, you still need an arguments struct:

```rust
use clawless::prelude::*;

#[derive(Debug, Args)]
pub struct VersionArgs {}

/// Display version information
#[command]
pub async fn version(_args: VersionArgs, context: Context) -> CommandResult {
    println!("myapp v{}", env!("CARGO_PKG_VERSION"));
    Ok(())
}
```

The empty struct is required for the type system, but it has no runtime
overhead.

## Async commands

All commands are async by default, even if they don't use async features:

```rust
#[command]
pub async fn hello(_args: HelloArgs, context: Context) -> CommandResult {
    println!("Hello!");  // No await needed
    Ok(())
}
```

This design choice enables:

- Consistent function signatures across all commands
- Easy addition of async operations later without signature changes
- Framework control over the async runtime

If you need to call async functions:

```rust
#[command]
pub async fn fetch(args: FetchArgs, context: Context) -> CommandResult {
    let client = reqwest::Client::new();
    let response = client.get(&args.url).send().await?;
    let body = response.text().await?;
    println!("{}", body);
    Ok(())
}
```

The Tokio runtime is managed automatically - you just write async code.

## What's next

Now that you understand commands, learn about:

- **[Arguments](./arguments)** - How to define command inputs
- **[Context](./context)** - Accessing framework features in commands
- **[Project Structure](./project-structure)** - Organizing commands in your
  project
