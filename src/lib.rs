pub mod orchestrator;
pub mod desktop;
pub mod llm;
pub mod tui;
pub mod workflows;
pub mod gui;

// Re-exports for convenience
pub use orchestrator::{Orchestrator, OrchestratorConfig};
pub use desktop::{CursorController, TerminalController};
pub use llm::{LlmRouter, LlmConfig, Provider};
pub use tui::App as TuiApp;
pub use workflows::{WorkflowManager, WorkflowType, WorkflowStatus};
pub use gui::{GuiApp, run_gui};

use anyhow::Result;

/// Initialize the DeskAgent system
pub async fn init() -> Result<()> {
    env_logger::init();
    log::info!("DeskAgent v1.0 initialized");
    Ok(())
}

/// Run the GUI application
pub async fn run_gui_app() -> Result<()> {
    init().await?;
    run_gui().await
}

/// Run the TUI application  
pub async fn run_tui_app() -> Result<()> {
    init().await?;
    
    let config = orchestrator::OrchestratorConfig::default();
    let orchestrator = Orchestrator::new(config).await?;
    let mut app = TuiApp::new(orchestrator);
    app.run().await
}