#[test]
#[cfg(unix)]
fn sigint_triggers_graceful_cancellation() {
    use std::io::{BufRead, BufReader};
    use std::process::Stdio;

    #[allow(deprecated)]
    use assert_cmd::cargo::CommandCargoExt;

    #[allow(deprecated)]
    let mut child = std::process::Command::cargo_bin("hello-world")
        .unwrap()
        .arg("wait")
        .stdout(Stdio::piped())
        .spawn()
        .expect("should start hello-world");

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
