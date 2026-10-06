//! Integration tests for the exit example
//!
//! Each `.toml` case in `tests/exit_code/` runs the example with a different version or with
//! different output flags. The case then asserts the exit code, the stdout, and the stderr of the
//! run.
//!
//! The test registers the binary with the path that Cargo gives at compile time. trycmd
//! then does not have to guess the build directory, which changes between Cargo versions.

// An assertion in a test panics by design. A `# Panics` section on every test
// would repeat that and give the reader no information.
#![allow(clippy::missing_panics_doc)]

use std::path::Path;

#[test]
fn exit_code() {
    trycmd::TestCases::new()
        .register_bin("exit-code", Path::new(env!("CARGO_BIN_EXE_exit-code")))
        .case("tests/exit_code/*.toml");
}
