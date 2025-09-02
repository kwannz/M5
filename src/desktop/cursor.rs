use anyhow::{Result, anyhow};
use std::process::Command;
use std::time::Duration;
use tokio::time::sleep;

#[derive(Debug)]
pub struct CursorController {
    app_name: String,
    timeout: Duration,
    retry_attempts: u32,
}

#[derive(Debug, Clone)]
pub struct FilePosition {
    pub file_path: String,
    pub line: Option<u32>,
    pub column: Option<u32>,
}

impl CursorController {
    pub fn new() -> Self {
        Self {
            app_name: "Cursor".to_string(),
            timeout: Duration::from_secs(5),
            retry_attempts: 3,
        }
    }
    
    pub fn with_config(app_name: String, timeout_ms: u64, retry_attempts: u32) -> Self {
        Self {
            app_name,
            timeout: Duration::from_millis(timeout_ms),
            retry_attempts,
        }
    }
    
    /// Open Cursor IDE and optionally navigate to a specific file and position
    pub async fn open_cursor(&self, position: Option<FilePosition>) -> Result<()> {
        for attempt in 1..=self.retry_attempts {
            let result = self.try_open_cursor(position.clone()).await;
            
            match result {
                Ok(_) => return Ok(()),
                Err(e) if attempt < self.retry_attempts => {
                    log::warn!("Attempt {} failed, retrying: {}", attempt, e);
                    sleep(Duration::from_millis(500 * attempt as u64)).await;
                }
                Err(e) => return Err(anyhow!("Failed to open Cursor after {} attempts: {}", self.retry_attempts, e)),
            }
        }
        
        unreachable!()
    }
    
    async fn try_open_cursor(&self, position: Option<FilePosition>) -> Result<()> {
        match position {
            Some(pos) => {
                // Open file at specific position
                let mut args = vec![pos.file_path.clone()];
                
                if let (Some(line), Some(column)) = (pos.line, pos.column) {
                    args.push("--goto".to_string());
                    args.push(format!("{}:{}", line, column));
                } else if let Some(line) = pos.line {
                    args.push("--goto".to_string());
                    args.push(line.to_string());
                }
                
                let output = Command::new("cursor")
                    .args(&args)
                    .output()
                    .map_err(|e| anyhow!("Failed to execute cursor command: {}", e))?;
                
                if !output.status.success() {
                    return Err(anyhow!(
                        "Cursor command failed: {}",
                        String::from_utf8_lossy(&output.stderr)
                    ));
                }
            }
            None => {
                // Just open Cursor
                let output = Command::new("cursor")
                    .output()
                    .map_err(|e| anyhow!("Failed to execute cursor command: {}", e))?;
                
                if !output.status.success() {
                    return Err(anyhow!(
                        "Cursor command failed: {}",
                        String::from_utf8_lossy(&output.stderr)
                    ));
                }
            }
        }
        
        // Wait for Cursor to fully load
        sleep(Duration::from_millis(1000)).await;
        Ok(())
    }
    
    /// Insert text at current cursor position using AppleScript
    pub async fn insert_text_at_cursor(&self, text: &str) -> Result<()> {
        for attempt in 1..=self.retry_attempts {
            let result = self.try_insert_text(text).await;
            
            match result {
                Ok(_) => return Ok(()),
                Err(e) if attempt < self.retry_attempts => {
                    log::warn!("Insert text attempt {} failed, retrying: {}", attempt, e);
                    sleep(Duration::from_millis(500 * attempt as u64)).await;
                }
                Err(e) => return Err(anyhow!("Failed to insert text after {} attempts: {}", self.retry_attempts, e)),
            }
        }
        
        unreachable!()
    }
    
    async fn try_insert_text(&self, text: &str) -> Result<()> {
        // First, focus on Cursor
        self.focus_cursor().await?;
        
        // Use AppleScript to insert text
        let escaped_text = text.replace("\"", "\\\"").replace("\n", "\\n");
        let script = format!(
            r#"
            tell application "{}"
                activate
                delay 0.2
                tell application "System Events"
                    keystroke "{}"
                end tell
            end tell
            "#,
            self.app_name, escaped_text
        );
        
        let output = Command::new("osascript")
            .arg("-e")
            .arg(&script)
            .output()
            .map_err(|e| anyhow!("Failed to execute AppleScript: {}", e))?;
        
        if !output.status.success() {
            return Err(anyhow!(
                "AppleScript failed: {}",
                String::from_utf8_lossy(&output.stderr)
            ));
        }
        
        Ok(())
    }
    
