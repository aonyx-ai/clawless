//! Compiler user-interface tests for the Clawless derive macros
//!
//! The `pass-*.rs` cases must compile. Each case therefore proves that one macro invocation
//! style expands to valid code.
//!
//! The `fail-*.rs` cases must not compile. A test compares their stderr against a `.stderr`
//! file in the repository, which keeps the diagnostics helpful as the macros change.

// An assertion in a test panics by design. A `# Panics` section on every test
// would repeat that and give the reader no information.
#![allow(clippy::missing_panics_doc)]

#[test]
fn ui() {
    let t = trybuild::TestCases::new();
    t.pass("tests/ui/pass-*.rs");
    t.compile_fail("tests/ui/fail-*.rs");
}
