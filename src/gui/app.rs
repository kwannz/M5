use super::state::{AppState, ViewType};
use super::dashboard::Dashboard;
use super::sprint_panel::SprintPanel;
use super::review_workspace::ReviewWorkspace;
use anyhow::Result;
use eframe::egui;

pub struct GuiApp {
    state: AppState,
    dashboard: Dashboard,
    sprint_panel: SprintPanel,
    review_workspace: ReviewWorkspace,
}

impl GuiApp {
    pub async fn new() -> Result<Self> {
        let state = AppState::load_from_files().await?;
        
        Ok(Self {
            state: state.clone(),
            dashboard: Dashboard::new(state.clone()),
            sprint_panel: SprintPanel::new(state.clone()),
            review_workspace: ReviewWorkspace::new(state.clone()),
        })
    }
    
    fn render_top_bar(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.heading("🤖 DeskAgent v1.0");
            
            ui.separator();
            
            // View tabs
            ui.selectable_value(&mut self.state.current_view, ViewType::Dashboard, "📊 Dashboard");
            ui.selectable_value(&mut self.state.current_view, ViewType::SprintPanel, "📋 Sprint");
            ui.selectable_value(&mut self.state.current_view, ViewType::ReviewWorkspace, "🔍 Review");
            
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                // Settings button
                if ui.button("⚙️ Settings").clicked() {
                    // TODO: Open settings dialog
                }
                
                // Refresh button
                if ui.button("🔄 Refresh").clicked() {
                    // TODO: Refresh data
                }
                
                // Repository info
                ui.label(format!("📁 {} ({})", self.state.dashboard_state.repo_name, self.state.dashboard_state.branch));
            });
        });
    }
    
    fn handle_keyboard_shortcuts(&mut self, ctx: &egui::Context) {
        // Handle global keyboard shortcuts
        ctx.input(|i| {
            if i.key_pressed(egui::Key::P) {
                self.state.current_view = ViewType::Dashboard;
            }
            if i.key_pressed(egui::Key::R) {
                self.state.current_view = ViewType::ReviewWorkspace;
            }
            if i.key_pressed(egui::Key::S) {
                self.state.current_view = ViewType::SprintPanel;
            }
        });
    }
}

impl eframe::App for GuiApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Handle keyboard shortcuts
        self.handle_keyboard_shortcuts(ctx);
        
        // Configure style
        ctx.style_mut(|style| {
            // Use system theme detection
            style.visuals = if ctx.style().visuals.dark_mode {
                egui::Visuals::dark()
            } else {
                egui::Visuals::light()
            };
        });
        
        // Top panel for navigation
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            ui.add_space(4.0);
            self.render_top_bar(ui);
            ui.add_space(4.0);
        });
        
        // Main content area
        egui::CentralPanel::default().show(ctx, |ui| {
            match self.state.current_view {
                ViewType::Dashboard => {
                    self.dashboard.update(&mut self.state, ui);
                }
                ViewType::SprintPanel => {
                    self.sprint_panel.update(&mut self.state, ui);
                }
                ViewType::ReviewWorkspace => {
                    self.review_workspace.update(&mut self.state, ui);
                }
            }
        });
        
        // Auto-refresh every few seconds
        if self.state.settings.auto_refresh {
            ctx.request_repaint_after(std::time::Duration::from_secs(self.state.settings.refresh_interval));
        }
    }
}