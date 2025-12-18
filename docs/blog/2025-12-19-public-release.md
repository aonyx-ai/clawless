---
title: Introducing Clawless
description: We're excited to announce the first public release of Clawless, a batteries-included framework for building command-line applications in Rust.
slug: introducing-clawless
authors: jdno
---

We're thrilled to announce the first public release of **Clawless 0.4.0**, a
batteries-included framework for building command-line applications in Rust!

Building CLIs should be about solving problems, not wiring up boilerplate.
Clawless takes care of the infrastructure so you can focus on what makes your
tool unique.

<!-- truncate -->

## What is Clawless?

Clawless is a framework that transforms how you build command-line applications
in Rust. Instead of manually registering commands, parsing arguments, and
setting up help text, you write simple functions and let the framework handle
the rest.

Here's a complete command:

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
pub async fn greet(args: GreetArgs, context: Context) -> CommandResult {
    println!("Hello, {}!", args.name);
    Ok(())
}
```

That's it. The function name becomes your command name, doc comments become help
text, and the `Context` parameter gives you access to framework features.
Everything you need is already there.

## Convention over configuration

The core philosophy of Clawless is simple: **your file structure becomes your
command structure**.

Want to add a `db migrate` subcommand? Create this structure:

```
src/
├── main.rs
├── commands.rs
└── commands/
    ├── db.rs
    └── db/
        └── migrate.rs
```

The module hierarchy maps directly to CLI commands. No manual registration, no
configuration files, no macro soup. Just Rust modules doing what Rust modules
do.

```bash
myapp db migrate
```

This convention-based approach means you can understand a CLI's structure just
by looking at the file tree. New team members know exactly where to find code.
Adding features is as simple as creating a new file.

## Getting started in minutes

We've built scaffolding tools to get you up and running fast:

```bash
# Install the Clawless CLI
cargo install clawless-cli

# Create a new project
clawless new my-cli
cd my-cli

# Run the example command
cargo run -- greet
```

You'll have a working CLI in seconds. Want to add a command? Generate the
boilerplate:

```bash
clawless generate command deploy
```

The CLI creates the file, updates module declarations, and sets up the
structure. You just fill in the logic.

## Built for real CLIs

Clawless isn't just about simple commands. It's designed for production
applications that need:

- **Nested command hierarchies** - Organize large CLIs into logical command
  groups
- **Type-safe arguments** - Full compiler guarantees from Clap's derive API
- **Async by default** - Tokio runtime managed automatically
- **Context system** - Access environment information, configuration, and
  framework features
- **Doc-driven help** - Write documentation once, in your code

The framework provides the infrastructure that every CLI needs, so you don't
build it from scratch each time.

## What's next?

We're actively developing Clawless and have exciting features planned:

- Structured output and logging system
- Configuration management (files + environment variables)
- Shell completion generation
- JSON output mode for scripting

Check out the [roadmap on GitHub](https://github.com/aonyx-ai/clawless/issues)
to see what's coming and share your ideas.

## Try it today

Ready to build your first CLI with Clawless?

- **[Quick Start](/docs/quick-start)** - Get up and running in 5 minutes
- **[Concepts](/docs/concepts)** - Understand how Clawless works
- **[How-To Guides](/docs/how-to)** - Practical recipes for common tasks

Or jump straight to the code:

```bash
cargo install clawless-cli
clawless new my-awesome-cli
```

We can't wait to see what you build! If you have questions, find bugs, or want
to contribute, visit us on [GitHub](https://github.com/aonyx-ai/clawless).

Happy building!
