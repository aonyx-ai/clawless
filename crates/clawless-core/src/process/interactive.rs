//! The run of a program that takes over the terminal

use std::process::ExitStatus;
use std::time::{Duration, Instant};

use kawauso_process::error::RunCommandError;
use kawauso_process::invocation::Streams;
use kawauso_process::{InteractiveRun, Invocation, ProcessId};

use super::{Process, RunInteractiveProcessError};
use crate::event::SendError;
use crate::event::process::{Outcome, ProcessEvent, RunId};
use crate::event::terminal::{TerminalLoan, TerminalRequest};

/// The way in which a program that held the terminal ended
#[derive(Debug)]
enum Ended {
    /// The program ended by itself with this status
    Exited(ExitStatus),

    /// The end of the program could not be waited for, and Clawless killed it
    Incomplete(RunCommandError),
}

impl Process {
    /// Runs a program that takes over the terminal, and returns its exit status
    ///
    /// Use this method for a program that draws on the terminal or reads the keys of the user.
    /// An editor, a pager, and `ssh` are such programs. Use [`run`] for a program whose output
    /// the command reads.
    ///
    /// The method first asks the presenter for the terminal. The request arrives after the
    /// events that the command sent before it, so the presenter shows that output first. While
    /// the program runs, the presenter writes nothing. Events that other tasks of the command
    /// send in the meantime appear after the program has ended.
    ///
    /// The program reads the standard input and writes its errors to the standard error of the
    /// application. Its standard output goes to the terminal that the presenter chose. Clawless
    /// captures nothing, so the run reports only its start and its end, after the program ended.
    ///
    /// A program that ends without success is not a failure of this method. The method returns
    /// the exit status, and the caller decides what it means. An editor that exits with a
    /// failure, for example, is how a user abandons an edit.
    ///
    /// The future of this method owns the program and the terminal. A caller that drops it kills
    /// the program and every program that it started, gives the terminal back, and reports
    /// nothing.
    ///
    /// # Signals
    ///
    /// On Unix, the program runs in a process group of its own. When the application holds the
    /// foreground of its terminal, the program holds it while it runs. Ctrl+C, Ctrl+\\, and
    /// Ctrl+Z then reach the program alone. They do not cancel the command, and a pager such as
    /// `less` handles Ctrl+C itself. On Windows, every program on the console receives Ctrl+C,
    /// so Ctrl+C cancels the command too.
    ///
    /// # Errors
    ///
    /// Returns [`UnlentTerminal`][unlent] when the presenter did not lend the terminal. Returns
    /// [`UnrunnableCommand`][unrunnable] when the program does not start or its end cannot be
    /// waited for. Returns [`UndeliverableTerminalRequest`][undeliverable] and
    /// [`UnreportableRun`][unreportable] when the presenter stopped listening.
    ///
    /// # Panics
    ///
    /// Panics when no Tokio runtime drives the future. Commands always run under one.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use clawless_core::process::{Invocation, Process};
    ///
    /// # async fn example(process: Process) -> Result<(), Box<dyn std::error::Error>> {
    /// let status = process
    ///     .run_interactive(Invocation::new("vi").arg("CHANGELOG.md"))
    ///     .await?;
    ///
    /// let saved = status.success();
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// [`run`]: Process::run
    /// [undeliverable]: RunInteractiveProcessError::UndeliverableTerminalRequest
    /// [unlent]: RunInteractiveProcessError::UnlentTerminal
    /// [unreportable]: RunInteractiveProcessError::UnreportableRun
    /// [unrunnable]: RunInteractiveProcessError::UnrunnableCommand
    pub async fn run_interactive(
        &self,
        invocation: Invocation,
    ) -> Result<ExitStatus, RunInteractiveProcessError> {
        let loan = self.borrow_terminal(&invocation).await?;
        let (stdout, hold) = loan.into_parts();

        let id = RunId::next();
        let started = Instant::now();

        let run = invocation
            .start_interactive(Streams::inherit().stdout(stdout))
            .map_err(|source| RunInteractiveProcessError::UnrunnableCommand { source })?;
        let process_id = run.id();

        let ended = self.wait_for_program(run).await;
        let duration = started.elapsed();
        drop(hold);

        let outcome = match &ended {
            Ended::Exited(status) => Outcome::Exited(*status),
            Ended::Incomplete(_) => Outcome::Incomplete,
        };
        let reported = self
            .report_interactive(id, &invocation, process_id, outcome, duration)
            .await;

        match ended {
            Ended::Exited(status) => match reported {
                Ok(()) => Ok(status),
                Err(source) => Err(RunInteractiveProcessError::UnreportableRun {
                    invocation: Box::new(invocation),
                    status,
                    source,
                }),
            },
            Ended::Incomplete(source) => {
                Err(RunInteractiveProcessError::UnrunnableCommand { source })
            }
        }
    }

