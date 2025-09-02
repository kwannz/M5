use anyhow::Result;
use log::info;
use std::env;

mod orchestrator;
mod desktop;
mod llm;
mod tui;
mod workflows;

use orchestrator::{Orchestrator, OrchestratorConfig};
use tui::App;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    env_logger::init();
    
    // Load configuration
    let config_path = env::var("CONFIG_PATH").unwrap_or_else(|_| "config.yaml".to_string());
    
    info!("Starting DeskAgent v1.0");
    info!("Loading configuration from: {}", config_path);
    
    // Initialize orchestrator
    let config = OrchestratorConfig::default(); // TODO: Load from config file
    let orchestrator = Orchestrator::new(config).await?;
    
    // Start TUI application
    let mut app = App::new(orchestrator);
    app.run().await?;
    
    info!("DeskAgent shutdown complete");
    Ok(())
}