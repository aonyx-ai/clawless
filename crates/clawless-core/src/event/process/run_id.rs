//! The identity of one run of an external program

use std::fmt::{self, Display};
use std::sync::atomic::{AtomicU64, Ordering};

/// The source of the identities that [`RunId::next`] hands out
///
/// The counter is a single static, so every identity is unique within the
/// process regardless of which [`Process`] handed it out. A counter per handle
/// would repeat itself as soon as a command cloned its context.
///
/// [`Process`]: crate::process::Process
static NEXT_RUN_ID: AtomicU64 = AtomicU64::new(0);

/// The identity of one run of an external program
///
/// A command can run several programs at the same time, and the lines of those
/// programs then reach the presenter interleaved. Every [`ProcessEvent`]
/// carries this value so that a consumer can put a line back with the run that
/// produced it, group the output of one program, and know when that program
/// ended.
///
/// The identity is Clawless's own, and it counts runs. It is not the
/// [`ProcessId`] that the operating system assigns, which names the program
/// while it runs and which the operating system reuses afterwards. A run has an
/// identity before the program starts and keeps it after the program ended,
/// which is what an event stream needs.
///
/// [`ProcessId`]: crate::process::ProcessId
///
/// # Examples
///
/// ```
/// use clawless_core::event::process::RunId;
///
/// let first = RunId::next();
/// let second = RunId::next();
///
/// assert_ne!(first, second);
/// ```
///
/// [`ProcessEvent`]: super::ProcessEvent
// r[impl process.event.correlation]
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub struct RunId(u64);

impl RunId {
    /// Returns an identity that no earlier call returned
    ///
    /// [`Process::run`] calls this once per run, so a command that runs a
    /// program does not need it. A test that stands in for a run builds its
    /// events with identities from here.
    ///
    /// The counter wraps after `2^64` runs. An application that started a
    /// program every nanosecond would reach that point after more than five
    /// hundred years, so the wrap is not a case that callers handle.
    ///
    /// # Examples
    ///
    /// ```
    /// use clawless_core::event::process::RunId;
    ///
    /// let id = RunId::next();
    ///
    /// assert_eq!(id, id);
    /// ```
    ///
    /// [`Process::run`]: crate::process::Process::run
    // r[impl process.event.correlation.unique]
    pub fn next() -> Self {
        Self(NEXT_RUN_ID.fetch_add(1, Ordering::Relaxed))
    }

    /// Returns the identity as a number
    ///
    /// A consumer that keys a map or a widget by the run reads the number here.
    ///
    /// # Examples
    ///
    /// ```
    /// use clawless_core::event::process::RunId;
    ///
    /// let id = RunId::next();
    ///
    /// assert_eq!(id.get(), id.get());
    /// ```
    pub fn get(&self) -> u64 {
        self.0
    }
}

/// Shows the identity for a reader
///
/// The text is the number alone, so that a log line can name the run without
/// the punctuation that [`Debug`] adds.
impl Display for RunId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        Display::fmt(&self.0, formatter)
    }
}

#[cfg(test)]
mod tests {
    // An assertion in a test panics by design. A `# Panics` section on every test
    // would repeat that and give the reader no information.
    #![allow(clippy::missing_panics_doc)]

    use super::*;

    #[test]
    fn display_returns_the_number() {
        let id = RunId::next();

        let text = id.to_string();

        assert_eq!(text, id.get().to_string());
    }

    // r[verify process.event.correlation]
    // r[verify process.event.correlation.unique]
    #[test]
    fn next_returns_a_new_identity_every_time() {
        let first = RunId::next();
        let second = RunId::next();

        assert_ne!(first, second);
    }

    #[test]
    fn trait_send() {
        fn assert_send<T: Send>() {}
        assert_send::<RunId>();
    }

    #[test]
    fn trait_sync() {
        fn assert_sync<T: Sync>() {}
        assert_sync::<RunId>();
    }

    #[test]
    fn trait_unpin() {
        fn assert_unpin<T: Unpin>() {}
        assert_unpin::<RunId>();
    }
}
