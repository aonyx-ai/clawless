# Clawless Specifications

This directory contains the specifications for Clawless, a Rust CLI framework
built around convention over configuration, domain-driven design, and
production-grade engineering.

## Start here

1. **[Architecture](architecture.md)** - Understand the domain model, hexagonal
   architecture, and ubiquitous language

## Project specs

Larger work that should be broken into smaller, atomic changes is considered a
_project_. Projects are documented in `projects/` and have two or more feature
specs.

Each project spec should include:

| Section        | Purpose                                           |
| -------------- | ------------------------------------------------- |
| Summary        | One paragraph description                         |
| Motivation     | Why this project matters                          |
| Feature specs  | Ordered list of feature specs with implementation |
|                | sequence and dependencies                         |
| Out of scope   | Explicitly excluded                               |
| Open questions | Decisions still to make                           |

## Feature specs

Individual feature specifications live in `features/`. Each spec is
self-contained and follows a consistent template.

Each feature spec should include:

| Section                     | Purpose                             |
| --------------------------- | ----------------------------------- |
| Summary                     | One paragraph description           |
| Motivation                  | Why this feature matters            |
| Domain concepts             | Types, traits, and their roles      |
| Functional requirements     | What the feature must do            |
| Non-functional requirements | Portability, etc.                   |
| API surface                 | Public types, functions, macros     |
| Edge cases                  | Error handling, boundary conditions |
| Out of scope                | Explicitly excluded                 |
| Open questions              | Decisions still to make             |

## Contributing to specs

### Workflow

1. **Discuss** - Talk through the feature or concept in conversation
2. **Draft** - Create a comprehensive spec based on the discussion
3. **Review** - Review and provide feedback
4. **Refine** - Iterate until the spec is solid
5. **Commit** - Add the spec on a feature branch, PR to main

### Principles

- **Comprehensive over brief** - Specs should answer questions before they arise
- **Concrete over abstract** - Use specific examples and scenarios
- **Consistent terminology** - Use terms from
  the [architecture](architecture.md)
- **Small increments** - Prefer small, deliverable features over large chunks

### Verification checklist

Before considering a spec complete:

- [ ] Terminology matches the architecture's ubiquitous language
- [ ] Hexagonal boundaries are respected (domain vs. ports vs. adapters)
- [ ] Edge cases are documented
- [ ] Open questions are resolved or explicitly deferred
