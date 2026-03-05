use clawless::artifact;
use clawless::context::Context;

async fn artifact_with_expression(context: Context) {
    artifact!("plain string");
}

fn main() {}
