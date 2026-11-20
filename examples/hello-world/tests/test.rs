//! Integration tests for the hello-world example
//!
//! The tests run the binary and examine its output. They cover the default greeting and an
//! explicit name argument.

#![allow(
    clippy::missing_panics_doc,
    reason = "an assertion in a test panics by design, so a `# Panics` section would repeat it"
)]

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
