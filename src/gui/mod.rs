pub mod app;
pub mod state;
pub mod dashboard;
pub mod sprint_panel;
pub mod review_workspace;
pub mod components;
pub mod utils;

pub use app::GuiApp;
pub use state::AppState;

use anyhow::Result;
use eframe::egui;

/// Initialize and run the GUI application
pub async fn run_gui() -> Result<()> {
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1200.0, 800.0])
            .with_min_inner_size([800.0, 600.0]),
        ..Default::default()
    };

    let app = GuiApp::new().await?;
    
    eframe::run_native(
        "DeskAgent v1.0",
        native_options,
        Box::new(|_cc| Ok(Box::new(app)))
    ).map_err(|e| anyhow::anyhow!("Failed to run GUI: {}", e))
}