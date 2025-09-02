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
    log::info!("🚀 Starting DeskAgent GUI...");
    
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("DeskAgent v1.0")
            .with_inner_size([1200.0, 800.0])
            .with_min_inner_size([800.0, 600.0])
            .with_visible(true)
            .with_resizable(true)
            .with_position([100.0, 100.0]),
        centered: true,
        ..Default::default()
    };

    log::info!("📋 Creating GUI application state...");
    let app = match GuiApp::new().await {
        Ok(app) => {
            log::info!("✅ GUI application state created successfully");
            app
        }
        Err(e) => {
            log::error!("❌ Failed to create GUI application: {}", e);
            return Err(e);
        }
    };
    
    log::info!("🖥️ Launching native window...");
    let result = eframe::run_native(
        "DeskAgent v1.0",
        native_options,
        Box::new(move |cc| {
            log::info!("🎨 Initializing graphics context");
            
            // Configure egui style for better visibility
            cc.egui_ctx.set_pixels_per_point(1.0);
            
            Ok(Box::new(app))
        })
    );
    
    match result {
        Ok(_) => {
            log::info!("✅ GUI application closed normally");
            Ok(())
        }
        Err(e) => {
            log::error!("❌ GUI application error: {}", e);
            Err(anyhow::anyhow!("Failed to run GUI: {}", e))
        }
    }
}