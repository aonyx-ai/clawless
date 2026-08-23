# Clawless

## For Humans

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

### Continuous Improvement

This document is a living artifact. After completing a plan or at the end of a
session, reflect on the work and consider whether AGENTS.md should be updated:

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

### Working Style

- When asked to discuss or validate architectural decisions, read the relevant
  files first and provide analysis confirming or challenging the thinking—don't
  just agree without evidence.
- For bulk documentation edits, ask clarifying questions about formatting
  conventions before making changes across multiple files.

## Project

### Philosophy

#### Correctness over Convenience

- Model the full error space—no shortcuts or simplified error handling.
- Handle all edge cases, including race conditions, signal timing, and platform
  differences.
- Use the type system to encode correctness constraints.
- Prefer compile-time guarantees over runtime checks where possible.

#### User Experience as a Primary Driver

- Provide structured, helpful error messages using `.context("description")?`
  from `anyhow::Context`.
- Make progress reporting responsive and informative.
- Write user-facing messages in clear, present tense.

#### Pragmatic Incrementalism

- "Not overly generic"—prefer specific, composable logic over abstract
  frameworks.
- Evolve the design incrementally rather than attempting perfect upfront
  architecture.

#### Production-Grade Engineering

- Use type system extensively: newtypes, builder patterns, type states,
  lifetimes.
- Test comprehensively, including edge cases, race conditions, and stress tests.
- Pay attention to what facilities already exist for testing, and aim to reuse
  them.
- Getting the details right is really important!

### Specifications

The `specs/` directory contains the project's design specifications.
[`specs/README.md`][specs-readme] defines the ubiquitous language, hexagonal
architecture, and layered crate structure; consult it before introducing new
domain concepts or modifying architectural boundaries. New features should have
a spec before implementation begins. Specs for already-shipped features are
historical records; do not update their API references when the API changes in
a later PR.

### Structure

```text
crates/
  ├── cargo-clawless/        # Scaffolding tool (`cargo clawless`)
  ├── clawless/              # Facade re-exporting core, cli, and tui
  ├── clawless-cli/          # CLI presentation layer (push-based)
  ├── clawless-core/         # Domain types, events, and abstract ports
  ├── clawless-derive/       # Procedural macros
  └── clawless-tui/          # TUI presentation layer (pull-based)
examples/
  ├── cancellation/          # Cooperative cancellation example
  └── hello-world/           # Reference example project
docs/                        # Docusaurus documentation site
specs/                       # Design specifications
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
    message!("Hello, {}!", args.name);
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

#### Command Registry

Clawless uses an inventory-based command registry for compile-time command
collection. The `#[command]` derive macro registers async functions as CLI
commands, and the `#[main]` macro generates the application entry point.

#### Convention over Configuration

File paths map to subcommand hierarchy automatically. Placing a command at
`commands/generate/command.rs` creates the `<cli> generate command` subcommand.

#### Layered Architecture

- **clawless-core** (library): Domain types, event system, and abstract ports.
  Platform-agnostic.
- **clawless-cli** (library): CLI presentation layer. Push-based adapter that
  renders events as they arrive (stateless commands).
- **clawless-tui** (library): TUI presentation layer. Pull-based adapter that
  aggregates events into a queryable projection (stateful applications).
- **clawless-derive** (proc macros): The `#[command]`, `#[commands]`, and
  `#[main]` macros that generate the CLI structure.
- **clawless** (facade): Re-exports `clawless-core`, `clawless-cli`, and
  `clawless-tui` so users need a single dependency.
- **cargo-clawless** (binary): Scaffolding tool for creating new Clawless
  projects and generating commands.

### Development Environment

The development environment is managed using [Flox][flox]. The justfile uses
`flox activate` as its shell, so all `just` recipes automatically run within
the Flox environment.

For ad-hoc commands outside of just:

```shell
flox activate -- <command>
```

## Quick Reference

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

### Helpful Git Commands

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

### Edition and Formatting

- Use Rust 2024 edition.
- Format with `just format-rust true` (uses unstable formatting options).
- Formatting is enforced in CI—always run `just format-rust` before committing.

### Module Organization

