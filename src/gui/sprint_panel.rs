use super::state::{AppState, ModuleStatus};
use eframe::egui;

pub struct SprintPanel {
    // State for interactive elements
    expand_all: bool,
}

impl SprintPanel {
    pub fn new(_state: AppState) -> Self {
        Self {
            expand_all: false,
        }
    }
    
    pub fn update(&mut self, state: &mut AppState, ui: &mut egui::Ui) {
        ui.spacing_mut().item_spacing.y = 8.0;
        
        egui::ScrollArea::vertical().show(ui, |ui| {
            self.render_sprint_header(state, ui);
            
            ui.add_space(16.0);
            
            self.render_module_tree(state, ui);
            
            ui.add_space(16.0);
            
            self.render_deliverables(state, ui);
            
            ui.add_space(16.0);
            
            self.render_actions(state, ui);
        });
    }
    
    fn render_sprint_header(&self, state: &AppState, ui: &mut egui::Ui) {
        ui.group(|ui| {
            ui.set_min_height(80.0);
            
            ui.horizontal(|ui| {
                ui.vertical(|ui| {
                    ui.heading(&format!("📋 {}: DeskAgent v1.0 Core Implementation", state.sprint_state.sprint_name));
                    
                    ui.horizontal(|ui| {
                        let (icon, color) = match state.sprint_state.status.as_str() {
                            "COMPLETED" => ("✅", egui::Color32::GREEN),
                            "IN_PROGRESS" => ("🔄", egui::Color32::BLUE),
                            "FAILED" => ("❌", egui::Color32::RED),
                            _ => ("⏸️", egui::Color32::YELLOW),
                        };
                        
                        ui.colored_label(color, format!("Status: {} {}", icon, state.sprint_state.status));
                        ui.separator();
                        ui.label(format!("Duration: {}", state.sprint_state.duration));
                        ui.separator();
                        ui.label(format!("Progress: {:.0}%", state.sprint_state.progress));
                    });
                });
                
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    // Progress indicator
                    let progress = state.sprint_state.progress / 100.0;
                    let progress_bar = egui::ProgressBar::new(progress)
                        .text(format!("{:.0}%", state.sprint_state.progress))
                        .desired_height(20.0);
                    ui.add_sized([200.0, 20.0], progress_bar);
                });
            });
        });
    }
    
    fn render_module_tree(&mut self, state: &mut AppState, ui: &mut egui::Ui) {
        ui.group(|ui| {
            ui.horizontal(|ui| {
                ui.label("📦 Module Tree:");
                
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.small_button("Collapse All").clicked() {
                        self.expand_all = false;
                        for module in &mut state.sprint_state.modules {
                            module.expanded = false;
                        }
                    }
                    
                    if ui.small_button("Expand All").clicked() {
                        self.expand_all = true;
                        for module in &mut state.sprint_state.modules {
                            module.expanded = true;
                        }
                    }
                });
            });
            
            ui.separator();
            
            if state.sprint_state.modules.is_empty() {
                ui.label("Loading modules from progress data...");
                self.render_sample_modules(state, ui);
            } else {
                for module in &mut state.sprint_state.modules {
                    self.render_module_node(module, ui);
                }
            }
        });
    }
    
    fn render_sample_modules(&self, _state: &mut AppState, ui: &mut egui::Ui) {
        // Sample module data for demo
        let modules = [
            ("M1: Orchestrator", ModuleStatus::Completed, 11, 11),
            ("M2: Desktop Control", ModuleStatus::Completed, 17, 17),
            ("M3: LLM Router", ModuleStatus::Completed, 13, 13),
            ("M4: TUI Dashboard", ModuleStatus::Completed, 3, 3),
            ("M5: Workflow Integration", ModuleStatus::Completed, 12, 12),
        ];
        
        for (name, status, passed, total) in modules.iter() {
            ui.horizontal(|ui| {
                // Expand/collapse icon (non-functional in sample)
                ui.label("├─");
                
                // Status icon
                let (icon, color) = match status {
                    ModuleStatus::Completed => ("✅", egui::Color32::GREEN),
                    ModuleStatus::InProgress => ("🔄", egui::Color32::BLUE),
                    ModuleStatus::Failed => ("❌", egui::Color32::RED),
                    ModuleStatus::Paused => ("⏸️", egui::Color32::YELLOW),
                };
                
                ui.colored_label(color, icon);
                ui.label(*name);
                
                // Test count
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(format!("[{}/{}]", passed, total));
                });
            });
        }
    }
    
    fn render_module_node(&self, module: &mut super::state::ModuleInfo, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            // Expand/collapse toggle
            let expand_icon = if module.expanded { "📂" } else { "📁" };
            if ui.small_button(expand_icon).clicked() {
                module.expanded = !module.expanded;
            }
            
            // Module status icon
            let (icon, color) = match module.status {
                ModuleStatus::Completed => ("✅", egui::Color32::GREEN),
                ModuleStatus::InProgress => ("🔄", egui::Color32::BLUE),
                ModuleStatus::Failed => ("❌", egui::Color32::RED),
                ModuleStatus::Paused => ("⏸️", egui::Color32::YELLOW),
            };
            
            ui.colored_label(color, icon);
            ui.label(&module.name);
            
            // Test results
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.label(format!("[{}/{}]", module.tests_passed, module.tests_total));
            });
        });
        
        // Show sub-tasks if expanded
        if module.expanded {
            ui.indent("subtasks", |ui| {
                if module.sub_tasks.is_empty() {
                    ui.label("└─ No sub-tasks defined");
                } else {
                    for (i, subtask) in module.sub_tasks.iter().enumerate() {
                        let prefix = if i == module.sub_tasks.len() - 1 { "└─" } else { "├─" };
                        
                        ui.horizontal(|ui| {
                            ui.label(prefix);
                            
                            let (icon, color) = match subtask.status {
                                ModuleStatus::Completed => ("✅", egui::Color32::GREEN),
                                ModuleStatus::InProgress => ("🔄", egui::Color32::BLUE),
                                ModuleStatus::Failed => ("❌", egui::Color32::RED),
                                ModuleStatus::Paused => ("⏸️", egui::Color32::YELLOW),
                            };
                            
                            ui.colored_label(color, icon);
                            ui.label(&subtask.name);
                        });
                    }
                }
            });
        }
    }
    
    fn render_deliverables(&self, state: &AppState, ui: &mut egui::Ui) {
        ui.group(|ui| {
            ui.label("📄 Deliverables:");
            ui.separator();
            
            if state.sprint_state.deliverables.is_empty() {
                self.render_sample_deliverables(ui);
            } else {
                for deliverable in &state.sprint_state.deliverables {
                    ui.horizontal(|ui| {
                        let icon = if deliverable.completed { "✅" } else { "⏳" };
                        let color = if deliverable.completed { 
                            egui::Color32::GREEN 
                        } else { 
                            egui::Color32::YELLOW 
                        };
                        
                        ui.colored_label(color, icon);
                        ui.label(&deliverable.name);
                        
                        if ui.small_button("📂 Open").clicked() {
                            // TODO: Open file in editor
                        }
                    });
                }
            }
        });
    }
    
    fn render_sample_deliverables(&self, ui: &mut egui::Ui) {
        let deliverables = [
            ("plans/sprint-01.plan.json", true),
            ("reviews/AI_REVIEW.md", true),
            ("progress/sprint-01.progress.json", true),
            ("status/REPORT.md", true),
            ("routing/log.jsonl", true),
        ];
        
        for (name, completed) in deliverables.iter() {
            ui.horizontal(|ui| {
                let (icon, color) = if *completed {
                    ("✅", egui::Color32::GREEN)
                } else {
                    ("⏳", egui::Color32::YELLOW)
                };
                
                ui.colored_label(color, icon);
                ui.label(*name);
                
                if ui.small_button("📂 Open").clicked() {
                    // TODO: Open file
                }
            });
        }
    }
    
    fn render_actions(&self, _state: &mut AppState, ui: &mut egui::Ui) {
        ui.group(|ui| {
            ui.label("🎬 Actions:");
            
            ui.horizontal_wrapped(|ui| {
                if ui.button("📊 Export Report").clicked() {
                    // TODO: Export sprint report
                }
                
                if ui.button("🔄 Refresh Data").clicked() {
                    // TODO: Refresh sprint data
                }
                
                if ui.button("➡️ Next Sprint").clicked() {
                    // TODO: Generate next sprint plan
                }
                
                if ui.button("📋 Clone Sprint").clicked() {
                    // TODO: Create sprint template
                }
            });
        });
    }
}