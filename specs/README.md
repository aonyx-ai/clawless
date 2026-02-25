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

| #    | Project                                      | Status   |
| ---- | -------------------------------------------- | -------- |
| P001 | [Cancellation](projects/001-cancellation.md) | Complete |
| P002 | [Output](projects/002-output.md)             | Complete |
| P003 | [Presenter](projects/003-presenter.md)       | Active   |

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

| #    | Feature                                                      | Project      | Status   |
| ---- | ------------------------------------------------------------ | ------------ | -------- |
| F001 | [Cancellation token](features/001-cancellation-token.md)     | Cancellation | Complete |
| F002 | [Signal handling](features/002-signal-handling.md)           | Cancellation | Complete |
| F003 | [Command cancellation](features/003-command-cancellation.md) | Cancellation | Complete |
| F004 | [Output types](features/004-output-types.md)                 | Output       | Complete |
| F005 | [Command output](features/005-command-output.md)             | Output       | Complete |
| F006 | [Event types](features/006-event-types.md)                   | Presenter    | Complete |
| F007 | [Event channel](features/007-event-channel.md)               | Presenter    | Active   |
| F008 | [Presenter](features/008-presenter.md)                       | Presenter    | Active   |
| F009 | [Presenter rendering](features/009-presenter-rendering.md)   | Presenter    | Active   |
| F010 | [Presenter macros](features/010-presenter-macros.md)         | Presenter    | Active   |
| F011 | [Output events](features/011-output-events.md)               | Presenter    | Active   |

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
- **Specs are historical records** - Once a feature ships, its spec is frozen.
  Do not update API references in completed specs when the API changes later.

### Verification checklist

Before considering a spec complete:

- [ ] Terminology matches the architecture's ubiquitous language
- [ ] Hexagonal boundaries are respected (domain vs. ports vs. adapters)
- [ ] Edge cases are documented
- [ ] Open questions are resolved or explicitly deferred
