//! Running external programs from a command
//!
//! Most command-line applications drive other programs. A release tool calls
//! `git`, a build tool calls `cargo`, and a deployment tool calls a provider's
//! own client. This module defines [`Process`], the interface that runs such a
//! program and reports it through the event system while it runs.
//!
//! A run reports itself as [`ProcessEvent`]s: one for the start, one for every
//! line that the program writes, and one for the end. A presenter therefore
//! shows a long build as it happens, and a TUI can keep the last few lines of
//! the program in a corner of the screen. The command's own code is the same in
//! both cases.
//!
//! The description of a command is an [`Invocation`], and the result of a run
//! is an [`Execution`]. Both come from the `kawauso-process` crate, which owns
//! the mechanics of starting a program and reading its streams. This module
//! re-exports what a command names, so that an application needs Clawless
//! alone.
//!
//! # Examples
//!
//! ```no_run
//! use clawless_core::process::{Invocation, Process};
//!
//! # async fn example(process: Process) -> Result<(), Box<dyn std::error::Error>> {
//! let execution = process
//!     .run(Invocation::new("git").arg("status").arg("--short"))
//!     .await?
//!     .require_success()?;
//!
//! let status = execution.stdout().to_string_lossy();
//! # Ok(())
//! # }
//! ```
//!
//! [`ProcessEvent`]: crate::event::process::ProcessEvent

use std::time::{Duration, Instant};

use bon::Builder;
pub use kawauso_process::error::{RequireSuccessError, RunCommandError};
pub use kawauso_process::execution::Capture;
pub use kawauso_process::invocation::{Argument, Program, WorkingDirectory};
pub use kawauso_process::run::{Line, Stream, Text};
pub use kawauso_process::{Execution, Invocation, ProcessId, Run};

pub use self::error::RunProcessError;
use crate::cancellation::Cancellation;
use crate::event::SendError;
use crate::event::process::{Outcome, ProcessEvent, RunId};
use crate::output::Output;

/// The time a program gets to end itself before Clawless kills it
///
/// A program that answers the request costs what it takes to answer, so this
/// period is only ever paid in full by a program that ignores it. Five seconds
/// is therefore generous where it costs nothing and still short enough that a
/// user who pressed Ctrl+C on such a program does not think the application
/// ignored them. A program that cleans up for longer says so through the
/// builder.
const DEFAULT_GRACE_PERIOD: Duration = Duration::from_secs(5);

/// The error of running an external program from a command
mod error;

/// The way in which the reading of the output of a program ended
///
/// A read that ends because the program closed its streams still owes the
/// caller the exit status, and a read that ends because cancellation stopped
/// the run owes nothing. The two ends therefore travel apart.
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
enum Streamed {
    /// Cancellation stopped the run
    Cancelled,

    /// Both output streams of the program reached their end
    Closed,
}

/// Runs external programs and reports them through the event system
///
/// `Process` holds the two things that a run needs: the [`Output`] that carries
/// its events, and the [`Cancellation`] that stops it. [`Context::process`]
/// builds one from the context of a command, which is how a command reaches
/// this interface.
///
/// A run reports every line that the program writes as it arrives, so a
/// presenter can render the progress of a program that takes minutes. The same
/// lines also reach the caller in the [`Execution`], which means that a command
/// can show the output and read it afterwards without running the program
/// twice.
///
/// Cancellation ends the program. A command that observes its cancellation token
/// therefore does not leave a build running after the user pressed Ctrl+C, and
/// it needs no code of its own to achieve that. The program is asked to end
/// first and killed only if it does not, so a build tool gets the moment it
/// needs to remove its lock file. The [builder] names how long that moment is.
///
/// [builder]: Process::builder
///
/// `Process` is cheaply clonable. Cloning produces another handle to the same
/// event channel and the same cancellation token.
///
/// # Examples
///
/// ```no_run
/// use clawless_core::process::{Invocation, Process};
///
/// # async fn example(process: Process) -> Result<(), Box<dyn std::error::Error>> {
/// let execution = process.run(Invocation::new("cargo").arg("build")).await?;
///
/// let succeeded = execution.status().success();
/// # Ok(())
/// # }
/// ```
///
/// [`Context::process`]: crate::context::Context::process
// r[impl process.new]
// r[impl process.new.grace]
// r[impl process.safety.clone]
// r[impl process.safety.send]
// r[impl process.safety.sync]
#[derive(Clone, Debug, Builder)]
pub struct Process {
    /// The channel that carries the events of a run
    output: Output,

