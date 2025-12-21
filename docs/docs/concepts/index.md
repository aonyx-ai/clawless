---
sidebar_position: 2
---

# Concepts

This section explains the core concepts and design principles behind Clawless.
Understanding these concepts will help you make the most of the framework and
build CLIs that follow best practices.

## Core concepts

### [Commands](./commands)

Commands are the building blocks of your CLI. Learn how the `#[command]` macro
transforms regular Rust functions into CLI commands, how command functions are
structured, and what happens under the hood.

### [Arguments](./arguments)

Arguments define the inputs your commands accept. Discover how Clawless uses
Clap's derive API for type-safe argument parsing, how to define flags and
options, and how arguments integrate with commands.

### [Context](./context)

The Context system provides commands with access to framework features. Explore
how Context gives you environment information today and will provide access to
configuration, output abstractions, and more in the future.

### [Macros](./macros)

Clawless uses three macros to wire up your CLI. Understand how `main!`,
`commands!`, and `#[command]` work together to generate the glue code that makes
convention-based development possible.

### [Project Structure](./project-structure)

Your file structure becomes your command structure. Learn how Clawless maps
module hierarchies to command hierarchies, where to place command files, and how
to organize large CLIs.

### [Naming Conventions](./naming-conventions)

Names matter in Clawless. See how file names, function names, and module names
automatically become command names, subcommands, and help text.

## Design principles

These are the principles that guided the design of Clawless:

### Convention over configuration

Clawless makes decisions for you so you can focus on building features. By
following conventions, the framework is able to provide sensible defaults and
advanced features without requiring extensive configuration.

### Batteries included

Common CLI needs like configuration management, structured output, and logging
are built into the framework rather than requiring you to integrate separate
libraries. This ensures consistency across projects and reduces the decision
fatigue of choosing and configuring dependencies.

### Rapid development

Clawless optimizes for speed of development. The scaffolding CLI (
`clawless new`, `clawless generate command`) gets you from idea to working code
quickly.

### Progressive disclosure

Start simple and add complexity only when needed. A basic command is just a
function with a macro. As your needs grow, you can add arguments, use context
features, organize into nested hierarchies, and leverage advanced framework
capabilities.

## Learning path

If you're new to Clawless:

1. Start with **[Commands](./commands)** and **[Arguments](./arguments)** to
   understand the basics
2. Learn about **[Context](./context)** to access framework features
3. Explore **[Project Structure](./project-structure)** to understand how to
   organize your CLI
4. Read **[Macros](./macros)** if you're curious about implementation details
5. Reference **[Naming Conventions](./naming-conventions)** when you need to
   look up specific rules

For experienced developers, the **[Macros](./macros)** section provides insight
into how Clawless generates code, which can help with debugging and
understanding limitations.
