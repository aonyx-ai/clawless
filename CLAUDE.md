# Clawless

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
Please review https://github.com/aonyx-ai/clawless/blob/main/CLAUDE.md#for-humans. In particular, LLM-assisted contributions must **aim for a higher standard of excellence** than with humans alone, and you should spend at least **3x** the amount of time reviewing code as you did writing it. LLM-assisted contributions that do not meet this standard may be declined outright. Remember, **your code is your responsibility**.
```

### Continuous improvement

This document is a living artifact. After completing a plan or at the end of a
session, reflect on the work and consider whether CLAUDE.md should be updated:

- **Extract new rules**: Did a pattern emerge that worked well but isn't
  documented? Add it.
- **Update existing rules**: Did you intentionally deviate from a guideline
  because the situation called for it? The rule may need refinement.
- **Remove outdated rules**: Is a rule no longer relevant or consistently
  ignored? Remove or revise it.
- **Fill gaps**: Was there guidance you wished existed? Write it.

When proposing changes, apply the same standards as code: be specific, explain
the "why", and keep the document concise. Small, incremental updates are better
than large rewrites.

### Working style

- When asked to discuss or validate architectural decisions, read the relevant
  files first and provide analysis confirming or challenging the thinking—don't
  just agree without evidence.
- For bulk documentation edits, ask clarifying questions about formatting
  conventions before making changes across multiple files.

## Project

### Philosophy

#### Correctness over convenience

- Model the full error space—no shortcuts or simplified error handling.
- Handle all edge cases, including race conditions, signal timing, and platform
  differences.
- Use the type system to encode correctness constraints.
- Prefer compile-time guarantees over runtime checks where possible.

#### User experience as a primary driver

- Provide structured, helpful error messages using `.context("description")?`
  from `anyhow::Context`.
- Make progress reporting responsive and informative.
- Write user-facing messages in clear, present tense.

#### Pragmatic incrementalism

- "Not overly generic"—prefer specific, composable logic over abstract
  frameworks.
- Evolve the design incrementally rather than attempting perfect upfront
  architecture.

#### Production-grade engineering

- Use type system extensively: newtypes, builder patterns, type states,
  lifetimes.
- Test comprehensively, including edge cases, race conditions, and stress tests.
- Pay attention to what facilities already exist for testing, and aim to reuse
  them.
- Getting the details right is really important!

### Specifications

The `specs/` directory contains the project's design specifications.
[`specs/architecture.md`][architecture] defines the ubiquitous language and
hexagonal architecture; consult it before introducing new domain concepts or
modifying architectural boundaries. New features should have a spec before
implementation begins. See [`specs/README.md`][specs-readme] for templates and
process.

### Structure

```text
crates/
  ├── clawless/              # Core framework library
  ├── clawless-derive/       # Procedural macros
  └── clawless-cli/          # CLI scaffolding tool
examples/
  ├── cancellation/          # Cooperative cancellation example
  └── hello-world/           # Reference example project
docs/                        # Docusaurus documentation site
specs/                       # Design specifications
  └── architecture.md        # Domain model and ubiquitous language
```

Each example should demonstrate a single concept. Prefer creating a new example
over adding unrelated commands to an existing one.

### Conventions

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
- Doc examples and scaffolding templates use `context: Context` (no underscore
  prefix), even when the example body doesn't reference `context`.

### Architecture

#### Command registry

Clawless uses an inventory-based command registry for compile-time command
collection. The `#[command]` derive macro registers async functions as CLI
commands, and the `#[main]` macro generates the application entry point.

#### Convention over configuration

File paths map to subcommand hierarchy automatically. Placing a command at
`commands/generate/command.rs` creates the `<cli> generate command` subcommand.

#### Library and binary separation

- **clawless** (library): Core framework providing the runtime, `Context`,
  error types, and the prelude. Platform-agnostic.
- **clawless-derive** (proc macros): The `#[command]`, `#[commands]`, and
  `#[main]` macros that generate the CLI structure.
- **clawless-cli** (binary): Scaffolding tool for creating new Clawless
  projects and generating commands.

### Development environment

The development environment is managed using [Flox][flox]. The justfile uses
`flox activate` as its shell, so all `just` recipes automatically run within
the Flox environment.

