pub mod task;
pub mod state;
pub mod logger;

use anyhow::Result;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tokio::sync::{mpsc, RwLock};
use uuid::Uuid;

pub use task::{Task, TaskType, TaskRequest, TaskAction};
pub use state::{TaskState, StateManager};
use logger::EventLogger;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrchestratorConfig {
    pub max_concurrent_tasks: usize,
    pub task_timeout_ms: u64,
    pub log_directory: String,
}

impl Default for OrchestratorConfig {
    fn default() -> Self {
        Self {
            max_concurrent_tasks: 5,
            task_timeout_ms: 30000,
            log_directory: "runs".to_string(),
        }
    }
}

#[derive(Debug)]
pub struct Orchestrator {
    tasks: RwLock<HashMap<Uuid, Task>>,
    state_manager: StateManager,
    event_logger: EventLogger,
    config: OrchestratorConfig,
    task_sender: mpsc::UnboundedSender<TaskRequest>,
    task_receiver: RwLock<Option<mpsc::UnboundedReceiver<TaskRequest>>>,
}

impl Orchestrator {
    pub async fn new(config: OrchestratorConfig) -> Result<Self> {
        let state_manager = StateManager::new();
        let event_logger = EventLogger::new(&config.log_directory).await?;
        let (task_sender, task_receiver) = mpsc::unbounded_channel();
        
        Ok(Self {
            tasks: RwLock::new(HashMap::new()),
            state_manager,
            event_logger,
            config,
            task_sender,
            task_receiver: RwLock::new(Some(task_receiver)),
        })
    }
    
    pub async fn submit_task(&mut self, task_type: TaskType, description: String, payload: serde_json::Value) -> Result<Uuid> {
        let task_id = Uuid::new_v4();
        let task = Task::new(task_id, task_type, description, payload);
        
        // Store task
        {
            let mut tasks = self.tasks.write().await;
            tasks.insert(task_id, task.clone());
        }
        
        // Log task creation
        self.event_logger.log_task_created(&task).await?;
        
        // Send task to processing queue
        let request = TaskRequest {
            task_id,
            action: task::TaskAction::Execute,
        };
        
        self.task_sender.send(request)?;
        
        Ok(task_id)
    }
    
    pub async fn get_task(&self, task_id: &Uuid) -> Option<Task> {
        let tasks = self.tasks.read().await;
        tasks.get(task_id).cloned()
    }
    
    pub async fn get_all_tasks(&self) -> Vec<Task> {
        let tasks = self.tasks.read().await;
        tasks.values().cloned().collect()
    }
    
    pub async fn update_task_state(&mut self, task_id: &Uuid, new_state: TaskState) -> Result<()> {
        {
            let mut tasks = self.tasks.write().await;
            if let Some(task) = tasks.get_mut(task_id) {
                let old_state = task.state.clone();
                task.state = new_state.clone();
                task.updated_at = Utc::now();
                
                // Log state transition
                self.event_logger.log_state_transition(task_id, &old_state, &new_state).await?;
            }
        }
        
        Ok(())
    }
    
    pub async fn start_processing(&self) -> Result<()> {
        let mut receiver = {
            let mut receiver_lock = self.task_receiver.write().await;
            receiver_lock.take().ok_or_else(|| anyhow::anyhow!("Task receiver already taken"))?
        };
        
        tokio::spawn(async move {
            while let Some(request) = receiver.recv().await {
                // Process task request based on action type
                match request.action {
                    TaskAction::Execute => {
                        log::info!("Executing task: {:?}", request.task_id);
                        // Implementation would update task state to Running
                    }
                    TaskAction::Retry => {
                        log::info!("Retrying task: {:?}", request.task_id);
                        // Implementation would retry failed task
                    }
                    TaskAction::Cancel => {
                        log::info!("Cancelling task: {:?}", request.task_id);
                        // Implementation would cancel task
                    }
                    TaskAction::Pause => {
                        log::info!("Pausing task: {:?}", request.task_id);
                        // Implementation would pause task execution
                    }
                    TaskAction::Resume => {
                        log::info!("Resuming task: {:?}", request.task_id);
                        // Implementation would resume paused task
                    }
                }
                log::info!("Task request processed: {:?}", request);
            }
        });
        
        Ok(())
    }

    #[cfg(test)]
    pub fn new_sync(config: OrchestratorConfig) -> Self {
        let state_manager = StateManager::new();
        let event_logger = EventLogger::new_sync(&config.log_directory);
        let (task_sender, task_receiver) = mpsc::unbounded_channel();
        
        Self {
            tasks: RwLock::new(HashMap::new()),
            state_manager,
            event_logger,
            config,
            task_sender,
            task_receiver: RwLock::new(Some(task_receiver)),
        }
    }
}