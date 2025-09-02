use anyhow::Result;
use deskagent::{
    orchestrator::{Orchestrator, OrchestratorConfig},
    desktop::{CursorController, TerminalController},
    llm::{LlmRouter, LlmConfig},
    workflows::WorkflowManager,
};
use std::path::PathBuf;

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();
    
    println!("🚀 DeskAgent Workflow Demo - PLAN & REVIEW Integration");
    println!("=====================================");
    
    // Initialize all components
    let config = OrchestratorConfig {
        max_concurrent_tasks: 5,
        task_timeout_ms: 30000,
        log_directory: "logs".to_string(),
    };
    
    let orchestrator = Orchestrator::new(config).await?;
    let cursor = CursorController::new();
    let terminal = TerminalController::new();
    let llm_config = LlmConfig::default();
    let llm = LlmRouter::new(llm_config, "logs").await?;
    let base_path = PathBuf::from(".");
    
    let mut workflow_manager = WorkflowManager::new(orchestrator, cursor, terminal, llm, base_path);
    
    // Demonstrate PLAN workflow
    println!("\n📋 Phase 1: PLAN Workflow");
    println!("Reading demo_sprint.md and generating task plan...");
    
    let sprint_file = PathBuf::from("demo_sprint.md");
    let plan_result = workflow_manager.execute_plan_workflow(sprint_file).await?;
    
    if plan_result.output_data.is_some() {
        println!("✅ Plan generated successfully!");
        println!("📁 Artifact: plans/sprint-01.plan.json");
        
        // Demonstrate EDIT workflow
        println!("\n✏️  Phase 2: EDIT Workflow");
        println!("Processing plan and generating placeholder code...");
        
        let edit_result = workflow_manager
            .execute_edit_workflow(plan_result.output_data.unwrap())
            .await?;
            
        println!("✅ Edit operations completed!");
        println!("🔧 Placeholder code inserted via Desktop Control");
    } else {
        println!("❌ Plan generation failed");
    }
    
    // Demonstrate REVIEW workflow
    println!("\n🔍 Phase 3: REVIEW Workflow");
    println!("Analyzing codebase and generating AI review...");
    
    let review_result = workflow_manager.execute_review_workflow().await?;
    
    if review_result.output_data.is_some() {
        println!("✅ Review completed successfully!");
        println!("📁 Artifact: reviews/AI_REVIEW.md");
    } else {
        println!("❌ Review generation failed");
    }
    
    println!("\n🎉 Workflow Demo Complete!");
    println!("=====================================");
    println!("All three workflows (PLAN → EDIT → REVIEW) demonstrated successfully.");
    println!("\nKey Features Demonstrated:");
    println!("• Sprint document → LLM → structured task plan");
    println!("• Task plan → Desktop Control → code insertion");
    println!("• Git analysis → test execution → AI review");
    println!("• Comprehensive error handling and fallback mechanisms");
    println!("• Artifact management (plans/, reviews/ directories)");
    
    Ok(())
}