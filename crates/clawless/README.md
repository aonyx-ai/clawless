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
clawless::main()!;
```

You can now start creating commands for your application. Inside `src/main.rs`,
add the following line at the top:

```rust
pub mod command;
```

Then go ahead, create `src/command.rs`, and add a struct and a function to
the file:

```rust
use clap::Args;
use clawless::command;

#[derive(Debug, Args)]
pub struct CommandArgs {
    #[arg(short, long)]
    name: String,
}

#[command]
pub async fn command(args: CommandArgs) {
    println!("Running a command: {}", args.name);
}
```

You can execute the command by calling your command-line application:

```shell
cargo run -- command
```

## License

Licensed under either of

- Apache License, Version 2.0 (<http://www.apache.org/licenses/LICENSE-2.0>)
- MIT license (<http://opensource.org/licenses/MIT>)

at your option.

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.
