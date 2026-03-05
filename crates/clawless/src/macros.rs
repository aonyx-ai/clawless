//! Output macros for commands
//!
//! The [`message!`], [`detail!`], and [`artifact!`] macros provide ergonomic shorthand for the
//! three methods on [`Output`]. They resolve a `context` variable at the call site, so they work
//! inside any `#[command]` function without extra imports — `use clawless::prelude::*` brings
//! them into scope.
//!
//! # Convention
//!
//! Every `#[command]` function receives a [`Context`] parameter conventionally named `context`.
//! The macros rely on this name: they expand to `context.output().method(...)`. If the parameter
//! has a different name, use the method form instead.
//!
//! # Expansion
//!
//! [`message!`] and [`detail!`] accept `format!`-style arguments and expand to an async send
//! through the event channel:
//!
//! ```rust,ignore
//! // This:
//! message!("found {} items", count);
//!
//! // Expands to:
//! context.output().message(format!("found {} items", count))
//!     .await
//!     .expect("event channel closed");
//! ```
//!
//! [`artifact!`] takes an expression rather than format arguments, because artifacts must
//! implement [`Display`], [`Serialize`], and [`Debug`]:
//!
//! ```rust,ignore
//! // This:
//! artifact!(count);
//!
//! // Expands to:
//! context.output().artifact(count)
//!     .await
//!     .expect("event channel closed");
//! ```
//!
//! Because the expansion includes `.await`, all three macros must be called from an async
//! function.
//!
//! # Examples
//!
//! ```rust,ignore
//! use clawless::prelude::*;
//!
//! #[command]
//! pub async fn count(args: CountArgs, context: Context) -> CommandResult {
//!     detail!("input: {}", args.sentence);
//!     message!("counting words");
//!     artifact!(WordCount { words: 42 });
//!     Ok(())
//! }
//! ```
//!
//! [`Context`]: crate::context::Context
//! [`Debug`]: std::fmt::Debug
//! [`Display`]: std::fmt::Display
//! [`Output`]: clawless_core::output::Output
//! [`Serialize`]: serde::Serialize
//! [`artifact!`]: crate::artifact
//! [`detail!`]: crate::detail
//! [`message!`]: crate::message
