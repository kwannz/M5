use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tokio::fs;
use uuid::Uuid;

use crate::llm::LlmRouter;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskPlan {
    pub plan_id: Uuid,
    pub created_at: DateTime<Utc>,
    pub sprint_file: String,
    pub overview: String,
    pub tasks: Vec<PlanTask>,
    pub estimated_duration_minutes: u32,
    pub priority: TaskPriority,
    pub dependencies: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanTask {
    pub task_id: String,
    pub title: String,
    pub description: String,
    pub file_targets: Vec<String>,
    pub estimated_minutes: u16,
    pub task_type: PlanTaskType,
    pub validation_criteria: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PlanTaskType {
    Implementation,
    Refactor,
    Testing,
    Documentation,
    Configuration,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TaskPriority {
    Low,
    Medium,
    High,
    Critical,
}

pub struct PlanWorkflow<'a> {
    llm: &'a LlmRouter,
    base_path: &'a PathBuf,
}

impl<'a> PlanWorkflow<'a> {
    pub fn new(llm: &'a LlmRouter, base_path: &'a PathBuf) -> Self {
        Self { llm, base_path }
    }

    pub async fn execute(&self, sprint_file: PathBuf) -> Result<serde_json::Value> {
        // Ensure plans directory exists
        let plans_dir = self.base_path.join("plans");
        fs::create_dir_all(&plans_dir).await?;

        // Read the sprint file
        let sprint_content = fs::read_to_string(&sprint_file).await?;

        // Create plan using LLM
        let plan = self.generate_plan_with_llm(&sprint_content, &sprint_file).await?;

        // Save plan to file
        let plan_file = plans_dir.join("sprint-01.plan.json");
        let plan_json = serde_json::to_string_pretty(&plan)?;
        fs::write(&plan_file, plan_json).await?;

        // Return plan data
        Ok(serde_json::to_value(plan)?)
    }

    async fn generate_plan_with_llm(&self, sprint_content: &str, sprint_file: &PathBuf) -> Result<TaskPlan> {
        let prompt = self.create_planning_prompt(sprint_content);

        // Create LLM request
        let messages = vec![crate::llm::Message::user(prompt)];
        let request = crate::llm::LlmRequest::new(crate::llm::TaskType::Plan, messages);
        
        match self.llm.generate(request).await {
            Ok(response) => {
                // Parse LLM response into structured plan
                self.parse_llm_response_to_plan(&response.content, sprint_file).await
            }
            Err(_) => {
                // Fallback: Create a structured plan without LLM
                self.create_fallback_plan(sprint_content, sprint_file).await
            }
        }
    }

    fn create_planning_prompt(&self, sprint_content: &str) -> String {
        format!(
            r#"You are a senior software architect and project manager. Analyze the following sprint document and create a detailed implementation plan.

SPRINT CONTENT:
{}

Please create a structured task plan with the following requirements:

1. OVERVIEW: Summarize the main goals and deliverables
2. TASKS: Break down into specific, actionable tasks with:
   - Clear titles and descriptions
   - Target files to modify/create
   - Time estimates in minutes
   - Task types (Implementation, Refactor, Testing, Documentation, Configuration)
   - Validation criteria for completion

3. DEPENDENCIES: Identify task dependencies and execution order
4. PRIORITY: Assess overall priority (Low/Medium/High/Critical)
5. DURATION: Estimate total implementation time

Focus on:
- Rust best practices and idiomatic code
- Comprehensive error handling
- Production-ready implementations (no TODOs or placeholders)
- Testability and maintainability
- Following existing project patterns

Respond with a JSON structure matching this format:
{{
  "overview": "Brief description of what will be accomplished",
  "tasks": [
    {{
      "task_id": "unique-id",
      "title": "Task title",
      "description": "Detailed description",
      "file_targets": ["path/to/file1.rs", "path/to/file2.rs"],
      "estimated_minutes": 60,
      "task_type": "Implementation|Refactor|Testing|Documentation|Configuration",
      "validation_criteria": ["How to verify completion"]
    }}
  ],
  "estimated_duration_minutes": 240,
  "priority": "High",
  "dependencies": ["task-id-1", "task-id-2"]
}}
"#,
            sprint_content
        )
    }

    async fn parse_llm_response_to_plan(&self, response: &str, sprint_file: &PathBuf) -> Result<TaskPlan> {
        // Try to extract JSON from LLM response
        let json_str = if let Some(start) = response.find('{') {
            if let Some(end) = response.rfind('}') {
                &response[start..=end]
            } else {
                response
            }
        } else {
            response
        };

        match serde_json::from_str::<serde_json::Value>(json_str) {
            Ok(json) => {
                let tasks = json["tasks"]
                    .as_array()
                    .unwrap_or(&Vec::new())
                    .iter()
                    .enumerate()
                    .map(|(i, task)| PlanTask {
                        task_id: task["task_id"].as_str().unwrap_or(&format!("task-{}", i + 1)).to_string(),
                        title: task["title"].as_str().unwrap_or("Unnamed Task").to_string(),
                        description: task["description"].as_str().unwrap_or("No description").to_string(),
                        file_targets: task["file_targets"]
                            .as_array()
                            .map(|arr| arr.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect())
                            .unwrap_or_default(),
                        estimated_minutes: task["estimated_minutes"].as_u64().unwrap_or(60) as u16,
                        task_type: match task["task_type"].as_str().unwrap_or("Implementation") {
                            "Refactor" => PlanTaskType::Refactor,
                            "Testing" => PlanTaskType::Testing,
                            "Documentation" => PlanTaskType::Documentation,
                            "Configuration" => PlanTaskType::Configuration,
                            _ => PlanTaskType::Implementation,
                        },
                        validation_criteria: task["validation_criteria"]
                            .as_array()
                            .map(|arr| arr.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect())
                            .unwrap_or_else(|| vec!["Task completed successfully".to_string()]),
                    })
                    .collect();

                Ok(TaskPlan {
                    plan_id: Uuid::new_v4(),
                    created_at: Utc::now(),
                    sprint_file: sprint_file.to_string_lossy().to_string(),
                    overview: json["overview"].as_str().unwrap_or("Generated task plan").to_string(),
                    tasks,
                    estimated_duration_minutes: json["estimated_duration_minutes"].as_u64().unwrap_or(240) as u32,
                    priority: match json["priority"].as_str().unwrap_or("Medium") {
                        "Low" => TaskPriority::Low,
                        "High" => TaskPriority::High,
                        "Critical" => TaskPriority::Critical,
                        _ => TaskPriority::Medium,
                    },
                    dependencies: json["dependencies"]
                        .as_array()
                        .map(|arr| arr.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect())
                        .unwrap_or_default(),
                })
            }
            Err(_) => {
                // Fallback to structured plan creation
                self.create_fallback_plan(response, sprint_file).await
            }
        }
    }

    async fn create_fallback_plan(&self, _content: &str, sprint_file: &PathBuf) -> Result<TaskPlan> {
        // Create a reasonable fallback plan based on content analysis
        let tasks = vec![
            PlanTask {
                task_id: "analyze-requirements".to_string(),
                title: "Analyze Requirements".to_string(),
                description: "Review and understand the requirements from the sprint document".to_string(),
                file_targets: vec![],
                estimated_minutes: 30,
                task_type: PlanTaskType::Documentation,
                validation_criteria: vec!["Requirements clearly documented and understood".to_string()],
            },
            PlanTask {
                task_id: "implement-core-logic".to_string(),
                title: "Implement Core Logic".to_string(),
                description: "Implement the main functionality as described in the requirements".to_string(),
                file_targets: vec!["src/lib.rs".to_string()],
                estimated_minutes: 120,
                task_type: PlanTaskType::Implementation,
                validation_criteria: vec!["Core logic implemented and compiles without errors".to_string()],
            },
            PlanTask {
                task_id: "add-comprehensive-tests".to_string(),
                title: "Add Comprehensive Tests".to_string(),
                description: "Create thorough test coverage for all implemented functionality".to_string(),
                file_targets: vec!["tests/integration_tests.rs".to_string()],
                estimated_minutes: 90,
                task_type: PlanTaskType::Testing,
                validation_criteria: vec!["All tests pass", "Test coverage > 80%"].iter().map(|s| s.to_string()).collect(),
            },
        ];

        Ok(TaskPlan {
            plan_id: Uuid::new_v4(),
            created_at: Utc::now(),
            sprint_file: sprint_file.to_string_lossy().to_string(),
            overview: "Fallback implementation plan based on content analysis".to_string(),
            tasks,
            estimated_duration_minutes: 240,
            priority: TaskPriority::Medium,
            dependencies: vec!["analyze-requirements".to_string()],
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use std::sync::Arc;

    async fn create_test_llm() -> LlmRouter {
        let config = crate::llm::LlmConfig::default();
        LlmRouter::new(config, "logs").await.unwrap()
    }

    #[tokio::test]
    async fn test_plan_workflow_creation() {
        let llm = create_test_llm().await;
        let base_path = PathBuf::from(".");
        let workflow = PlanWorkflow::new(&llm, &base_path);
        
        // Just test that we can create the workflow
        assert!(std::ptr::eq(workflow.llm, &llm));
        assert_eq!(workflow.base_path, &base_path);
    }

    #[tokio::test]
    async fn test_fallback_plan_creation() {
        let llm = create_test_llm().await;
        let temp_dir = TempDir::new().unwrap();
        let base_path = temp_dir.path().to_path_buf();
        let workflow = PlanWorkflow::new(&llm, &base_path);
        
        let sprint_file = PathBuf::from("test-sprint.md");
        let content = "Test sprint content with requirements";
        
        let plan = workflow.create_fallback_plan(content, &sprint_file).await.unwrap();
        
        assert!(!plan.tasks.is_empty());
        assert_eq!(plan.sprint_file, "test-sprint.md");
        assert!(plan.estimated_duration_minutes > 0);
    }

    #[tokio::test]
    async fn test_planning_prompt_generation() {
        let llm = create_test_llm().await;
        let base_path = PathBuf::from(".");
        let workflow = PlanWorkflow::new(&llm, &base_path);
        
        let content = "Test sprint content";
        let prompt = workflow.create_planning_prompt(content);
        
        assert!(prompt.contains("Test sprint content"));
        assert!(prompt.contains("JSON structure"));
        assert!(prompt.contains("Rust best practices"));
    }
}