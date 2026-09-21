//! Reads the lines that a user types, one at a time and only on request
//!
//! The presenter runs on an asynchronous runtime, and reading a terminal blocks. [`LineReader`]
//! therefore reads on a thread of its own and sends each line to the presenter through a channel.

use std::io;
use std::sync::mpsc as blocking;

use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender, unbounded_channel};

/// The name of the thread that reads the input
const THREAD_NAME: &str = "clawless-input";

/// Reads one line from the input into the buffer and returns the count of the bytes that it read
///
/// The contract is the one of [`io::BufRead::read_line`]. A count of zero means that the input
/// has ended.
type ReadLine = Box<dyn FnMut(&mut String) -> io::Result<usize> + Send>;

/// One read of the input: a line, the end of the input, or a failure
type Read = io::Result<Option<String>>;

/// Reads the lines that a user types, one at a time and only on request
///
/// Each call to [`next_line`] requests exactly one read. Between requests, the reader does not
/// read the input, so a program that the command starts can read the terminal.
///
/// The reader uses a plain thread and not a blocking task of the runtime. A runtime waits for
/// its blocking tasks at shutdown, and a read of a terminal returns only when the user presses
/// Enter. A cancelled application would therefore not end until then. Nothing waits for a plain
/// thread.
///
/// If a caller drops [`next_line`] before it returns, the read stays open, because a blocked
/// read cannot be cancelled. The next call starts no second read and returns the line of the
/// open read.
///
/// The thread starts with the first request.
///
/// [`next_line`]: LineReader::next_line
pub(super) struct LineReader {
    /// The state of the reader
    state: State,
}

/// The state of a [`LineReader`]
enum State {
    /// No line was requested yet, so no thread runs
    Idle {
        /// The function that reads one line from the input
        read_line: ReadLine,
    },

    /// The thread runs and answers requests
    Started {
        /// The channel that asks the thread for one read
        requests: blocking::Sender<()>,

        /// The channel that carries each read back
        reads: UnboundedReceiver<Read>,

        /// Whether a read is requested and not yet returned to a caller
        open: OpenRead,
    },

    /// The thread could not start, or it has ended
    Stopped,
}

/// Whether a read is requested and not yet returned to a caller
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
enum OpenRead {
    /// No read is open
    No,

    /// A read is requested, and no caller has received its result
    Yes,
}

impl LineReader {
    /// Creates a reader over a function that reads one line from the input
    pub(super) fn new(
        read_line: impl FnMut(&mut String) -> io::Result<usize> + Send + 'static,
    ) -> Self {
        Self {
            state: State::Idle {
                read_line: Box::new(read_line),
            },
        }
    }

    /// Creates a reader over the standard input of the application
    pub(super) fn stdin() -> Self {
        Self::new(|buffer| io::stdin().read_line(buffer))
    }

    /// Returns the next line that the user types, without the characters that end it
    ///
    /// Returns `None` when the input has ended, which Ctrl+D causes on a terminal. A last line
    /// without a line break is a line.
    ///
    /// The returned future is safe to drop. The read that it requested stays open, and the next
    /// call returns its line.
    ///
    /// # Errors
    ///
    /// Returns the error of the input if the read fails. Returns an error as well if the thread
    /// that reads the input could not start, and if it has ended.
    pub(super) async fn next_line(&mut self) -> Read {
        self.start()?;

        let State::Started {
            requests,
            reads,
            open,
        } = &mut self.state
        else {
            return Err(stopped());
        };

        match open {
            OpenRead::Yes => {}
            OpenRead::No => {
                requests.send(()).map_err(|_closed| stopped())?;
                *open = OpenRead::Yes;
            }
        }

        let Some(read) = reads.recv().await else {
            self.state = State::Stopped;
            return Err(stopped());
        };
        *open = OpenRead::No;

        read
    }

    /// Starts the thread that reads the input, unless it runs already
    ///
    /// # Errors
    ///
    /// Returns the error of the operating system if the thread cannot start.
    fn start(&mut self) -> io::Result<()> {
        let read_line = match std::mem::replace(&mut self.state, State::Stopped) {
            State::Idle { read_line } => read_line,
            state @ (State::Started { .. } | State::Stopped) => {
                self.state = state;
                return Ok(());
            }
        };

        let (requests, requested) = blocking::channel();
        let (sender, reads) = unbounded_channel();

        std::thread::Builder::new()
            .name(THREAD_NAME.to_owned())
            .spawn(move || serve(read_line, &requested, &sender))?;

        self.state = State::Started {
            requests,
            reads,
            open: OpenRead::No,
        };

        Ok(())
    }
}

