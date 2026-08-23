# 🦦 Clawless

`clawless` is an opinionated, batteries-included framework for building
command-line applications with Rust. It features low-level building blocks and
high-level abstractions that can be used to build stateless CLIs or stateful
TUI applications.

This crate is the facade of the framework. It re-exports [`clawless-core`],
[`clawless-cli`], and [`clawless-tui`], along with the macros from
[`clawless-derive`], so that applications need only a single dependency. The
full documentation and guides are available at [clawless.rs].

## Usage

The quickest way to create a new project is the scaffolding tool:

```shell
cargo install cargo-clawless
cargo clawless new my-cli
```

To set up a project manually instead, create a new binary crate with
`cargo new --bin <name>`. Then add `clawless` as a dependency. Inside the
crate, open `src/main.rs` and replace the generated contents with the
following snippet:

```rust,ignore
mod commands;

clawless::main!();
```

Next, create `src/commands.rs` to set up your commands module:

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
pub async fn greet(args: GreetArgs, context: Context) -> CommandResult {
    message!("Hello, {}!", args.name);
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
├── commands.rs
└── commands/
    ├── greet.rs
    ├── db.rs
    └── db/
        ├── migrate.rs
        └── seed.rs
```

With this structure:

- `cargo run -- greet` runs the `greet` command
- `cargo run -- db migrate` runs the `db::migrate` command
- `cargo run -- db seed` runs the `db::seed` command

Parent modules declare their children, so `src/commands/db.rs` contains
`mod migrate;` and `mod seed;`.

## License

Licensed under either of

- Apache License, Version 2.0 (<http://www.apache.org/licenses/LICENSE-2.0>)
- MIT license (<http://opensource.org/licenses/MIT>)

at your option.

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.

[clawless.rs]: https://clawless.rs
[`clawless-cli`]: https://docs.rs/clawless-cli
[`clawless-core`]: https://docs.rs/clawless-core
[`clawless-derive`]: https://docs.rs/clawless-derive
[`clawless-tui`]: https://docs.rs/clawless-tui