    /// The token that stops a run
    #[builder(default)]
    cancellation: Cancellation,

    /// The time a program gets to end itself before Clawless kills it
    #[builder(default = DEFAULT_GRACE_PERIOD)]
    grace_period: Duration,
}

impl Process {
    /// Runs a program and reports its output while it runs
    ///
    /// The method starts the program, sends every line that it writes into the
    /// event channel as the line arrives, waits for the program to end, and
    /// returns what the program produced. The [`Execution`] holds the exit
    /// status, both output streams as the program wrote them, and the time that
    /// the run took.
    ///
    /// A program that ends without success is not a failure of this method. The
    /// status travels in the [`Execution`], and
    /// [`require_success`][success] turns a status that the caller cannot
    /// accept into an error.
    ///
    /// Cancellation ends the program: it is asked to end, and killed only if it
    /// does not answer within the grace period. The run then returns
    /// [`CancelledRun`][cancelled], and the events of the run end with an
    /// [`Outcome::Cancelled`].
    ///
    /// The program starts with a standard input that is null, so a program that
    /// asks for a password ends instead of waiting for an answer. It inherits
    /// the environment of the application, and the operating system resolves
    /// its name.
    ///
    /// The future of this method owns the program. A caller that drops it, in a
    /// timeout of its own for example, kills the program with it and no event
    /// reports the end of the run. Stop a run through the cancellation token
    /// instead, which asks the program to end and closes its events.
    ///
    /// # Errors
    ///
    /// Returns [`UnrunnableCommand`][unrunnable] when the program does not
    /// start or the run produces no result, [`CancelledRun`][cancelled] when
    /// cancellation stopped the program, and
    /// [`UnreportableOutput`][unreportable] when the presenter stopped
    /// listening while the program ran.
    ///
    /// # Panics
    ///
    /// Panics when no Tokio runtime drives the future. Commands always run
    /// under one.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use clawless_core::process::{Invocation, Process};
    ///
    /// # async fn example(process: Process) -> Result<(), Box<dyn std::error::Error>> {
    /// let execution = process
    ///     .run(Invocation::new("git").arg("rev-parse").arg("HEAD"))
    ///     .await?
    ///     .require_success()?;
    ///
    /// let commit = execution.stdout().to_string_lossy().trim().to_owned();
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// [`Outcome::Cancelled`]: crate::event::process::Outcome::Cancelled
    /// [cancelled]: RunProcessError::CancelledRun
    /// [success]: Execution::require_success
    /// [unreportable]: RunProcessError::UnreportableOutput
    /// [unrunnable]: RunProcessError::UnrunnableCommand
    // r[impl process.run]
    // r[impl process.run.capture]
    // r[impl process.run.stream]
    pub async fn run(&self, invocation: Invocation) -> Result<Execution, RunProcessError> {
        let started = Instant::now();
        let id = RunId::next();

        let mut run = invocation
            .start()
            .map_err(|source| RunProcessError::UnrunnableCommand { source })?;

        let event = ProcessEvent::Started {
            id,
            invocation: invocation.clone(),
            process_id: run.id(),
        };
        self.report(&invocation, event).await?;

        let streamed = self.stream(id, &invocation, &mut run).await;

        match streamed {
            Ok(Streamed::Closed) => self.complete(id, invocation, run, started).await,
            Ok(Streamed::Cancelled) => Err(self.cancel(id, invocation, run, started).await),
            Err(error) => {
                drop(run.stop(self.grace_period).await);
                drop(
                    self.end(id, &invocation, Outcome::Incomplete, started)
                        .await,
                );

                Err(error)
            }
        }
    }