- Use `mod.rs` for modules that contain submodules.
- One public type per module, use submodules for related types.
- In library crates, give every item the narrowest visibility that still
  compiles. `unreachable_pub` enforces this, so a `pub` item is genuinely part
  of the public API and a `pub(crate)` item is genuinely internal.
- Binary crates allow `unreachable_pub` at the crate root. Nothing in them is
  reachable from outside, so the lint would only demand `pub(crate)` everywhere
  without telling the reader anything.
- Test helpers in dedicated modules/files.
- Use fully qualified imports rarely, prefer importing the type most of the
  time, or otherwise a module if it is conventional.
- Strongly prefer importing types or modules at the very top of the module.
  Never import types or modules within function contexts, unless the function is
  gated by a `cfg()` of some kind.
- It is okay to import enum variants for pattern matching, though.

### Memory and Performance

- Use `Arc` or borrows for shared immutable data.
- Careful attention to copying vs. referencing.
- Stream data where possible rather than buffering.

### Dependencies

#### Workspace Dependencies

- All versions managed in root `Cargo.toml` `[workspace.dependencies]`.
- Internal crates use exact version pinning: `version = "=0.4.0"`.
- Require the lowest version of a dependency that still compiles, so that
  applications keep the widest choice of versions. Verify the floor with
  `just check-minimal-deps`.
- Write dependency entries without comments. Do not describe what a package
  does, and do not explain a version requirement. Reasoning that matters,
  such as why a floor cannot go lower, belongs in the commit message.
- When adding dependencies, run `just check-dependencies` to verify license
  compatibility. If new licenses need allowlisting in `deny.toml`, include
  that in the same commit, again without a comment. Allowlist licenses that
  are OSI- or FSF-approved, ask for any other licenses.

#### Key Dependencies

- **anyhow**: Error handling with context.
- **bon**: Derive builder patterns for complex types.
- **clap**: CLI argument parsing (with derive macros).
- **getset**: Derive getters and setters for struct fields.
- **inventory**: Compile-time command registration.
- **tokio**: Async runtime.
- **typed-fields**: Macros to generate newtypes.

### Type System

#### Primitives Only at Boundaries

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

#### Enums over Bools

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

#### Derive Conventions

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

#### Fields Are Never Public

Fields must never be `pub`. Implement getters instead, e.g. with `getset`.
This keeps the invariants of a type in one place, and it lets a field change
representation without breaking every caller.

### Coding Patterns

#### Control Flow

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

### Error Handling

- Use `anyhow` for error handling. `CommandResult` is an alias for
  `anyhow::Result<()>`.
- Provide rich error context using `.context("description")?`.
- Error context messages should be lowercase sentence fragments suitable for
  "failed to {context}".
- Where a library returns a typed error, define one error enum per fallible
  action, named after the action and its object (e.g. `LoadCommandError`,
  `DiscoverProjectError`), never after the component that raises it.
- Use struct variants. The underlying cause is a field named `source`, and the
  context the message needs is carried in named fields. A variant only carries
  context its own layer knows.
- Variants name the failure condition together with its object (e.g.
  `UnresolvedCommand`, `FormatterUnavailable`, `MissingArgument`), never the
  step that failed. Name the condition at the certainty you have: a failed
  spawn is `FormatterUnavailable`, not `FormatterNotInstalled`.
- A variant with a `source` reads "failed to ..." in its message; a variant
  that is itself the diagnosis states its condition declaratively.

### Testing

#### Test Organization

- Unit tests in the same file as the code they test.
- Integration tests for `clawless-cli` using [trycmd][trycmd].
- UI tests for macros using [trybuild][trybuild] (`crates/clawless/tests/ui/`).
  Files use `pass-` and `fail-` prefixes. Each pass-test function should
  isolate a single invocation style for targeted failure diagnostics.
- Test functions ordered alphabetically within modules.
- Name tests descriptively: `function_name_<condition>_<result>`, e.g.
  `greet_with_name_returns_greeting`.
- Each test should have exactly one assertion.

Testing tools:

- **nextest**: Test runner (used by `just test-rust`).
- **trycmd**: CLI integration tests for `clawless-cli`.
- **trybuild**: UI tests for macros (pass and compile-fail).

#### Test Structure

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

