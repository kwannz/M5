use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tokio::fs::{self, OpenOptions};
use tokio::io::AsyncWriteExt;
use uuid::Uuid;

use super::task::Task;
use super::state::TaskState;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskEvent {
    pub event_id: Uuid,
    pub task_id: Uuid,
    pub event_type: EventType,
    pub timestamp: DateTime<Utc>,
    pub details: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EventType {
    TaskCreated,
    TaskStarted,
    TaskCompleted,
    TaskFailed,
    TaskCancelled,
    TaskRetried,
    StateTransition,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunSession {
    pub session_id: Uuid,
    pub start_time: DateTime<Utc>,
    pub end_time: Option<DateTime<Utc>>,
    pub events: Vec<TaskEvent>,
}

#[derive(Debug)]
pub struct EventLogger {
    base_dir: PathBuf,
    current_session: RunSession,
}

impl EventLogger {
    pub async fn new<P: Into<PathBuf>>(base_dir: P) -> Result<Self> {
        let base_dir = base_dir.into();
        fs::create_dir_all(&base_dir).await?;
        
        let session_id = Uuid::new_v4();
        let current_session = RunSession {
            session_id,
            start_time: Utc::now(),
            end_time: None,
            events: Vec::new(),
        };
        
        let logger = Self {
            base_dir,
            current_session,
        };
        
        // Create session directory
        logger.create_session_directory().await?;
        
        Ok(logger)
    }
    
    #[cfg(test)]
    pub fn new_sync<P: Into<PathBuf>>(base_dir: P) -> Self {
        let base_dir = base_dir.into();
        
        let session_id = Uuid::new_v4();
        let current_session = RunSession {
            session_id,
            start_time: Utc::now(),
            end_time: None,
            events: Vec::new(),
        };
        
        Self {
            base_dir,
            current_session,
        }
    }
    
    async fn create_session_directory(&self) -> Result<()> {
        let session_dir = self.get_session_directory();
        fs::create_dir_all(&session_dir).await?;
        Ok(())
    }
    
    fn get_session_directory(&self) -> PathBuf {
        let timestamp = self.current_session.start_time.format("%Y%m%d_%H%M%S");
        self.base_dir.join(format!("{}_{}", timestamp, &self.current_session.session_id.to_string()[..8]))
    }
    
    async fn log_event(&mut self, event: TaskEvent) -> Result<()> {
        // Add event to current session
        self.current_session.events.push(event.clone());
        
        // Write event to log file
        let session_dir = self.get_session_directory();
        let log_file = session_dir.join("events.jsonl");
        
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&log_file)
            .await?;
        
        let event_line = serde_json::to_string(&event)?;
        file.write_all(format!("{}\n", event_line).as_bytes()).await?;
        file.flush().await?;
        
        Ok(())
    }
    
    pub async fn log_task_created(&mut self, task: &Task) -> Result<()> {
        let event = TaskEvent {
            event_id: Uuid::new_v4(),
            task_id: task.id,
            event_type: EventType::TaskCreated,
            timestamp: Utc::now(),
            details: serde_json::json!({
                "task_type": task.task_type,
                "description": task.description,
                "payload": task.payload
            }),
        };
        
        self.log_event(event).await
    }
    
    pub async fn log_task_started(&mut self, task_id: &Uuid) -> Result<()> {
        let event = TaskEvent {
            event_id: Uuid::new_v4(),
            task_id: *task_id,
            event_type: EventType::TaskStarted,
            timestamp: Utc::now(),
            details: serde_json::json!({}),
        };
        
        self.log_event(event).await
    }
    
    pub async fn log_task_completed(&mut self, task_id: &Uuid, result: &serde_json::Value) -> Result<()> {
        let event = TaskEvent {
            event_id: Uuid::new_v4(),
            task_id: *task_id,
            event_type: EventType::TaskCompleted,
            timestamp: Utc::now(),
            details: serde_json::json!({
                "result": result
            }),
        };
        
        self.log_event(event).await
    }
    
    pub async fn log_task_failed(&mut self, task_id: &Uuid, error: &str) -> Result<()> {
        let event = TaskEvent {
            event_id: Uuid::new_v4(),
            task_id: *task_id,
            event_type: EventType::TaskFailed,
            timestamp: Utc::now(),
            details: serde_json::json!({
                "error": error
            }),
        };
        
        self.log_event(event).await
    }
    
    pub async fn log_state_transition(&mut self, task_id: &Uuid, from: &TaskState, to: &TaskState) -> Result<()> {
        let event = TaskEvent {
            event_id: Uuid::new_v4(),
            task_id: *task_id,
            event_type: EventType::StateTransition,
            timestamp: Utc::now(),
            details: serde_json::json!({
                "from_state": from,
                "to_state": to
            }),
        };
        
        self.log_event(event).await
    }
    
    pub async fn finalize_session(&mut self) -> Result<()> {
        self.current_session.end_time = Some(Utc::now());
        
        // Write session summary
        let session_dir = self.get_session_directory();
        let summary_file = session_dir.join("run.json");
        
        let summary = serde_json::to_string_pretty(&self.current_session)?;
        fs::write(&summary_file, summary).await?;
        
        Ok(())
    }
    
    pub fn get_session_id(&self) -> Uuid {
        self.current_session.session_id
    }
    
    pub fn get_events(&self) -> &[TaskEvent] {
        &self.current_session.events
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use crate::orchestrator::task::{Task, TaskType};
    
    #[tokio::test]
    async fn test_event_logger_creation() {
        let temp_dir = TempDir::new().unwrap();
        let logger = EventLogger::new(temp_dir.path()).await.unwrap();
        
        // Check session directory was created
        let session_dir = logger.get_session_directory();
        assert!(session_dir.exists());
    }
    
    #[tokio::test]
    async fn test_task_event_logging() {
        let temp_dir = TempDir::new().unwrap();
        let mut logger = EventLogger::new(temp_dir.path()).await.unwrap();
        
        // Create a test task
        let task = Task::new(
            Uuid::new_v4(),
            TaskType::Plan,
            "Test task".to_string(),
            serde_json::json!({"test": "data"}),
        );
        
        // Log task creation
        logger.log_task_created(&task).await.unwrap();
        
        // Check event was recorded
        assert_eq!(logger.get_events().len(), 1);
        assert_eq!(logger.get_events()[0].task_id, task.id);
        assert!(matches!(logger.get_events()[0].event_type, EventType::TaskCreated));
        
        // Check event file was created
        let session_dir = logger.get_session_directory();
        let log_file = session_dir.join("events.jsonl");
        assert!(log_file.exists());
    }
    
    #[tokio::test]
    async fn test_session_finalization() {
        let temp_dir = TempDir::new().unwrap();
        let mut logger = EventLogger::new(temp_dir.path()).await.unwrap();
        
        // Create and log a task
        let task = Task::new(
            Uuid::new_v4(),
            TaskType::Plan,
            "Test task".to_string(),
            serde_json::json!({}),
        );
        logger.log_task_created(&task).await.unwrap();
        
        // Finalize session
        logger.finalize_session().await.unwrap();
        
        // Check summary file was created
        let session_dir = logger.get_session_directory();
        let summary_file = session_dir.join("run.json");
        assert!(summary_file.exists());
        
        // Check end time was set
        assert!(logger.current_session.end_time.is_some());
    }
}