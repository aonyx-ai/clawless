//! Questions that a terminal asks on one line
//!
//! The terminal presenter writes a question and reads one line as the answer. It does not change
//! the mode of the terminal, so Ctrl+C still reaches the application as a signal.
//!
//! [`LineQuestion`] gives the text of a question and parses the line that the user typed. [`ask`]
//! asks one question until the user answers it.

use std::io::{self, Write};

use clawless_core::event::prompt::Reply;
use clawless_core::prompt::AnswerPromptError;

use super::line_reader::LineReader;

mod confirm;
mod text;

/// A question that a user answers with one line of text
///
/// The terminal presenter shows [`prompt`], reads a line, and passes the line to [`parse`]. If
/// the line is no answer, [`parse`] returns a hint. The presenter shows the hint and asks again.
///
/// [`parse`]: LineQuestion::parse
/// [`prompt`]: LineQuestion::prompt
pub(super) trait LineQuestion {
    /// The answer that the question asks for
    type Answer;

    /// Returns the text that precedes the answer of the user
    ///
    /// The text ends with a space and without a line break.
    fn prompt(&self) -> String;

    /// Parses a line that the user typed
    ///
    /// The line does not contain the characters that ended it.
    ///
    /// # Errors
    ///
    /// Returns a hint for the user if the line is no answer. The hint is a full sentence.
    fn parse(&self, line: &str) -> Result<Self::Answer, String>;
}

/// Asks the user one question and sends the outcome to the command that asked
///
/// The function writes the question to the display and reads the answer from the input. After a
/// line that is no answer, it writes the hint and asks again.
///
/// If the command stops waiting, the function writes a line break and returns, so the next
/// output starts on a new line. If the command stopped waiting before the call, the function
/// writes nothing.
///
/// The end of the input sends [`AnswerPromptError::ClosedInput`] to the command, and never a
/// default. A failure of the display or the input sends [`AnswerPromptError::UnusableTerminal`].
pub(super) async fn ask<Q: LineQuestion>(
    question: &Q,
    mut reply: Reply<Q::Answer>,
    input: &mut LineReader,
    display: &mut impl Write,
) {
    if reply.is_abandoned() {
        return;
    }

    match converse(question, &mut reply, input, display).await {
        Ok(Some(answer)) => reply.answer(answer),
        Ok(None) => {}
        Err(error) => reply.fail(error),
    }
}

/// Shows the question until the user answers it, and returns the answer
///
/// Returns `None` if the command stopped waiting before the user answered.
///
/// # Errors
///
/// Returns [`AnswerPromptError::ClosedInput`] if the input ends, and
/// [`AnswerPromptError::UnusableTerminal`] if the display or the input fails.
async fn converse<Q: LineQuestion>(
    question: &Q,
    reply: &mut Reply<Q::Answer>,
    input: &mut LineReader,
    display: &mut impl Write,
) -> Result<Option<Q::Answer>, AnswerPromptError> {
    loop {
        write!(display, "{}", question.prompt()).map_err(unusable)?;
        display.flush().map_err(unusable)?;

        let line = tokio::select! {
            biased;

            () = reply.abandoned() => {
                writeln!(display).map_err(unusable)?;
                return Ok(None);
            }
            line = input.next_line() => line.map_err(unusable)?,
        };

        let Some(line) = line else {
            writeln!(display).map_err(unusable)?;
            return Err(AnswerPromptError::ClosedInput);
        };

        match question.parse(&line) {
            Ok(answer) => return Ok(Some(answer)),
            Err(hint) => writeln!(display, "{hint}").map_err(unusable)?,
        }
    }
}

/// Returns the error for a terminal that could not show a question or read an answer
fn unusable(source: io::Error) -> AnswerPromptError {
    AnswerPromptError::UnusableTerminal { source }
}

#[cfg(test)]
mod tests {
    // An assertion in a test panics by design. A `# Panics` section on every test
    // would repeat that and give the reader no information.
    #![allow(clippy::missing_panics_doc)]

    use std::collections::VecDeque;
    use std::sync::mpsc;

    use clawless_core::event::prompt::{PendingAnswer, PromptRequest};
    use clawless_core::prompt::{Confirm, Confirmation};

    use super::*;

    /// Returns the parts of a request for a confirmation without a default
    fn request() -> (Confirm, Reply<Confirmation>, PendingAnswer<Confirmation>) {
        let question = Confirm::new("Release?");
        let (request, pending) = PromptRequest::confirm(question.clone());

        match request {
            PromptRequest::Confirm { reply, .. } => (question, reply, pending),
            PromptRequest::Text { .. } => unreachable!("the request is a confirmation"),
        }
    }

    /// Returns a reader over the lines that a user types
    fn typed(lines: &[&str]) -> LineReader {
        let mut lines = lines
            .iter()
            .map(|line| (*line).to_owned())
            .collect::<VecDeque<_>>();

        LineReader::new(move |buffer| {
            let line = lines.pop_front().unwrap_or_default();
            buffer.push_str(&line);
            Ok(line.len())
        })
    }

    #[tokio::test]
    async fn ask_with_a_command_that_stops_waiting_ends_the_line_of_the_question() {
        let (question, reply, pending) = request();
        let mut display = Vec::new();
        let (_feed, source) = mpsc::channel::<String>();
        let mut input = LineReader::new(move |buffer| {
            let line = source.recv().unwrap_or_default();
            buffer.push_str(&line);
            Ok(line.len())
        });
        let command = tokio::spawn(async move {
            tokio::task::yield_now().await;
            drop(pending);
        });

        ask(&question, reply, &mut input, &mut display).await;
        command.await.expect("should join");

        assert_eq!(String::from_utf8_lossy(&display), "Release? [y/n] \n");
    }

    #[tokio::test]
    async fn ask_with_a_line_that_is_no_answer_shows_the_hint_and_asks_again() {
        let (question, reply, _pending) = request();
        let mut display = Vec::new();

        ask(
            &question,
            reply,
            &mut typed(&["maybe\n", "n\n"]),
            &mut display,
        )
        .await;

        assert_eq!(
            String::from_utf8_lossy(&display),
            "Release? [y/n] Please answer \"y\" or \"n\".\nRelease? [y/n] "
        );
    }

    #[tokio::test]
    async fn ask_with_an_abandoned_reply_shows_nothing() {
        let (question, reply, pending) = request();
        let mut display = Vec::new();
        drop(pending);

        ask(&question, reply, &mut typed(&["y\n"]), &mut display).await;

        assert_eq!(String::from_utf8_lossy(&display), "");
    }

    #[tokio::test]
    async fn ask_with_an_answer_sends_it_to_the_command() {
        let (question, reply, pending) = request();
        let mut display = Vec::new();
        ask(&question, reply, &mut typed(&["y\n"]), &mut display).await;

        let answer = pending.wait().await.expect("should answer");

        assert_eq!(answer, Confirmation::Yes);
    }

    #[tokio::test]
    async fn ask_with_the_end_of_the_input_ends_the_line_of_the_question() {
        let (question, reply, _pending) = request();
        let mut display = Vec::new();

        ask(&question, reply, &mut typed(&[]), &mut display).await;

        assert_eq!(String::from_utf8_lossy(&display), "Release? [y/n] \n");
    }

    #[tokio::test]
    async fn ask_with_the_end_of_the_input_reports_the_closed_input() {
        let (question, reply, pending) = request();
        let mut display = Vec::new();
        ask(&question, reply, &mut typed(&[]), &mut display).await;

        let error = pending.wait().await.expect_err("should fail");

        assert_eq!(
            error.to_string(),
            "the input ended before the user answered"
        );
    }
}
