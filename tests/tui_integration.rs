use anyhow::Result;
use chrono::Utc;
use deskagent::{
    orchestrator::{Orchestrator, OrchestratorConfig, TaskState},
    tui::{App, TaskSummary, PendingAction},
};
use std::time::Duration;
use tempfile::TempDir;
use tokio::fs;
use uuid::Uuid;

/// Test TUI app initialization
#[tokio::test]
async fn test_tui_app_creation() {
    let temp_dir = TempDir::new().unwrap();
    let config = OrchestratorConfig {
        max_concurrent_tasks: 5,
        task_timeout_ms: 30000,
        log_directory: temp_dir.path().to_str().unwrap().to_string(),
    };
    
    let orchestrator = Orchestrator::new(config).await.unwrap();
    let app = App::new(orchestrator);
    
    assert!(!app.should_quit);
    assert_eq!(app.status_message, "Ready");
    assert!(!app.show_confirmation);
    assert!(app.pending_action.is_none());
}

/// Test repository detection functionality
#[tokio::test]
async fn test_repository_detection() {
    let temp_dir = TempDir::new().unwrap();
    let config = OrchestratorConfig {
        max_concurrent_tasks: 5,
        task_timeout_ms: 30000,
        log_directory: temp_dir.path().to_str().unwrap().to_string(),
    };
    
    let orchestrator = Orchestrator::new(config).await.unwrap();
    let app = App::new(orchestrator);
    
    // Test repo detection (will use current directory)
    let repo = app.detect_current_repo().await;
    // Repo detection is optional and depends on git setup - just check it doesn't crash
    println!("Detected repo: {:?}", repo);
    
    // Test branch detection (may or may not be in a git repo)
    let branch = app.detect_current_branch().await;
    // Branch detection is optional and depends on git setup
    println!("Detected branch: {:?}", branch);
}

/// Test task summary loading  
#[tokio::test]
async fn test_task_summary_loading() {
    let temp_dir = TempDir::new().unwrap();
    let config = OrchestratorConfig {
        max_concurrent_tasks: 5,
        task_timeout_ms: 30000,
        log_directory: temp_dir.path().to_str().unwrap().to_string(),
    };
    
    let orchestrator = Orchestrator::new(config).await.unwrap();
    let app = App::new(orchestrator);
    
    // Create mock runs directory in a separate temp dir for testing
    let test_temp_dir = TempDir::new().unwrap();
    let runs_dir = test_temp_dir.path().join("runs");
    fs::create_dir_all(&runs_dir).await.unwrap();
    
    let run1_dir = runs_dir.join("test-run-1");
    fs::create_dir_all(&run1_dir).await.unwrap();
    
    let run_json = serde_json::json!({
        "task_type": "PLAN", 
        "success": true,
        "duration_ms": 1500,
        "error": null
    });
    
    fs::write(run1_dir.join("run.json"), run_json.to_string()).await.unwrap();
    
    // Change working directory temporarily to test directory
    let original_dir = std::env::current_dir().unwrap();
    std::env::set_current_dir(test_temp_dir.path()).unwrap();
    
    let tasks = app.load_recent_tasks().await.unwrap();
    
    // Restore original directory
    std::env::set_current_dir(original_dir).unwrap();
    
    // Should find our test task
    assert!(!tasks.is_empty(), "Expected at least one task");
    
    // Find our specific test task
    let plan_task = tasks.iter().find(|t| t.task_type == "PLAN" && t.duration_ms == Some(1500));
    assert!(plan_task.is_some(), "Should find our PLAN task with 1500ms duration");
    
    let plan_task = plan_task.unwrap();
    assert_eq!(plan_task.task_type, "PLAN");
    assert!(plan_task.success);
    assert_eq!(plan_task.duration_ms, Some(1500));
    assert_eq!(plan_task.status, TaskState::Completed);
}

