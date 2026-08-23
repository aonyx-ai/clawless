//! Integration tests for signal-driven cancellation
//!
//! The test starts the example as a real child process and sends it a real signal. Only the
//! operating system can exercise the full path from a signal to a cancellation.

// An assertion in a test panics by design. A `# Panics` section on every test
// would repeat that and give the reader no information.
#![allow(clippy::missing_panics_doc)]

use std::io::{BufRead, BufReader};
use std::process::Stdio;

use assert_cmd::cargo::*;

// r[verify cancel.os.first]
#[test]
fn sigint_triggers_graceful_cancellation() {
    let mut child = std::process::Command::new(cargo_bin!("cancellation"))
        .arg("wait")
        .stdout(Stdio::piped())
        .spawn()
        .expect("should start cancellation");

    let stdout = child.stdout.take().unwrap();
    let mut lines = BufReader::new(stdout).lines();

    let line = lines.next().unwrap().unwrap();
    assert_eq!(line, "waiting");

    // Sending a signal to another process has no safe equivalent in the standard library.
    // The target is a child this test spawned and has not yet reaped, so the pid is valid.
    #[allow(unsafe_code)]
    unsafe {
        libc::kill(child.id() as libc::pid_t, libc::SIGINT)
    };

    let status = child.wait().unwrap();

    let line = lines.next().unwrap().unwrap();
    assert_eq!(line, "cancelled");
    assert!(status.success());
}