    /// Save current file using Cmd+S
    pub async fn save_file(&self) -> Result<()> {
        self.focus_cursor().await?;
        
        let script = format!(
            r#"
            tell application "{}"
                activate
                delay 0.1
                tell application "System Events"
                    key code 1 using command down
                end tell
            end tell
            "#,
            self.app_name
        );
        
        let output = Command::new("osascript")
            .arg("-e")
            .arg(&script)
            .output()
            .map_err(|e| anyhow!("Failed to execute save AppleScript: {}", e))?;
        
        if !output.status.success() {
            return Err(anyhow!(
                "Save AppleScript failed: {}",
                String::from_utf8_lossy(&output.stderr)
            ));
        }
        
        // Wait for save to complete
        sleep(Duration::from_millis(500)).await;
        Ok(())
    }
    
    async fn focus_cursor(&self) -> Result<()> {
        let script = format!(
            r#"
            tell application "{}"
                activate
            end tell
            "#,
            self.app_name
        );
        
        let output = Command::new("osascript")
            .arg("-e")
            .arg(&script)
            .output()
            .map_err(|e| anyhow!("Failed to focus Cursor: {}", e))?;
        
        if !output.status.success() {
            return Err(anyhow!(
                "Focus AppleScript failed: {}",
                String::from_utf8_lossy(&output.stderr)
            ));
        }
        
        // Brief delay to ensure focus
        sleep(Duration::from_millis(200)).await;
        Ok(())
    }
    
    /// Check if Cursor is running
    pub fn is_cursor_running(&self) -> Result<bool> {
        let output = Command::new("pgrep")
            .arg("-f")
            .arg(&self.app_name)
            .output()
            .map_err(|e| anyhow!("Failed to check if Cursor is running: {}", e))?;
        
        Ok(output.status.success() && !output.stdout.is_empty())
    }
    
    /// Navigate to a specific file and position (integrated operation)
    pub async fn navigate_and_edit(&self, file_path: &str, line: Option<u32>, column: Option<u32>, text_to_insert: &str) -> Result<()> {
        let position = FilePosition {
            file_path: file_path.to_string(),
            line,
            column,
        };
        
        // Open file at position
        self.open_cursor(Some(position)).await?;
        
        // Insert text
        if !text_to_insert.is_empty() {
            sleep(Duration::from_millis(500)).await; // Wait for file to load
            self.insert_text_at_cursor(text_to_insert).await?;
        }
        
        // Save file
        self.save_file().await?;
        
        Ok(())
    }
    
    /// Insert text in a specific file by opening it and inserting at the end
    pub async fn insert_text(&self, file_path: &str, text: &str) -> Result<()> {
        // First open the file
        let position = Some(FilePosition {
            file_path: file_path.to_string(),
            line: None,
            column: None,
        });
        
        self.open_cursor(position).await?;
        
        // Wait for file to load
        sleep(Duration::from_millis(500)).await;
        
        // Move to end of file and insert text
        self.focus_cursor().await?;
        
        let script = format!(
            r#"
            tell application "{}"
                activate
                delay 0.1
                tell application "System Events"
                    key code 125 using command down -- Cmd+Down to end of file
                    delay 0.1
                    keystroke "{}"
                end tell
            end tell
            "#,
            self.app_name,
            text.replace("\"", "\\\"").replace("\n", "\\n")
        );
        
        let output = Command::new("osascript")
            .arg("-e")
            .arg(&script)
            .output()
            .map_err(|e| anyhow!("Failed to execute insert text AppleScript: {}", e))?;
        
        if !output.status.success() {
            return Err(anyhow!(
                "Insert text AppleScript failed: {}",
                String::from_utf8_lossy(&output.stderr)
            ));
        }
        
        // Save the file
        self.save_file().await?;
        
        Ok(())
    }
    
    /// Append text to end of file
    pub async fn append_to_file(&self, file_path: &str, text: &str) -> Result<()> {
        self.insert_text(file_path, text).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    use std::io::Write;
    
    #[test]
    fn test_cursor_controller_creation() {
        let controller = CursorController::new();
        assert_eq!(controller.app_name, "Cursor");
        assert_eq!(controller.timeout, Duration::from_secs(5));
        assert_eq!(controller.retry_attempts, 3);
    }
    
    #[test]
    fn test_cursor_controller_with_config() {
        let controller = CursorController::with_config("TestCursor".to_string(), 3000, 5);
        assert_eq!(controller.app_name, "TestCursor");
        assert_eq!(controller.timeout, Duration::from_millis(3000));
        assert_eq!(controller.retry_attempts, 5);
    }
    
    #[test] 
    fn test_file_position() {
        let position = FilePosition {
            file_path: "/path/to/file.rs".to_string(),
            line: Some(42),
            column: Some(10),
        };
        
        assert_eq!(position.file_path, "/path/to/file.rs");
        assert_eq!(position.line, Some(42));
        assert_eq!(position.column, Some(10));
    }
    
    // Note: Integration tests for actual Cursor interaction would require
    // Cursor to be installed and would be platform-specific
}