    /// Ends a run that cancellation stopped and reports it
    ///
    /// The program is asked to end and killed only if it does not answer within
    /// the grace period, so a build tool that holds a lock file gets the moment
    /// it needs to remove it.
    ///
    /// What the program writes while it ends reaches the capture of the run and
    /// not the event channel, because the ending owns the streams. A consumer
    /// therefore sees the lines up to the cancellation and then the end of the
    /// run.
    ///
    /// The grace period bounds the whole ending, so a program that ignores the
    /// request costs that period and a program that answers costs what it takes
    /// to answer.
    ///
    /// The result of the ending is dropped. The caller is cancelling, so it
    /// discards the output of the run either way, and a program that could not
    /// be waited for is no longer this application's concern.
    // r[impl process.run.cancel.grace]
    // r[impl process.run.cancel.bound]
    async fn cancel(
        &self,
        id: RunId,
        invocation: Invocation,
        run: Run,
        started: Instant,
    ) -> RunProcessError {
        drop(run.stop(self.grace_period).await);
        drop(self.end(id, &invocation, Outcome::Cancelled, started).await);

        RunProcessError::CancelledRun { invocation }
    }

    /// Reports every line of the program until it stops writing
    ///
    /// The read and the cancellation token race on every line, so a program
    /// that writes without end still stops when the user asks for it. The
    /// cancellation branch comes first, which means that a token that is
    /// already cancelled ends the run instead of reporting one more line.
    ///
    /// # Errors
    ///
    /// Returns [`UnrunnableCommand`][unrunnable] when a stream of the program
    /// cannot be read, and [`UnreportableOutput`][unreportable] when a line
    /// cannot be reported.
    ///
    /// [unreportable]: RunProcessError::UnreportableOutput
    /// [unrunnable]: RunProcessError::UnrunnableCommand
    // r[impl process.run.cancel]
    async fn stream(
        &self,
        id: RunId,
        invocation: &Invocation,
        run: &mut Run,
    ) -> Result<Streamed, RunProcessError> {
        loop {
            let line = tokio::select! {
                biased;

                () = self.cancellation.cancelled() => return Ok(Streamed::Cancelled),
                line = run.next_line() => line,
            };

            let line = line.map_err(|source| RunProcessError::UnrunnableCommand { source })?;

            let Some(line) = line else {
                return Ok(Streamed::Closed);
            };

            self.report(invocation, ProcessEvent::Line { id, line })
                .await?;
        }
    }

    /// Waits for a program that stopped writing and reports how it ended
    ///
    /// A program can close both of its output streams and keep running, so the
    /// wait races the cancellation token as the read does.
    ///
    /// The wait leaves the handle where it is, so cancellation here ends the
    /// program the same way it does while the program is writing: a request,
    /// the grace period, and a kill only if the program does not answer.
    ///
    /// # Errors
    ///
    /// Returns [`CancelledRun`][cancelled] when cancellation stopped the
    /// program, [`UnrunnableCommand`][unrunnable] when the end of the program
    /// cannot be waited for, and [`UnreportableOutput`][unreportable] when the
    /// end of the run cannot be reported.
    ///
    /// [cancelled]: RunProcessError::CancelledRun
    /// [unreportable]: RunProcessError::UnreportableOutput
    /// [unrunnable]: RunProcessError::UnrunnableCommand
    // r[impl process.run.cancel]
    async fn complete(
        &self,
        id: RunId,
        invocation: Invocation,
        mut run: Run,
        started: Instant,
    ) -> Result<Execution, RunProcessError> {
        let ended = tokio::select! {
            biased;

            () = self.cancellation.cancelled() => None,
            ended = run.wait_for_end() => Some(ended),
        };

        let Some(ended) = ended else {
            return Err(self.cancel(id, invocation, run, started).await);
        };

        let waited = match ended {
            Ok(()) => run.wait().await,
            Err(source) => {
                drop(run.stop(self.grace_period).await);

                Err(source)
            }
        };

        let execution = match waited {
            Ok(execution) => execution,
            Err(source) => {
                drop(
                    self.end(id, &invocation, Outcome::Incomplete, started)
                        .await,
                );

                return Err(RunProcessError::UnrunnableCommand { source });
            }
        };

        let outcome = Outcome::Exited(execution.status());

        self.end(id, &invocation, outcome, started)
            .await
            .map_err(|source| RunProcessError::UnreportableOutput { invocation, source })?;

        Ok(execution)
    }

