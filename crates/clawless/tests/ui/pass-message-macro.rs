use clawless::context::Context;
use clawless::message;

async fn message_with_literal(context: Context) {
    message!("hello");
}

async fn message_with_format_args(context: Context) {
    message!("hello, {}", "world");
}

fn main() {}
