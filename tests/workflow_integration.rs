use deskagent::{
    orchestrator::{Orchestrator, OrchestratorConfig},
    desktop::{CursorController, TerminalController},
    llm::LlmRouter,
    workflows::{WorkflowManager, WorkflowType, WorkflowStatus, PlanWorkflow, EditWorkflow, ReviewWorkflow},
};
use std::path::PathBuf;
use tempfile::TempDir;
use tokio::fs;

/// Test workflow manager creation and basic functionality
#[tokio::test]
async fn test_workflow_manager_creation() {
    let temp_dir = TempDir::new().unwrap();
    let config = OrchestratorConfig {
        max_concurrent_tasks: 5,
        task_timeout_ms: 30000,
        log_directory: temp_dir.path().join("logs").to_string_lossy().to_string(),
    };
    
    let orchestrator = Orchestrator::new(config).await.unwrap();
    let cursor = CursorController::new();
    let terminal = TerminalController::new();
    let llm_config = deskagent::llm::LlmConfig::default();
    let llm = LlmRouter::new(llm_config, "logs").await.unwrap();
    let base_path = temp_dir.path().to_path_buf();
    
    let manager = WorkflowManager::new(orchestrator, cursor, terminal, llm, base_path.clone());
    
    assert_eq!(manager.get_base_path(), &base_path);
}

/// Test PLAN workflow execution with mock sprint file
#[tokio::test]
async fn test_plan_workflow_execution() {
    let temp_dir = TempDir::new().unwrap();
    let base_path = temp_dir.path().to_path_buf();
    
    // Create test sprint file
    let sprint_content = r#"
# Test Sprint

## M5. PLAN & REVIEW Workflow

**Goal**: Implement basic PLAN → EDIT → REVIEW flow

**DoD**:
- [ ] Read SPRINTx.md → LLM generates task plan
- [ ] Insert placeholder code in Cursor  
- [ ] Generate review report with git diff + tests

**Tasks**:
- [ ] Implement plan generation
- [ ] Add desktop integration
- [ ] Create review analysis
"#;
    
    let sprint_file = base_path.join("test-sprint.md");
    fs::write(&sprint_file, sprint_content).await.unwrap();
    
    let llm_config = deskagent::llm::LlmConfig::default();
    let llm = LlmRouter::new(llm_config, "logs").await.unwrap();
    
    let plan_workflow = PlanWorkflow::new(&llm, &base_path);
    let result = plan_workflow.execute(sprint_file).await.unwrap();
    
    // Verify plan was created
    assert!(result.is_object());
    assert!(result["tasks"].is_array());
    assert!(result["overview"].is_string());
    
    // Verify plan file was created
    let plan_file = base_path.join("plans/sprint-01.plan.json");
    assert!(plan_file.exists());
}

/// Test EDIT workflow execution with plan data
#[tokio::test]
async fn test_edit_workflow_execution() {
    let cursor = CursorController::new();
    let edit_workflow = EditWorkflow::new(&cursor);
    
    let plan_data = serde_json::json!({
        "tasks": [
            {
                "task_id": "test-task-1",
                "title": "Implement Test Function",
                "description": "Add a test function for validation",
                "file_targets": ["src/test_module.rs"],
                "task_type": "Implementation"
            },
            {
                "task_id": "test-task-2", 
                "title": "Add Test Coverage",
                "description": "Create comprehensive tests",
                "file_targets": ["tests/test_coverage.rs"],
                "task_type": "Testing"
            }
        ]
    });
    
    let result = edit_workflow.execute(plan_data).await.unwrap();
    
    // Verify edit result structure
    assert!(result.is_object());
    assert!(result["operations"].is_array());
    assert!(result["files_modified"].is_array());
    assert!(result["success"].is_boolean());
}

/// Test REVIEW workflow execution
#[tokio::test]
async fn test_review_workflow_execution() {
    let temp_dir = TempDir::new().unwrap();
    let base_path = temp_dir.path().to_path_buf();
    
    let llm_config = deskagent::llm::LlmConfig::default();
    let llm = LlmRouter::new(llm_config, "logs").await.unwrap();
    
    let review_workflow = ReviewWorkflow::new(&llm, &base_path);
    let result = review_workflow.execute().await.unwrap();
    
    // Verify review result structure
    assert!(result.is_object());
    assert!(result["review_id"].is_string());
    assert!(result["git_analysis"].is_object());
    assert!(result["code_quality"].is_object());
    assert!(result["test_results"].is_object());
    assert!(result["overall_score"].is_number());
    
    // Verify review file was created
    let review_file = base_path.join("reviews/AI_REVIEW.md");
    assert!(review_file.exists());
}

