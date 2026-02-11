//! Cooperative shutdown
//!
//! This module defines [`Cancellation`], a token-based shutdown primitive. Commands and tasks
//! observe a cancellation token to know when to stop work. Tokens form a tree: cancelling a parent
//! cancels all children, but cancelling a child leaves the parent and siblings unaffected.
//!
//! `Cancellation` is a newtype over [`CancellationToken`] that exposes a curated API matching the
//! Clawless domain vocabulary, decoupling downstream crates from `tokio-util`.
//!
//! [`CancellationToken`]: tokio_util::sync::CancellationToken

use tokio_util::sync::CancellationToken;
pub use tokio_util::sync::WaitForCancellationFuture;

/// Cooperative shutdown token
///
/// `Cancellation` is a cooperative shutdown primitive. Commands and Tasks observe a cancellation
/// token to know when to stop work. Tokens form a tree: cancelling a parent cancels all children,
/// but cancelling a child leaves the parent and siblings unaffected.
///
/// Internally, `Cancellation` wraps a [`CancellationToken`] from `tokio-util`. Cloning a
/// `Cancellation` produces a handle to the same underlying token, not an independent copy.
///
/// # Examples
///
/// ```
/// use clawless::prelude::*;
///
/// let root = Cancellation::new();
/// let child = root.child();
///
/// assert!(!root.is_cancelled());
/// assert!(!child.is_cancelled());
///
/// root.cancel();
///
/// assert!(root.is_cancelled());
/// assert!(child.is_cancelled());
/// ```
///
/// [`CancellationToken`]: tokio_util::sync::CancellationToken
#[derive(Clone, Debug, Default)]
pub struct Cancellation {
    token: CancellationToken,
}

impl Cancellation {
    /// Creates a new, uncancelled token
    ///
    /// # Examples
    ///
    /// ```
    /// use clawless::prelude::*;
    ///
    /// let cancellation = Cancellation::new();
    /// assert!(!cancellation.is_cancelled());
    /// ```
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates a child token linked to this parent
    ///
    /// Cancelling the parent also cancels the child. Cancelling the child does not affect the
    /// parent or siblings.
    ///
    /// # Examples
    ///
    /// ```
    /// use clawless::prelude::*;
    ///
    /// let parent = Cancellation::new();
    /// let child = parent.child();
    ///
    /// child.cancel();
    ///
    /// assert!(child.is_cancelled());
    /// assert!(!parent.is_cancelled());
    /// ```
    pub fn child(&self) -> Self {
        Self {
            token: self.token.child_token(),
        }
    }

    /// Cancels the token and all children
    ///
    /// Calling `cancel()` on an already-cancelled token is a no-op (idempotent).
    ///
    /// # Examples
    ///
    /// ```
    /// use clawless::prelude::*;
    ///
    /// let cancellation = Cancellation::new();
    /// cancellation.cancel();
    /// cancellation.cancel(); // idempotent
    ///
    /// assert!(cancellation.is_cancelled());
    /// ```
    pub fn cancel(&self) {
        self.token.cancel();
    }

    /// Check if the token has been cancelled
    ///
    /// Returns `true` if the token has been cancelled. This is a synchronous, non-blocking,
    /// allocation-free check.
    ///
    /// # Examples
    ///
    /// ```
    /// use clawless::prelude::*;
    ///
    /// let cancellation = Cancellation::new();
    /// assert!(!cancellation.is_cancelled());
    ///
    /// cancellation.cancel();
    /// assert!(cancellation.is_cancelled());
    /// ```
    pub fn is_cancelled(&self) -> bool {
        self.token.is_cancelled()
    }

    /// Returns a future that completes when the token is cancelled
    ///
    /// If the token is already cancelled, the returned future completes immediately.
    ///
    /// The returned future is suitable for use in `tokio::select!` branches.
    ///
    /// # Examples
    ///
    /// ```
    /// use clawless::prelude::*;
    ///
    /// # #[tokio::main]
    /// # async fn main() {
    /// let cancellation = Cancellation::new();
    /// cancellation.cancel();
    ///
    /// // Completes immediately because the token is already cancelled
    /// cancellation.cancelled().await;
    /// # }
    /// ```
    pub fn cancelled(&self) -> WaitForCancellationFuture<'_> {
        self.token.cancelled()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cancel_marks_token_as_cancelled() {
        let cancellation = Cancellation::new();

        cancellation.cancel();

        assert!(cancellation.is_cancelled());

        cancellation.cancel();

        assert!(cancellation.is_cancelled());
    }

    #[tokio::test]
    async fn cancelled_on_already_cancelled_completes_immediately() {
        let cancellation = Cancellation::new();
        cancellation.cancel();

        cancellation.cancelled().await;
    }

    #[test]
    fn child_cancel_does_not_affect_parent() {
        let parent = Cancellation::new();
        let child = parent.child();

        child.cancel();

        assert!(child.is_cancelled());
        assert!(!parent.is_cancelled());
    }

    #[test]
    fn child_cancelled_by_parent() {
        let parent = Cancellation::new();
        let child = parent.child();

        parent.cancel();

        assert!(parent.is_cancelled());
        assert!(child.is_cancelled());
    }

    #[test]
    fn child_outlives_dropped_parent() {
        let child = {
            let parent = Cancellation::new();
            let child = parent.child();
            parent.cancel();
            child
        };

        assert!(child.is_cancelled());
    }

    #[test]
    fn deeply_nested_children_propagate() {
        let root = Cancellation::new();
        let level1 = root.child();
        let level2 = level1.child();
        let level3 = level2.child();

        root.cancel();

        assert!(root.is_cancelled());
        assert!(level1.is_cancelled());
        assert!(level2.is_cancelled());
        assert!(level3.is_cancelled());
    }

    #[test]
    fn default_creates_uncancelled_token() {
        let cancellation = Cancellation::default();

        assert!(!cancellation.is_cancelled());
    }

    #[test]
    fn new_creates_uncancelled_token() {
        let cancellation = Cancellation::new();

        assert!(!cancellation.is_cancelled());
    }

    #[test]
    fn trait_send() {
        fn assert_send<T: Send>() {}
        assert_send::<Cancellation>();
    }

    #[test]
    fn trait_sync() {
        fn assert_sync<T: Sync>() {}
        assert_sync::<Cancellation>();
    }

    #[test]
    fn trait_unpin() {
        fn assert_unpin<T: Unpin>() {}
        assert_unpin::<Cancellation>();
    }
}
