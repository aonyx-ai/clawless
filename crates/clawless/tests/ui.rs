//! Compiler user-interface tests for the Clawless derive macros
//!
//! The `pass-*.rs` cases must compile. Each case therefore proves that one macro invocation
//! style expands to valid code.
//!
//! The `fail-*.rs` cases must not compile. A test compares their stderr against a `.stderr`
//! file in the repository, which keeps the diagnostics helpful as the macros change.

#![allow(
    clippy::missing_panics_doc,
    reason = "an assertion in a test panics by design, so a `# Panics` section would repeat it"
)]

#[test]
fn ui() {
    let t = trybuild::TestCases::new();

    t.pass("tests/ui/pass-*.rs");
    t.compile_fail("tests/ui/fail-*.rs");
}