/// Test full workflow pipeline: PLAN → EDIT → REVIEW
#[tokio::test]
async fn test_full_workflow_pipeline() {
    let temp_dir = TempDir::new().unwrap();
    let base_path = temp_dir.path().to_path_buf();
    
    // Setup workflow manager
    let config = OrchestratorConfig {
        max_concurrent_tasks: 5,
        task_timeout_ms: 30000,
        log_directory: temp_dir.path().join("logs").to_string_lossy().to_string(),
    };
    
    let orchestrator = Orchestrator::new(config).await.unwrap();
    let cursor = CursorController::new();
    let llm_config = deskagent::llm::LlmConfig::default();
    let llm = LlmRouter::new(llm_config, "logs").await.unwrap();
    
    // desktop becomes cursor and terminal
    let terminal = TerminalController::new();
    let mut manager = WorkflowManager::new(orchestrator, cursor, terminal, llm, base_path.clone());
    
    // Create test sprint file
    let sprint_content = r#"
# Integration Test Sprint

## M5. Full Workflow Test

**Goal**: Test complete PLAN → EDIT → REVIEW pipeline

**DoD**:
- [ ] Generate comprehensive task plan
- [ ] Execute edit operations with Desktop Control
- [ ] Produce detailed review analysis

**Tasks**:
- [ ] Parse requirements and create structured plan
- [ ] Insert placeholder implementations in target files
- [ ] Analyze changes and generate review report with recommendations
"#;
    
    let sprint_file = base_path.join("integration-test-sprint.md");
    fs::write(&sprint_file, sprint_content).await.unwrap();
    
    // Execute full workflow
    let results = manager.execute_full_workflow(sprint_file).await.unwrap();
    
    // Verify all three workflows were executed
    assert_eq!(results.len(), 3);
    
    // Verify workflow types and statuses
    let workflow_types: Vec<_> = results.iter().map(|r| &r.workflow_type).collect();
    assert!(workflow_types.contains(&&WorkflowType::Plan));
    assert!(workflow_types.contains(&&WorkflowType::Edit));
    assert!(workflow_types.contains(&&WorkflowType::Review));
    
    // Verify all workflows have completion status
    for result in &results {
        assert!(matches!(result.status, WorkflowStatus::Completed | WorkflowStatus::Failed));
        assert!(result.completed_at.is_some());
    }
    
    // Verify artifacts were created
    let plan_file = base_path.join("plans/sprint-01.plan.json");
    let review_file = base_path.join("reviews/AI_REVIEW.md");
    
    if results[0].status == WorkflowStatus::Completed {
        assert!(plan_file.exists());
    }
    
    if results[2].status == WorkflowStatus::Completed {
        assert!(review_file.exists());
    }
}

/// Test workflow error handling and recovery
#[tokio::test]
async fn test_workflow_error_handling() {
    let temp_dir = TempDir::new().unwrap();
    let base_path = temp_dir.path().to_path_buf();
    
    let config = OrchestratorConfig {
        max_concurrent_tasks: 5,
        task_timeout_ms: 30000,
        log_directory: temp_dir.path().join("logs").to_string_lossy().to_string(),
    };
    
    let orchestrator = Orchestrator::new(config).await.unwrap();
    let cursor = CursorController::new();
    let llm_config = deskagent::llm::LlmConfig::default();
    let llm = LlmRouter::new(llm_config, "logs").await.unwrap();
    
    // desktop becomes cursor and terminal
    let terminal = TerminalController::new();
    let mut manager = WorkflowManager::new(orchestrator, cursor, terminal, llm, base_path.clone());
    
    // Try to execute with non-existent sprint file
    let non_existent_file = base_path.join("does-not-exist.md");
    let result = manager.execute_plan_workflow(non_existent_file).await;
    
    // Should handle error gracefully
    assert!(result.is_ok());
    let workflow_result = result.unwrap();
    assert_eq!(workflow_result.status, WorkflowStatus::Failed);
    assert!(workflow_result.error_message.is_some());
}

