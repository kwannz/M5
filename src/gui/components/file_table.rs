use eframe::egui;

#[derive(Debug, Clone)]
pub struct FileEntry {
    pub path: String,
    pub additions: u32,
    pub deletions: u32,
    pub status: FileStatus,
    pub issues: Vec<String>,
}

#[derive(Debug, Clone)]
pub enum FileStatus {
    Added,
    Modified,
    Deleted,
    Renamed,
}

pub struct FileTable {
    entries: Vec<FileEntry>,
    selected_index: Option<usize>,
    sort_column: SortColumn,
    sort_ascending: bool,
}

#[derive(Debug, Clone, Copy)]
enum SortColumn {
    Path,
    Changes,
    Status,
}

impl FileTable {
    pub fn new(entries: Vec<FileEntry>) -> Self {
        Self {
            entries,
            selected_index: None,
            sort_column: SortColumn::Path,
            sort_ascending: true,
        }
    }
    
    pub fn ui(&mut self, ui: &mut egui::Ui) -> Option<&FileEntry> {
        let mut selected_entry = None;
        
        ui.horizontal(|ui| {
            // Column headers
            if ui.button("📁 File").clicked() {
                self.toggle_sort(SortColumn::Path);
            }
            
            ui.separator();
            
            if ui.button("📊 +/-").clicked() {
                self.toggle_sort(SortColumn::Changes);
            }
            
            ui.separator();
            
            if ui.button("🏷️ Status").clicked() {
                self.toggle_sort(SortColumn::Status);
            }
            
            ui.separator();
            ui.label("Issues");
        });
        
        ui.separator();
        
        // Sort entries
        self.sort_entries();
        
        // File rows
        egui::ScrollArea::vertical()
            .max_height(400.0)
            .show(ui, |ui| {
                for (i, entry) in self.entries.iter().enumerate() {
                    let selected = self.selected_index == Some(i);
                    
                    ui.horizontal(|ui| {
                        // File path (clickable)
                        let response = ui.selectable_label(selected, &entry.path);
                        if response.clicked() {
                            self.selected_index = if selected { None } else { Some(i) };
                            if !selected {
                                selected_entry = Some(entry);
                            }
                        }
                        
                        ui.separator();
                        
                        // Change count
                        let change_text = if entry.deletions > 0 {
                            format!("+{} -{}", entry.additions, entry.deletions)
                        } else {
                            format!("+{}", entry.additions)
                        };
                        
                        let change_color = if entry.additions > entry.deletions {
                            egui::Color32::GREEN
                        } else if entry.deletions > entry.additions {
                            egui::Color32::RED
                        } else {
                            egui::Color32::YELLOW
                        };
                        
                        ui.colored_label(change_color, change_text);
                        
                        ui.separator();
                        
                        // Status
                        let (status_icon, status_color) = match entry.status {
                            FileStatus::Added => ("✅", egui::Color32::GREEN),
                            FileStatus::Modified => ("📝", egui::Color32::BLUE),
                            FileStatus::Deleted => ("❌", egui::Color32::RED),
                            FileStatus::Renamed => ("🔄", egui::Color32::YELLOW),
                        };
                        
                        ui.colored_label(status_color, status_icon);
                        
                        ui.separator();
                        
                        // Issue count
                        if entry.issues.is_empty() {
                            ui.label("None");
                        } else {
                            ui.colored_label(
                                egui::Color32::YELLOW,
                                format!("{} issues", entry.issues.len())
                            );
                        }
                    });
                    
                    // Show issues if selected
                    if selected && !entry.issues.is_empty() {
                        ui.indent("file_issues", |ui| {
                            for issue in &entry.issues {
                                ui.label(format!("• {}", issue));
                            }
                        });
                    }
                }
            });
        
        selected_entry
    }
    
    fn toggle_sort(&mut self, column: SortColumn) {
        if std::mem::discriminant(&self.sort_column) == std::mem::discriminant(&column) {
            self.sort_ascending = !self.sort_ascending;
        } else {
            self.sort_column = column;
            self.sort_ascending = true;
        }
    }
    
    fn sort_entries(&mut self) {
        self.entries.sort_by(|a, b| {
            let cmp = match self.sort_column {
                SortColumn::Path => a.path.cmp(&b.path),
                SortColumn::Changes => (a.additions + a.deletions).cmp(&(b.additions + b.deletions)),
                SortColumn::Status => format!("{:?}", a.status).cmp(&format!("{:?}", b.status)),
            };
            
            if self.sort_ascending {
                cmp
            } else {
                cmp.reverse()
            }
        });
    }
}