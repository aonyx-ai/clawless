//! The stream on which a program that takes over the terminal draws

use std::io::{self, IsTerminal, Stderr};
#[cfg(unix)]
use std::os::fd::AsFd;
#[cfg(windows)]
use std::os::windows::io::AsHandle;
use std::process::Stdio;

use bon::bon;
use clawless_core::event::terminal::Refusal;

/// Whether a stream of the application is a terminal
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub(super) enum Attachment {
    /// The stream is a terminal
    Terminal,

    /// The stream is a file, a pipe, or another stream that is not a terminal
    Redirected,
}

impl Attachment {
    /// Returns whether the stream is a terminal
    pub(super) fn of(stream: &impl IsTerminal) -> Self {
        if stream.is_terminal() {
            Self::Terminal
        } else {
            Self::Redirected
        }
    }
}

/// The stream of the application on which a program that takes over the terminal draws
///
/// The program writes its standard output there. The standard error comes first, because the
/// standard output of the application carries the result of the command. A user who sends the
/// result to a file then still sees the editor.
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub(super) enum Screen {
    /// The standard error of the application
    StandardError,

    /// The standard output of the application
    StandardOutput,
}

#[bon]
impl Screen {
    /// Chooses the screen for a program from the streams of the application
    ///
    /// The program reads the keys of the user, so the standard input must be a terminal. The
    /// screen is the standard error if it is a terminal, and else the standard output if that is
    /// one. A user who runs `mycli edit 2>/dev/null` therefore still sees the editor.
    ///
    /// # Errors
    ///
    /// Returns [`Refusal::MissingTerminal`] when the standard input is not a terminal, or when
    /// neither output stream is one. A program would then wait for a user who is not there.
    #[builder(finish_fn = choose)]
    pub(super) fn for_streams(
        input: Attachment,
        error: Attachment,
        output: Attachment,
    ) -> Result<Self, Refusal> {
        match input {
            Attachment::Terminal => {}
            Attachment::Redirected => return Err(Refusal::MissingTerminal),
        }

        match (error, output) {
            (Attachment::Terminal, Attachment::Terminal) => Ok(Self::StandardError),
            (Attachment::Terminal, Attachment::Redirected) => Ok(Self::StandardError),
            (Attachment::Redirected, Attachment::Terminal) => Ok(Self::StandardOutput),
            (Attachment::Redirected, Attachment::Redirected) => Err(Refusal::MissingTerminal),
        }
    }

    /// Returns a stream for the standard output of a program, which writes to this screen
    ///
    /// `stderr` is the standard error of the application. The stream for that screen is a copy
    /// of its handle, so each program receives a stream of its own. A program that draws on the
    /// standard output inherits it.
    ///
    /// # Errors
    ///
    /// Returns the error of the operating system when it cannot copy the handle, which happens
    /// when the application has no handles left.
    pub(super) fn stream(self, stderr: &Stderr) -> io::Result<Stdio> {
        match self {
            Self::StandardError => duplicate(stderr),
            Self::StandardOutput => Ok(Stdio::inherit()),
        }
    }
}

/// Returns a copy of the handle of a stream
///
/// # Errors
///
/// Returns the error of the operating system when it cannot copy the handle.
#[cfg(unix)]
fn duplicate(stream: &impl AsFd) -> io::Result<Stdio> {
    stream.as_fd().try_clone_to_owned().map(Stdio::from)
}

/// Returns a copy of the handle of a stream
///
/// # Errors
///
/// Returns the error of the operating system when it cannot copy the handle.
#[cfg(windows)]
fn duplicate(stream: &impl AsHandle) -> io::Result<Stdio> {
    stream.as_handle().try_clone_to_owned().map(Stdio::from)
}

#[cfg(test)]
mod tests {
    #![allow(
        clippy::missing_panics_doc,
        reason = "an assertion in a test panics by design. A `# Panics` section on every test would repeat that and give the reader no information"
    )]

    use super::*;

    #[test]
    fn choose_with_a_redirected_error_returns_the_standard_output() {
        let screen = Screen::for_streams()
            .input(Attachment::Terminal)
            .error(Attachment::Redirected)
            .output(Attachment::Terminal)
            .choose();

        assert_eq!(screen, Ok(Screen::StandardOutput));
    }

    #[test]
    fn choose_with_a_redirected_input_refuses() {
        let screen = Screen::for_streams()
            .input(Attachment::Redirected)
            .error(Attachment::Terminal)
            .output(Attachment::Terminal)
            .choose();

        assert_eq!(screen, Err(Refusal::MissingTerminal));
    }

    #[test]
    fn choose_with_a_redirected_output_returns_the_standard_error() {
        let screen = Screen::for_streams()
            .input(Attachment::Terminal)
            .error(Attachment::Terminal)
            .output(Attachment::Redirected)
            .choose();

        assert_eq!(screen, Ok(Screen::StandardError));
    }

    #[test]
    fn choose_with_a_terminal_on_both_outputs_returns_the_standard_error() {
        let screen = Screen::for_streams()
            .input(Attachment::Terminal)
            .error(Attachment::Terminal)
            .output(Attachment::Terminal)
            .choose();

        assert_eq!(screen, Ok(Screen::StandardError));
    }

    #[test]
    fn choose_with_both_outputs_redirected_refuses() {
        let screen = Screen::for_streams()
            .input(Attachment::Terminal)
            .error(Attachment::Redirected)
            .output(Attachment::Redirected)
            .choose();

        assert_eq!(screen, Err(Refusal::MissingTerminal));
    }

    #[test]
    fn trait_send_attachment() {
        fn assert_send<T: Send>() {}
        assert_send::<Attachment>();
    }

    #[test]
    fn trait_send_screen() {
        fn assert_send<T: Send>() {}
        assert_send::<Screen>();
    }

    #[test]
    fn trait_sync_attachment() {
        fn assert_sync<T: Sync>() {}
        assert_sync::<Attachment>();
    }

    #[test]
    fn trait_sync_screen() {
        fn assert_sync<T: Sync>() {}
        assert_sync::<Screen>();
    }

    #[test]
    fn trait_unpin_attachment() {
        fn assert_unpin<T: Unpin>() {}
        assert_unpin::<Attachment>();
    }

    #[test]
    fn trait_unpin_screen() {
        fn assert_unpin<T: Unpin>() {}
        assert_unpin::<Screen>();
    }
}
