use super::state::{AppState, ActivityStatus, RiskLevel};
use eframe::egui;

pub struct Dashboard {
    // State for interactive elements
    selected_activity: Option<usize>,
}

impl Dashboard {
    pub fn new(_state: AppState) -> Self {
        Self {
            selected_activity: None,
        }
    }
    
    pub fn update(&mut self, state: &mut AppState, ui: &mut egui::Ui) {
        ui.spacing_mut().item_spacing.y = 8.0;
        
        // Main dashboard content
        egui::ScrollArea::vertical().show(ui, |ui| {
            self.render_header_info(state, ui);
            
            ui.add_space(16.0);
            
            self.render_progress_section(state, ui);
            
            ui.add_space(16.0);
            
            self.render_activity_section(state, ui);
            
            ui.add_space(16.0);
            
            self.render_quick_actions(state, ui);
        });
    }
    
    fn render_header_info(&self, state: &AppState, ui: &mut egui::Ui) {
        ui.group(|ui| {
            ui.set_min_height(80.0);
            
            ui.horizontal(|ui| {
                // Repository info
                ui.vertical(|ui| {
                    ui.label("🏠 Repository");
                    ui.heading(&format!("{} v1.0", state.dashboard_state.repo_name));
                });
                
                ui.separator();
                
                // Branch info
                ui.vertical(|ui| {
                    ui.label("📍 Branch");
                    ui.heading(&state.dashboard_state.branch);
                });
                
                ui.separator();
                
                // Task summary
                ui.vertical(|ui| {
                    ui.label("📊 Tasks");
                    ui.horizontal(|ui| {
                        ui.label(&format!("{} total", state.dashboard_state.total_tasks));
                        ui.separator();
                        ui.colored_label(
                            egui::Color32::GREEN,
                            &format!("{} done", state.dashboard_state.completed_tasks)
                        );
                        ui.separator();
                        let pending = state.dashboard_state.total_tasks - state.dashboard_state.completed_tasks - state.dashboard_state.failed_tasks;
                        ui.colored_label(
                            egui::Color32::YELLOW,
                            &format!("{} pending", pending)
                        );
                        if state.dashboard_state.failed_tasks > 0 {
                            ui.separator();
                            ui.colored_label(
                                egui::Color32::RED,
                                &format!("{} failed", state.dashboard_state.failed_tasks)
                            );
                        }
                    });
                });
            });
        });
    }
    
    fn render_progress_section(&self, state: &AppState, ui: &mut egui::Ui) {
        ui.group(|ui| {
            ui.set_min_height(100.0);
            
            ui.label("📈 Sprint Progress");
            
            // Progress bar
            let progress = state.dashboard_state.progress_percentage / 100.0;
            let progress_bar = egui::ProgressBar::new(progress)
                .text(format!("{:.0}%", state.dashboard_state.progress_percentage))
                .desired_height(24.0);
            ui.add_sized([ui.available_width(), 24.0], progress_bar);
            
            ui.add_space(8.0);
            
            ui.horizontal(|ui| {
                // Last review status
                ui.vertical(|ui| {
                    ui.label("Last Review:");
                    let (icon, color) = match state.dashboard_state.last_review_status.as_str() {
                        "PASS" => ("✅", egui::Color32::GREEN),
                        "PENDING" => ("⏳", egui::Color32::YELLOW),
                        "FAIL" => ("❌", egui::Color32::RED),
                        _ => ("❓", egui::Color32::GRAY),
                    };
                    ui.colored_label(color, format!("{} {}", icon, state.dashboard_state.last_review_status));
                });
                
                ui.separator();
                
                // Risk summary
                ui.vertical(|ui| {
                    ui.label("Risks:");
                    ui.horizontal(|ui| {
                        for risk in &state.dashboard_state.risks {
                            let (icon, color) = match risk.level {
                                RiskLevel::High => ("🔴", egui::Color32::RED),
                                RiskLevel::Medium => ("🟡", egui::Color32::YELLOW),
                                RiskLevel::Low => ("🟢", egui::Color32::GREEN),
                            };
                            ui.colored_label(color, format!("{} {} {:?}", icon, risk.count, risk.level));
                        }
                    });
                });
            });
        });
    }
    
    fn render_activity_section(&mut self, state: &AppState, ui: &mut egui::Ui) {
        ui.group(|ui| {
            ui.label("🔔 Recent Activity");
            
            if state.dashboard_state.recent_activities.is_empty() {
                ui.label("No recent activities");
                
                // Add some sample activities for demo
                ui.separator();
                ui.label("Sample Activities:");
                self.render_sample_activities(ui);
            } else {
                egui::ScrollArea::vertical()
                    .max_height(200.0)
                    .show(ui, |ui| {
                        for (i, activity) in state.dashboard_state.recent_activities.iter().enumerate() {
                            let selected = self.selected_activity == Some(i);
                            
                            ui.horizontal(|ui| {
                                let (icon, color) = match activity.status {
                                    ActivityStatus::Success => ("✅", egui::Color32::GREEN),
                                    ActivityStatus::Failed => ("❌", egui::Color32::RED),
                                    ActivityStatus::Running => ("🔄", egui::Color32::BLUE),
                                    ActivityStatus::Paused => ("⏸️", egui::Color32::YELLOW),
                                };
                                
                                ui.colored_label(color, icon);
                                ui.label(activity.timestamp.format("%H:%M").to_string());
                                
                                let response = ui.selectable_label(selected, &activity.title);
                                if response.clicked() {
                                    self.selected_activity = if selected { None } else { Some(i) };
                                }
                            });
                        }
                    });
            }
        });
    }
    
    fn render_sample_activities(&self, ui: &mut egui::Ui) {
        let sample_activities = [
            ("✅", "15:42 M5 Workflow Integration", egui::Color32::GREEN),
            ("✅", "14:23 M4 TUI Dashboard", egui::Color32::GREEN),
            ("✅", "13:15 M3 LLM Router", egui::Color32::GREEN),
            ("✅", "12:01 M2 Desktop Control", egui::Color32::GREEN),
        ];
        
        for (icon, text, color) in sample_activities.iter() {
            ui.horizontal(|ui| {
                ui.colored_label(*color, *icon);
                ui.label(*text);
            });
        }
    }
    
    fn render_quick_actions(&self, state: &mut AppState, ui: &mut egui::Ui) {
        ui.group(|ui| {
            ui.label("⚡ Quick Actions");
            
            ui.horizontal_wrapped(|ui| {
                if ui.button("📋 [P] Plan").clicked() {
                    // TODO: Trigger plan workflow
                }
                
                if ui.button("🔍 [R] Review").clicked() {
                    state.current_view = super::state::ViewType::ReviewWorkspace;
                }
                
                if ui.button("📊 [S] Status").clicked() {
                    // TODO: Show status dialog
                }
                
                if ui.button("🔄 [F] Follow").clicked() {
                    // TODO: Trigger follow-up workflow
                }
                
                if ui.button("🛠️ [A] Apply").clicked() {
                    // TODO: Apply changes
                }
                
                if ui.button("📢 [N] Notify").clicked() {
                    // TODO: Send notifications
                }
                
                if ui.button("💾 [O] Offline").clicked() {
                    // TODO: Toggle offline mode
                }
                
                if ui.button("❓ [H] Help").clicked() {
                    // TODO: Show help dialog
                }
            });
        });
    }
}