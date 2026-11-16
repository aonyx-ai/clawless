//! Integration tests for the exit example
//!
//! Each `.toml` case in `tests/exit_code/` runs the example with a different version or with
//! different output flags. The case then asserts the exit code, the stdout, and the stderr of the
//! run.

#![allow(
    clippy::missing_panics_doc,
    reason = "an assertion in a test panics by design, so a `# Panics` section would repeat it"
)]

#[test]
fn exit_code() {
    trycmd::TestCases::new().case("tests/exit_code/*.toml");
}
