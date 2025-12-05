# 🦦 Clawless CLI

`clawless-cli` is the official command-line tool for working with Clawless
projects. It provides scaffolding and code generation capabilities to help you
quickly build and extend command-line applications.

The CLI itself is built using the Clawless framework, serving as both a useful
tool and a reference implementation.

## Installation

Install the CLI using `cargo`:

```shell
cargo install clawless-cli
```

## Commands

### `clawless new`

Create a new Clawless project with a complete setup:

```shell
clawless new my-app
```

This command:

- Creates a new binary crate with `cargo new`
- Adds `clawless` as a dependency
- Sets up the project structure with `main.rs` and `commands.rs`
- Creates a sample `greet` command to get you started

### `clawless generate command`

Generate a new command in an existing Clawless project:

```shell
clawless generate command my-command
```

For nested commands, use slash notation:

```shell
clawless generate command db/migrate
```

This command:

- Creates the command file with boilerplate code
- Adds the necessary `mod` statement to the parent module
- Supports nested command hierarchies

## Usage in Projects

The typical workflow is:

1. Create a new project:

   ```shell
   clawless new my-cli
   cd my-cli
   ```

2. Generate additional commands:

   ```shell
   clawless generate command deploy
   clawless generate command config/set
   ```

3. Build and run your CLI:

   ```shell
   cargo run -- greet World
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