    /// Reports the end of a run
    ///
    /// A caller that already has an error to report drops the result of this
    /// method. The consumer that would have read the event is the one that is
    /// gone, so a second failure says nothing that the first one does not.
    ///
    /// # Errors
    ///
    /// Returns [`SendError`] if the consumer of the events has stopped
    /// listening.
    // r[impl process.run.lifecycle]
    async fn end(
        &self,
        id: RunId,
        invocation: &Invocation,
        outcome: Outcome,
        started: Instant,
    ) -> Result<(), SendError> {
        let event = ProcessEvent::Finished {
            id,
            invocation: invocation.clone(),
            outcome,
            duration: started.elapsed(),
        };

        self.output.process_event(event).await
    }

    /// Sends one event of a run, naming the command if the send fails
    ///
    /// # Errors
    ///
    /// Returns [`UnreportableOutput`][unreportable] if the consumer of the
    /// events has stopped listening.
    ///
    /// [unreportable]: RunProcessError::UnreportableOutput
    async fn report(
        &self,
        invocation: &Invocation,
        event: ProcessEvent,
    ) -> Result<(), RunProcessError> {
        self.output.process_event(event).await.map_err(|source| {
            RunProcessError::UnreportableOutput {
                invocation: invocation.clone(),
                source,
            }
        })
    }
}

#[cfg(test)]
mod tests {
    // An assertion in a test panics by design. A `# Panics` section on every test
    // would repeat that and give the reader no information.
    #![allow(clippy::missing_panics_doc)]

    #[cfg(unix)]
    use std::fs::read_to_string;
    use std::time::Duration;

    use kawauso_process::run::Stream;
    #[cfg(unix)]
    use tempfile::TempDir;
    use tokio::time::{sleep, timeout};

    use super::*;
    use crate::event::{Event, EventReceiver, event_channel};

    /// Returns the events of every run that the channel holds
    ///
    /// The channel closes when the test drops the handle that sent the events,
    /// so the drain reads to the end and needs no timeout of its own.
    async fn drain(mut receiver: EventReceiver) -> Vec<ProcessEvent> {
        let mut events = Vec::new();

        while let Some(event) = receiver.recv().await {
            match event {
                Event::Process(event) => events.push(*event),
                Event::Message(_) | Event::Detail(_) | Event::Artifact(_) => {}
            }
        }

        events
    }

    /// Returns whether the error reports a run that cancellation stopped
    fn is_cancelled(error: &RunProcessError) -> bool {
        match error {
            RunProcessError::CancelledRun { .. } => true,
            RunProcessError::UnrunnableCommand { .. } => false,
            RunProcessError::UnreportableOutput { .. } => false,
        }
    }

    /// Returns every line of output that the events carry, in the order of arrival
    fn lines(events: &[ProcessEvent]) -> Vec<(Stream, String)> {
        events
            .iter()
            .filter_map(|event| match event {
                ProcessEvent::Line { line, .. } => {
                    Some((line.stream(), line.text().get().to_owned()))
                }
                ProcessEvent::Started { .. } => None,
                ProcessEvent::Finished { .. } => None,
            })
            .collect()
    }

