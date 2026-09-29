//! Integration tests for the exit example
//!
//! Each `.toml` case in `tests/exit_code/` runs the example with a different version or with
//! different output flags. The case then asserts the exit code, the stdout, and the stderr of the
//! run.

// An assertion in a test panics by design. A `# Panics` section on every test
// would repeat that and give the reader no information.
#![allow(clippy::missing_panics_doc)]

#[test]
fn exit_code() {
    trycmd::TestCases::new().case("tests/exit_code/*.toml");
}
