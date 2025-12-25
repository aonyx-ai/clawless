---
sidebar_position: 4
---

# Test with trycmd

Use [trycmd](https://docs.rs/trycmd) to write snapshot tests for your CLI
commands. Tests are defined as simple TOML files with expected output, making
them easy to write and maintain.

## Add trycmd to your project

Add trycmd as a dev dependency:

```bash
cargo add --dev trycmd
```

## Create the test file

Create a test file at `tests/cli.rs`:

```rust
#[test]
fn cli_tests() {
    trycmd::TestCases::new()
        .case("tests/cmd/*.toml");
}
```

This tells trycmd to look for test cases in `tests/cmd/`.

## Write your first test

Create `tests/cmd/greet.toml`:

```toml
bin.name = "my-cli"
args = ["greet", "World"]
status.code = 0
```

This test runs `my-cli greet World` and verifies it exits with code 0.

Run the test:

```bash
cargo test
```

## Test stdout output

To verify the command's output, add a `.stdout` file with the same name:

`tests/cmd/greet.toml`:

```toml
bin.name = "my-cli"
args = ["greet", "World"]
status.code = 0
```

`tests/cmd/greet.stdout`:

```
Hello, World!
```

The test passes if the command's stdout matches the file exactly.

## Test file system changes

For commands that create or modify files, use `.in/` and `.out/` directories:

```
tests/cmd/
├── init.toml
├── init.in/
│   └── (empty or initial files)
└── init.out/
    ├── config.toml
    └── src/
        └── main.rs
```

`tests/cmd/init.toml`:

```toml
bin.name = "my-cli"
args = ["init"]
fs.sandbox = true
status.code = 0
```

With `fs.sandbox = true`:

- The test runs in a temporary directory
- Files from `.in/` are copied to the sandbox before running
- After the command runs, the sandbox is compared to `.out/`

This is useful for testing commands that generate files or modify project
structure.

## Test error cases

Test that commands fail correctly:

`tests/cmd/missing-arg.toml`:

```toml
bin.name = "my-cli"
args = ["deploy"]
status.code = 2
```

`tests/cmd/missing-arg.stderr`:

```
error: the following required arguments were not provided:
  <ENVIRONMENT>
...
```

Use `...` to match any text (useful for help text that may change).

## Update expected output

When your CLI's output changes, update the expected files:

```bash
TRYCMD=dump cargo test
```

This overwrites `.stdout`, `.stderr`, and `.out/` files with the actual output.
Review the changes before committing.

## Project structure

A typical test setup looks like this:

```
my-cli/
├── src/
│   ├── main.rs
│   ├── commands.rs
│   └── commands/
│       └── greet.rs
├── tests/
│   ├── cli.rs
│   └── cmd/
│       ├── greet-default.toml
│       ├── greet-default.stdout
│       ├── greet-name.toml
│       ├── greet-name.stdout
│       ├── greet-help.toml
│       └── greet-help.stdout
└── Cargo.toml
```

## See also

- [trycmd documentation](https://docs.rs/trycmd)
- [Commands](../concepts/commands) - Writing command functions
