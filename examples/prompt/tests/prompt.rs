//! Integration tests for the prompt example without a terminal
//!
//! A test has no terminal, so every case in `tests/prompt/` runs the example without a user. The
//! cases cover the output of the commands at each verbosity and in each output mode.
//!
//! The test registers the binary with the path that Cargo gives at compile time. trycmd
//! then does not have to guess the build directory, which changes between Cargo versions.

// An assertion in a test panics by design. A `# Panics` section on every test
// would repeat that and give the reader no information.
#![allow(clippy::missing_panics_doc)]

use std::path::Path;

#[test]
fn prompt() {
    trycmd::TestCases::new()
        .register_bin("prompt", Path::new(env!("CARGO_BIN_EXE_prompt")))
        .case("tests/prompt/*.toml");
}
