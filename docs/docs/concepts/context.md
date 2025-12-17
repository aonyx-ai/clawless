---
sidebar_position: 2
---

# Context

The `Context` is the second parameter every command receives, providing access
to framework features and environment information. It's how Clawless delivers
batteries-included functionality to your commands without requiring manual
setup.

:::warning[Work in Progress]
The Context system is still evolving. Currently, it only provides access to the
working directory. Many of the features described below (configuration, output
abstraction, structured logging) are planned but not yet implemented. See
the [Future features](#future-features) section for what's coming.
:::

## What is Context?

Context is a struct passed to every command that provides:

- **Currently available:** Environment information (working directory)
- **Coming soon:** Configuration, output abstraction, structured logging, and
  more

```rust
use clawless::prelude::*;

#[command]
pub async fn deploy(args: DeployArgs, context: Context) -> CommandResult {
    // Access context features here
    let cwd = context.current_working_directory();
    println!("Deploying from: {}", cwd.display());
    Ok(())
}
```

Every command must accept `Context` as its second parameter, even if it doesn't
use it. Use `_context` to indicate intentionally unused context:

```rust
#[command]
pub async fn version(_args: VersionArgs, _context: Context) -> CommandResult {
    println!("v{}", env!("CARGO_PKG_VERSION"));
    Ok(())
}
```

## Current features

### Current working directory

Access the directory from which the CLI was invoked:

```rust
use clawless::prelude::*;

#[command]
pub async fn status(_args: StatusArgs, context: Context) -> CommandResult {
    let cwd = context.current_working_directory();

    println!("Working directory: {}", cwd.display());

    // Build paths relative to the working directory
    let config_path = cwd.join("config.toml");
    if config_path.exists() {
        println!("Found config at: {}", config_path.display());
    }

    Ok(())
}
```

The `current_working_directory()` method returns a `CurrentWorkingDirectory`
type, which is a wrapper around `PathBuf` with the same API. You can:

- Use `.display()` to show the path
- Use `.join()` to build relative paths
- Use all standard `Path` and `PathBuf` methods

**Important:** The working directory is captured when the CLI starts. If your
command changes directories (e.g., with `std::env::set_current_dir()`), the
context value won't update. This is by design to provide a stable reference
point.

## Future features

The Context system is designed to be the central access point for all framework
features. Planned additions include:

### Configuration (coming soon)

Access application configuration loaded from files and environment variables:

```rust
#[command]
pub async fn deploy(args: DeployArgs, context: Context) -> CommandResult {
    // Future API (not yet implemented)
    let config = context.config();
    let api_url = config.api_url();
    let timeout = config.timeout();

    // Use config values...
    Ok(())
}
```

See [issue #118](https://github.com/aonyx-ai/clawless/issues/118) for the
configuration system design.

### Output abstraction (coming soon)

Consistent logging and output with built-in verbosity levels:

```rust
#[command]
pub async fn build(args: BuildArgs, context: Context) -> CommandResult {
    // Future API (not yet implemented)
    let output = context.output();

    output.info("Starting build...");
    output.debug("Loading configuration...");
    output.success("Build completed!");

    Ok(())
}
```

The output abstraction will respect `--quiet` and `--verbose` flags
automatically. See
issues [#151](https://github.com/aonyx-ai/clawless/issues/151)
and [#152](https://github.com/aonyx-ai/clawless/issues/152).

### Structured logging (coming soon)

Built-in observability and tracing support:

```rust
#[command]
pub async fn process(args: ProcessArgs, context: Context) -> CommandResult {
    // Future API (not yet implemented)
    context.trace("Processing started");

    // Structured fields automatically captured
    context.info_with_fields("Processing item", |fields| {
        fields.add("item_id", &item.id);
        fields.add("duration_ms", elapsed);
    });

    Ok(())
}
```

See [issue #155](https://github.com/aonyx-ai/clawless/issues/155) for
observability plans.

## Why Context?

The Context pattern provides several benefits:

**Consistent access** - All framework features available through one parameter,
rather than scattered across different imports and globals.

**Easy testing** - Context can be constructed with test values, making commands
easy to test in isolation:

```rust
use clawless::context::Context;

#[test]
fn test_deploy_command() {
    let context = Context::builder()
        .current_working_directory("/tmp/test".into())
        .build();

    let args = DeployArgs { environment: "test".into() };

    // Test your command
    let result = tokio::runtime::Runtime::new()
        .unwrap()
        .block_on(deploy(args, context));

    assert!(result.is_ok());
}
```

**Forward compatibility** - New framework features can be added to Context
without breaking existing commands. Your commands automatically gain access to
new capabilities.

**Separation of concerns** - Command arguments (user input) are separate from
context (framework features). This makes the distinction between "what the user
asked for" and "what the framework provides" explicit.

## Context lifecycle

Context is created once when your CLI starts and is cloned for each command
execution:

1. **Startup** - `Context::try_new()` is called by the `main!` macro
2. **Initialization** - Environment information is captured (working directory,
   etc.)
3. **Execution** - Context is cloned and passed to your command
4. **Access** - Your command uses context methods to access features

## What's next

Now that you understand Context, learn about:

- **[Commands](./commands)** - How Context integrates with command functions
- **[Arguments](./arguments)** - The other parameter commands receive
- **[Project Structure](./project-structure)** - Organizing commands in your
  project
