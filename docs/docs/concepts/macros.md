---
sidebar_position: 4
---

# Macros

Clawless uses four procedural macros plus three output macros to wire up your
CLI application. Understanding how these macros work together helps you debug
issues and appreciate the convention-based design.

## Procedural macros

### `clawless::main!()`

Called in `src/main.rs` to generate your application entry point.

**What it does:**

1. Generates the `main()` function
2. Builds the clap command tree and parses arguments
3. Resolves the subcommand tree to find the target leaf
4. Delegates to the appropriate runner (`CommandRunner` for CLI commands,
   `ApplicationRunner` for TUI applications)

The macro uses a two-phase dispatch strategy: first it resolves which leaf the
user invoked, then it hands off to the runner that matches the leaf type. This
separation keeps the generated code small and lets the runners manage their own
lifecycles (event channels, presenters, projections, signal handlers).

**Usage:**

```rust
// src/main.rs
mod commands;

clawless::main!();
```

### `clawless::commands!()`

Called in `src/commands.rs` to set up the root command.

**What it does:**

1. Creates a root command using your crate's description from `Cargo.toml`
2. Provides an entry point for the inventory system to collect subcommands
3. Generates initialization and resolve functions for the root level

**Usage:**

```rust
// src/commands.rs
mod greet;
mod deploy;

clawless::commands!();
```

### `#[command]`

Marks a function as a CLI command, generating the glue code to integrate it with
Clap and the inventory system.

**What it does:**

1. Generates a Clap `Command` with help text from doc comments
2. Generates a resolve function that returns the command as a leaf for dispatch
3. Registers the command with the inventory system for discovery

**Usage:**

```rust
#[derive(Debug, Args)]
pub struct GreetArgs {
    name: String,
}

/// Greet the user
#[command]
pub async fn greet(args: GreetArgs, context: Context) -> CommandResult {
    message!("Hello, {}!", args.name);
    Ok(())
}
```

### `#[application]`

Marks a function as a TUI application leaf. Works like `#[command]` but accepts
a third parameter for pull-based event consumption and is executed through
`ApplicationRunner` instead of `CommandRunner`.

**What it does:**

1. Generates a Clap `Command` with help text from doc comments (same as
   `#[command]`)
2. Generates a resolve function that returns the application as a leaf for
   dispatch
3. Registers the application with the inventory system for discovery

**Usage:**

```rust
#[derive(Debug, Args)]
pub struct DashboardArgs {}

/// Interactive dashboard
#[application]
pub async fn dashboard(
    args: DashboardArgs,
    context: Context,
    projection: Projection,
) -> CommandResult {
    // Use projection to consume events in a pull-based loop
    Ok(())
}
```

Application functions must accept exactly three parameters: arguments, context,
and projection. The `Projection` provides a queryable view of execution state
for stateful rendering (e.g., with ratatui).

### Generated function names

Each `#[command]` or `#[application]` on a function named `greet` generates two
companion functions: `greet_init` (builds the clap `Command`) and
`greet_resolve` (returns a `ResolvedLeaf` for dispatch). You may encounter these
names in compiler errors, stack traces, or `cargo expand` output.

Similarly, `commands!()` generates `clawless_init` and `clawless_resolve` at the
root level.

## Output macros

Clawless provides three output macros that offer a convenient shorthand for
writing to the framework-controlled [output](./output) system. These macros
access the `context` parameter that every command and application receives, so
they can only be used inside `#[command]` and `#[application]` functions.

### `message!`

Writes an informational message that the user should see during normal
operation. Suppressed by `--quiet`.

```rust
message!("Deploying to {}", environment);
message!("Done");
```

`message!` accepts the same arguments as `format!`. A single string literal
with no formatting arguments is also accepted.

### `detail!`

Writes a detail message that is only shown when `--verbose` is passed.
Use this for progress information, diagnostics, or other context that helps
during debugging but would be noisy in normal use.

```rust
detail!("loading configuration from {}", path.display());
detail!("connection established");
```

### `artifact!`

Writes the primary data output of a command. This is the machine-readable
payload: the thing a script would pipe to `jq`, or the structured result a
user specifically asked for. When `--json` is active, `artifact!` emits JSON;
otherwise it uses the type's `Display` implementation.

```rust
artifact!(BuildResult { success: true });
artifact!(users);
```

Unlike `message!` and `detail!`, `artifact!` takes a single expression rather
than a format string.

### When to use each macro

| Macro       | Purpose                             | Shown by default | Hidden by `--quiet` | Requires `--verbose` |
| ----------- | ----------------------------------- | ---------------- | ------------------- | -------------------- |
| `message!`  | Informational messages for the user | Yes              | Yes                 | No                   |
| `detail!`   | Debug and progress information      | No               | Yes                 | Yes                  |
| `artifact!` | Primary command data or results     | Yes              | No                  | No                   |

## How they work together

Here's the flow when your CLI runs:

```
1. User runs: myapp greet World

2. main!() generates main():
   - Calls clawless_init() to build the full clap command tree
   - Clap parses arguments

3. Resolution phase:
   - Calls clawless_resolve(matches) to walk the subcommand tree
   - Each level checks its inventory-registered children
   - Finds greet_resolve(), which returns a ResolvedLeaf

4. Execution phase:
   - main() matches on the ResolvedLeaf variant
   - For commands: CommandRunner sets up the event channel,
     context, presenter, and signal handler, then runs the command
   - For applications: ApplicationRunner sets up a projection
     instead of a presenter for pull-based rendering
```

## The inventory system

Clawless uses the [`inventory`](https://docs.rs/inventory) crate for command
discovery. This is what enables convention-based registration without manual
wiring.

**How it works:**

1. Each `#[command]` macro submits a registration to the inventory at compile
   time
2. The `commands!()` macro generates code that collects all registered commands
3. At runtime, the inventory provides the complete command tree to Clap

This approach means:

- No central registry file to maintain
- Adding a command is just `mod new_command;` + `#[command]`

## Macro attributes

### `main!()` attributes

The `main!()` macro accepts no attributes.

### `commands!()` attributes

The `commands!()` macro accepts no attributes.

### `#[command]` and `#[application]` attributes

Both macros accept the same optional attributes:

- **`alias = "name"`** - Add a command alias
- **`require_subcommand`** - Prevent execution without a subcommand

See [Commands](./commands#macro-attributes) for details.

## Debugging generated code

If you need to see what the macros generate, use `cargo expand`:

```bash
# Install cargo-expand
cargo install cargo-expand

# Expand a specific file
cargo expand --bin myapp

# View just main.rs expansion
cargo expand --bin myapp main
```

This shows the actual Rust code generated by the macros, which can help debug
issues or understand behavior.

## Limitations

Understanding the macros helps you work within their constraints:

**Function signature requirements:**

- Commands and applications must be `pub async fn`
- Commands must accept exactly two parameters: args, then context
- Applications must accept exactly three parameters: args, context, then
  projection
- Both must return `CommandResult`

These requirements enable the macros to generate correct wrapper code.

**Module structure requirements:**

- `commands.rs` must exist
- Must call `commands!()` macro in commands module

These conventions enable the inventory system to discover commands.

## What's next

Now that you understand the macro system, learn about:

- **[Commands](./commands)** - What the `#[command]` macro does to functions
- **[Project Structure](./project-structure)** - How module hierarchy becomes
  command hierarchy
- **[Naming Conventions](./naming-conventions)** - How names map to CLI commands