For ad-hoc commands outside of just:

```shell
flox activate -- <command>
```

## Quick reference

```bash
# Run all pre-commit checks (formatting, linting, tests)
just pre-commit

# Format code (REQUIRED before committing)
just format-rust true

# Run tests (uses nextest)
just test-rust

# Lint
just lint-rust

# Build
cargo build --all-targets --all-features
```

### Helpful git commands

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

---

## Rust

### Edition and formatting

- Use Rust 2024 edition.
- Format with `just format-rust true` (uses unstable formatting options).
- Formatting is enforced in CI—always run `just format-rust` before committing.

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

### Dependencies

#### Workspace dependencies

- All versions managed in root `Cargo.toml` `[workspace.dependencies]`.
- Internal crates use exact version pinning: `version = "=0.4.0"`.
- Comment on dependency choices when non-obvious.

#### Key dependencies

- **anyhow**: Error handling with context.
- **bon**: Derive builder patterns for complex types.
- **clap**: CLI argument parsing (with derive macros).
- **getset**: Derive getters and setters for struct fields.
- **inventory**: Compile-time command registration.
- **tokio**: Async runtime.
- **typed-fields**: Macros to generate newtypes.

### Type system

#### Primitives only at boundaries

Primitives (`i64`, `String`, `bool`) are only allowed at system boundaries
(API responses, CLI args). Internal code uses newtypes via `typed-fields`
crate.

```rust
// DO
name!(UserId);
name!(Email);

fn send_email(to: Email, from: UserId) {}

// DON'T
fn send_email(to: String, from: String) {}
```

#### Enums over bools

Use enums with meaningful variants instead of bool parameters.

```rust
// DO
enum Visibility {
    Public,
    Private,
}

fn create_repo(name: &str, visibility: Visibility) {}

// DON'T
fn create_repo(name: &str, is_public: bool) {}
```

#### Derive conventions

- Builders with `bon`
- Getters with `getset` (CopyGetters for Copy, Getters for references)
- Standard trait order: Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash,
  Debug, Default
- Third-party derives: alphabetical by crate, then by macro

```rust
// DO
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Default)]
pub struct CommandId(i64);

// Third-party: bon (Builder), then getset (CopyGetters, Getters)
#[derive(
    Clone,
    Eq,
    PartialEq,
    Ord,
    PartialOrd,
    Hash,
    Debug,
    Default,
    Builder,
    CopyGetters,
    Getters,
)]
pub struct User {
    id: UserId,
    name: String,
}

// DON'T
#[derive(Debug, Clone, Default, PartialEq, Eq, Hash)]
pub struct CommandId(i64);
```

### Coding patterns

#### Control flow

