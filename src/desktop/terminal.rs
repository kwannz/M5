use anyhow::{Result, anyhow};
use std::process::{Command, Stdio, Child};
use std::time::Duration;
use tokio::time::sleep;
use std::io::{BufRead, BufReader};
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Debug)]
pub struct TerminalController {
    app_name: String,
    timeout: Duration,
    retry_attempts: u32,
}

#[derive(Debug, Clone)]
pub struct CommandResult {
    pub stdout: String,
    pub stderr: String,
    pub exit_code: i32,
    pub duration_ms: u64,
}

#[derive(Debug)]
pub struct TerminalSession {
    process: Arc<Mutex<Option<Child>>>,
    session_id: String,
}

impl TerminalController {
    pub fn new() -> Self {
        Self {
            app_name: "Terminal".to_string(),
            timeout: Duration::from_secs(10),
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
    
    /// Execute a command in the terminal and return the result
    pub async fn execute_command(&self, command: &str) -> Result<CommandResult> {
        let start_time = std::time::Instant::now();
        
        for attempt in 1..=self.retry_attempts {
            let result = self.try_execute_command(command).await;
            
            match result {
                Ok(mut cmd_result) => {
                    cmd_result.duration_ms = start_time.elapsed().as_millis() as u64;
                    return Ok(cmd_result);
                }
                Err(e) if attempt < self.retry_attempts => {
                    log::warn!("Command execution attempt {} failed, retrying: {}", attempt, e);
                    sleep(Duration::from_millis(500 * attempt as u64)).await;
                }
                Err(e) => return Err(anyhow!("Command failed after {} attempts: {}", self.retry_attempts, e)),
            }
        }
        
        unreachable!()
    }
    
    async fn try_execute_command(&self, command: &str) -> Result<CommandResult> {
        // Use bash to execute the command for better compatibility
        let output = if cfg!(target_os = "macos") || cfg!(target_os = "linux") {
            Command::new("bash")
                .arg("-c")
                .arg(command)
                .output()
                .map_err(|e| anyhow!("Failed to execute command '{}': {}", command, e))?
        } else {
            return Err(anyhow!("Terminal controller only supports macOS and Linux"));
        };
        
        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        let exit_code = output.status.code().unwrap_or(-1);
        
        Ok(CommandResult {
            stdout,
            stderr,
            exit_code,
            duration_ms: 0, // Will be set by caller
        })
    }
    
    /// Execute a command and capture output with real-time streaming
    pub async fn execute_command_streaming<F>(&self, command: &str, mut output_handler: F) -> Result<CommandResult>
    where
        F: FnMut(&str),
    {
        let start_time = std::time::Instant::now();
        
        let mut child = if cfg!(target_os = "macos") || cfg!(target_os = "linux") {
            Command::new("bash")
                .arg("-c")
                .arg(command)
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .spawn()
                .map_err(|e| anyhow!("Failed to spawn command '{}': {}", command, e))?
        } else {
            return Err(anyhow!("Terminal controller only supports macOS and Linux"));
        };
        
        let stdout = child.stdout.take().ok_or_else(|| anyhow!("Failed to capture stdout"))?;
        let stderr = child.stderr.take().ok_or_else(|| anyhow!("Failed to capture stderr"))?;
        
        let stdout_reader = BufReader::new(stdout);
        let stderr_reader = BufReader::new(stderr);
        
        let mut stdout_lines = Vec::new();
        let mut stderr_lines = Vec::new();
        
        // Read stdout
        for line in stdout_reader.lines() {
            let line = line.map_err(|e| anyhow!("Failed to read stdout line: {}", e))?;
            output_handler(&line);
            stdout_lines.push(line);
        }
        
        // Read stderr
        for line in stderr_reader.lines() {
            let line = line.map_err(|e| anyhow!("Failed to read stderr line: {}", e))?;
            output_handler(&format!("STDERR: {}", line));
            stderr_lines.push(line);
        }
        
        let exit_status = child.wait().map_err(|e| anyhow!("Failed to wait for command: {}", e))?;
        let exit_code = exit_status.code().unwrap_or(-1);
        let duration_ms = start_time.elapsed().as_millis() as u64;
        
        Ok(CommandResult {
            stdout: stdout_lines.join("\n"),
            stderr: stderr_lines.join("\n"),
            exit_code,
            duration_ms,
        })
    }
    
    /// Open Terminal app using AppleScript (macOS specific)
    pub async fn open_terminal(&self) -> Result<()> {
        if !cfg!(target_os = "macos") {
            return Err(anyhow!("open_terminal is only supported on macOS"));
        }
        
        let script = format!(
            r#"
            tell application "{}"
                activate
                do script ""
            end tell
            "#,
            self.app_name
        );
        
        let output = Command::new("osascript")
            .arg("-e")
            .arg(&script)
            .output()
            .map_err(|e| anyhow!("Failed to execute AppleScript: {}", e))?;
        
        if !output.status.success() {
            return Err(anyhow!(
                "Terminal AppleScript failed: {}",
                String::from_utf8_lossy(&output.stderr)
            ));
        }
        
        // Wait for Terminal to open
        sleep(Duration::from_millis(500)).await;
        Ok(())
    }
    
    /// Execute command in Terminal app using AppleScript (macOS specific)
    pub async fn execute_in_terminal(&self, command: &str) -> Result<()> {
        if !cfg!(target_os = "macos") {
            return Err(anyhow!("execute_in_terminal is only supported on macOS"));
        }
        
        let escaped_command = command.replace("\"", "\\\"");
        let script = format!(
            r#"
            tell application "{}"
                activate
                do script "{}"
            end tell
            "#,
            self.app_name, escaped_command
        );
        
        let output = Command::new("osascript")
            .arg("-e")
            .arg(&script)
            .output()
            .map_err(|e| anyhow!("Failed to execute AppleScript: {}", e))?;
        
        if !output.status.success() {
            return Err(anyhow!(
                "Terminal command AppleScript failed: {}",
                String::from_utf8_lossy(&output.stderr)
            ));
        }
        
        Ok(())
    }
    
    /// Check if Terminal app is running
    pub fn is_terminal_running(&self) -> Result<bool> {
        let output = Command::new("pgrep")
            .arg("-f")
            .arg(&self.app_name)
            .output()
            .map_err(|e| anyhow!("Failed to check if Terminal is running: {}", e))?;
        
        Ok(output.status.success() && !output.stdout.is_empty())
    }
    
    /// Create a new terminal session for interactive use
    pub async fn create_session(&self) -> Result<TerminalSession> {
        let session_id = uuid::Uuid::new_v4().to_string();
        
        let session = TerminalSession {
            process: Arc::new(Mutex::new(None)),
            session_id,
        };
        
        Ok(session)
    }
    
    /// Validate command before execution (basic safety checks)
    pub fn validate_command(&self, command: &str) -> Result<()> {
        let command = command.trim();
        
        if command.is_empty() {
            return Err(anyhow!("Command cannot be empty"));
        }
        
        // Basic safety checks for potentially dangerous commands
        let dangerous_patterns = [
            "rm -rf /",
            "sudo rm -rf",
            "format",
            "> /dev/",
            "dd if=",
        ];
        
        for pattern in &dangerous_patterns {
            if command.contains(pattern) {
                return Err(anyhow!("Potentially dangerous command detected: {}", pattern));
            }
        }
        
        Ok(())
    }
    
    /// Execute a safe command with validation
    pub async fn execute_safe_command(&self, command: &str) -> Result<CommandResult> {
        self.validate_command(command)?;
        self.execute_command(command).await
    }
}

impl TerminalSession {
    pub fn get_session_id(&self) -> &str {
        &self.session_id
    }
    
    pub async fn is_active(&self) -> bool {
        let process = self.process.lock().await;
        process.is_some()
    }
    
    pub async fn terminate(&self) -> Result<()> {
        let mut process = self.process.lock().await;
        if let Some(mut child) = process.take() {
            child.kill().map_err(|e| anyhow!("Failed to terminate session: {}", e))?;
            child.wait().map_err(|e| anyhow!("Failed to wait for session termination: {}", e))?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_terminal_controller_creation() {
        let controller = TerminalController::new();
        assert_eq!(controller.app_name, "Terminal");
        assert_eq!(controller.timeout, Duration::from_secs(10));
        assert_eq!(controller.retry_attempts, 3);
    }
    
    #[test]
    fn test_terminal_controller_with_config() {
        let controller = TerminalController::with_config("TestTerminal".to_string(), 5000, 2);
        assert_eq!(controller.app_name, "TestTerminal");
        assert_eq!(controller.timeout, Duration::from_millis(5000));
        assert_eq!(controller.retry_attempts, 2);
    }
    
    #[test]
    fn test_command_validation() {
        let controller = TerminalController::new();
        
        // Valid commands
        assert!(controller.validate_command("echo hello").is_ok());
        assert!(controller.validate_command("ls -la").is_ok());
        assert!(controller.validate_command("git status").is_ok());
        
        // Invalid commands
        assert!(controller.validate_command("").is_err());
        assert!(controller.validate_command("rm -rf /").is_err());
        assert!(controller.validate_command("sudo rm -rf /Users").is_err());
    }
    
    #[tokio::test]
    async fn test_simple_command_execution() {
        let controller = TerminalController::new();
        
        // Test simple echo command
        let result = controller.execute_command("echo 'test'").await.unwrap();
        assert_eq!(result.exit_code, 0);
        assert!(result.stdout.contains("test"));
        assert!(result.stderr.is_empty());
    }
    
    #[tokio::test]
    async fn test_command_with_error() {
        let controller = TerminalController::new();
        
        // Test command that should fail
        let result = controller.execute_command("nonexistent_command").await.unwrap();
        assert_ne!(result.exit_code, 0);
        assert!(!result.stderr.is_empty());
    }
    
    #[tokio::test]
    async fn test_session_creation() {
        let controller = TerminalController::new();
        let session = controller.create_session().await.unwrap();
        
        assert!(!session.get_session_id().is_empty());
        assert!(!session.is_active().await);
    }
}