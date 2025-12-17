---
sidebar_position: 1
---

# Quick Start

Get a working Clawless CLI in 5 minutes.

## Prerequisites

You'll need:

- Rust (latest stable recommended)
- Cargo (comes with Rust)

## Install the Clawless CLI

The Clawless CLI provides scaffolding and code generation for your projects:

```bash
cargo install clawless-cli
```

## Create your first project

Generate a new CLI application:

```bash
clawless new my-cli
cd my-cli
```

This creates a complete project structure with:

- `src/main.rs` - Application entry point
- `src/commands.rs` - Command discovery setup
- `src/commands/greet.rs` - A sample command to get you started
- `Cargo.toml` - With Clawless already configured

## Try it out

Run the sample `greet` command:

```bash
cargo run -- greet
```

Output:

```
Hello, World!
```

Try it with a name:

```bash
cargo run -- greet Rust
```

Output:

```
Hello, Rust!
```

Check the help text:

```bash
cargo run -- --help
cargo run -- greet --help
```

## Add a new command

Generate a new command:

```bash
clawless generate command goodbye
```

This creates `src/commands/goodbye.rs` and updates the module declarations
automatically.

Open `src/commands/goodbye.rs` and you'll see the generated boilerplate:

```rust
use clawless::prelude::*;

#[derive(Debug, Args)]
pub struct GoodbyeArgs {}

#[command]
pub async fn goodbye(args: GoodbyeArgs, context: Context) -> CommandResult {
    todo!()
}
```

Update it to actually do something:

```rust
use clawless::prelude::*;

#[derive(Debug, Args)]
pub struct GoodbyeArgs {
    /// The name to say goodbye to
    #[arg(default_value = "World")]
    name: String,
}

/// Say goodbye to someone
#[command]
pub async fn goodbye(args: GoodbyeArgs, _context: Context) -> CommandResult {
    println!("Goodbye, {}!", args.name);
    Ok(())
}
```

Run it:

```bash
cargo run -- goodbye Rust
```

Output:

```
Goodbye, Rust!
```

## Create nested commands

For larger CLIs, organize commands into groups using nested modules:

```bash
clawless generate command db/migrate
clawless generate command db/seed
```

This creates:

```
src/commands/
├── greet.rs
├── goodbye.rs
├── db.rs
└── db/
    ├── migrate.rs
    └── seed.rs
```

Run nested commands:

```bash
cargo run -- db migrate
cargo run -- db seed
```

The file hierarchy automatically becomes your command hierarchy!

## What's next?

You now have a working CLI with multiple commands. To learn more:

- **[Tutorial](/tutorial)** - Step-by-step guide through all Clawless features
- **[How-To Guides](/how-to)** - Practical recipes for common tasks
- **[Concepts](/concepts)** - Deep dive into how Clawless works

Or jump right into building by exploring the generated code in
`src/commands/greet.rs` to see how commands, arguments, and context work
together.
