//! Integration tests for signal-driven cancellation
//!
//! The test starts the example as a real child process and sends it a real signal. Only the
//! operating system can exercise the full path from a signal to a cancellation.

#![allow(
    clippy::missing_panics_doc,
    reason = "an assertion in a test panics by design, so a `# Panics` section would repeat it"
)]

use std::io::{BufRead, BufReader, Lines};
use std::process::{Child, ChildStdout, Stdio};

use assert_cmd::cargo::*;

/// Starts the example and returns it with the lines that it writes to its standard output
#[allow(
    clippy::expect_used,
    clippy::unwrap_used,
    reason = "a child that does not start, or that has no standard output, fails the test"
)]
fn start() -> (Child, Lines<BufReader<ChildStdout>>) {
    let mut child = std::process::Command::new(cargo_bin!("cancellation"))
        .arg("wait")
        .stdout(Stdio::piped())
        .spawn()
        .expect("should start cancellation");

    let stdout = child.stdout.take().unwrap();

    (child, BufReader::new(stdout).lines())
}

/// Sends SIGINT to a child that the test has not yet reaped
#[allow(
    unsafe_code,
    reason = "sending a signal to another process has no safe equivalent in the standard library, and the target is a child this test spawned and has not yet reaped, so the pid is valid"
)]
fn interrupt(child: &Child) {
    unsafe { libc::kill(child.id() as libc::pid_t, libc::SIGINT) };
}

#[test]
fn sigint_ends_the_process_successfully() {
    let (mut child, mut lines) = start();
    lines.next().unwrap().unwrap();
    interrupt(&child);

    let status = child.wait().unwrap();

    assert!(status.success());
}

#[test]
fn sigint_triggers_graceful_cancellation() {
    let (mut child, mut lines) = start();
    lines.next().unwrap().unwrap();
    interrupt(&child);

    let line = lines.next().unwrap().unwrap();
    child.wait().unwrap();

    assert_eq!(line, "cancelled");
}

#[test]
fn wait_announces_that_it_is_waiting() {
    let (mut child, mut lines) = start();

    let line = lines.next().unwrap().unwrap();
    interrupt(&child);
    child.wait().unwrap();

    assert_eq!(line, "waiting");
}
