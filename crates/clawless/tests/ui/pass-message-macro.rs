use clawless::context::Context;
use clawless::message;

fn message_with_literal(context: Context) {
    message!("hello");
}

fn message_with_format_args(context: Context) {
    message!("hello, {}", "world");
}

fn main() {}
