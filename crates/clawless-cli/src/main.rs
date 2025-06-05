mod commands;

#[tokio::main]
async fn main() {
    let app = commands::clawless_init();
    commands::clawless_exec(app.get_matches()).await;
}
