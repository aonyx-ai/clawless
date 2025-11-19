# 🦦 `clawless-derive`

`clawless` is a framework for building command-line applications with Rust, and
the `clawless-derive` crate implements the procedural macros that power this
framework.

The crate defines three main macros which the `clawless` crate re-exports:

- `main!` - Generates the application entry point and main function
- `commands!` - Sets up the commands module with the root command and inventory
- `#[command]` - Marks functions as CLI commands and registers them

The typical structure is:

```rust
// src/main.rs
mod commands;
clawless::main!();

// src/commands.rs
mod my_command;
clawless::commands!();
```

The `commands!` macro generates the root command for your CLI, while the
`#[command]` macro does the heavy lifting of creating individual commands and
registering them with their parent module.

Check the documentation of the macros to get a better understanding of how
this crate works.

## License

Licensed under either of

- Apache License, Version 2.0 (<http://www.apache.org/licenses/LICENSE-2.0>)
- MIT license (<http://opensource.org/licenses/MIT>)

at your option.

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.
