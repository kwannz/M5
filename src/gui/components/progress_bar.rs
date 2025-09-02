use eframe::egui;

pub struct EnhancedProgressBar {
    progress: f32,
    text: Option<String>,
    show_percentage: bool,
    color: Option<egui::Color32>,
}

impl EnhancedProgressBar {
    pub fn new(progress: f32) -> Self {
        Self {
            progress: progress.clamp(0.0, 1.0),
            text: None,
            show_percentage: true,
            color: None,
        }
    }
    
    pub fn text(mut self, text: impl Into<String>) -> Self {
        self.text = Some(text.into());
        self
    }
    
    pub fn show_percentage(mut self, show: bool) -> Self {
        self.show_percentage = show;
        self
    }
    
    pub fn color(mut self, color: egui::Color32) -> Self {
        self.color = Some(color);
        self
    }
    
    pub fn ui(self, ui: &mut egui::Ui) {
        let desired_size = egui::vec2(ui.available_width(), 24.0);
        let (rect, _) = ui.allocate_exact_size(desired_size, egui::Sense::hover());
        
        if ui.is_rect_visible(rect) {
            let visuals = ui.style().visuals.clone();
            
            // Background
            ui.painter().rect_filled(
                rect,
                visuals.widgets.inactive.rounding,
                visuals.extreme_bg_color,
            );
            
            // Progress bar
            let progress_width = rect.width() * self.progress;
            let progress_rect = egui::Rect::from_min_size(
                rect.min,
                egui::vec2(progress_width, rect.height()),
            );
            
            let bar_color = self.color.unwrap_or(visuals.selection.bg_fill);
            ui.painter().rect_filled(
                progress_rect,
                visuals.widgets.inactive.rounding,
                bar_color,
            );
            
            // Text
            let text = if let Some(custom_text) = self.text {
                custom_text
            } else if self.show_percentage {
                format!("{:.0}%", self.progress * 100.0)
            } else {
                String::new()
            };
            
            if !text.is_empty() {
                ui.painter().text(
                    rect.center(),
                    egui::Align2::CENTER_CENTER,
                    text,
                    egui::FontId::default(),
                    visuals.text_color(),
                );
            }
            
            // Border
            ui.painter().rect_stroke(
                rect,
                visuals.widgets.inactive.rounding,
                visuals.widgets.inactive.bg_stroke,
            );
        }
    }
}