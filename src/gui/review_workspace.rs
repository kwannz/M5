use super::state::{AppState, ReviewStatus, RiskLevel};
use eframe::egui;

pub struct ReviewWorkspace {
    // State for interactive elements
    selected_file: Option<usize>,
    show_analysis_details: bool,
}

impl ReviewWorkspace {
    pub fn new(_state: AppState) -> Self {
        Self {
            selected_file: None,
            show_analysis_details: false,
        }
    }
    
    pub fn update(&mut self, state: &mut AppState, ui: &mut egui::Ui) {
        ui.spacing_mut().item_spacing.y = 8.0;
        
        egui::ScrollArea::vertical().show(ui, |ui| {
            self.render_review_header(state, ui);
            
            ui.add_space(16.0);
            
            self.render_file_changes(state, ui);
            
            ui.add_space(16.0);
            
            self.render_analysis_summary(state, ui);
            
            ui.add_space(16.0);
            
            self.render_recommendations(state, ui);
            
            ui.add_space(16.0);
            
            self.render_actions(state, ui);
        });
    }
    
    fn render_review_header(&self, state: &AppState, ui: &mut egui::Ui) {
        ui.group(|ui| {
            ui.set_min_height(80.0);
            
            ui.horizontal(|ui| {
                ui.vertical(|ui| {
                    ui.heading(&format!("🔍 Review: {}", state.review_state.review_id));
                    
                    ui.horizontal(|ui| {
                        // Status badge
                        let (icon, color) = match state.review_state.status {
                            ReviewStatus::Approved => ("✅", egui::Color32::GREEN),
                            ReviewStatus::Pending => ("⏳", egui::Color32::YELLOW),
                            ReviewStatus::ChangesRequested => ("❌", egui::Color32::RED),
                        };
                        
                        ui.colored_label(color, format!("Status: {} {:?}", icon, state.review_state.status));
                        ui.separator();
                        
                        // Quality score
                        let score_color = if state.review_state.quality_score >= 90 {
                            egui::Color32::GREEN
                        } else if state.review_state.quality_score >= 70 {
                            egui::Color32::YELLOW
                        } else {
                            egui::Color32::RED
                        };
                        ui.colored_label(score_color, format!("Quality: {}/100", state.review_state.quality_score));
                        ui.separator();
                        
                        ui.label(format!("Reviewer: {}", state.review_state.reviewer));
                        ui.separator();
                        ui.label(format!("Time: {}", state.review_state.review_time));
                    });
                });
                
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    // Quality score ring/progress
                    let score = state.review_state.quality_score as f32 / 100.0;
                    let progress_bar = egui::ProgressBar::new(score)
                        .text(format!("{}%", state.review_state.quality_score))
                        .desired_height(24.0);
                    ui.add_sized([120.0, 24.0], progress_bar);
                });
            });
        });
    }
    
    fn render_file_changes(&mut self, state: &AppState, ui: &mut egui::Ui) {
        ui.group(|ui| {
            ui.label(&format!("📁 File Changes ({} files modified):", 
                state.review_state.file_changes.len()));
            ui.separator();
            
            if state.review_state.file_changes.is_empty() {
                self.render_sample_file_changes(ui);
            } else {
                // Table header
                ui.horizontal(|ui| {
                    ui.label("File");
                    ui.separator();
                    ui.label("+/-");
                    ui.separator();
                    ui.label("Risk");
                    ui.separator();
                    ui.label("Issues");
                });
                
                ui.separator();
                
                // File rows
                for (i, file_change) in state.review_state.file_changes.iter().enumerate() {
                    let selected = self.selected_file == Some(i);
                    
                    ui.horizontal(|ui| {
                        // File name (clickable)
                        let response = ui.selectable_label(selected, &file_change.file);
                        if response.clicked() {
                            self.selected_file = if selected { None } else { Some(i) };
                        }
                        
                        ui.separator();
                        
                        // Change count
                        ui.label(format!("+{}-{}", file_change.additions, file_change.deletions));
                        
                        ui.separator();
                        
                        // Risk level
                        let (risk_icon, risk_color) = match file_change.risk {
                            RiskLevel::High => ("🔴", egui::Color32::RED),
                            RiskLevel::Medium => ("🟡", egui::Color32::YELLOW),
                            RiskLevel::Low => ("🟢", egui::Color32::GREEN),
                        };
                        ui.colored_label(risk_color, risk_icon);
                        
                        ui.separator();
                        
                        // Issues summary
                        if file_change.issues.is_empty() {
                            ui.label("None");
                        } else {
                            ui.label(format!("{} issues", file_change.issues.len()));
                        }
                    });
                    
                    // Show issues if file is selected
                    if selected && !file_change.issues.is_empty() {
                        ui.indent("file_issues", |ui| {
                            for issue in &file_change.issues {
                                ui.label(format!("• {}", issue));
                            }
                        });
                    }
                }
            }
        });
    }
    
    fn render_sample_file_changes(&mut self, ui: &mut egui::Ui) {
        let sample_files = [
            ("src/orchestrator/", 45, 0, RiskLevel::Medium, "Complex state logic"),
            ("src/desktop/cursor", 32, 0, RiskLevel::High, "AppleScript safety"),
            ("src/llm/router.rs", 28, 0, RiskLevel::Low, "Clean implementation"),
            ("src/tui/mod.rs", 156, 0, RiskLevel::Medium, "Large UI module"),
            ("src/workflows/edit", 89, 0, RiskLevel::Low, "Good error handling"),
        ];
        
        // Table header
        ui.horizontal(|ui| {
            ui.label("File");
            ui.separator();
            ui.label("+/-");
            ui.separator();
            ui.label("Risk");
            ui.separator();
            ui.label("Issues");
        });
        
        ui.separator();
        
        for (i, (file, additions, _deletions, risk, issue)) in sample_files.iter().enumerate() {
            let selected = self.selected_file == Some(i);
            
            ui.horizontal(|ui| {
                let response = ui.selectable_label(selected, *file);
                if response.clicked() {
                    self.selected_file = if selected { None } else { Some(i) };
                }
                
                ui.separator();
                ui.label(format!("+{}", additions));
                ui.separator();
                
                let (risk_icon, risk_color) = match risk {
                    RiskLevel::High => ("🔴", egui::Color32::RED),
                    RiskLevel::Medium => ("🟡", egui::Color32::YELLOW),
                    RiskLevel::Low => ("🟢", egui::Color32::GREEN),
                };
                ui.colored_label(risk_color, risk_icon);
                
                ui.separator();
                ui.label(*issue);
            });
        }
    }
    
    fn render_analysis_summary(&mut self, state: &AppState, ui: &mut egui::Ui) {
        ui.group(|ui| {
            ui.horizontal(|ui| {
                ui.label("📊 Analysis Summary");
                
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let button_text = if self.show_analysis_details { "▼ Hide Details" } else { "▶ Show Details" };
                    if ui.small_button(button_text).clicked() {
                        self.show_analysis_details = !self.show_analysis_details;
                    }
                });
            });
            
            ui.separator();
            
            // High-level summary
            let summary_items = [
                ("Architecture:", &state.review_state.analysis_summary.architecture, "✅"),
                ("Error Handling:", &state.review_state.analysis_summary.error_handling, "✅"),
                ("Testing:", &state.review_state.analysis_summary.testing, "✅"),
                ("Performance:", &state.review_state.analysis_summary.performance, "⚠️"),
                ("Security:", &state.review_state.analysis_summary.security, "⚠️"),
            ];
            
            for (category, description, status) in summary_items.iter() {
                ui.horizontal(|ui| {
                    let color = match *status {
                        "✅" => egui::Color32::GREEN,
                        "⚠️" => egui::Color32::YELLOW,
                        "❌" => egui::Color32::RED,
                        _ => egui::Color32::GRAY,
                    };
                    
                    ui.colored_label(color, *status);
                    ui.label(*category);
                    
                    if self.show_analysis_details {
                        ui.label(*description);
                    }
                });
            }
        });
    }
    
    fn render_recommendations(&self, state: &AppState, ui: &mut egui::Ui) {
        ui.group(|ui| {
            ui.label("💡 Recommendations");
            ui.separator();
            
            if state.review_state.recommendations.is_empty() {
                self.render_sample_recommendations(ui);
            } else {
                for (i, recommendation) in state.review_state.recommendations.iter().enumerate() {
                    ui.horizontal(|ui| {
                        ui.label(&format!("{}.", i + 1));
                        ui.label(recommendation);
                    });
                }
            }
        });
    }
    
    fn render_sample_recommendations(&self, ui: &mut egui::Ui) {
        let recommendations = [
            "Move API key management to secure storage",
            "Replace blocking I/O with async alternatives",
            "Add input validation for AppleScript commands",
            "Consider rate limiting for LLM API calls",
        ];
        
        for (i, recommendation) in recommendations.iter().enumerate() {
            ui.horizontal(|ui| {
                ui.label(&format!("{}.", i + 1));
                ui.label(*recommendation);
                
                if ui.small_button("📋 Copy").clicked() {
                    // TODO: Copy to clipboard
                }
            });
        }
    }
    
    fn render_actions(&self, _state: &mut AppState, ui: &mut egui::Ui) {
        ui.group(|ui| {
            ui.label("🎬 Actions");
            
            ui.horizontal_wrapped(|ui| {
                if ui.button("✅ Approve").clicked() {
                    // TODO: Approve review
                }
                
                if ui.button("❌ Request Changes").clicked() {
                    // TODO: Request changes
                }
                
                if ui.button("📄 Export Report").clicked() {
                    // TODO: Export review report
                }
                
                if ui.button("🔍 View Diff").clicked() {
                    // TODO: Open diff viewer
                }
                
                if ui.button("🔄 Refresh").clicked() {
                    // TODO: Refresh review data
                }
            });
        });
    }
}