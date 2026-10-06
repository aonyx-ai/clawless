//! Integration tests for the `cargo clawless` scaffolding commands
//!
//! Each `.toml` case in `tests/commands/` gives one invocation. A case lists the arguments,
//! the expected output, and the directory tree that the command must produce.
//!
//! The `README.md` case runs the examples from the README of the crate. The documentation
//! therefore stays correct.
//!
//! The test registers the binary with the path that Cargo gives at compile time. trycmd
//! then does not have to guess the build directory, which changes between Cargo versions.

// An assertion in a test panics by design. A `# Panics` section on every test
// would repeat that and give the reader no information.
#![allow(clippy::missing_panics_doc)]

use std::path::Path;

#[test]
fn commands() {
    trycmd::TestCases::new()
        .register_bin(
            "cargo-clawless",
            Path::new(env!("CARGO_BIN_EXE_cargo-clawless")),
        )
        .case("tests/commands/*.toml")
        .case("../../README.md");
}
