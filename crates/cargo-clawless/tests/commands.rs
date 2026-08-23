//! Integration tests for the `cargo clawless` scaffolding commands
//!
//! Each `.toml` case in `tests/commands/` gives one invocation. A case lists the arguments,
//! the expected output, and the directory tree that the command must produce.
//!
//! The `README.md` case runs the examples from the README of the crate. The documentation
//! therefore stays correct.

// An assertion in a test panics by design. A `# Panics` section on every test
// would repeat that and give the reader no information.
#![allow(clippy::missing_panics_doc)]

#[test]
fn commands() {
    trycmd::TestCases::new()
        .case("tests/commands/*.toml")
        .case("../../README.md");
}
