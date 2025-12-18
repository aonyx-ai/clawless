---
sidebar_position: 1
---

# Arguments

Arguments define what inputs your commands accept from users. In Clawless,
arguments are defined using structs that derive Clap's `Args` trait, giving you
full access to Clap's powerful argument parsing capabilities.

## Defining arguments

Every command needs an arguments struct, even if it takes no arguments:

```rust
use clawless::prelude::*;

#[derive(Debug, Args)]
pub struct GreetArgs {
    /// The name to greet
    #[arg(default_value = "World")]
    name: String,
}
```

The struct must:

- Derive `Args` from Clap (imported via `clawless::prelude::*`)
- Be public (`pub`)

Field-level doc comments are optional but recommended - they become the help
text shown to users.

## Clap integration

Clawless uses Clap's derive API for argument parsing. This means you have access
to all of Clap's features through the `#[arg(...)]` attribute on struct fields.

### Positional arguments

Fields without `#[arg]` attributes become positional arguments:

```rust
#[derive(Debug, Args)]
pub struct DeployArgs {
    /// The environment to deploy to
    environment: String,
}
```

Usage: `myapp deploy production`

### Options (flags with values)

Use `#[arg(short, long)]` to create named options:

```rust
#[derive(Debug, Args)]
pub struct ServerArgs {
    /// Port to listen on
    #[arg(short, long, default_value = "8080")]
    port: u16,

    /// Host address
    #[arg(short = 'H', long, default_value = "localhost")]
    host: String,
}
```

Usage:

```bash
myapp server --port 3000 --host 0.0.0.0
myapp server -p 3000 -H 0.0.0.0
```

### Boolean flags

Use `bool` type for flags that don't take values:

```rust
#[derive(Debug, Args)]
pub struct BuildArgs {
    /// Enable verbose output
    #[arg(short, long)]
    verbose: bool,

    /// Skip running tests
    #[arg(long)]
    skip_tests: bool,
}
```

Usage:

```bash
myapp build --verbose
myapp build -v --skip-tests
```

### Optional arguments

Use `Option<T>` for optional arguments:

```rust
#[derive(Debug, Args)]
pub struct ConnectArgs {
    /// API key (optional)
    #[arg(long, env = "API_KEY")]
    api_key: Option<String>,
}
```

The value is `None` if not provided, `Some(value)` if provided.

### Multiple values

Use `Vec<T>` to accept multiple values:

```rust
#[derive(Debug, Args)]
pub struct AddArgs {
    /// Files to add
    #[arg(required = true)]
    files: Vec<String>,
}
```

Usage: `myapp add file1.txt file2.txt file3.txt`

## Doc comments as help text

Doc comments on fields become the help text shown to users:

```rust
#[derive(Debug, Args)]
pub struct DeployArgs {
    /// The environment to deploy to.
    ///
    /// Valid environments: dev, staging, production
    environment: String,

    /// Skip confirmation prompts
    #[arg(short = 'y', long)]
    yes: bool,
}
```

Running `myapp deploy --help` shows:

```
Arguments:
  <ENVIRONMENT>  The environment to deploy to.

                 Valid environments: dev, staging, production

Options:
  -y, --yes  Skip confirmation prompts
  -h, --help Print help
```

The first line becomes the short description, and subsequent lines become the
long description.

## Empty arguments structs

Commands that don't take arguments still need an arguments struct:

```rust
#[derive(Debug, Args)]
pub struct VersionArgs {}
```

This is a requirement of the type system but has no runtime overhead.

## Type safety

Clap validates and parses arguments at runtime, converting strings to the
appropriate Rust types:

```rust
#[derive(Debug, Args)]
pub struct ConfigArgs {
    #[arg(short, long)]
    timeout: u64,        // Parsed from string to u64

    #[arg(short, long)]
    retries: usize,      // Parsed from string to usize

    #[arg(short, long)]
    endpoint: url::Url,  // Any type implementing FromStr works
}
```

## Advanced Clap features

Since Clawless uses Clap's derive API directly, you have access to all Clap
features:

- **Value parsers** - Custom parsing logic
- **Value hints** - Shell completion hints
- **Argument groups** - Group related arguments
- **Custom validation** - Validate argument combinations
- **Styling** - Customize help text appearance

Refer to the [Clap documentation](https://docs.rs/clap/latest/clap/) for
comprehensive coverage of all available attributes and features.

## What's next

Now that you understand arguments, learn about:

- **[Commands](./commands)** - How arguments integrate with command functions
- **[Context](./context)** - The other parameter commands receive
- **[Naming Conventions](./naming-conventions)** - How argument struct names
  relate to commands
