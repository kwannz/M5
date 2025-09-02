use eframe::egui;

struct SimpleApp {
    name: String,
}

impl SimpleApp {
    fn new() -> Self {
        Self {
            name: "DeskAgent Test".to_string(),
        }
    }
    
    fn configure_fonts(ctx: &egui::Context) {
        let mut fonts = egui::FontDefinitions::default();
        
        // Configure larger font sizes for better readability
        let mut style = (*ctx.style()).clone();
        style.text_styles.insert(
            egui::TextStyle::Body,
            egui::FontId::new(16.0, egui::FontFamily::Proportional)
        );
        style.text_styles.insert(
            egui::TextStyle::Heading, 
            egui::FontId::new(24.0, egui::FontFamily::Proportional)
        );
        style.text_styles.insert(
            egui::TextStyle::Button,
            egui::FontId::new(16.0, egui::FontFamily::Proportional)
        );
        
        ctx.set_style(style);
        ctx.set_fonts(fonts);
    }
}

impl eframe::App for SimpleApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Configure fonts on first frame
        Self::configure_fonts(ctx);
        
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("🤖 DeskAgent GUI Test");
            ui.separator();
            ui.label("如果你看到这个消息，说明GUI基础设施工作正常。");
            ui.label("GUI infrastructure is working correctly if you see this message.");
            ui.label("Emoji test: 📊 🔍 📋 ⚡ 🛡️ 📦 🎨");
            
            ui.add_space(20.0);
            
            if ui.button("测试按钮 Test Button 🚀").clicked() {
                println!("Button clicked! GUI is responsive. 按钮被点击了！");
            }
        });
    }
}

fn main() -> Result<(), eframe::Error> {
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([800.0, 600.0])
            .with_min_inner_size([400.0, 300.0]),
        ..Default::default()
    };

    eframe::run_native(
        "DeskAgent GUI Test",
        native_options,
        Box::new(|_cc| Ok(Box::new(SimpleApp::new()))),
    )
}