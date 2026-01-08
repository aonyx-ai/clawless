# Instructions for Coding Agents

This repository contains an opinionated, batteries-included framework for
building command-line applications in Rust.

## Development Commands

The development environment is managed using [Flox]. Run all commands through
Flox to ensure you have the necessary tools and dependencies:

- `flox activate -- <command>`

Many common tasks are defined as recipes in [just]. Run `just --list` to see a
full list. Here are the most important ones:

```shell
# Run all pre-commit checks (formatting, linting, tests, etc.)
just pre-commit

# Format the code using rustfmt
just format-rust true

# Run the test suite
just test-rust
```

## Testing

- Write unit tests for all functions in their respective `tests` module
  - Test edge cases and different inputs
  - Name tests descriptively using the format
    `function_name_<condition>_<result>`, e.g.
    `greet_with_name_returns_greeting`
- Write integration tests for the `clawless-cli` using [trycmd]

## Rust

See the top-level `Cargo.toml` for Edition and MSRV requirements.

## Project Structure

```text
crates/
  ├── clawless/              # Core framework library
  ├── clawless-derive/       # Procedural macros
  └── clawless-cli/          # CLI scaffolding tool
examples/
  └── hello-world/           # Reference example project
docs/                        # Docusaurus documentation site
```

## Clawless Conventions

Use the prelude for common imports:

```rust
use clawless::prelude::*;
```

Commands follow this pattern:

```rust
#[derive(Debug, Args)]
pub struct GreetArgs {
    /// Name to greet
    name: String,
}

/// Greet someone by name
#[command(alias = "g")]
pub async fn greet(args: GreetArgs, context: Context) -> CommandResult {
    println!("Hello, {}!", args.name);
    Ok(())
}
```

- Doc comments become help text
- Module paths map to subcommands: `commands/generate/command.rs` →
  `<cli> generate command`
- Use `.context("description")?` from `anyhow::Context` for error context

## Code Style

- Write idiomatic Rust code following the Rust API Guidelines
- Write clear and concise documentation comments for all items
- Functions should do one thing and do it well
- Keep cyclomatic complexity low; refactor complex functions into smaller ones
- Use `rustfmt` for code formatting (run `just format-rust true`)
- Use `clippy` for linting (run `just lint-rust`)

## Git Workflow

- Use feature branches for new features and bug fixes
- Refactor code before adding new features
- Commit often with small, focused changes
- Write clear and descriptive commit messages

[flox]: https://flox.dev
[just]: https://just.systems
[trycmd]: https://docs.rs/trycmd/latest/trycmd/