    /// Returns the outcome of the run that the events describe
    fn outcome(events: &[ProcessEvent]) -> Option<Outcome> {
        events.iter().find_map(|event| match event {
            ProcessEvent::Finished { outcome, .. } => Some(outcome.clone()),
            ProcessEvent::Started { .. } => None,
            ProcessEvent::Line { .. } => None,
        })
    }

    /// Returns an invocation that gives the commands to the shell of the platform
    ///
    /// A test needs a program that writes what the test expects, and the shell
    /// is the program that every machine has. The separator between two
    /// commands differs between the platforms, so the helper joins them for the
    /// shell that runs them.
    fn shell(commands: &[&str]) -> Invocation {
        if cfg!(windows) {
            Invocation::new("cmd").arg("/C").arg(commands.join(" & "))
        } else {
            Invocation::new("sh").arg("-c").arg(commands.join("; "))
        }
    }

    // r[verify process.new]
    // r[verify process.safety.clone]
    #[tokio::test]
    async fn clone_reports_through_the_same_channel() {
        let (sender, receiver) = event_channel();
        let process = Process::builder().output(Output::new(sender)).build();
        let clone = process.clone();

        clone.run(shell(&["exit 0"])).await.expect("should run");
        drop(process);
        drop(clone);

        assert_eq!(drain(receiver).await.len(), 2);
    }

    // r[verify process.event.finished]
    #[tokio::test]
    async fn run_reports_the_command_and_the_time_of_the_run() {
        let (sender, receiver) = event_channel();
        let process = Process::builder().output(Output::new(sender)).build();
        let invocation = shell(&["exit 0"]);

        process.run(invocation.clone()).await.expect("should run");
        drop(process);

        assert_eq!(
            drain(receiver).await.iter().find_map(|event| match event {
                ProcessEvent::Finished {
                    invocation,
                    duration,
                    ..
                } => Some((invocation.to_string(), *duration > Duration::ZERO)),
                ProcessEvent::Started { .. } => None,
                ProcessEvent::Line { .. } => None,
            }),
            Some((invocation.to_string(), true))
        );
    }

    // r[verify process.event.lifecycle]
    // r[verify process.run.lifecycle]
    #[tokio::test]
    async fn run_reports_the_lifecycle_of_the_program() {
        let (sender, receiver) = event_channel();
        let process = Process::builder().output(Output::new(sender)).build();

        process
            .run(shell(&["echo only"]))
            .await
            .expect("should run");
        drop(process);

        assert_eq!(
            drain(receiver)
                .await
                .iter()
                .map(|event| match event {
                    ProcessEvent::Started { .. } => "started",
                    ProcessEvent::Line { .. } => "line",
                    ProcessEvent::Finished { .. } => "finished",
                })
                .collect::<Vec<_>>(),
            vec!["started", "line", "finished"]
        );
    }

    // r[verify process.event.line]
    // r[verify process.run]
    // r[verify process.run.stream]
    #[tokio::test]
    async fn run_reports_the_lines_of_the_program() {
        let (sender, receiver) = event_channel();
        let process = Process::builder().output(Output::new(sender)).build();

        process
            .run(shell(&["echo one", "echo two"]))
            .await
            .expect("should run");
        drop(process);

        assert_eq!(
            lines(&drain(receiver).await),
            vec![
                (Stream::StandardOutput, "one".to_owned()),
                (Stream::StandardOutput, "two".to_owned()),
            ]
        );
    }

    // r[verify process.event.outcome]
    // r[verify process.event.outcome.exited]
    #[tokio::test]
    async fn run_reports_the_outcome_of_the_program() {
        let (sender, receiver) = event_channel();
        let process = Process::builder().output(Output::new(sender)).build();

        let execution = process.run(shell(&["exit 0"])).await.expect("should run");
        drop(process);

        assert_eq!(
            outcome(&drain(receiver).await),
            Some(Outcome::Exited(execution.status()))
        );
    }

