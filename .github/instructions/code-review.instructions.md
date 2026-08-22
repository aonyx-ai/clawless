---
applyTo: "**/*"
excludeAgent: "coding-agent"
---

# Clawless code review instructions

## Project identity

Clawless is a Rust CLI framework built around domain-driven design and hexagonal
architecture. The authoritative source for domain vocabulary and architectural
boundaries is [`specs/architecture.md`][architecture].

The full set of coding conventions lives in [`AGENTS.md`][agents-md]. These
instructions orient you to the project's architecture and spec-driven
development process; they do not duplicate the coding rules.

## Domain model

The architecture defines 13 domain concepts organized into three layers:

- **Core entities** (identity matters, have lifecycles): Application, Command,
  Argument, Context, Task, Hook
- **Value objects** (data, state, or tokens): Progress, Artifact, Diagnostic,
  Outcome, Cancellation
- **Ports** (interfaces with swappable adapters): Formatter, Prompt

Use these terms precisely. If a PR introduces a new concept, it should be
consistent with this vocabulary.

## Hexagonal architecture

The architecture follows a ports-and-adapters pattern with a strict dependency
rule:

- **Domain core** does not depend on any particular runtime, terminal, or I/O
  mechanism.
- **Ports** define contracts that the domain declares.
- **Adapters** are concrete implementations that plug into ports. Swappable per
  environment (interactive terminal, CI, testing, scripting).

Arrows flow outward: domain -> ports -> adapters. The domain never depends on a
specific adapter.

## Spec-driven review

When reviewing a PR:

1. **Identify domain concepts touched.** Determine which of the 13 domain
   concepts the change involves.
2. **Check for a feature spec.** Look in `specs/features/` and
   `specs/projects/` for a relevant specification. See
   [`specs/README.md`][specs] for the spec template and process.
3. **Verify hexagonal boundaries.** Domain core must not depend on adapters.
   Adapters depend on port interfaces, not on each other.
4. **Verify ubiquitous language.** New or modified concepts should use the terms
   defined in the architecture spec, not synonyms or ad hoc names.

## Architecture red flags

Flag these in review:

- **Domain types importing infrastructure or adapters.** The domain core must
  remain independent of concrete I/O, rendering, or platform concerns.
- **New public types not documented in a spec.** Significant new domain concepts
  should have a feature spec before implementation begins.
- **Terminology drift.** Using "job" instead of "Task", "result" instead of
  "Outcome", "error" instead of "Diagnostic", or other terms that diverge from
  the ubiquitous language.
- **Commands accessing I/O directly.** Commands should produce structured data
  (Artifacts, Progress, Diagnostics) that flows through ports, not call
  `println!` or interact with the terminal directly — except where the current
  implementation status makes this acceptable (see the "current state" table in
  the architecture spec).

## Spec verification checklist

When a feature spec exists for the work under review, verify:

- [ ] All functional requirements from the spec are implemented
- [ ] Edge cases documented in the spec are tested
- [ ] API surface matches the spec (types, function signatures, prelude exports)
- [ ] Out-of-scope items listed in the spec are not accidentally included
- [ ] Non-functional requirements (portability, performance) are addressed
- [ ] Terminology in the implementation matches the spec and architecture

## Project structure

```text
crates/
  clawless/              # Core framework library
  clawless-derive/       # Procedural macros
  clawless-cli/          # CLI scaffolding tool
examples/
  hello-world/           # Reference example project
specs/
  architecture.md        # Domain model and ubiquitous language
  features/              # Feature specifications
  projects/              # Project specifications
```

## Coding standards

For the full set of coding conventions — including Rust-specific rules, type
system guidelines, error handling, testing patterns, documentation standards,
commit message format, and Markdown conventions — refer to
[`AGENTS.md`][agents-md].

Key principles from the project philosophy:

- **Correctness over convenience**: model the full error space, handle all edge
  cases, use the type system to encode constraints.
- **User experience as a primary driver**: structured error messages, responsive
  progress reporting, clear present-tense user-facing messages.
- **Pragmatic incrementalism**: prefer specific, composable logic over abstract
  frameworks.

## Markdown conventions

All Markdown in this project uses sentence case headings (not title case), the
Oxford comma, and reference-style links.

[architecture]: ../../specs/architecture.md
[agents-md]: ../../AGENTS.md
[specs]: ../../specs/README.md
