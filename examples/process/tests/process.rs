//! Integration tests for the external program example
//!
//! Each `.toml` case in `tests/process/` runs the example against a program that every Unix
//! machine has. The cases cover what the command reports at each verbosity, in each output
//! mode, and when the program fails or does not exist at all.

// An assertion in a test panics by design. A `# Panics` section on every test
// would repeat that and give the reader no information.
#![allow(clippy::missing_panics_doc)]

// r[verify process.render.verbosity]
// r[verify process.render.streams]
#[test]
fn process() {
    trycmd::TestCases::new().case("tests/process/*.toml");
}
