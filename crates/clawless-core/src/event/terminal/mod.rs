//! The request that borrows the terminal from the presenter
//!
//! A program such as an editor, a pager, or `ssh` draws on the terminal and reads the keys of the
//! user. The presenter writes to the same terminal, so the two must take turns. A command
//! therefore asks the presenter for the terminal before such a program starts. It gives the
//! terminal back when the program ends.
//!
//! A presenter that cannot lend the terminal answers with a [`Refusal`]. The command then does
//! not start the program.

pub use self::refusal::Refusal;

mod refusal;