    /// Asks the presenter for the terminal and waits until it lends it
    ///
    /// # Errors
    ///
    /// Returns [`UndeliverableTerminalRequest`][undeliverable] when the presenter stopped
    /// listening, and [`UnlentTerminal`][unlent] when the presenter refused the terminal.
    ///
    /// [undeliverable]: RunInteractiveProcessError::UndeliverableTerminalRequest
    /// [unlent]: RunInteractiveProcessError::UnlentTerminal
    async fn borrow_terminal(
        &self,
        invocation: &Invocation,
    ) -> Result<TerminalLoan, RunInteractiveProcessError> {
        let (request, pending) = TerminalRequest::new(invocation.clone());

        self.output
            .request_terminal(request)
            .await
            .map_err(
                |source| RunInteractiveProcessError::UndeliverableTerminalRequest {
                    invocation: Box::new(invocation.clone()),
                    source,
                },
            )?;

        pending
            .wait()
            .await
            .map_err(|refusal| RunInteractiveProcessError::UnlentTerminal {
                invocation: Box::new(invocation.clone()),
                refusal,
            })
    }

    /// Waits for a program that holds the terminal to end
    ///
    /// A wait that fails drops the run, which kills the program and every program that it
    /// started, so no program outlives the run.
    async fn wait_for_program(&self, mut run: InteractiveRun) -> Ended {
        match run.wait().await {
            Ok(status) => Ended::Exited(status),
            Err(source) => Ended::Incomplete(source),
        }
    }

    /// Reports the start and the end of a run that held the terminal
    ///
    /// Both events go out after the program gave the terminal back. The presenter reads no event
    /// while a program holds the terminal, so a send into a full channel would wait forever.
    ///
    /// # Errors
    ///
    /// Returns [`SendError`] if the consumer of the events has stopped listening.
    async fn report_interactive(
        &self,
        id: RunId,
        invocation: &Invocation,
        process_id: Option<ProcessId>,
        outcome: Outcome,
        duration: Duration,
    ) -> Result<(), SendError> {
        let started = ProcessEvent::Started {
            id,
            invocation: invocation.clone(),
            process_id,
        };
        self.output.process_event(started).await?;

        let finished = ProcessEvent::Finished {
            id,
            invocation: invocation.clone(),
            outcome,
            duration,
        };
        self.output.process_event(finished).await
    }
}

