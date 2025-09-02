use deskagent::run_gui_app;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    run_gui_app().await
}