    // r[verify process.event.started.identifier]
    #[tokio::test]
    async fn run_reports_the_identifier_of_the_program() {
        let (sender, receiver) = event_channel();
        let process = Process::builder().output(Output::new(sender)).build();

        process.run(shell(&["exit 0"])).await.expect("should run");
        drop(process);

        assert_eq!(
            drain(receiver).await.iter().find_map(|event| match event {
                ProcessEvent::Started { process_id, .. } => Some(process_id.is_some()),
                ProcessEvent::Line { .. } => None,
                ProcessEvent::Finished { .. } => None,
            }),
            Some(true)
        );
    }

    // r[verify process.event.started]
    #[tokio::test]
    async fn run_reports_the_start_of_the_program() {
        let (sender, receiver) = event_channel();
        let process = Process::builder().output(Output::new(sender)).build();
        let invocation = shell(&["exit 0"]);

        process.run(invocation.clone()).await.expect("should run");
        drop(process);

        assert_eq!(
            drain(receiver).await.first().map(ProcessEvent::to_string),
            Some(format!("$ {invocation}"))
        );
    }

    // r[verify process.run.capture]
    #[tokio::test]
    async fn run_returns_the_capture_of_the_program() {
        let (sender, _receiver) = event_channel();
        let process = Process::builder().output(Output::new(sender)).build();

        let execution = process
            .run(shell(&["echo hello"]))
            .await
            .expect("should run");

        assert_eq!(execution.stdout().to_string_lossy().trim(), "hello");
    }

    // r[verify process.event.line]
    #[tokio::test]
    async fn run_separates_the_streams_of_the_program() {
        let (sender, receiver) = event_channel();
        let process = Process::builder().output(Output::new(sender)).build();

        process
            .run(shell(&["echo out", "echo err 1>&2"]))
            .await
            .expect("should run");
        drop(process);

        let mut found = lines(&drain(receiver).await);
        found.sort();

        assert_eq!(
            found,
            vec![
                (Stream::StandardError, "err".to_owned()),
                (Stream::StandardOutput, "out".to_owned()),
            ]
        );
    }

    // r[verify process.run.cancel.grace]
    #[cfg(unix)]
    #[tokio::test]
    async fn run_with_a_cancelled_token_lets_the_program_end_itself() {
        let directory = TempDir::new().expect("should create a directory");
        let marker = directory.path().join("cleaned");
        let cancellation = Cancellation::new();
        let (sender, mut receiver) = event_channel();
        let process = Process::builder()
            .output(Output::new(sender))
            .cancellation(cancellation.clone())
            .build();
        let invocation = shell(&[
            &format!("trap 'touch \"{}\"; exit 0' TERM", marker.display()),
            "echo ready",
            "sleep 20 & wait",
        ]);

        let handle = tokio::spawn(async move { process.run(invocation).await });
        while let Some(event) = receiver.recv().await {
            let ready = match event {
                Event::Process(event) => match *event {
                    ProcessEvent::Line { .. } => true,
                    ProcessEvent::Started { .. } => false,
                    ProcessEvent::Finished { .. } => false,
                },
                Event::Message(_) | Event::Detail(_) | Event::Artifact(_) => false,
            };

            if ready {
                break;
            }
        }
        cancellation.cancel();
        timeout(Duration::from_secs(20), handle)
            .await
            .expect("should not time out")
            .expect("should join")
            .expect_err("should fail");

        assert!(marker.exists());
    }

