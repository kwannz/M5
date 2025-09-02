use eframe::egui;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AppTheme {
    Light,
    Dark,
    System,
}

pub struct ThemeManager {
    current_theme: AppTheme,
    custom_colors: ThemeColors,
}

#[derive(Debug, Clone)]
pub struct ThemeColors {
    pub success: egui::Color32,
    pub warning: egui::Color32,
    pub error: egui::Color32,
    pub info: egui::Color32,
    pub accent: egui::Color32,
}

impl Default for ThemeColors {
    fn default() -> Self {
        Self {
            success: egui::Color32::from_rgb(34, 197, 94),   // Green-500
            warning: egui::Color32::from_rgb(234, 179, 8),   // Yellow-500
            error: egui::Color32::from_rgb(239, 68, 68),     // Red-500
            info: egui::Color32::from_rgb(59, 130, 246),     // Blue-500
            accent: egui::Color32::from_rgb(147, 51, 234),   // Purple-500
        }
    }
}

impl ThemeManager {
    pub fn new(theme: AppTheme) -> Self {
        Self {
            current_theme: theme,
            custom_colors: ThemeColors::default(),
        }
    }
    
    pub fn apply_theme(&self, ctx: &egui::Context) {
        ctx.style_mut(|style| {
            match self.current_theme {
                AppTheme::Light => {
                    style.visuals = egui::Visuals::light();
                }
                AppTheme::Dark => {
                    style.visuals = egui::Visuals::dark();
                }
                AppTheme::System => {
                    // Use system default
                    style.visuals = if self.is_system_dark() {
                        egui::Visuals::dark()
                    } else {
                        egui::Visuals::light()
                    };
                }
            }
            
            // Apply custom colors
            self.customize_visuals(&mut style.visuals);
        });
    }
    
    fn customize_visuals(&self, visuals: &mut egui::Visuals) {
        // Customize progress bar colors
        visuals.selection.bg_fill = self.custom_colors.accent;
        
        // Customize hyperlink color
        visuals.hyperlink_color = self.custom_colors.info;
        
        // Round corners more
        visuals.widgets.noninteractive.rounding = egui::Rounding::same(6.0);
        visuals.widgets.inactive.rounding = egui::Rounding::same(6.0);
        visuals.widgets.hovered.rounding = egui::Rounding::same(6.0);
        visuals.widgets.active.rounding = egui::Rounding::same(6.0);
        visuals.widgets.open.rounding = egui::Rounding::same(6.0);
        
        // Panel rounding
        visuals.window_rounding = egui::Rounding::same(8.0);
        visuals.panel_fill = if visuals.dark_mode {
            egui::Color32::from_gray(27)
        } else {
            egui::Color32::from_gray(248)
        };
    }
    
    fn is_system_dark(&self) -> bool {
        // On macOS, we can check the system theme
        #[cfg(target_os = "macos")]
        {
            use std::process::Command;
            
            if let Ok(output) = Command::new("defaults")
                .args(&["read", "-g", "AppleInterfaceStyle"])
                .output()
            {
                return String::from_utf8_lossy(&output.stdout).trim() == "Dark";
            }
        }
        
        // Default to light theme if we can't detect
        false
    }
    
    pub fn set_theme(&mut self, theme: AppTheme) {
        self.current_theme = theme;
    }
    
    pub fn current_theme(&self) -> AppTheme {
        self.current_theme
    }
    
    pub fn colors(&self) -> &ThemeColors {
        &self.custom_colors
    }
    
    pub fn success_color(&self) -> egui::Color32 {
        self.custom_colors.success
    }
    
    pub fn warning_color(&self) -> egui::Color32 {
        self.custom_colors.warning
    }
    
    pub fn error_color(&self) -> egui::Color32 {
        self.custom_colors.error
    }
    
    pub fn info_color(&self) -> egui::Color32 {
        self.custom_colors.info
    }
    
    pub fn accent_color(&self) -> egui::Color32 {
        self.custom_colors.accent
    }
}

impl Default for ThemeManager {
    fn default() -> Self {
        Self::new(AppTheme::System)
    }
}