/// Test workflow result serialization and deserialization
#[tokio::test]
async fn test_workflow_result_serialization() {
    let temp_dir = TempDir::new().unwrap();
    let config = OrchestratorConfig {
        max_concurrent_tasks: 5,
        task_timeout_ms: 30000,
        log_directory: temp_dir.path().join("logs").to_string_lossy().to_string(),
    };
    
    let orchestrator = Orchestrator::new(config).await.unwrap();
    let cursor = CursorController::new();
    let llm_config = deskagent::llm::LlmConfig::default();
    let llm = LlmRouter::new(llm_config, "logs").await.unwrap();
    let base_path = temp_dir.path().to_path_buf();
    
    // desktop becomes cursor and terminal
    let terminal = TerminalController::new();
    let mut manager = WorkflowManager::new(orchestrator, cursor, terminal, llm, base_path.clone());
    
    // Create simple sprint file
    let sprint_file = base_path.join("simple-sprint.md");
    fs::write(&sprint_file, "# Simple Sprint\n\nTest content").await.unwrap();
    
    let result = manager.execute_plan_workflow(sprint_file).await.unwrap();
    
    // Test serialization
    let serialized = serde_json::to_string(&result);
    assert!(serialized.is_ok());
    
    // Test deserialization
    let deserialized = serde_json::from_str(&serialized.unwrap());
    assert!(deserialized.is_ok());
    
    let deserialized_result: deskagent::workflows::WorkflowResult = deserialized.unwrap();
    assert_eq!(deserialized_result.workflow_type, result.workflow_type);
    assert_eq!(deserialized_result.workflow_id, result.workflow_id);
}

/// Test plan workflow with various content types
#[tokio::test]
async fn test_plan_workflow_content_variations() {
    let temp_dir = TempDir::new().unwrap();
    let base_path = temp_dir.path().to_path_buf();
    
    let llm_config = deskagent::llm::LlmConfig::default();
    let llm = LlmRouter::new(llm_config, "logs").await.unwrap();
    
    let plan_workflow = PlanWorkflow::new(&llm, &base_path);
    
    // Test with minimal content
    let minimal_sprint = base_path.join("minimal-sprint.md");
    fs::write(&minimal_sprint, "# Minimal").await.unwrap();
    
    let result = plan_workflow.execute(minimal_sprint).await.unwrap();
    assert!(result["tasks"].is_array());
    
    // Test with detailed content
    let detailed_content = r#"
# Detailed Sprint Plan

## Requirements
- Implement authentication system
- Add user management
- Create dashboard interface
- Integrate with external APIs

## Technical Specifications
- Use Rust with tokio for async operations
- PostgreSQL database
- REST API with JSON responses
- JWT for authentication
- Rate limiting and caching

## Acceptance Criteria
- All endpoints have >90% test coverage
- Response time <100ms for cached requests
- Support 1000+ concurrent users
- Full documentation with examples
"#;
    
    let detailed_sprint = base_path.join("detailed-sprint.md");
    fs::write(&detailed_sprint, detailed_content).await.unwrap();
    
    let detailed_result = plan_workflow.execute(detailed_sprint).await.unwrap();
    assert!(detailed_result["tasks"].is_array());
    
    // Detailed plan should have more tasks
    let minimal_tasks = result["tasks"].as_array().unwrap().len();
    let detailed_tasks = detailed_result["tasks"].as_array().unwrap().len();
    assert!(detailed_tasks >= minimal_tasks);
}

/// Test edit workflow with different file types
#[tokio::test]
async fn test_edit_workflow_file_types() {
    let cursor = CursorController::new();
    let edit_workflow = EditWorkflow::new(&cursor);
    
    // Test with Rust files
    let rust_plan = serde_json::json!({
        "tasks": [{
            "task_id": "rust-impl",
            "title": "Rust Implementation",
            "description": "Implement core logic in Rust",
            "file_targets": ["src/core.rs", "src/utils.rs"],
            "task_type": "Implementation"
        }]
    });
    
    let rust_result = edit_workflow.execute(rust_plan).await.unwrap();
    assert!(rust_result["success"].as_bool().unwrap_or(false));
    
    // Test with test files
    let test_plan = serde_json::json!({
        "tasks": [{
            "task_id": "test-coverage",
            "title": "Add Test Coverage", 
            "description": "Create comprehensive tests",
            "file_targets": ["tests/integration.rs", "tests/unit.rs"],
            "task_type": "Testing"
        }]
    });
    
    let test_result = edit_workflow.execute(test_plan).await.unwrap();
    assert!(test_result["operations"].is_array());
    
    // Test with documentation files
    let doc_plan = serde_json::json!({
        "tasks": [{
            "task_id": "documentation",
            "title": "Update Documentation",
            "description": "Add API documentation",
            "file_targets": ["docs/api.md", "README.md"],
            "task_type": "Documentation"
        }]
    });
    
    let doc_result = edit_workflow.execute(doc_plan).await.unwrap();
    assert!(doc_result["files_modified"].is_array());
}

