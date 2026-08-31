//! The vocabulary of external programs
//!
//! Most command-line applications drive other programs. This module names what
//! such a program is described by and what it produces, so that a command, a
//! presenter, and a projection all speak of a run in the same terms.
//!
//! The description of a command is an [`Invocation`]: a program, its arguments,
//! and optionally the directory it runs in. It is a value, so an application
//! can build one, write it to a log, and name it in an error without running
//! anything.
//!
//! The result of a run is an [`Execution`]: the exit status, what the program
//! wrote to each of its streams, and the time the run took. A status that is
//! not a success is data rather than a failure — the check mode of a formatter
//! ends without success when it finds a file to format, and that is the answer
//! the caller asked for.
//!
//! These types come from the `kawauso-process` crate, which owns the mechanics
//! of starting a program and reading its streams. This module re-exports what
//! an application names, so that it needs Clawless alone.
//!
//! # Examples
//!
//! ```
//! use clawless_core::process::Invocation;
//!
//! let invocation = Invocation::new("git").arg("status").arg("--short");
//!
//! assert_eq!(invocation.to_string(), "git status --short");
//! ```

pub use kawauso_process::error::{RequireSuccessError, RunCommandError};
pub use kawauso_process::execution::Capture;
pub use kawauso_process::invocation::{Argument, Program, WorkingDirectory};
pub use kawauso_process::run::{Line, Stream, Text};
pub use kawauso_process::{Execution, Invocation, ProcessId, Run};
