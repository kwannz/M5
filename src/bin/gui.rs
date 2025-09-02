use deskagent::run_gui_app;
use std::panic;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Note: Logging is initialized in deskagent::init()
    
    // Set up panic handler for better debugging
    panic::set_hook(Box::new(|panic_info| {
        eprintln!("🚨 PANIC occurred in DeskAgent GUI:");
        eprintln!("Location: {}", panic_info.location().map_or("Unknown".to_string(), |l| format!("{}:{}:{}", l.file(), l.line(), l.column())));
        eprintln!("Message: {}", panic_info.payload().downcast_ref::<&str>().unwrap_or(&"Unknown error"));
        
        // Log to file as well
        log::error!("🚨 PANIC: {} at {}", 
            panic_info.payload().downcast_ref::<&str>().unwrap_or(&"Unknown error"),
            panic_info.location().map_or("Unknown".to_string(), |l| format!("{}:{}:{}", l.file(), l.line(), l.column()))
        );
        
        // Show dialog on macOS
        #[cfg(target_os = "macos")]
        {
            std::process::Command::new("osascript")
                .args(&["-e", &format!("display dialog \"DeskAgent GUI crashed: {}\" buttons {{\"OK\"}} default button \"OK\"", 
                    panic_info.payload().downcast_ref::<&str>().unwrap_or(&"Unknown error"))])
                .output()
                .ok();
        }
    }));
    
    log::info!("🚀 DeskAgent GUI starting...");
    
    match run_gui_app().await {
        Ok(_) => {
            log::info!("✅ DeskAgent GUI closed successfully");
            Ok(())
        }
        Err(e) => {
            log::error!("❌ DeskAgent GUI error: {}", e);
            
            // Show error dialog on macOS
            #[cfg(target_os = "macos")]
            {
                std::process::Command::new("osascript")
                    .args(&["-e", &format!("display dialog \"DeskAgent GUI failed to start: {}\" buttons {{\"OK\"}} default button \"OK\"", e)])
                    .output()
                    .ok();
            }
            
            Err(e)
        }
    }
}