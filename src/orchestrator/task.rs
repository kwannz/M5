use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::state::TaskState;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TaskType {
    Plan,
    Review,
    Status,
    Followup,
    Apply,
}

impl std::fmt::Display for TaskType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TaskType::Plan => write!(f, "PLAN"),
            TaskType::Review => write!(f, "REVIEW"),
            TaskType::Status => write!(f, "STATUS"),
            TaskType::Followup => write!(f, "FOLLOWUP"),
            TaskType::Apply => write!(f, "APPLY"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub id: Uuid,
    pub task_type: TaskType,
    pub description: String,
    pub payload: serde_json::Value,
    pub state: TaskState,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub retry_count: u32,
    pub error_message: Option<String>,
    pub result: Option<serde_json::Value>,
}

impl Task {
    pub fn new(id: Uuid, task_type: TaskType, description: String, payload: serde_json::Value) -> Self {
        let now = Utc::now();
        
        Self {
            id,
            task_type,
            description,
            payload,
            state: TaskState::Pending,
            created_at: now,
            updated_at: now,
            started_at: None,
            completed_at: None,
            retry_count: 0,
            error_message: None,
            result: None,
        }
    }
    
    pub fn start(&mut self) {
        self.state = TaskState::Running;
        self.started_at = Some(Utc::now());
        self.updated_at = Utc::now();
    }
    
    pub fn complete(&mut self, result: serde_json::Value) {
        self.state = TaskState::Completed;
        self.result = Some(result);
        self.completed_at = Some(Utc::now());
        self.updated_at = Utc::now();
    }
    
    pub fn fail(&mut self, error: String) {
        self.state = TaskState::Failed;
        self.error_message = Some(error);
        self.updated_at = Utc::now();
    }
    
    pub fn retry(&mut self) -> bool {
        self.retry_count += 1;
        self.state = TaskState::Pending;
        self.updated_at = Utc::now();
        self.retry_count <= 3 // Max 3 retries
    }
    
    pub fn can_retry(&self) -> bool {
        matches!(self.state, TaskState::Failed) && self.retry_count < 3
    }
    
    pub fn duration(&self) -> Option<chrono::Duration> {
        match (self.started_at, self.completed_at) {
            (Some(start), Some(end)) => Some(end - start),
            _ => None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct TaskRequest {
    pub task_id: Uuid,
    pub action: TaskAction,
}

#[derive(Debug, Clone)]
pub enum TaskAction {
    Execute,
    Retry,
    Cancel,
    Pause,
    Resume,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    
    #[test]
    fn test_task_creation() {
        let id = Uuid::new_v4();
        let task = Task::new(
            id,
            TaskType::Plan,
            "Test task".to_string(),
            json!({"test": "data"}),
        );
        
        assert_eq!(task.id, id);
        assert_eq!(task.task_type, TaskType::Plan);
        assert_eq!(task.description, "Test task");
        assert_eq!(task.state, TaskState::Pending);
        assert_eq!(task.retry_count, 0);
    }
    
    #[test]
    fn test_task_lifecycle() {
        let id = Uuid::new_v4();
        let mut task = Task::new(
            id,
            TaskType::Plan,
            "Test task".to_string(),
            json!({}),
        );
        
        // Start task
        task.start();
        assert_eq!(task.state, TaskState::Running);
        assert!(task.started_at.is_some());
        
        // Complete task
        let result = json!({"status": "success"});
        task.complete(result.clone());
        assert_eq!(task.state, TaskState::Completed);
        assert_eq!(task.result, Some(result));
        assert!(task.completed_at.is_some());
    }
    
    #[test]
    fn test_task_retry_logic() {
        let id = Uuid::new_v4();
        let mut task = Task::new(
            id,
            TaskType::Plan,
            "Test task".to_string(),
            json!({}),
        );
        
        // Fail task
        task.fail("Test error".to_string());
        assert_eq!(task.state, TaskState::Failed);
        assert_eq!(task.error_message, Some("Test error".to_string()));
        assert!(task.can_retry());
        
        // Retry task
        assert!(task.retry());
        assert_eq!(task.state, TaskState::Pending);
        assert_eq!(task.retry_count, 1);
        
        // Fail and retry multiple times
        task.fail("Another error".to_string());
        assert!(task.retry()); // retry 2
        task.fail("Third error".to_string());
        assert!(task.retry()); // retry 3
        task.fail("Final error".to_string());
        assert!(!task.retry()); // Should not retry after 3 attempts
        assert!(!task.can_retry());
    }
}