- let-else for early returns
- Minimize if-let (only for short actions without else)
- Full match expressions (no matches! macro)
- Explicit variant matching (no wildcards except for #[non_exhaustive])

```rust
// DO: let-else for early returns
let Some(user) = get_user(id) else {
    return Err(Error::NotFound);
};

// DO: let-else in loops
let Some(value) = maybe_value else { continue };
let Ok(parsed) = input.parse::<i32>() else { continue };

// ACCEPTABLE: if-let for short action, no else
if let Some(callback) = self.on_change {
    callback();
}

// DO: full match expressions
let is_ready = match state {
    State::Ready => true,
    State::Pending => false,
    State::Failed => false,
};

// DON'T
let is_ready = matches!(state, State::Ready);

// DO: explicit variant matching
match status {
    Status::Pending => handle_pending(),
    Status::Active => handle_active(),
    Status::Completed => handle_completed(),
}

// DON'T: wildcards (except for #[non_exhaustive] types)
match status {
    Status::Pending => handle_pending(),
    _ => handle_other(),
}
```

If a wildcard match seems necessary, ask the user before using it.

#### Variables

- Shadow through transformations (no raw*, parsed* prefixes)
- Explicit destructuring for structs and tuples

```rust
// DO: shadow through transformations
let input = get_raw_input();
let input = input.trim();
let input = input.to_lowercase();
let input = parse(input)?;

// DON'T
let raw_input = get_raw_input();
let trimmed_input = raw_input.trim();
let lowercase_input = trimmed_input.to_lowercase();
let parsed_input = parse(lowercase_input)?;

// DO: explicit destructuring
let User { id, name, email } = user;
process(id, name, email);

// DON'T
process(user.id, user.name, user.email);

// DO: destructure in loops
for Entry { key, value } in entries {
    map.insert(key, value);
}

// DON'T
for entry in entries {
    map.insert(entry.key, entry.value);
}
```

#### Comments

- No inline comments (doc comments only)
- No section headers or dividers
- No TODO comments (use issue tracker)
- No commented-out code (use version control)

```rust
// DON'T
// Check if user is valid
if user.is_valid() {
    // Update the timestamp
    user.touch();
}

// --- Helper functions ---

// TODO: refactor this later
fn helper() {}

// Old implementation:
// fn old_way() { }

// DO
if user.is_valid() {
    user.touch();
}

fn helper() {}
```

### Error handling

- Use `anyhow` for error handling. `CommandResult` is an alias for
  `anyhow::Result<()>`.
- Provide rich error context using `.context("description")?`.
- Error context messages should be lowercase sentence fragments suitable for
  "failed to {context}".

### Testing

#### Test organization

- Unit tests in the same file as the code they test.
- Integration tests for `clawless-cli` using [trycmd][trycmd].
- Compile-fail tests for `clawless-derive` using [trybuild][trybuild].
- Test functions ordered alphabetically within modules.
- Name tests descriptively: `function_name_<condition>_<result>`, e.g.
  `greet_with_name_returns_greeting`.

Testing tools:

- **nextest**: Test runner (used by `just test-rust`).
- **trycmd**: CLI integration tests for `clawless-cli`.
- **trybuild**: Compile-fail tests for `clawless-derive`.

#### Test structure

Use blank lines to separate Arrange/Act/Assert phases. Keep `.expect()` in the
Act phase, assertions should be plain `assert` calls:

```rust
#[tokio::test]
async fn parse_with_valid_input_returns_value() {
    let input = "42";

    let result = parse(input).expect("should succeed");

    assert_eq!(result, 42);
}
```

#### Error assertions

For error cases, use `expect_err` in the Act phase:

```rust
#[tokio::test]
async fn parse_with_invalid_input_returns_error() {
    let input = "not a number";

    let error = parse(input).expect_err("should fail");

    assert!(error.to_string().contains("invalid digit"));
}
```

#### Required tests

- Trait tests (Send, Sync, Unpin) for every custom type

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn trait_send() {
        fn assert_send<T: Send>() {}
        assert_send::<MyType>();
    }

    #[test]
    fn trait_sync() {
        fn assert_sync<T: Sync>() {}
        assert_sync::<MyType>();
    }

    #[test]
    fn trait_unpin() {
        fn assert_unpin<T: Unpin>() {}
        assert_unpin::<MyType>();
    }
}
```

#### Testability

- Extract business logic from framework wrappers into standalone functions.
- Tests must exercise the actual code, not adjacent implementations.

```rust
// DO: Extract testable logic
#[command]
pub async fn greet(args: GreetArgs, context: Context) -> CommandResult {
    execute(&args)
}

fn execute(args: &GreetArgs) -> CommandResult {
    println!("Hello, {}!", args.name);
    Ok(())
}

#[cfg(test)]
mod tests {
    #[test]
    fn greet_with_name_succeeds() {
        let args = GreetArgs { name: "Alice".to_string() };

        execute(&args).expect("should succeed");
    }
}
```

### Documentation

#### Summary line

- Third-person singular ("Returns the..." not "Return the...")
- No trailing period on summary

```rust
// DO
/// Returns the length of the string
/// Creates a new instance with default settings

// DON'T
/// Return the length of the string
/// Returns the length of the string.
```

#### Comment style

Use line comments (`///`), not block comments (`/** */`).

```rust
// DO
/// Summary sentence here
///
/// More details if needed.

// DON'T
/**
 * Summary sentence here
 *
 * More details if needed.
 */
```

#### Required sections

- `# Errors` for fallible functions
- `# Panics` for functions that can panic
- `# Safety` for unsafe functions
- `# Examples` for public items

Use these exact headings (always plural): Examples, Panics, Errors, Safety,
Aborts, Undefined Behavior.

