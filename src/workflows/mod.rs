pub mod plan;
pub mod edit;
pub mod review;

use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use uuid::Uuid;

pub use plan::PlanWorkflow;
pub use edit::EditWorkflow;
pub use review::ReviewWorkflow;

use crate::orchestrator::Orchestrator;
use crate::desktop::{CursorController, TerminalController};
use crate::llm::LlmRouter;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum WorkflowType {
    Plan,
    Edit,
    Review,
    Apply,
    Followup,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum WorkflowStatus {
    Pending,
    Running,
    Completed,
    Failed,
    Cancelled,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowResult {
    pub workflow_id: Uuid,
    pub workflow_type: WorkflowType,
    pub status: WorkflowStatus,
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub input_data: serde_json::Value,
    pub output_data: Option<serde_json::Value>,
    pub error_message: Option<String>,
    pub artifacts: Vec<PathBuf>,
}

pub struct WorkflowManager {
    orchestrator: Orchestrator,
    cursor: CursorController,
    terminal: TerminalController,
    llm: LlmRouter,
    base_path: PathBuf,
}

impl WorkflowManager {
    pub fn new(orchestrator: Orchestrator, cursor: CursorController, terminal: TerminalController, llm: LlmRouter, base_path: PathBuf) -> Self {
        Self {
            orchestrator,
            cursor,
            terminal,
            llm,
            base_path,
        }
    }

    pub async fn execute_plan_workflow(&mut self, sprint_file: PathBuf) -> Result<WorkflowResult> {
        let workflow_id = Uuid::new_v4();
        let mut result = WorkflowResult {
            workflow_id,
            workflow_type: WorkflowType::Plan,
            status: WorkflowStatus::Running,
            started_at: Utc::now(),
            completed_at: None,
            input_data: serde_json::json!({"sprint_file": sprint_file}),
            output_data: None,
            error_message: None,
            artifacts: Vec::new(),
        };

        let plan_workflow = PlanWorkflow::new(&self.llm, &self.base_path);
        
        match plan_workflow.execute(sprint_file).await {
            Ok(plan_data) => {
                result.status = WorkflowStatus::Completed;
                result.completed_at = Some(Utc::now());
                result.output_data = Some(plan_data.clone());
                result.artifacts.push(self.base_path.join("plans/sprint-01.plan.json"));
            }
            Err(e) => {
                result.status = WorkflowStatus::Failed;
                result.completed_at = Some(Utc::now());
                result.error_message = Some(e.to_string());
            }
        }

        Ok(result)
    }

    pub async fn execute_edit_workflow(&mut self, plan_data: serde_json::Value) -> Result<WorkflowResult> {
        let workflow_id = Uuid::new_v4();
        let mut result = WorkflowResult {
            workflow_id,
            workflow_type: WorkflowType::Edit,
            status: WorkflowStatus::Running,
            started_at: Utc::now(),
            completed_at: None,
            input_data: plan_data.clone(),
            output_data: None,
            error_message: None,
            artifacts: Vec::new(),
        };

        let edit_workflow = EditWorkflow::new(&self.cursor);
        
        match edit_workflow.execute(plan_data).await {
            Ok(edit_data) => {
                result.status = WorkflowStatus::Completed;
                result.completed_at = Some(Utc::now());
                result.output_data = Some(edit_data);
            }
            Err(e) => {
                result.status = WorkflowStatus::Failed;
                result.completed_at = Some(Utc::now());
                result.error_message = Some(e.to_string());
            }
        }

        Ok(result)
    }

    pub async fn execute_review_workflow(&mut self) -> Result<WorkflowResult> {
        let workflow_id = Uuid::new_v4();
        let mut result = WorkflowResult {
            workflow_id,
            workflow_type: WorkflowType::Review,
            status: WorkflowStatus::Running,
            started_at: Utc::now(),
            completed_at: None,
            input_data: serde_json::json!({}),
            output_data: None,
            error_message: None,
            artifacts: Vec::new(),
        };

        let review_workflow = ReviewWorkflow::new(&self.llm, &self.base_path);
        
        match review_workflow.execute().await {
            Ok(review_data) => {
                result.status = WorkflowStatus::Completed;
                result.completed_at = Some(Utc::now());
                result.output_data = Some(review_data);
                result.artifacts.push(self.base_path.join("reviews/AI_REVIEW.md"));
            }
            Err(e) => {
                result.status = WorkflowStatus::Failed;
                result.completed_at = Some(Utc::now());
                result.error_message = Some(e.to_string());
            }
        }

        Ok(result)
    }

    pub async fn execute_full_workflow(&mut self, sprint_file: PathBuf) -> Result<Vec<WorkflowResult>> {
        let mut results = Vec::new();

        // Execute PLAN workflow
        let plan_result = self.execute_plan_workflow(sprint_file).await?;
        let plan_data = plan_result.output_data.clone();
        results.push(plan_result);

        // Execute EDIT workflow if plan succeeded
        if let Some(plan_data) = plan_data {
            let edit_result = self.execute_edit_workflow(plan_data).await?;
            results.push(edit_result);
        }

        // Execute REVIEW workflow
        let review_result = self.execute_review_workflow().await?;
        results.push(review_result);

        Ok(results)
    }

    pub fn get_base_path(&self) -> &PathBuf {
        &self.base_path
    }
}

impl Default for WorkflowManager {
    fn default() -> Self {
        // This is a placeholder implementation for testing
        // In production, proper instances should be passed via new()
        use crate::orchestrator::OrchestratorConfig;
        use std::sync::Arc;
        use tokio::runtime::Handle;

        // Create minimal instances for testing
        let config = OrchestratorConfig::default();
        let orchestrator = Handle::current().block_on(async {
            Orchestrator::new(config).await.unwrap()
        });
        
        let cursor = CursorController::new();
        let terminal = TerminalController::new();
        
        // Create a basic LlmConfig for testing
        let llm_config = crate::llm::LlmConfig::default();
        let llm = Handle::current().block_on(async {
            LlmRouter::new(llm_config, "logs").await.unwrap()
        });
        
        Self::new(orchestrator, cursor, terminal, llm, PathBuf::from("."))
    }
}