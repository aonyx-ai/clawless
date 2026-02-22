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
//! [`message!`] and [`detail!`] accept `format_args!`-style arguments and expand to a
//! `format_args!` call, avoiding an intermediate [`String`] allocation:
//!
//! ```rust,ignore
//! // This:
//! message!("found {} items", count);
//!
//! // Expands to:
//! context.output().message(format_args!("found {} items", count));
//! ```
//!
//! [`artifact!`] takes an expression rather than format arguments, because artifacts must
//! implement both [`Display`] and [`Serialize`]:
//!
//! ```rust,ignore
//! // This:
//! artifact!(count);
//!
//! // Expands to:
//! context.output().artifact(&(count));
//! ```
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
//! [`Display`]: std::fmt::Display
//! [`Output`]: crate::output::Output
//! [`Serialize`]: serde::Serialize
//! [`artifact!`]: crate::artifact
//! [`detail!`]: crate::detail
//! [`message!`]: crate::message
