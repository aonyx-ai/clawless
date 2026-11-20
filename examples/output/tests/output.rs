//! Integration tests for the output example
//!
//! Each `.toml` case in `tests/output/` runs the example with a different set of output
//! flags. The case then asserts the stdout and the stderr. Together the cases cover how
//! verbosity and output mode interact.

#![allow(
    clippy::missing_panics_doc,
    reason = "an assertion in a test panics by design, so a `# Panics` section would repeat it"
)]

#[test]
fn output() {
    trycmd::TestCases::new().case("tests/output/*.toml");
}