    // r[verify process.run.cancel]
    #[cfg(unix)]
    #[tokio::test]
    async fn run_with_a_cancelled_token_kills_the_program() {
        let directory = TempDir::new().expect("should create a directory");
        let beats = directory.path().join("beats");
        let cancellation = Cancellation::new();
        let (sender, _receiver) = event_channel();
        let process = Process::builder()
            .output(Output::new(sender))
            .cancellation(cancellation.clone())
            .build();
        let invocation = shell(&[&format!(
            "while true; do echo beat >> '{}'; sleep 0.05; done",
            beats.display()
        )]);

        let handle = tokio::spawn(async move { process.run(invocation).await });
        sleep(Duration::from_millis(300)).await;
        cancellation.cancel();
        timeout(Duration::from_secs(30), handle)
            .await
            .expect("should not time out")
            .expect("should join")
            .expect_err("should fail");
        let written = read_to_string(&beats).expect("should read the beats");
        written
            .lines()
            .next()
            .expect("the program should have written a beat");
        sleep(Duration::from_millis(400)).await;

        assert_eq!(
            read_to_string(&beats).expect("should read the beats"),
            written
        );
    }

    // r[verify process.event.outcome.cancelled]
    #[tokio::test]
    async fn run_with_a_cancelled_token_reports_the_cancellation() {
        let cancellation = Cancellation::new();
        let (sender, receiver) = event_channel();
        let process = Process::builder()
            .output(Output::new(sender))
            .cancellation(cancellation.clone())
            .build();
        cancellation.cancel();

        process
            .run(shell(&["sleep 30"]))
            .await
            .expect_err("should fail");
        drop(process);

        assert_eq!(outcome(&drain(receiver).await), Some(Outcome::Cancelled));
    }

    // r[verify process.run.cancel.error]
    #[tokio::test]
    async fn run_with_a_cancelled_token_returns_an_error() {
        let cancellation = Cancellation::new();
        let (sender, _receiver) = event_channel();
        let process = Process::builder()
            .output(Output::new(sender))
            .cancellation(cancellation.clone())
            .build();
        cancellation.cancel();

        let error = process
            .run(shell(&["sleep 30"]))
            .await
            .expect_err("should fail");

        assert!(is_cancelled(&error));
    }

    // r[verify process.run.report.error]
    #[tokio::test]
    async fn run_with_a_closed_channel_returns_an_error() {
        let (sender, receiver) = event_channel();
        let process = Process::builder().output(Output::new(sender)).build();
        drop(receiver);

        let error = process
            .run(shell(&["exit 0"]))
            .await
            .expect_err("should fail");

        assert!(matches_unreportable_output(&error));
    }

    // r[verify process.run.error]
    #[tokio::test]
    async fn run_with_a_missing_program_returns_an_error() {
        let (sender, _receiver) = event_channel();
        let process = Process::builder().output(Output::new(sender)).build();

        let error = process
            .run(Invocation::new("clawless-no-such-program"))
            .await
            .expect_err("should fail");

        assert!(matches_unrunnable_command(&error));
    }

    // r[verify process.run.cancel.grace]
    #[cfg(unix)]
    #[tokio::test]
    async fn run_with_a_program_that_closed_its_streams_ends_itself_on_cancellation() {
        let directory = TempDir::new().expect("should create a directory");
        let marker = directory.path().join("cleaned");
        let cancellation = Cancellation::new();
        let (sender, _receiver) = event_channel();
        let process = Process::builder()
            .output(Output::new(sender))
            .cancellation(cancellation.clone())
            .build();
        let invocation = shell(&[
            &format!("trap 'touch \"{}\"; exit 0' TERM", marker.display()),
            "exec 1>&- 2>&-",
            "sleep 20 & wait",
        ]);

        let handle = tokio::spawn(async move { process.run(invocation).await });
        sleep(Duration::from_millis(250)).await;
        cancellation.cancel();
        timeout(Duration::from_secs(30), handle)
            .await
            .expect("should not time out")
            .expect("should join")
            .expect_err("should fail");

        assert!(marker.exists());
    }

    // r[verify process.run.cancel]
    #[cfg(unix)]
    #[tokio::test]
    async fn run_with_a_program_that_closed_its_streams_stops_on_cancellation() {
        let cancellation = Cancellation::new();
        let (sender, _receiver) = event_channel();
        let process = Process::builder()
            .output(Output::new(sender))
            .cancellation(cancellation.clone())
            .build();
        let invocation = shell(&["exec 1>&- 2>&-", "sleep 30"]);

        let handle = tokio::spawn(async move { process.run(invocation).await });
        sleep(Duration::from_millis(250)).await;
        cancellation.cancel();
        let error = timeout(Duration::from_secs(30), handle)
            .await
            .expect("should not time out")
            .expect("should join")
            .expect_err("should fail");

        assert!(is_cancelled(&error));
    }

