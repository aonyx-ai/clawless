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
