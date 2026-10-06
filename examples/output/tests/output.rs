//! Integration tests for the output example
//!
//! Each `.toml` case in `tests/output/` runs the example with a different set of output
//! flags. The case then asserts the stdout and the stderr. Together the cases cover how
//! verbosity and output mode interact.
//!
//! The test registers the binary with the path that Cargo gives at compile time. trycmd
//! then does not have to guess the build directory, which changes between Cargo versions.

// An assertion in a test panics by design. A `# Panics` section on every test
// would repeat that and give the reader no information.
#![allow(clippy::missing_panics_doc)]

use std::path::Path;

#[test]
fn output() {
    trycmd::TestCases::new()
        .register_bin("output", Path::new(env!("CARGO_BIN_EXE_output")))
        .case("tests/output/*.toml");
}
