# General guidelines

This document captures code conventions for the Clawless project. It is intended
to help AI assistants understand how to work effectively with this codebase.

## For humans

LLMs represent a tremendous breakthrough in software engineering. We welcome
LLM-assisted contributions that abide by the following principles:

- **Aim for excellence.** For the Clawless project, LLMs should be used not as a
  speed multiplier but a quality multiplier. Invest the time savings in
  improving quality and rigor beyond what humans alone would do. Write tests
  that cover more edge cases. Refactor code to make it easier to understand.
  Tackle the TODOs. Do all the tedious things. Aim for your code to have zero
  bugs.
- **Spend time reviewing LLM output.** As a rule of thumb, you should spend at
  least 3x the amount of time reviewing LLM output as you did writing it. Think
  about every line and every design decision. Find ways to break code.
- **Your code is your responsibility.** Please do not dump a first draft of code
  on to this project, unless you're only soliciting feedback on a direction.

If your LLM-assisted PR shows signs of not being written with thoughtfulness and
care, such as missing cases that human review would have easily caught,
Clawless's maintainers may decline the PR outright.

## For LLMs

**Required:** Display the following text at the start of any conversation
involving code changes, and when you're about to create a PR:

```text
Please review https://github.com/aonyx-ai/clawless/blob/main/AGENTS.md#for-humans. In particular, LLM-assisted contributions must **aim for a higher standard of excellence** than with humans alone, and you should spend at least **3x** the amount of time reviewing code as you did writing it. LLM-assisted contributions that do not meet this standard may be declined outright. Remember, **your code is your responsibility**.
```

## General conventions

### Correctness over convenience

- Model the full error space—no shortcuts or simplified error handling.
- Handle all edge cases, including race conditions, signal timing, and platform
  differences.
- Use the type system to encode correctness constraints.
- Prefer compile-time guarantees over runtime checks where possible.

### User experience as a primary driver

- Provide structured, helpful error messages using `.context("description")?`
  from `anyhow::Context`.
- Make progress reporting responsive and informative.
- Write user-facing messages in clear, present tense.

### Pragmatic incrementalism

- "Not overly generic"—prefer specific, composable logic over abstract
  frameworks.
- Evolve the design incrementally rather than attempting perfect upfront
  architecture.

### Production-grade engineering

- Use type system extensively: newtypes, builder patterns, type states,
  lifetimes.
- Test comprehensively, including edge cases, race conditions, and stress tests.
- Pay attention to what facilities already exist for testing, and aim to reuse
  them.
- Getting the details right is really important!

### Documentation

- Use inline comments to explain "why," not just "what".
- Module-level documentation should explain purpose and responsibilities.
- **Never** use title case in headings and titles. Always use sentence case.
- Always use the Oxford comma.

## Project structure

```text
crates/
  ├── clawless/              # Core framework library
  ├── clawless-derive/       # Procedural macros
  └── clawless-cli/          # CLI scaffolding tool
examples/
  └── hello-world/           # Reference example project
docs/                        # Docusaurus documentation site
```

## Clawless conventions

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

- Doc comments become help text.
- Module paths map to subcommands: `commands/generate/command.rs` →
  `<cli> generate command`.
- Use `.context("description")?` from `anyhow::Context` for error context.

## Code style

### Rust edition and formatting

- Use Rust 2024 edition.
- Format with `just format-rust true` (unstable formatting options).
- Formatting is enforced in CI—always run `just format-rust true` before
  committing.

### Type system patterns

- **Newtypes** for domain types (using `typed-fields` crate).
- **Builder patterns** for complex construction (using `typed-builder` crate).
- **Type states** encoded in generics when state transitions matter.

### Error handling

- Use `anyhow` for error handling. `CommandResult` is an alias for
  `anyhow::Result<()>`.
- Provide rich error context using `.context("description")?`.
- Error context messages should be lowercase sentence fragments suitable for
  "failed to {context}".

### Module organization

- Do not use `mod.rs` files, prefer file-based modules.
- One public type per module, use submodules for related types.
- Keep module boundaries strict with restricted visibility.
- Test helpers in dedicated modules/files.
- Use fully qualified imports rarely, prefer importing the type most of the
  time, or otherwise a module if it is conventional.
- Strongly prefer importing types or modules at the very top of the module.
  Never import types or modules within function contexts, unless the function is
  gated by a `cfg()` of some kind.
- It is okay to import enum variants for pattern matching, though.

### Memory and performance