#### Error Assertions

For error cases, use `expect_err` in the Act phase:

```rust
#[tokio::test]
async fn parse_with_invalid_input_returns_error() {
    let input = "not a number";

    let error = parse(input).expect_err("should fail");

    assert!(error.to_string().contains("invalid digit"));
}
```

#### Required Tests

- Trait tests (Send, Sync, Unpin) for every custom type.
- Do not test compiler-derived traits (Eq, Ord, Hash, Clone, etc.). Only test
  auto traits and custom behavior, such as a builder round-trip.

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

#### Summary Line

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

#### Comment Style

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

#### Required Sections

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
- Paths like `super::` and `crate::` should not appear in rendered
  documentation

```rust
// DO
/// Returns [`Option<T>`] if the value exists
///
/// [`Option<T>`]: std::option::Option

// DON'T
/// Returns `Option` if the value exists
```

#### Depth

Documentation should explain the "why", not just the "what". Write it for a
reader that has no prior context, and especially no knowledge of the
conversation that led to the code.

Write for a consumer of the published crate. A published crate bundles neither
the specifications nor the design notes, so never reference them from a doc
comment. Internal rationale, such as which library a function hides, stays out
of the documentation as well. Document what the API does, what it requires of
the caller, and how it fails.

What that means for each kind of item:

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

#### Module vs Type Docs

- Module docs: high-level summaries, when to use this module.
- Type docs: comprehensive, self-contained.
- Some duplication between module and type docs is acceptable.

#### Language

Use American English spelling: "color" not "colour", "serialize" not
"serialise".

Use the `/simple-english::simple-english` skill to adhere to the ASD-STE100
standard for Simplified Technical English.

---

## Markdown

- Use title case in headings and titles.
- Always use the Oxford comma.
- Use reference-style Markdown links, not inline links.
- Table cells must be single-line. Markdown does not support multi-line cells;
  each newline starts a new row. Ignore line length limits for table rows.

## Git

### Commit Messages

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
- Wrap the body at 72 characters.
- Write commit messages for a reader that has no prior context and no access
  to the session history.
- Keep commit messages concise. Aim for two or three paragraphs, not more.
- **Never** write conventional commit messages
- **Never** add yourself as a co-author.
- Commit messages should be Markdown. Don't use backticks in commit message
  titles, but do use them in bodies.

### Commit Quality

- **Never commit directly to main**: Always create a feature branch and submit a
  pull request.
- **Atomic commits**: Each commit should be a logical unit of change.
- **Bisect-able history**: Every commit must build and pass all checks.
- **Separate concerns**: Format fixes and refactoring should be in separate
  commits from feature changes.
- **One primary commit per pull request**: the primary commit carries the
  well-crafted message, and that is what lands in the history. Follow-up
  fixups within the same pull request can use one-liner messages, because
  they get squashed into the primary commit on merge.
- **Diff against the baseline when reversing or modifying a prior commit**: use
  `git diff <commit>~1` (against the working tree) to verify you haven't
  introduced unintentional changes relative to the pre-commit state.

### Pull Requests

Assign every pull request to yourself with `--assignee @me`. The title is the
summary line of the primary commit.

The description carries the same prose as the commit message, but not the same
line breaks. GitHub renders a single newline in a description as a line break,
so a body wrapped at 72 characters arrives narrow and ragged. Reflow each
paragraph onto one line, and keep the blank line between paragraphs:

```bash
gh pr create --assignee @me --title "Summary line" --body "First paragraph on one line.

Second paragraph on one line."
```

Do not use `--fill`. It copies the commit body verbatim, wrapping included.

---

## Acknowledgments

This `AGENTS.md` file was adopted from
[nextest's AGENTS.md][nextest-agents], which is published under the
Apache-2.0 or MIT license.

[flox]: https://flox.dev
[nextest-agents]: https://github.com/nextest-rs/nextest/blob/main/AGENTS.md
[specs-readme]: specs/README.md
[tbaggery]: https://tbaggery.com/2008/04/19/a-note-about-git-commit-messages.html
[trycmd]: https://docs.rs/trycmd/latest/trycmd/
[trybuild]: https://docs.rs/trybuild/latest/trybuild/