#[cfg(test)]
mod tests {
    #![allow(
        clippy::missing_panics_doc,
        reason = "an assertion in a test panics by design. A `# Panics` section on every test would repeat that and give the reader no information"
    )]

    use std::fs::File;
    #[cfg(unix)]
    use std::fs::read_to_string;
    #[cfg(unix)]
    use std::process::Stdio;

    use tempfile::TempDir;
    use tokio::task::JoinHandle;
    use tokio::time::timeout;

    use super::*;
    use crate::event::terminal::Refusal;
    use crate::event::{Event, EventReceiver, event_channel};
    use crate::output::Output;
    use crate::prompt::ScriptedUser;

    /// Returns the name of the variant of the error, which a test compares in one assertion
    fn variant(error: &RunInteractiveProcessError) -> &'static str {
        match error {
            RunInteractiveProcessError::CancelledRun { .. } => "CancelledRun",
            RunInteractiveProcessError::UndeliverableTerminalRequest { .. } => "Undeliverable",
            RunInteractiveProcessError::UnlentTerminal { .. } => "UnlentTerminal",
            RunInteractiveProcessError::UnreportableRun { .. } => "UnreportableRun",
            RunInteractiveProcessError::UnrunnableCommand { .. } => "UnrunnableCommand",
        }
    }

    /// Plays a presenter that lends the terminal, and returns the events of every run
    ///
    /// The standard output of each program goes to `display`. The presenter waits for the
    /// terminal to come back before it reads the next event, as the terminal presenter does.
    fn lender(mut receiver: EventReceiver, display: File) -> JoinHandle<Vec<ProcessEvent>> {
        tokio::spawn(async move {
            let mut events = Vec::new();

            while let Some(event) = receiver.recv().await {
                match event {
                    Event::Terminal(request) => {
                        let stdout = display.try_clone().expect("should clone the display");
                        request.lend(stdout).await;
                    }
                    Event::Process(event) => events.push(*event),
                    Event::Message(_)
                    | Event::Detail(_)
                    | Event::Artifact(_)
                    | Event::Prompt(_) => {}
                }
            }

            events
        })
    }

    /// Returns a process that reports to a lender, and the lender
    ///
    /// The lender sends the standard output of every program to the file `display` in the
    /// directory.
    fn lent(directory: &TempDir) -> (Process, JoinHandle<Vec<ProcessEvent>>) {
        let display =
            File::create(directory.path().join("display")).expect("should create the display");
        let (sender, receiver) = event_channel();
        let process = Process::builder().output(Output::new(sender)).build();

        (process, lender(receiver, display))
    }

    /// Returns an invocation that gives the commands to the shell of the platform
    fn shell(commands: &[&str]) -> Invocation {
        if cfg!(windows) {
            Invocation::new("cmd").arg("/C").arg(commands.join(" & "))
        } else {
            Invocation::new("sh").arg("-c").arg(commands.join("; "))
        }
    }

    #[tokio::test]
    async fn run_interactive_reports_the_run_after_the_program_ended() {
        let directory = TempDir::new().expect("should create a directory");
        let (process, lender) = lent(&directory);

        let status = process
            .run_interactive(shell(&["exit 0"]))
            .await
            .expect("should run");
        drop(process);

        assert_eq!(
            timeout(Duration::from_secs(20), lender)
                .await
                .expect("the presenter should get the terminal back")
                .expect("should join")
                .iter()
                .map(|event| match event {
                    ProcessEvent::Started { process_id, .. } =>
                        format!("started {}", process_id.is_some()),
                    ProcessEvent::Line { .. } => "line".to_owned(),
                    ProcessEvent::Finished { outcome, .. } => format!("finished {outcome:?}"),
                })
                .collect::<Vec<_>>(),
            vec![
                "started true".to_owned(),
                format!("finished {:?}", Outcome::Exited(status)),
            ]
        );
    }

    #[cfg(unix)]
    #[tokio::test]
    async fn run_interactive_sends_the_standard_output_to_the_lent_stream() {
        let directory = TempDir::new().expect("should create a directory");
        let (process, _lender) = lent(&directory);

        process
            .run_interactive(shell(&["echo hello"]))
            .await
            .expect("should run");

        assert_eq!(
            read_to_string(directory.path().join("display")).expect("should read the display"),
            "hello\n"
        );
    }

    #[tokio::test]
    async fn run_interactive_with_a_closed_channel_returns_an_error() {
        let (sender, receiver) = event_channel();
        let process = Process::builder().output(Output::new(sender)).build();
        drop(receiver);

        let error = process
            .run_interactive(shell(&["exit 0"]))
            .await
            .expect_err("should fail");

        assert_eq!(variant(&error), "Undeliverable");
    }

    #[tokio::test]
    async fn run_interactive_with_a_missing_program_returns_an_error() {
        let directory = TempDir::new().expect("should create a directory");
        let (process, _lender) = lent(&directory);

        let error = process
            .run_interactive(Invocation::new("clawless-no-such-program"))
            .await
            .expect_err("should fail");

        assert_eq!(variant(&error), "UnrunnableCommand");
    }

    #[cfg(unix)]
    #[tokio::test]
    async fn run_interactive_with_a_presenter_that_stops_listening_returns_the_status() {
        let (sender, mut receiver) = event_channel();
        let process = Process::builder().output(Output::new(sender)).build();
        let presenter = tokio::spawn(async move {
            if let Some(Event::Terminal(request)) = receiver.recv().await {
                let returned = request.lend(Stdio::null());
                drop(receiver);
                returned.await;
            }
        });

        let error = process
            .run_interactive(shell(&["exit 3"]))
            .await
            .expect_err("should fail");
        drop(presenter.await);
        let RunInteractiveProcessError::UnreportableRun { status, .. } = error else {
            panic!("the run should be unreportable, not {error}");
        };

        assert_eq!(status.code(), Some(3));
    }

    #[tokio::test]
    async fn run_interactive_with_an_unsuccessful_program_returns_the_status() {
        let directory = TempDir::new().expect("should create a directory");
        let (process, _lender) = lent(&directory);

        let status = process
            .run_interactive(shell(&["exit 3"]))
            .await
            .expect("should run");

        assert_eq!(status.code(), Some(3));
    }

    #[tokio::test]
    async fn run_interactive_without_a_lender_does_not_start_the_program() {
        let directory = TempDir::new().expect("should create a directory");
        let marker = directory.path().join("started");
        let (sender, receiver) = event_channel();
        let process = Process::builder().output(Output::new(sender)).build();
        let refuser = tokio::spawn(ScriptedUser::default().attend(receiver));

        drop(
            process
                .run_interactive(shell(&[&format!("echo > \"{}\"", marker.display())]))
                .await,
        );
        drop(process);
        drop(refuser.await);

        assert!(!marker.exists());
    }

    #[tokio::test]
    async fn run_interactive_without_a_lender_returns_the_refusal() {
        let (sender, receiver) = event_channel();
        let process = Process::builder().output(Output::new(sender)).build();
        let _refuser = tokio::spawn(ScriptedUser::default().attend(receiver));

        let error = process
            .run_interactive(shell(&["exit 0"]))
            .await
            .expect_err("should fail");

        assert!(
            error
                .to_string()
                .ends_with(&Refusal::DroppedRequest.to_string())
        );
    }

    #[test]
    fn trait_send() {
        fn assert_send<T: Send>() {}
        assert_send::<Ended>();
    }

    #[test]
    fn trait_sync() {
        fn assert_sync<T: Sync>() {}
        assert_sync::<Ended>();
    }

    #[test]
    fn trait_unpin() {
        fn assert_unpin<T: Unpin>() {}
        assert_unpin::<Ended>();
    }
}
