---
applyTo: "**/*.rs"
excludeAgent: "coding-agent"
---

# Rust code review rules

These are the highest-signal Rust conventions for Clawless code review. The
complete set of rules lives in [`AGENTS.md`][agents-md].

## Derive order

Standard traits first, then third-party derives alphabetical by crate:

1. Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Default
2. Third-party: alphabetical by crate, then by macro (e.g., `Builder` from bon,
   then `CopyGetters`, `Getters` from getset)

## Type system

- **Primitives only at boundaries.** `String`, `i64`, `bool`, etc. are only
  allowed at system boundaries (API responses, CLI args). Internal code uses
  newtypes via the `typed-fields` crate.
- **Enums over bools.** Use enums with meaningful variants instead of `bool`
  parameters.
- **Builders with `bon`.** Getters with `getset` (`CopyGetters` for
  `Copy` types, `Getters` for references).

## Control flow

- **Let-else for early returns.** Prefer `let Some(x) = expr else { return; }`
  over `if let` with an else branch.
- **Full match expressions.** Do not use the `matches!` macro.
- **No wildcards.** Match all variants explicitly, except for
  `#[non_exhaustive]` types from external crates.
- **Minimal if-let.** Only acceptable for short actions without an else branch.

## Module organization

- **No `mod.rs` files.** Use file-based modules.
- **Imports at module top.** Never import types or modules within function
  bodies, unless the function is gated by a `cfg()` attribute.
- **One public type per module.** Use submodules for related types.
- **Prefer importing types.** Use fully qualified paths only when conventional
  or necessary for disambiguation.

## Comments and code hygiene

- **Doc comments only.** No inline comments (`//` within function bodies).
- **No section headers or dividers** (`// --- section ---`).
- **No TODO comments.** Use the issue tracker instead.
- **No commented-out code.** Use version control.

## Error handling

- **Always use `.context("description")?`** from `anyhow::Context`.
- **Context messages are lowercase sentence fragments** suitable for "failed to
  {context}" — e.g., `.context("read config file")?`.

## Testing

- **Trait tests for every custom type**: Send, Sync, Unpin.
- **AAA separation**: blank lines between arrange, act, and assert phases.
- **Alphabetical test ordering** within `mod tests`.
- **Descriptive names**: `function_condition_result` — e.g.,
  `parse_with_valid_input_returns_value`.
- **`.expect()` in the act phase**, plain `assert` calls for assertions.
- **Error assertions**: use `.expect_err("should fail")` in the act phase.

## Documentation

- **Third-person singular summary**: "Returns the..." not "Return the..."
- **No trailing period** on the summary line.
- **Required sections** (always plural): Errors, Panics, Safety, Examples.
- **Line comments** (`///`), not block comments (`/** */`).
- **Reference-style links**: `[`Type`]` with link definitions, not inline URLs.

[agents-md]: ../../AGENTS.md
