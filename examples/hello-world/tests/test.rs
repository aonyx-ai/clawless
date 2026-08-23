//! Integration tests for the hello-world example
//!
//! The tests run the binary and examine its output. They cover the default greeting and an
//! explicit name argument.

// An assertion in a test panics by design. A `# Panics` section on every test
// would repeat that and give the reader no information.
#![allow(clippy::missing_panics_doc)]

use assert_cmd::cargo::*;
use predicates::prelude::*;

#[test]
fn greets_default() {
    let mut cmd = cargo_bin_cmd!("hello-world");

    cmd.arg("greet");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Hello, World!"));
}

#[test]
fn greets_name() {
    let mut cmd = cargo_bin_cmd!("hello-world");

    cmd.arg("greet").arg("Otter");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Hello, Otter!"));
}