/// Test completion percentage calculation
#[tokio::test]
async fn test_completion_percentage_calculation() {
    let temp_dir = TempDir::new().unwrap();
    let config = OrchestratorConfig {
        max_concurrent_tasks: 5,
        task_timeout_ms: 30000,
        log_directory: temp_dir.path().to_str().unwrap().to_string(),
    };
    
    let orchestrator = Orchestrator::new(config).await.unwrap();
    let app = App::new(orchestrator);
    
    let percentage = app.calculate_completion_percentage();
    assert_eq!(percentage, 60); // Based on M1, M2, M3 completed
}

/// Test summary text generation
#[tokio::test]
async fn test_summary_text_generation() {
    let temp_dir = TempDir::new().unwrap();
    let config = OrchestratorConfig {
        max_concurrent_tasks: 5,
        task_timeout_ms: 30000,
        log_directory: temp_dir.path().to_str().unwrap().to_string(),
    };
    
    let orchestrator = Orchestrator::new(config).await.unwrap();
    let app = App::new(orchestrator);
    
    let summary = app.generate_summary_text();
    let summary_string = format!("{:?}", summary);
    
    assert!(summary_string.contains("M1"));
    assert!(summary_string.contains("M2"));
    assert!(summary_string.contains("M3"));
    assert!(summary_string.contains("Tests: 33/33 passing"));
}

/// Test high-risk operation detection
#[tokio::test]
async fn test_high_risk_operation_detection() {
    let temp_dir = TempDir::new().unwrap();
    let config = OrchestratorConfig {
        max_concurrent_tasks: 5,
        task_timeout_ms: 30000,
        log_directory: temp_dir.path().to_str().unwrap().to_string(),
    };
    
    let orchestrator = Orchestrator::new(config).await.unwrap();
    let app = App::new(orchestrator);
    
    assert!(app.is_high_risk_operation("APPLY"));
    assert!(app.is_high_risk_operation("PLAN"));
    assert!(app.is_high_risk_operation("REVIEW"));
    assert!(app.is_high_risk_operation("FOLLOWUP"));
    assert!(!app.is_high_risk_operation("STATUS"));
    assert!(!app.is_high_risk_operation("UNKNOWN"));
}

/// Test confirmation dialog functionality
#[tokio::test]
async fn test_confirmation_dialog() {
    let temp_dir = TempDir::new().unwrap();
    let config = OrchestratorConfig {
        max_concurrent_tasks: 5,
        task_timeout_ms: 30000,
        log_directory: temp_dir.path().to_str().unwrap().to_string(),
    };
    
    let orchestrator = Orchestrator::new(config).await.unwrap();
    let mut app = App::new(orchestrator);
    
    // Initially no confirmation dialog
    assert!(!app.show_confirmation);
    assert!(app.pending_action.is_none());
    
    // Show confirmation dialog
    app.show_confirmation_dialog("Test message", PendingAction::Plan);
    
    assert!(app.show_confirmation);
    assert_eq!(app.confirmation_message, "Test message");
    assert!(matches!(app.pending_action, Some(PendingAction::Plan)));
    
    // Cancel action
    app.cancel_action();
    
    assert!(!app.show_confirmation);
    assert!(app.pending_action.is_none());
    assert_eq!(app.status_message, "Action cancelled");
}

/// Test task execution simulation
#[tokio::test]
async fn test_task_execution_simulation() {
    let temp_dir = TempDir::new().unwrap();
    let config = OrchestratorConfig {
        max_concurrent_tasks: 5,
        task_timeout_ms: 30000,
        log_directory: temp_dir.path().to_str().unwrap().to_string(),
    };
    
    let orchestrator = Orchestrator::new(config).await.unwrap();
    let mut app = App::new(orchestrator);
    
    // Initially no tasks
    assert_eq!(app.recent_tasks.len(), 0);
    
    // Execute a PLAN action
    app.execute_plan_action().await;
    
    // Should have created a task
    assert_eq!(app.recent_tasks.len(), 1);
    
    let task = &app.recent_tasks[0];
    assert_eq!(task.task_type, "PLAN");
    assert_eq!(task.status, TaskState::Completed);
    assert!(task.success);
    assert_eq!(task.duration_ms, Some(500));
    assert!(app.status_message.contains("PLAN operation completed"));
}

