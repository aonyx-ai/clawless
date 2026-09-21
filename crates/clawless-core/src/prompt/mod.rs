//! Questions that a command asks its user
//!
//! [`Confirm`] is a question that the user answers with yes or no, and [`Confirmation`] is the
//! answer. [`AnswerPromptError`] is the reason why a prompt has no answer.
//!
//! A question reaches the presenter as a [`PromptRequest`], and the presenter answers it.
//!
//! [`PromptRequest`]: crate::event::prompt::PromptRequest

pub use self::answer_prompt_error::AnswerPromptError;
pub use self::confirm::Confirm;
pub use self::confirmation::Confirmation;

/// The error that ends a prompt without an answer from the user
mod answer_prompt_error;
/// A question that a user answers with yes or no
mod confirm;
/// The answer of a user to a confirmation
mod confirmation;
