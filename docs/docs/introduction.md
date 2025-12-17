---
sidebar_position: 0
slug: /
---

# Introduction

Welcome to Clawless, a framework for building command-line applications in Rust.

## What is Clawless?

Clawless is a batteries-included framework that provides everything you need for
production CLIs: structured output, configuration management, environment
context, and scaffolding tools. You write command functions; Clawless provides
the infrastructure.

```rust
use clawless::prelude::*;

#[derive(Debug, Args)]
pub struct GreetArgs {
    /// The name to greet
    #[arg(default_value = "World")]
    name: String,
}

/// Greet the user
#[command]
pub async fn greet(args: GreetArgs, _context: Context) -> CommandResult {
    println!("Hello, {}!", args.name);
    Ok(())
}
```

That's it. The function name becomes your command name, doc comments become help
text, and the `Context` parameter gives you access to environment info,
configuration, and output abstractions. Everything you need to build a
professional CLI is already there.

## Why Clawless?

Most CLIs need the same features: structured logging, configuration management,
consistent output formatting, environment context, and more. Building these from
scratch for every project wastes time and leads to inconsistent patterns across
your tools.

Clawless is an **opinionated, batteries-included framework** that provides these
features out of the box. By following simple conventions, you get a
full-featured CLI with production-ready infrastructure.

### What Clawless provides

**Built-in features:**

- **Context system** - Every command receives environment information (working
  directory, etc.) automatically
- **Async runtime** - Tokio runtime managed for you; just write `async fn` and
  it works
- **Scaffolding tools** - Generate projects and commands with `clawless new` and
  `clawless generate command`
- **Convention-based structure** - File hierarchy becomes command hierarchy; no
  manual registration
- **Type-safe arguments** - Full compiler guarantees for your CLI arguments and
  flags

**Coming soon:**

- **Output abstraction** - Consistent logging interface with multiple verbosity
  levels (`--quiet`, `--verbose`)
- **Structured logging** - Built-in observability and tracing support
- **Configuration system** - Layered config loading (files + environment
  variables) with zero setup
- **Shell completions** - Auto-generated completions for bash, zsh, and fish
- **JSON output** - Structured output mode for scripting and automation

### Philosophy

Clawless is designed around three principles:

1. **Convention over configuration** - Sensible defaults that work for 90% of
   CLIs
2. **Batteries included** - Common features built-in, not bolted on
3. **Rapid development** - From idea to working CLI in minutes, not hours

## How it works

Clawless uses conventions and code generation to build your CLI:

1. **File structure = Command structure** - Your module hierarchy maps directly
   to CLI commands:

   ```
   src/commands/db/migrate.rs  →  myapp db migrate
   ```

2. **Context injection** - Framework features (config, output, environment)
   available through the `Context` parameter:

   ```rust
   pub async fn migrate(args: MigrateArgs, context: Context) -> CommandResult {
       let cwd = context.current_working_directory();
       // Future: context.config(), context.output(), etc.
   }
   ```

3. **Macro-driven generation** - Three macros work together to wire up your CLI:
   - `clawless::main!()` - Application entry point with runtime setup
   - `clawless::commands!()` - Command discovery and registration
   - `#[command]` - Marks functions as commands with automatic help text

## Who is this for?

Clawless is designed for anyone building command-line tools in Rust:

- **New to Rust?** Clawless provides guardrails and reduces decisions you need
  to make
- **Need a production CLI fast?** Get logging, config, output formatting, and
  more without setup
- **Building internal tools?** Consistency across all your team's CLIs with
  shared conventions
- **Growing a large CLI?** Structured conventions and built-in features scale
  better than ad-hoc solutions

## Project status

Clawless is in active early development (currently v0.3.0) and not yet
considered production-ready. The core concepts are stable, but APIs may change
and some features are still being developed.

If you're building internal tools, prototyping, or learning, Clawless is a great
choice. For production applications, review
the [open issues](https://github.com/aonyx-ai/clawless/issues) to understand
current limitations.

## Next steps

Ready to build your first CLI?

- **[Quick Start](/quick-start)** - Get up and running in 5 minutes
- **[Tutorial](/tutorial)** - Learn Clawless step-by-step
- **[Concepts](/concepts)** - Understand how Clawless works under the hood

Or jump straight to our [How-To Guides](/how-to) if you prefer learning by
doing.
