//! Output writer abstraction
//!
//! This module defines [`Writer`], the output destination for rendered events. The render task
//! uses a `Writer` for each output stream (messages and artifacts), selecting between stdout and
//! stderr based on the [`OutputMode`].
//!
//! In test builds, a buffer variant allows tests to capture rendered output without touching
//! real I/O handles.
//!
//! [`OutputMode`]: crate::output::OutputMode

use std::io::Write;
#[cfg(test)]
use std::sync::{Arc, Mutex};

/// Output destination for rendered events
///
/// `Writer` abstracts the output stream so that production code writes to stdout or stderr,
/// while tests capture output in an in-memory buffer. The render task holds one `Writer` for
/// messages and another for artifacts.
///
/// Write failures terminate the process, matching the behavior of `println!`.
#[derive(Clone, Debug)]
pub(super) enum Writer {
    Stdout,
    Stderr,
    #[cfg(test)]
    Buffer(Arc<Mutex<Vec<u8>>>),
}

impl Writer {
    /// Writes a message followed by a newline to the output stream
    ///
    /// # Panics
    ///
    /// Panics if writing to the underlying stream fails, matching `println!` behavior.
    pub(super) fn write_line(&self, message: &str) {
        match self {
            Writer::Stdout => {
                let mut handle = std::io::stdout().lock();
                writeln!(handle, "{message}").expect("should write to stdout");
            }
            Writer::Stderr => {
                let mut handle = std::io::stderr().lock();
                writeln!(handle, "{message}").expect("should write to stderr");
            }
            #[cfg(test)]
            Writer::Buffer(buffer) => {
                let mut guard = buffer.lock().expect("should lock buffer");
                writeln!(guard, "{message}").expect("should write to buffer");
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn trait_send() {
        fn assert_send<T: Send>() {}
        assert_send::<Writer>();
    }

    #[test]
    fn trait_sync() {
        fn assert_sync<T: Sync>() {}
        assert_sync::<Writer>();
    }

    #[test]
    fn trait_unpin() {
        fn assert_unpin<T: Unpin>() {}
        assert_unpin::<Writer>();
    }

    #[test]
    fn write_line_to_buffer_appends_with_newline() {
        let buffer = Arc::new(Mutex::new(Vec::new()));
        let writer = Writer::Buffer(Arc::clone(&buffer));

        writer.write_line("hello");

        let guard = buffer.lock().expect("should lock");
        assert_eq!(
            String::from_utf8(guard.clone()).expect("should be valid UTF-8"),
            "hello\n"
        );
    }
}
