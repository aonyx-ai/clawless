//! Integration tests for the external program example
//!
//! Each `.toml` case in `tests/process/` runs the example against a program that every Unix
//! machine has. The cases cover what the command reports at each verbosity, in each output
//! mode, and when the program fails or does not exist at all.

#![allow(
    clippy::missing_panics_doc,
    reason = "an assertion in a test panics by design, so a `# Panics` section would repeat it"
)]

#[test]
fn process() {
    trycmd::TestCases::new().case("tests/process/*.toml");
}