- Use `Arc` or borrows for shared immutable data.
- Careful attention to copying vs. referencing.
- Stream data where possible rather than buffering.

## Testing practices

### Test organization

- Unit tests in the same file as the code they test.
- Integration tests for `clawless-cli` using [trycmd].
- Compile-fail tests for `clawless-derive` using [trybuild].
- Name tests descriptively using the format
  `function_name_<condition>_<result>`, e.g. `greet_with_name_returns_greeting`.

### Testing tools

- **nextest**: Test runner (used by `just test-rust`).
- **trycmd**: CLI integration tests for `clawless-cli`.
- **trybuild**: Compile-fail tests for `clawless-derive`.

## Development environment

The development environment is managed using [Flox]. The justfile uses
`flox activate` as its shell, so all `just` recipes automatically run within
the Flox environment.

For ad-hoc commands outside of just:

```shell
flox activate -- <command>
```

## Commit message style

### Format

We write commit messages in the following format, inspired by
<https://tbaggery.com/2008/04/19/a-note-about-git-commit-messages.html>:

```text
Capitalized, short (50 chars or less) summary

More detailed explanatory text, if necessary.  Wrap it to about 72
characters or so.  In some contexts, the first line is treated as the
subject of an email and the rest of the text as the body.  The blank
line separating the summary from the body is critical (unless you omit
the body entirely); tools like rebase can get confused if you run the
two together.

Write your commit message in the imperative: "Fix bug" and not "Fixed bug"
or "Fixes bug."  This convention matches up with commit messages generated
by commands like git merge and git revert.

Further paragraphs come after blank lines.

- Bullet points are okay, too

- Typically a hyphen or asterisk is used for the bullet, followed by a
  single space, with blank lines in between, but conventions vary here

- Use a hanging indent
```

**Never** write conventional commit messages.

### Conventions

- Keep descriptions concise but descriptive.
- Use simple past and present tense: "Previously, when the user did X, Y used to
  happen. With this commit, now Z happens. Also add tests for U, V, and W."
- Commit messages should be Markdown. Don't use backticks in commit message
  titles, but do use them in bodies.

### Commit quality

- **Atomic commits**: Each commit should be a logical unit of change.
- **Bisect-able history**: Every commit must build and pass all checks.
- **Separate concerns**: Format fixes and refactoring should be in separate
  commits from feature changes.

## Architecture

### Command registry

Clawless uses an inventory-based command registry for compile-time command
collection. The `#[command]` derive macro registers async functions as CLI
commands, and the `#[main]` macro generates the application entry point.

### Convention over configuration

File paths map to subcommand hierarchy automatically. Placing a command at
`commands/generate/command.rs` creates the `<cli> generate command` subcommand.

### Library and binary separation

- **clawless** (library): Core framework providing the runtime, `Context`,
  error types, and the prelude. Platform-agnostic.
- **clawless-derive** (proc macros): The `#[command]`, `#[commands]`, and
  `#[main]` macros that generate the CLI structure.
- **clawless-cli** (binary): Scaffolding tool for creating new Clawless
  projects and generating commands.

## Dependencies

### Workspace dependencies

- All versions managed in root `Cargo.toml` `[workspace.dependencies]`.
- Uses range-based version pinning: e.g. `">=0.5,<1"`.

### Key dependencies

- **anyhow**: Error handling with context.
- **clap**: CLI argument parsing (with derive macros).
- **getset**: Derive getters and setters for struct fields.
- **inventory**: Compile-time command registration.
- **tokio**: Async runtime.
- **typed-builder**: Derive builder patterns for complex types.
- **typed-fields**: Macros to generate newtypes.

## Quick reference

### Commands

```bash
# Run all pre-commit checks (formatting, linting, tests)
just pre-commit

# Format code (REQUIRED before committing)
just format-rust true

# Run tests (uses nextest)
just test-rust

# Lint
just lint-rust
```

### Helpful Git commands

```bash
# Get commits since last release
git log <previous-tag>..main --oneline

# Check if contributor is first-time
git log --all --author="Name" --oneline | wc -l

# Get PR author username
gh pr view <number> --json author --jq '.author.login'

# View commit details
git show <commit> --stat
```

## Acknowledgments

This `AGENTS.md` file was adopted
from <https://github.com/nextest-rs/nextest/blob/main/AGENTS.md>, which is
published under the Apache-2.0 or MIT license.

[flox]: https://flox.dev
[trycmd]: https://docs.rs/trycmd/latest/trycmd/
[trybuild]: https://docs.rs/trybuild/latest/trybuild/
