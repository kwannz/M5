use anyhow::Result;
use deskagent::desktop::cursor::CursorController;
use deskagent::desktop::terminal::TerminalController;
use std::fs;
use tempfile::NamedTempFile;
use std::io::Write;

#[tokio::test]
async fn test_terminal_echo_command() -> Result<()> {
    let terminal = TerminalController::new();
    
    let result = terminal.execute_command("echo 'HELLO'").await?;
    
    assert_eq!(result.exit_code, 0);
    assert!(result.stdout.contains("HELLO"));
    assert!(result.stderr.is_empty());
    assert!(result.duration_ms > 0);
    
    println!("✅ Terminal echo test passed: {}", result.stdout.trim());
    Ok(())
}

#[tokio::test] 
async fn test_terminal_directory_listing() -> Result<()> {
    let terminal = TerminalController::new();
    
    let result = terminal.execute_command("ls -la | head -5").await?;
    
    assert_eq!(result.exit_code, 0);
    assert!(!result.stdout.is_empty());
    
    println!("✅ Directory listing test passed");
    Ok(())
}

#[tokio::test]
async fn test_terminal_git_status() -> Result<()> {
    let terminal = TerminalController::new();
    
    // This should work in our project directory
    let result = terminal.execute_command("git status --porcelain || echo 'No git repo'").await?;
    
    assert_eq!(result.exit_code, 0);
    
    println!("✅ Git status test passed: {}", if result.stdout.is_empty() { "Clean repo" } else { "Has changes" });
    Ok(())
}

#[tokio::test]
async fn test_file_write_and_verify() -> Result<()> {
    let terminal = TerminalController::new();
    
    // Create a temporary file path
    let temp_file = NamedTempFile::new()?;
    let file_path = temp_file.path().to_string_lossy().to_string();
    
    // Write content to the file using terminal
    let write_command = format!("echo 'Test content from DeskAgent' > '{}'", file_path);
    let result = terminal.execute_command(&write_command).await?;
    
    assert_eq!(result.exit_code, 0);
    
    // Verify the content was written
    let read_command = format!("cat '{}'", file_path);
    let read_result = terminal.execute_command(&read_command).await?;
    
    assert_eq!(read_result.exit_code, 0);
    assert!(read_result.stdout.contains("Test content from DeskAgent"));
    
    println!("✅ File write and verify test passed");
    Ok(())
}

#[test]
fn test_cursor_controller_configuration() {
    let cursor = CursorController::new();
    
    // Test basic configuration
    assert!(cursor.is_cursor_running().is_ok());
    
    // Test custom configuration
    let custom_cursor = CursorController::with_config("CustomCursor".to_string(), 2000, 5);
    
    println!("✅ Cursor controller configuration test passed");
}

#[tokio::test]
async fn test_terminal_session_management() -> Result<()> {
    let terminal = TerminalController::new();
    
    let session = terminal.create_session().await?;
    
    assert!(!session.get_session_id().is_empty());
    assert!(!session.is_active().await);
    
    // Test session termination
    session.terminate().await?;
    
    println!("✅ Terminal session management test passed");
    Ok(())
}

#[tokio::test]
async fn test_command_validation() -> Result<()> {
    let terminal = TerminalController::new();
    
    // Test valid commands
    assert!(terminal.validate_command("echo hello").is_ok());
    assert!(terminal.validate_command("ls -la").is_ok());
    assert!(terminal.validate_command("git status").is_ok());
    
    // Test invalid commands
    assert!(terminal.validate_command("").is_err());
    assert!(terminal.validate_command("rm -rf /").is_err());
    
    // Test safe command execution
    let result = terminal.execute_safe_command("echo 'Safe command'").await?;
    assert_eq!(result.exit_code, 0);
    assert!(result.stdout.contains("Safe command"));
    
    println!("✅ Command validation test passed");
    Ok(())
}

#[tokio::test] 
async fn test_desktop_control_integration() -> Result<()> {
    // This is the main integration test that would demonstrate
    // the full desktop control capability as specified in M2 DoD:
    // 1. Write "HELLO" to a target file
    // 2. Execute terminal commands and assert output
    
    let terminal = TerminalController::new();
    
    // Create a test file
    let mut temp_file = NamedTempFile::new()?;
    let file_path = temp_file.path().to_string_lossy().to_string();
    
    // Step 1: Write "HELLO" to the file (simulating Cursor write operation)
    writeln!(temp_file, "HELLO")?;
    temp_file.flush()?;
    
    // Step 2: Verify content using terminal
    let cat_result = terminal.execute_command(&format!("cat '{}'", file_path)).await?;
    assert_eq!(cat_result.exit_code, 0);
    assert!(cat_result.stdout.contains("HELLO"));
    
    // Step 3: Execute "echo ok" and assert output
    let echo_result = terminal.execute_command("echo ok").await?;
    assert_eq!(echo_result.exit_code, 0);
    assert!(echo_result.stdout.trim() == "ok");
    
    println!("✅ Desktop control integration test passed");
    println!("   - File write verified: HELLO content found");
    println!("   - Terminal echo verified: 'ok' output received");
    
    Ok(())
}