use std::fmt;

use clawless::context::Context;
use clawless::event::{Styling, TextForm};
use clawless::message;

struct Verdict;

impl TextForm for Verdict {
    fn fmt_text(&self, f: &mut fmt::Formatter<'_>, _styling: Styling) -> fmt::Result {
        f.write_str("passed")
    }
}

async fn message_with_literal(context: Context) {
    message!("hello");
}

async fn message_with_format_args(context: Context) {
    message!("hello, {}", "world");
}

async fn message_with_a_value_that_it_borrows(context: Context) {
    let verdict = Verdict;
    message!(verdict);
    drop(verdict);
}

fn main() {}
