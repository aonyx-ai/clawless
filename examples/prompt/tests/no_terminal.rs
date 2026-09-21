//! Integration test for a prompt whose input is not a terminal
//!
//! The test starts the example as a child process with its standard input from the null device.
//! Only a real process covers the path from the streams of the process to the error of the
//! prompt.

// An assertion in a test panics by design. A `# Panics` section on every test
// would repeat that and give the reader no information.
#![allow(clippy::missing_panics_doc)]

use std::process::Stdio;

use assert_cmd::cargo::*;

#[test]
fn confirm_with_the_input_from_the_null_device_shows_the_message_of_the_command() {
    let output = std::process::Command::new(cargo_bin!("prompt"))
        .arg("confirm")
        .stdin(Stdio::null())
        .output()
        .expect("should run the example");

    let stdout = String::from_utf8_lossy(&output.stdout);

    assert_eq!(
        stdout,
        "About to release version 1.4.0.\n\
         Nobody can confirm the release, because no terminal is present.\n"
    );
}
