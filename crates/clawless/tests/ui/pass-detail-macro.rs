use clawless::context::Context;
use clawless::detail;

fn detail_with_literal(context: Context) {
    detail!("extra info");
}

fn detail_with_format_args(context: Context) {
    detail!("extra: {}", 42);
}

fn main() {}