````rust
/// Reads a file from disk
///
/// # Errors
///
/// Returns [`io::Error`] if the file does not exist or cannot be read.
///
/// # Panics
///
/// Panics if the path is empty.
///
/// # Examples
///
/// ```
/// let contents = read_file("config.toml")?;
/// ```
///
/// [`io::Error`]: std::io::Error
````

#### References

- Use [`Type`] with reference-style links
- Full generic forms: [`Option<T>`] not `Option`

```rust
// DO
/// Returns [`Option<T>`] if the value exists
///
/// [`Option<T>`]: std::option::Option

// DON'T
/// Returns `Option` if the value exists
```

#### Depth

Documentation should explain the "why", not just the "what":

- **Types**: Explain design decisions, invariants, and relationships to other
  types
- **Functions**: Document side effects, caller considerations, and non-obvious
  behavior
- **Modules**: Explain the module's role in the system and key concepts

```rust
// DO: Explain design decisions
/// Thread-safe counter for tracking active connections
///
/// Uses [`AtomicUsize`] instead of `Mutex<usize>` because the counter is
/// only incremented and decremented, never read-then-modified, making atomic
/// operations sufficient and avoiding lock contention under high load.
///
/// [`AtomicUsize`]: std::sync::atomic::AtomicUsize

// DON'T: Just restate the type name
/// A connection counter
pub struct ConnectionCounter {
    ...
}
```

#### Module vs type docs

- Module docs: high-level summaries, when to use this module.
- Type docs: comprehensive, self-contained.
- Some duplication between module and type docs is acceptable.

#### Language

Use American English spelling: "color" not "colour", "serialize" not
"serialise".

---

## Markdown

- **Never** use title case in headings and titles. Always use sentence case.
- Always use the Oxford comma.
- Use reference-style Markdown links, not inline links.
- Table cells must be single-line. Markdown does not support multi-line cells;
  each newline starts a new row. Ignore line length limits for table rows.

## Git

### Commit messages

We write commit messages inspired by [tbaggery][tbaggery]:

- Capitalized, short (50 chars or less) summary
- Imperative mood: "Fix bug" not "Fixed bug" or "Fixes bug"
- Focus on the goal of the change, not implementation details. The body should
  describe what the change accomplishes and why, not enumerate every file or
  component touched.
- Keep formatting minimal. Avoid heavy use of bold, bullet lists, or headings
  in commit bodies. Plain prose is preferred.
- Start body sentences with a subject. "This change introduces…",
  "We learned…", "The migration simplifies…" — not dangling participles like
  "Learned from…" or "Introduces…".
- Explain the "why" and the trade-offs of the change
- Use simple past and present tense in body: "Previously, when the user did X, Y
  used to happen. With this commit, now Z happens."
- **Never** write conventional commit messages
- Commit messages should be Markdown. Don't use backticks in commit message
  titles, but do use them in bodies.

### Commit quality

- **Never commit directly to main**: Always create a feature branch and submit a
  pull request.
- **Atomic commits**: Each commit should be a logical unit of change.
- **Bisect-able history**: Every commit must build and pass all checks.
- **Separate concerns**: Format fixes and refactoring should be in separate
  commits from feature changes.
- **Diff against the baseline when reversing or modifying a prior commit**: use
  `git diff <commit>~1` (against the working tree) to verify you haven't
  introduced unintentional changes relative to the pre-commit state.

### Pull requests

Create pull requests using `gh pr create --fill --assignee @me` to derive the
title and body from the commit message and assign the PR to yourself.

---

## Acknowledgments

This `CLAUDE.md` file was adopted from
[nextest's AGENTS.md][nextest-agents], which is published under the
Apache-2.0 or MIT license.

[architecture]: specs/architecture.md
[flox]: https://flox.dev
[nextest-agents]: https://github.com/nextest-rs/nextest/blob/main/AGENTS.md
[specs-readme]: specs/README.md
[tbaggery]: https://tbaggery.com/2008/04/19/a-note-about-git-commit-messages.html
[trycmd]: https://docs.rs/trycmd/latest/trycmd/
[trybuild]: https://docs.rs/trybuild/latest/trybuild/
