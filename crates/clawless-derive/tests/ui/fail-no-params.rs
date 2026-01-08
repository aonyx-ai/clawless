use clawless::prelude::*;

#[command]
pub async fn greet() -> CommandResult {
    println!("Hello!");
    Ok(())
}

fn main() {}