/// Test review workflow analysis components
#[tokio::test]
async fn test_review_workflow_analysis() {
    let temp_dir = TempDir::new().unwrap();
    let base_path = temp_dir.path().to_path_buf();
    
    // Create a simple Rust project structure for testing
    fs::create_dir_all(base_path.join("src")).await.unwrap();
    fs::write(
        base_path.join("src/main.rs"),
        r#"fn main() { println!("Hello, world!"); }"#
    ).await.unwrap();
    
    fs::write(
        base_path.join("Cargo.toml"),
        r#"[package]
name = "test-project"
version = "0.1.0"
edition = "2021"

[dependencies]
"#
    ).await.unwrap();
    
    let llm_config = deskagent::llm::LlmConfig::default();
    let llm = LlmRouter::new(llm_config, "logs").await.unwrap();
    
    let review_workflow = ReviewWorkflow::new(&llm, &base_path);
    let result = review_workflow.execute().await.unwrap();
    
    // Verify comprehensive analysis
    let analysis = &result;
    assert!(analysis["git_analysis"].is_object());
    assert!(analysis["code_quality"].is_object());
    assert!(analysis["test_results"].is_object());
    assert!(analysis["llm_analysis"].is_object());
    assert!(analysis["recommendations"].is_array());
    
    // Check specific analysis components
    assert!(analysis["git_analysis"]["files_changed"].is_array());
    assert!(analysis["code_quality"]["compilation_status"].is_object());
    assert!(analysis["test_results"]["total_tests"].is_number());
    assert!(analysis["overall_score"].is_number());
    
    let score = analysis["overall_score"].as_f64().unwrap();
    assert!(score >= 0.0 && score <= 10.0);
}

/// Test workflow manager default constructor for testing
#[tokio::test]
async fn test_workflow_manager_default() {
    let manager = WorkflowManager::new_for_testing().await.unwrap();
    
    // Should be able to create default instance
    assert_eq!(manager.get_base_path(), &PathBuf::from("."));
}

/// Test concurrent workflow execution safety
#[tokio::test]
async fn test_concurrent_workflow_safety() {
    let temp_dir = TempDir::new().unwrap();
    let base_path = temp_dir.path().to_path_buf();
    
    let config = OrchestratorConfig {
        max_concurrent_tasks: 5,
        task_timeout_ms: 30000,
        log_directory: temp_dir.path().join("logs").to_string_lossy().to_string(),
    };
    
    let orchestrator = Orchestrator::new(config).await.unwrap();
    let cursor = CursorController::new();
    let llm_config = deskagent::llm::LlmConfig::default();
    let llm = LlmRouter::new(llm_config, "logs").await.unwrap();
    
    // desktop becomes cursor and terminal
    let terminal = TerminalController::new();
    let mut manager = WorkflowManager::new(orchestrator, cursor, terminal, llm, base_path.clone());
    
    // Create multiple sprint files
    let sprint1 = base_path.join("sprint1.md");
    let sprint2 = base_path.join("sprint2.md");
    
    fs::write(&sprint1, "# Sprint 1\nTest content 1").await.unwrap();
    fs::write(&sprint2, "# Sprint 2\nTest content 2").await.unwrap();
    
    // Execute workflows concurrently
    let handle1 = tokio::spawn({
        let sprint1 = sprint1.clone();
        async move {
            let manager = WorkflowManager::new_for_testing().await.unwrap();
            let mut manager = manager; // Make mutable for the method call
            manager.execute_plan_workflow(sprint1).await
        }
    });
    
    let handle2 = tokio::spawn(async move {
        manager.execute_review_workflow().await
    });
    
    let (result1, result2) = tokio::join!(handle1, handle2);
    
    // Both should complete without conflicts
    assert!(result1.is_ok());
    assert!(result2.is_ok());
    
    let workflow1 = result1.unwrap().unwrap();
    let workflow2 = result2.unwrap().unwrap();
    
    assert!(workflow1.workflow_id != workflow2.workflow_id);
}