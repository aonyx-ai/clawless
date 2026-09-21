//! Integration tests for the prompt example without a terminal
//!
//! A test has no terminal, so every case in `tests/prompt/` runs the example without a user. The
//! cases cover the output of the commands at each verbosity and in each output mode.

// An assertion in a test panics by design. A `# Panics` section on every test
// would repeat that and give the reader no information.
#![allow(clippy::missing_panics_doc)]

#[test]
fn prompt() {
    trycmd::TestCases::new().case("tests/prompt/*.toml");
}
