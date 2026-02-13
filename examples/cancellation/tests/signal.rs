use std::io::{BufRead, BufReader};
use std::process::Stdio;

use assert_cmd::cargo::*;

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

    unsafe { libc::kill(child.id() as libc::pid_t, libc::SIGINT) };

    let line = lines.next().unwrap().unwrap();
    assert_eq!(line, "cancelled");

    let status = child.wait().unwrap();
    assert!(status.success());
}
