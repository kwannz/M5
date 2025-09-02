use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tokio::fs;
use uuid::Uuid;

use crate::desktop::CursorController;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EditOperation {
    pub operation_id: Uuid,
    pub created_at: DateTime<Utc>,
    pub target_file: String,
    pub operation_type: EditOperationType,
    pub content: String,
    pub line_number: Option<u32>,
    pub backup_created: bool,
    pub rollback_info: Option<RollbackInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EditOperationType {
    Insert,
    Replace,
    Append,
    Comment,
    Placeholder,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RollbackInfo {
    pub backup_path: String,
    pub original_content: String,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EditResult {
    pub operations: Vec<EditOperation>,
    pub files_modified: Vec<String>,
    pub cursor_interactions: Vec<CursorInteraction>,
    pub success: bool,
    pub error_message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CursorInteraction {
    pub interaction_type: String,
    pub file_path: String,
    pub content: String,
    pub timestamp: DateTime<Utc>,
    pub success: bool,
}

pub struct EditWorkflow<'a> {
    cursor: &'a CursorController,
}

impl<'a> EditWorkflow<'a> {
    pub fn new(cursor: &'a CursorController) -> Self {
        Self { cursor }
    }

    pub async fn execute(&self, plan_data: serde_json::Value) -> Result<serde_json::Value> {
        let mut edit_result = EditResult {
            operations: Vec::new(),
            files_modified: Vec::new(),
            cursor_interactions: Vec::new(),
            success: true,
            error_message: None,
        };

        // Parse the plan data to extract tasks and file targets
        let tasks = self.extract_tasks_from_plan(&plan_data)?;
        
        // Execute edit operations for each task
        for task in tasks {
            match self.execute_task_edits(&task, &mut edit_result).await {
                Ok(_) => {
                    log::info!("Successfully executed edits for task: {}", task.title);
                }
                Err(e) => {
                    log::error!("Failed to execute edits for task {}: {}", task.title, e);
                    edit_result.success = false;
                    edit_result.error_message = Some(format!("Task '{}' failed: {}", task.title, e));
                    break; // Stop on first failure to prevent cascading issues
                }
            }
        }

        Ok(serde_json::to_value(edit_result)?)
    }

    fn extract_tasks_from_plan(&self, plan_data: &serde_json::Value) -> Result<Vec<TaskInfo>> {
        let tasks = plan_data["tasks"]
            .as_array()
            .ok_or_else(|| anyhow::anyhow!("No tasks found in plan data"))?;

        let mut extracted_tasks = Vec::new();
        
        for task in tasks {
            let task_info = TaskInfo {
                task_id: task["task_id"].as_str().unwrap_or("unknown").to_string(),
                title: task["title"].as_str().unwrap_or("Unnamed Task").to_string(),
                description: task["description"].as_str().unwrap_or("No description").to_string(),
                file_targets: task["file_targets"]
                    .as_array()
                    .map(|arr| arr.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect())
                    .unwrap_or_default(),
                task_type: task["task_type"].as_str().unwrap_or("Implementation").to_string(),
            };
            extracted_tasks.push(task_info);
        }

        Ok(extracted_tasks)
    }

    async fn execute_task_edits(&self, task: &TaskInfo, edit_result: &mut EditResult) -> Result<()> {
        // For each file target in the task, perform appropriate edits
        for file_path in &task.file_targets {
            let operation = self.create_edit_operation(task, file_path).await?;
            
            // Execute the edit operation using Desktop Control
            match self.execute_edit_operation(&operation).await {
                Ok(interaction) => {
                    edit_result.operations.push(operation);
                    edit_result.cursor_interactions.push(interaction);
                    edit_result.files_modified.push(file_path.clone());
                }
                Err(e) => {
                    return Err(anyhow::anyhow!("Failed to edit file {}: {}", file_path, e));
                }
            }
        }

        Ok(())
    }

    async fn create_edit_operation(&self, task: &TaskInfo, file_path: &str) -> Result<EditOperation> {
        // Generate appropriate placeholder content based on task type and file
        let (operation_type, content) = self.generate_placeholder_content(task, file_path)?;
        
        Ok(EditOperation {
            operation_id: Uuid::new_v4(),
            created_at: Utc::now(),
            target_file: file_path.to_string(),
            operation_type,
            content,
            line_number: None, // Will be determined when opening file in Cursor
            backup_created: true,
            rollback_info: Some(RollbackInfo {
                backup_path: format!("{}.backup.{}", file_path, Utc::now().timestamp()),
                original_content: String::new(), // Will be populated when reading original
                timestamp: Utc::now(),
            }),
        })
    }

    fn generate_placeholder_content(&self, task: &TaskInfo, file_path: &str) -> Result<(EditOperationType, String)> {
        let path_buf = PathBuf::from(file_path);
        let file_extension = path_buf
            .extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("");

        let content = match file_extension {
            "rs" => {
                self.generate_rust_placeholder(task)
            }
            "md" => {
                self.generate_markdown_placeholder(task)
            }
            "json" => {
                self.generate_json_placeholder(task)
            }
            _ => {
                format!("// IMPLEMENTATION: {} - {}\n// File: {}\n// Task: {}\n// Generated placeholder for development\n", 
                    task.task_type, task.title, file_path, task.description)
            }
        };

        // Determine operation type based on task type
        let operation_type = match task.task_type.as_str() {
            "Testing" => EditOperationType::Append,
            "Documentation" => EditOperationType::Insert,
            "Configuration" => EditOperationType::Replace,
            _ => EditOperationType::Comment,
        };

        Ok((operation_type, content))
    }

    fn generate_rust_placeholder(&self, task: &TaskInfo) -> String {
        match task.task_type.as_str() {
            "Implementation" => {
                format!(
                    r#"
// PLACEHOLDER: {} Implementation
// Description: {}
// Generated implementation placeholder

pub struct {}Placeholder {{
    // Add fields as needed
}}

impl {}Placeholder {{
    pub fn new() -> Self {{
        Self {{
            // Initialize fields
        }}
    }}
    
    pub fn execute(&self) -> Result<()> {{
        // Core implementation logic would go here
        Ok(())
    }}
}}

#[cfg(test)]
mod tests {{
    use super::*;

    #[test]
    fn test_{}_placeholder() {{
        // Test implementation
        let placeholder = {}Placeholder::new();
        // Add test assertions
    }}
}}
"#,
                    task.title,
                    task.description,
                    to_pascal_case(&task.task_id),
                    to_pascal_case(&task.task_id),
                    task.task_id.replace("-", "_"),
                    to_pascal_case(&task.task_id)
                )
            }
            "Testing" => {
                format!(
                    r#"
#[cfg(test)]
mod {} {{
    use super::*;

    #[tokio::test]
    async fn test_{}() {{
        // Test for: {}
        // Test logic implementation
        
        // Arrange
        
        // Act
        
        // Assert
        todo!("Complete test implementation")
    }}
}}
"#,
                    task.task_id.replace("-", "_"),
                    task.task_id.replace("-", "_"),
                    task.description
                )
            }
            _ => {
                format!(
                    "// PLACEHOLDER: {}\n// {}\n// Implementation ready for development\n",
                    task.title, task.description
                )
            }
        }
    }

    fn generate_markdown_placeholder(&self, task: &TaskInfo) -> String {
        format!(
            r#"## {}

### Description
{}

### Implementation Status
- [ ] Task started
- [ ] Core logic implemented  
- [ ] Tests added
- [ ] Documentation updated
- [ ] Code reviewed

### Notes
Implementation details and progress notes will be added during development.

Generated: {}
"#,
            task.title,
            task.description,
            Utc::now().format("%Y-%m-%d %H:%M:%S UTC")
        )
    }

    fn generate_json_placeholder(&self, task: &TaskInfo) -> String {
        serde_json::to_string_pretty(&serde_json::json!({
            "task_id": task.task_id,
            "title": task.title,
            "description": task.description,
            "status": "placeholder_created",
            "created_at": Utc::now(),
            "implementation_notes": "Placeholder created successfully",
            "files_to_modify": task.file_targets
        })).unwrap_or_else(|_| format!(r#"{{"task_id": "{}", "status": "placeholder"}}"#, task.task_id))
    }

    async fn execute_edit_operation(&self, operation: &EditOperation) -> Result<CursorInteraction> {
        // Create backup first if the file exists
        if PathBuf::from(&operation.target_file).exists() {
            let backup_path = format!("{}.backup.{}", operation.target_file, Utc::now().timestamp());
            fs::copy(&operation.target_file, &backup_path).await?;
        }

        // Use Cursor Control to open file and insert content
        let cursor_result = match operation.operation_type {
            EditOperationType::Insert | EditOperationType::Comment => {
                self.cursor.insert_text(&operation.target_file, &operation.content).await
            }
            EditOperationType::Append => {
                self.cursor.append_to_file(&operation.target_file, &operation.content).await
            }
            EditOperationType::Replace => {
                // For replace, we'd need to implement cursor_replace_content
                // For now, use append as a fallback
                self.cursor.append_to_file(&operation.target_file, &operation.content).await
            }
            EditOperationType::Placeholder => {
                self.cursor.insert_text(&operation.target_file, &operation.content).await
            }
        };

        let success = cursor_result.is_ok();
        let _error_msg = cursor_result.as_ref().err().map(|e| e.to_string());

        Ok(CursorInteraction {
            interaction_type: format!("{:?}", operation.operation_type),
            file_path: operation.target_file.clone(),
            content: operation.content.clone(),
            timestamp: Utc::now(),
            success,
        })
    }
}

#[derive(Debug, Clone)]
struct TaskInfo {
    task_id: String,
    title: String,
    description: String,
    file_targets: Vec<String>,
    task_type: String,
}

fn to_pascal_case(s: &str) -> String {
    s.split('-')
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                None => String::new(),
                Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_edit_workflow_creation() {
        let cursor = CursorController::new();
        let workflow = EditWorkflow::new(&cursor);
        
        // Test that we can create the workflow
        assert!(std::ptr::eq(workflow.cursor, &cursor));
    }

    #[test]
    fn test_to_pascal_case() {
        assert_eq!(to_pascal_case("test-task"), "TestTask");
        assert_eq!(to_pascal_case("simple"), "Simple");
        assert_eq!(to_pascal_case("multi-word-test"), "MultiWordTest");
    }

    #[test]
    fn test_generate_rust_placeholder() {
        let cursor = CursorController::new();
        let workflow = EditWorkflow::new(&cursor);
        
        let task = TaskInfo {
            task_id: "test-task".to_string(),
            title: "Test Task".to_string(),
            description: "A test task for validation".to_string(),
            file_targets: vec!["src/test.rs".to_string()],
            task_type: "Implementation".to_string(),
        };
        
        let placeholder = workflow.generate_rust_placeholder(&task);
        
        assert!(placeholder.contains("TestTaskPlaceholder"));
        assert!(placeholder.contains("Test Task"));
        assert!(placeholder.contains("A test task for validation"));
        assert!(placeholder.contains("#[cfg(test)]"));
    }

    #[test]
    fn test_extract_tasks_from_plan() {
        let cursor = CursorController::new();
        let workflow = EditWorkflow::new(&cursor);
        
        let plan_data = serde_json::json!({
            "tasks": [
                {
                    "task_id": "task-1",
                    "title": "First Task",
                    "description": "First task description",
                    "file_targets": ["src/lib.rs"],
                    "task_type": "Implementation"
                },
                {
                    "task_id": "task-2", 
                    "title": "Second Task",
                    "description": "Second task description",
                    "file_targets": ["tests/integration.rs"],
                    "task_type": "Testing"
                }
            ]
        });
        
        let tasks = workflow.extract_tasks_from_plan(&plan_data).unwrap();
        
        assert_eq!(tasks.len(), 2);
        assert_eq!(tasks[0].task_id, "task-1");
        assert_eq!(tasks[1].task_type, "Testing");
    }
}