/// Reads one line for each request, until the reader is dropped
///
/// A closed request channel and a closed read channel both mean that the reader was dropped.
fn serve(
    mut read_line: ReadLine,
    requested: &blocking::Receiver<()>,
    reads: &UnboundedSender<Read>,
) {
    while requested.recv().is_ok() {
        let mut line = String::new();

        let read = match read_line(&mut line) {
            Ok(0) => Ok(None),
            Ok(_count) => {
                let end = line.trim_end_matches(['\r', '\n']).len();
                line.truncate(end);
                Ok(Some(line))
            }
            Err(error) => Err(error),
        };

        if reads.send(read).is_err() {
            return;
        }
    }
}

/// Returns the error for a reader whose thread does not run
fn stopped() -> io::Error {
    io::Error::new(
        io::ErrorKind::BrokenPipe,
        "the reader of the input has stopped",
    )
}

#[cfg(test)]
mod tests {
    // An assertion in a test panics by design. A `# Panics` section on every test
    // would repeat that and give the reader no information.
    #![allow(clippy::missing_panics_doc)]

    use std::collections::VecDeque;
    use std::future::Future;
    use std::io;
    use std::pin::pin;
    use std::sync::Arc;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::mpsc;
    use std::task::Poll;
    use std::time::Duration;

    use super::*;

    /// Returns a reader over the given lines, and the count of the reads that it has started
    fn scripted(lines: &[&str]) -> (LineReader, Arc<AtomicUsize>) {
        let mut lines = lines
            .iter()
            .map(|line| (*line).to_owned())
            .collect::<VecDeque<_>>();
        let reads = Arc::new(AtomicUsize::new(0));
        let counter = Arc::clone(&reads);

        let reader = LineReader::new(move |buffer| {
            counter.fetch_add(1, Ordering::SeqCst);
            let line = lines.pop_front().unwrap_or_default();
            buffer.push_str(&line);
            Ok(line.len())
        });

        (reader, reads)
    }

    /// Gives the thread of a reader the time to start a read that it must not start
    fn settle() {
        std::thread::sleep(Duration::from_millis(50));
    }

    #[tokio::test]
    async fn new_starts_no_read() {
        let (_reader, reads) = scripted(&["y\n"]);

        settle();

        assert_eq!(reads.load(Ordering::SeqCst), 0);
    }

    #[tokio::test]
    async fn next_line_after_an_abandoned_read_returns_the_line_of_that_read() {
        let (feed, source) = mpsc::channel::<String>();
        let mut reader = LineReader::new(move |buffer| {
            let line = source.recv().unwrap_or_default();
            buffer.push_str(&line);
            Ok(line.len())
        });
        {
            let mut abandoned = pin!(reader.next_line());
            std::future::poll_fn(|context| {
                Poll::Ready(abandoned.as_mut().poll(context).is_pending())
            })
            .await;
        }
        feed.send("late\n".to_owned()).expect("should feed");

        let line = reader.next_line().await.expect("should read");

        assert_eq!(line, Some("late".to_owned()));
    }

    #[tokio::test]
    async fn next_line_at_the_end_of_the_input_returns_none() {
        let (mut reader, _reads) = scripted(&[]);

        let line = reader.next_line().await.expect("should read");

        assert_eq!(line, None);
    }

    #[tokio::test]
    async fn next_line_reads_no_further_than_the_line_it_returns() {
        let (mut reader, reads) = scripted(&["y\n", "n\n"]);

        drop(reader.next_line().await);
        settle();

        assert_eq!(reads.load(Ordering::SeqCst), 1);
    }

    #[tokio::test]
    async fn next_line_returns_the_lines_in_order() {
        let (mut reader, _reads) = scripted(&["y\n", "n\n"]);
        drop(reader.next_line().await);

        let line = reader.next_line().await.expect("should read");

        assert_eq!(line, Some("n".to_owned()));
    }

    #[tokio::test]
    async fn next_line_with_a_failing_input_returns_the_error() {
        let mut reader = LineReader::new(|_buffer| Err(io::Error::other("unplugged")));

        let error = reader.next_line().await.expect_err("should fail");

        assert_eq!(error.to_string(), "unplugged");
    }

    #[tokio::test]
    async fn next_line_with_a_last_line_without_an_end_returns_the_line() {
        let (mut reader, _reads) = scripted(&["y"]);

        let line = reader.next_line().await.expect("should read");

        assert_eq!(line, Some("y".to_owned()));
    }

    #[tokio::test]
    async fn next_line_with_a_windows_line_end_returns_the_line_without_it() {
        let (mut reader, _reads) = scripted(&["y\r\n"]);

        let line = reader.next_line().await.expect("should read");

        assert_eq!(line, Some("y".to_owned()));
    }
}
