//! Integration tests for the external program example
//!
//! Each `.toml` case in `tests/process/` runs the example against a program that every Unix
//! machine has. The cases cover what the command reports at each verbosity, in each output
//! mode, and when the program fails or does not exist at all.
//!
//! The test registers the binary with the path that Cargo gives at compile time. trycmd
//! then does not have to guess the build directory, which changes between Cargo versions.

// An assertion in a test panics by design. A `# Panics` section on every test
// would repeat that and give the reader no information.
#![allow(clippy::missing_panics_doc)]

use std::path::Path;

// r[verify process.render.verbosity]
// r[verify process.render.streams]
#[test]
fn process() {
    trycmd::TestCases::new()
        .register_bin("process", Path::new(env!("CARGO_BIN_EXE_process")))
        .case("tests/process/*.toml");
}