/// Test multiple task execution
#[tokio::test]
async fn test_multiple_task_execution() {
    let temp_dir = TempDir::new().unwrap();
    let config = OrchestratorConfig {
        max_concurrent_tasks: 5,
        task_timeout_ms: 30000,
        log_directory: temp_dir.path().to_str().unwrap().to_string(),
    };
    
    let orchestrator = Orchestrator::new(config).await.unwrap();
    let mut app = App::new(orchestrator);
    
    // Execute multiple actions
    app.execute_plan_action().await;
    app.execute_review_action().await;
    app.execute_apply_action().await;
    
    // Should have three tasks
    assert_eq!(app.recent_tasks.len(), 3);
    
    // Check task types (newest first)
    assert_eq!(app.recent_tasks[0].task_type, "APPLY");
    assert_eq!(app.recent_tasks[1].task_type, "REVIEW");
    assert_eq!(app.recent_tasks[2].task_type, "PLAN");
    
    // All should be completed successfully
    for task in &app.recent_tasks {
        assert_eq!(task.status, TaskState::Completed);
        assert!(task.success);
        assert!(task.duration_ms.is_some());
    }
}

/// Test TaskSummary serialization
#[test]
fn test_task_summary_serialization() {
    let task = TaskSummary {
        id: Uuid::new_v4(),
        task_type: "TEST".to_string(),
        status: TaskState::Completed,
        created_at: Utc::now(),
        duration_ms: Some(1000),
        success: true,
        error_message: None,
    };
    
    let json = serde_json::to_string(&task);
    assert!(json.is_ok());
    
    let deserialized: Result<TaskSummary, _> = serde_json::from_str(&json.unwrap());
    assert!(deserialized.is_ok());
    
    let deserialized = deserialized.unwrap();
    assert_eq!(deserialized.task_type, "TEST");
    assert_eq!(deserialized.status, TaskState::Completed);
    assert_eq!(deserialized.duration_ms, Some(1000));
    assert!(deserialized.success);
}

/// Test data refresh functionality
#[tokio::test]
async fn test_data_refresh() {
    let temp_dir = TempDir::new().unwrap();
    let config = OrchestratorConfig {
        max_concurrent_tasks: 5,
        task_timeout_ms: 30000,
        log_directory: temp_dir.path().to_str().unwrap().to_string(),
    };
    
    let orchestrator = Orchestrator::new(config).await.unwrap();
    let mut app = App::new(orchestrator);
    
    // Initially no refresh time
    let initial_time = app.last_refresh;
    
    // Add some delay to ensure time difference
    tokio::time::sleep(Duration::from_millis(10)).await;
    
    // Refresh data
    app.refresh_data().await.unwrap();
    
    // Refresh time should be updated
    assert!(app.last_refresh > initial_time);
}

/// Integration test: End-to-end TUI workflow simulation
#[tokio::test]
async fn test_e2e_tui_workflow() {
    let temp_dir = TempDir::new().unwrap();
    let config = OrchestratorConfig {
        max_concurrent_tasks: 5,
        task_timeout_ms: 30000,
        log_directory: temp_dir.path().to_str().unwrap().to_string(),
    };
    
    let orchestrator = Orchestrator::new(config).await.unwrap();
    let mut app = App::new(orchestrator);
    
    // 1. Initial state
    assert!(!app.should_quit);
    assert_eq!(app.recent_tasks.len(), 0);
    assert_eq!(app.status_message, "Ready");
    
    // 2. Refresh data (loads repo info and tasks)
    app.refresh_data().await.unwrap();
    
    // Repository detection is optional - just test that refresh works without crashing
    println!("Refreshed repo info: {:?}", app.current_repo);
    
    // 3. Simulate PLAN workflow
    app.handle_plan_action().await;
    
    // Should show confirmation dialog
    assert!(app.show_confirmation);
    assert!(matches!(app.pending_action, Some(PendingAction::Plan)));
    
    // 4. Confirm action
    app.confirm_action().await;
    
    // Dialog should be closed and task executed
    assert!(!app.show_confirmation);
    assert!(app.pending_action.is_none());
    assert_eq!(app.recent_tasks.len(), 1);
    assert_eq!(app.recent_tasks[0].task_type, "PLAN");
    assert_eq!(app.recent_tasks[0].status, TaskState::Completed);
    
    // 5. Simulate REVIEW workflow
    app.handle_review_action().await;
    app.confirm_action().await;
    
    assert_eq!(app.recent_tasks.len(), 2);
    assert_eq!(app.recent_tasks[0].task_type, "REVIEW");
    
    // 6. Test cancellation workflow
    app.handle_apply_action().await;
    assert!(app.show_confirmation);
    
    app.cancel_action();
    assert!(!app.show_confirmation);
    assert_eq!(app.status_message, "Action cancelled");
    assert_eq!(app.recent_tasks.len(), 2); // No new task added
    
    // 7. Test status action (no confirmation needed)
    app.handle_status_action().await;
    assert!(app.status_message.contains("Status updated successfully"));
}

