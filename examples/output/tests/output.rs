//! Integration tests for the output example
//!
//! Each `.toml` case in `tests/output/` runs the example with a different set of output
//! flags. The case then asserts the stdout and the stderr. Together the cases cover how
//! verbosity and output mode interact.

// An assertion in a test panics by design. A `# Panics` section on every test
// would repeat that and give the reader no information.
#![allow(clippy::missing_panics_doc)]

#[test]
fn output() {
    trycmd::TestCases::new().case("tests/output/*.toml");
}
