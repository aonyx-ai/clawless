---
applyTo: "**/*"
excludeAgent: "coding-agent"
---

# Clawless code review instructions

## Project identity

Clawless is a Rust CLI framework built around domain-driven design and hexagonal
architecture.

The full set of coding conventions lives in [`AGENTS.md`][agents-md]. These
instructions orient you to the project's architecture; they do not duplicate
the coding rules.

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

## Review process

When reviewing a PR:

1. **Identify domain concepts touched.** Determine which of the 13 domain
   concepts the change involves.
2. **Verify hexagonal boundaries.** Domain core must not depend on adapters.
   Adapters depend on port interfaces, not on each other.
3. **Verify ubiquitous language.** New or modified concepts should use the terms
   of the domain model above, not synonyms or ad hoc names.

## Architecture red flags

Flag these in review:

- **Domain types importing infrastructure or adapters.** The domain core must
  remain independent of concrete I/O, rendering, or platform concerns.
- **Terminology drift.** Using "job" instead of "Task", "result" instead of
  "Outcome", "error" instead of "Diagnostic", or other terms that diverge from
  the ubiquitous language.
- **Commands accessing I/O directly.** Commands should produce structured data
  (Artifacts, Progress, Diagnostics) that flows through ports, not call
  `println!` or interact with the terminal directly.

## Project structure

```text
crates/
  clawless/              # Core framework library
  clawless-derive/       # Procedural macros
  clawless-cli/          # CLI scaffolding tool
examples/
  hello-world/           # Reference example project
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

[agents-md]: ../../AGENTS.md
