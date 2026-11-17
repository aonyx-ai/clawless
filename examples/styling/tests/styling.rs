//! Integration tests for the styling example
//!
//! Each `.toml` case in `tests/styling/` runs the example with a different set of variables in
//! its environment. The case then asserts which form of the text reaches stdout. The cases run
//! without the environment of the test, so that a variable of the machine cannot change them.

// An assertion in a test panics by design. A `# Panics` section on every test
// would repeat that and give the reader no information.
#![allow(clippy::missing_panics_doc)]

#[test]
fn styling() {
    trycmd::TestCases::new().case("tests/styling/*.toml");
}
