# 🦦 Clawless

`clawless` is a framework for building command-line applications with Rust. Its
goal is to provide high-level building blocks and well-designed conventions so
that users can focus on their applications.

The library exports a few macros that create a command-line application, parse
arguments, and then call user-defined functions.

## Project Status

Clawless is in a very early prototyping phase and not considered ready for
production use. Follow the project and check out the open issues to understand
the crate's current limitations.

## Usage

First of all, generate a new binary crate using `cargo new --bin <name>`. Inside
the crate, open `src/main.rs` and replace the generated contents with the
following snippet:

```rust,ignore
mod commands;

clawless::main!();
```

Next, create `src/commands.rs` (or `src/commands/mod.rs`) to set up your
commands module:

```rust,ignore
clawless::commands!();
```

You can now start creating commands for your application. Commands should be
defined in modules under the `commands` module. For example, create
`src/commands/greet.rs`:

```rust
use clawless::prelude::*;

#[derive(Debug, Args)]
pub struct GreetArgs {
    #[arg(short, long)]
    name: String,
}

#[command]
pub async fn greet(args: GreetArgs) -> CommandResult {
    println!("Hello, {}!", args.name);
    Ok(())
}
```

Don't forget to declare the module in `src/commands.rs`:

```rust,ignore
mod greet;

clawless::commands!();
```

You can execute the command by calling your command-line application:

```shell
cargo run -- greet --name World
```

### Organizing Commands

For larger applications, you can organize commands into nested modules. The
module hierarchy naturally maps to subcommand groups:

```text
src/
├── main.rs
└── commands/
    ├── mod.rs
    ├── greet.rs
    └── db/
        ├── mod.rs
        ├── migrate.rs
        └── seed.rs
```

With this structure:

- `cargo run -- greet` runs the `greet` command
- `cargo run -- db migrate` runs the `db::migrate` command
- `cargo run -- db seed` runs the `db::seed` command

## License

Licensed under either of

- Apache License, Version 2.0 (<http://www.apache.org/licenses/LICENSE-2.0>)
- MIT license (<http://opensource.org/licenses/MIT>)

at your option.

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.