/// Test centered rectangle helper function
#[test]
fn test_centered_rect() {
    use ratatui::layout::Rect;
    use deskagent::tui::centered_rect;
    
    let area = Rect {
        x: 0,
        y: 0,
        width: 100,
        height: 100,
    };
    
    let centered = centered_rect(50, 50, area);
    
    // Should be centered
    assert_eq!(centered.width, 50);
    assert_eq!(centered.height, 50);
    assert_eq!(centered.x, 25); // (100 - 50) / 2
    assert_eq!(centered.y, 25); // (100 - 50) / 2
}

/// Test loading from non-existent runs directory
#[tokio::test]
async fn test_load_tasks_no_runs_dir() {
    let temp_dir = TempDir::new().unwrap();
    let config = OrchestratorConfig {
        max_concurrent_tasks: 5,
        task_timeout_ms: 30000,
        log_directory: temp_dir.path().to_str().unwrap().to_string(),
    };
    
    let orchestrator = Orchestrator::new(config).await.unwrap();
    let app = App::new(orchestrator);
    
    // Change to temp directory without runs folder
    let original_dir = std::env::current_dir().unwrap();
    std::env::set_current_dir(temp_dir.path()).unwrap();
    
    let tasks = app.load_recent_tasks().await.unwrap();
    
    std::env::set_current_dir(original_dir).unwrap();
    
    // Should handle missing directory gracefully
    assert_eq!(tasks.len(), 0);
}

/// Test malformed run.json handling
#[tokio::test]
async fn test_load_tasks_malformed_json() {
    // Use a completely separate temp dir just for testing, not connected to orchestrator
    let test_temp_dir = TempDir::new().unwrap();
    let orchestrator_temp_dir = TempDir::new().unwrap();
    
    let config = OrchestratorConfig {
        max_concurrent_tasks: 5,
        task_timeout_ms: 30000,
        log_directory: orchestrator_temp_dir.path().to_str().unwrap().to_string(),
    };
    
    let orchestrator = Orchestrator::new(config).await.unwrap();
    let app = App::new(orchestrator);
    
    // Create runs directory with malformed JSON in test directory
    let runs_dir = test_temp_dir.path().join("runs");
    fs::create_dir_all(&runs_dir).await.unwrap();
    
    let run1_dir = runs_dir.join("malformed-run");
    fs::create_dir_all(&run1_dir).await.unwrap();
    
    // Write invalid JSON
    fs::write(run1_dir.join("run.json"), "{ invalid json }").await.unwrap();
    
    // Also create a directory without run.json to test that case
    let run2_dir = runs_dir.join("no-json-run");
    fs::create_dir_all(&run2_dir).await.unwrap();
    
    let original_dir = std::env::current_dir().unwrap();
    std::env::set_current_dir(test_temp_dir.path()).unwrap();
    
    let tasks = app.load_recent_tasks().await.unwrap();
    
    std::env::set_current_dir(original_dir).unwrap();
    
    // Should handle malformed JSON gracefully - no valid tasks should be loaded
    assert_eq!(tasks.len(), 0);
}