    // r[verify process.new.grace]
    // r[verify process.run.cancel.bound]
    #[cfg(unix)]
    #[tokio::test]
    async fn run_with_a_program_that_leaves_a_child_returns_within_the_grace_period() {
        let cancellation = Cancellation::new();
        let (sender, _receiver) = event_channel();
        let process = Process::builder()
            .output(Output::new(sender))
            .cancellation(cancellation.clone())
            .grace_period(Duration::from_millis(200))
            .build();
        let invocation = shell(&["echo ready", "sleep 20"]);

        let handle = tokio::spawn(async move { process.run(invocation).await });
        sleep(Duration::from_millis(200)).await;
        cancellation.cancel();
        let started = Instant::now();
        timeout(Duration::from_secs(20), handle)
            .await
            .expect("should not time out")
            .expect("should join")
            .expect_err("should fail");

        assert!(started.elapsed() < Duration::from_secs(5));
    }

    #[tokio::test]
    async fn run_with_an_unsuccessful_program_returns_the_status() {
        let (sender, _receiver) = event_channel();
        let process = Process::builder().output(Output::new(sender)).build();

        let execution = process.run(shell(&["exit 3"])).await.expect("should run");

        assert_eq!(execution.status().code(), Some(3));
    }

    // r[verify process.run.cancel]
    #[tokio::test]
    async fn run_with_cancellation_while_the_program_runs_returns_an_error() {
        let cancellation = Cancellation::new();
        let (sender, mut receiver) = event_channel();
        let process = Process::builder()
            .output(Output::new(sender))
            .cancellation(cancellation.clone())
            .grace_period(Duration::from_millis(200))
            .build();
        let invocation = shell(&["echo ready", "sleep 30"]);

        let handle = tokio::spawn(async move { process.run(invocation).await });
        while let Some(event) = receiver.recv().await {
            let ready = match event {
                Event::Process(event) => match *event {
                    ProcessEvent::Line { .. } => true,
                    ProcessEvent::Started { .. } => false,
                    ProcessEvent::Finished { .. } => false,
                },
                Event::Message(_) | Event::Detail(_) | Event::Artifact(_) => false,
            };

            if ready {
                break;
            }
        }
        cancellation.cancel();
        let error = timeout(Duration::from_secs(30), handle)
            .await
            .expect("should not time out")
            .expect("should join")
            .expect_err("should fail");

        assert!(is_cancelled(&error));
    }

    // r[verify process.safety.send]
    #[test]
    fn trait_send() {
        fn assert_send<T: Send>() {}
        assert_send::<Process>();
    }

    // r[verify process.safety.sync]
    #[test]
    fn trait_sync() {
        fn assert_sync<T: Sync>() {}
        assert_sync::<Process>();
    }

    #[test]
    fn trait_unpin() {
        fn assert_unpin<T: Unpin>() {}
        assert_unpin::<Process>();
    }

    /// Returns whether the error reports output that could not be reported
    fn matches_unreportable_output(error: &RunProcessError) -> bool {
        match error {
            RunProcessError::UnreportableOutput { .. } => true,
            RunProcessError::CancelledRun { .. } => false,
            RunProcessError::UnrunnableCommand { .. } => false,
        }
    }

    /// Returns whether the error reports a program that could not be run
    fn matches_unrunnable_command(error: &RunProcessError) -> bool {
        match error {
            RunProcessError::UnrunnableCommand { .. } => true,
            RunProcessError::CancelledRun { .. } => false,
            RunProcessError::UnreportableOutput { .. } => false,
        }
    }
}
