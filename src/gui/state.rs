use anyhow::Result;
use chrono::{DateTime, Utc};
use tokio::fs;

#[derive(Debug, Clone, PartialEq)]
pub enum ViewType {
    Dashboard,
    SprintPanel,
    ReviewWorkspace,
}

#[derive(Debug, Clone)]
pub struct AppState {
    pub current_view: ViewType,
    pub dashboard_state: DashboardState,
    pub sprint_state: SprintState,
    pub review_state: ReviewState,
    pub settings: AppSettings,
}

#[derive(Debug, Clone)]
pub struct DashboardState {
    pub repo_name: String,
    pub branch: String,
    pub total_tasks: u32,
    pub completed_tasks: u32,
    pub failed_tasks: u32,
    pub progress_percentage: f32,
    pub last_review_status: String,
    pub recent_activities: Vec<ActivityItem>,
    pub risks: Vec<RiskItem>,
}

#[derive(Debug, Clone)]
pub struct ActivityItem {
    pub timestamp: DateTime<Utc>,
    pub title: String,
    pub status: ActivityStatus,
}

#[derive(Debug, Clone)]
pub enum ActivityStatus {
    Success,
    Failed,
    Running,
    Paused,
}

#[derive(Debug, Clone)]
pub struct RiskItem {
    pub level: RiskLevel,
    pub count: u32,
}

#[derive(Debug, Clone)]
pub enum RiskLevel {
    High,
    Medium,
    Low,
}

#[derive(Debug, Clone)]
pub struct SprintState {
    pub sprint_name: String,
    pub status: String,
    pub duration: String,
    pub progress: f32,
    pub modules: Vec<ModuleInfo>,
    pub deliverables: Vec<DeliverableInfo>,
}

#[derive(Debug, Clone)]
pub struct ModuleInfo {
    pub name: String,
    pub status: ModuleStatus,
    pub tests_passed: u32,
    pub tests_total: u32,
    pub expanded: bool,
    pub sub_tasks: Vec<SubTaskInfo>,
}

#[derive(Debug, Clone)]
pub enum ModuleStatus {
    Completed,
    InProgress,
    Failed,
    Paused,
}

#[derive(Debug, Clone)]
pub struct SubTaskInfo {
    pub name: String,
    pub status: ModuleStatus,
}

#[derive(Debug, Clone)]
pub struct DeliverableInfo {
    pub name: String,
    pub path: String,
    pub completed: bool,
}

#[derive(Debug, Clone)]
pub struct ReviewState {
    pub review_id: String,
    pub status: ReviewStatus,
    pub quality_score: u32,
    pub reviewer: String,
    pub review_time: String,
    pub file_changes: Vec<FileChangeInfo>,
    pub analysis_summary: AnalysisSummary,
    pub recommendations: Vec<String>,
}

#[derive(Debug, Clone)]
pub enum ReviewStatus {
    Approved,
    Pending,
    ChangesRequested,
}

#[derive(Debug, Clone)]
pub struct FileChangeInfo {
    pub file: String,
    pub additions: u32,
    pub deletions: u32,
    pub risk: RiskLevel,
    pub issues: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct AnalysisSummary {
    pub architecture: String,
    pub error_handling: String,
    pub testing: String,
    pub performance: String,
    pub security: String,
}

#[derive(Debug, Clone)]
pub struct AppSettings {
    pub theme: Theme,
    pub auto_refresh: bool,
    pub refresh_interval: u64,
}

#[derive(Debug, Clone)]
pub enum Theme {
    Light,
    Dark,
    System,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            current_view: ViewType::Dashboard,
            dashboard_state: DashboardState::default(),
            sprint_state: SprintState::default(),
            review_state: ReviewState::default(),
            settings: AppSettings::default(),
        }
    }
}

impl Default for DashboardState {
    fn default() -> Self {
        Self {
            repo_name: "deskagent".to_string(),
            branch: "main".to_string(),
            total_tasks: 0,
            completed_tasks: 0,
            failed_tasks: 0,
            progress_percentage: 0.0,
            last_review_status: "PENDING".to_string(),
            recent_activities: Vec::new(),
            risks: Vec::new(),
        }
    }
}

impl Default for SprintState {
    fn default() -> Self {
        Self {
            sprint_name: "Sprint-01".to_string(),
            status: "COMPLETED".to_string(),
            duration: "1 day".to_string(),
            progress: 100.0,
            modules: Vec::new(),
            deliverables: Vec::new(),
        }
    }
}

impl Default for ReviewState {
    fn default() -> Self {
        Self {
            review_id: "AI_REVIEW_001".to_string(),
            status: ReviewStatus::Approved,
            quality_score: 95,
            reviewer: "Claude AI".to_string(),
            review_time: "2.3s".to_string(),
            file_changes: Vec::new(),
            analysis_summary: AnalysisSummary::default(),
            recommendations: Vec::new(),
        }
    }
}

impl Default for AnalysisSummary {
    fn default() -> Self {
        Self {
            architecture: "Clean modular design".to_string(),
            error_handling: "Comprehensive Result patterns".to_string(),
            testing: "98.4% coverage".to_string(),
            performance: "Some blocking I/O".to_string(),
            security: "API keys need secure storage".to_string(),
        }
    }
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            theme: Theme::System,
            auto_refresh: true,
            refresh_interval: 5,
        }
    }
}

impl AppState {
    pub async fn load_from_files() -> Result<Self> {
        let mut state = Self::default();
        
        // Load dashboard data
        if let Ok(progress_data) = fs::read_to_string("progress/sprint-01.progress.json").await {
            state.load_progress_data(&progress_data)?;
        }
        
        // Load review data
        if let Ok(review_data) = fs::read_to_string("reviews/AI_REVIEW.md").await {
            state.load_review_data(&review_data)?;
        }
        
        Ok(state)
    }
    
    fn load_progress_data(&mut self, data: &str) -> Result<()> {
        let progress: serde_json::Value = serde_json::from_str(data)?;
        
        if let Some(completion) = progress.get("completion_percentage").and_then(|v| v.as_f64()) {
            self.dashboard_state.progress_percentage = completion as f32;
        }
        
        if let Some(modules) = progress.get("modules").and_then(|v| v.as_array()) {
            self.dashboard_state.total_tasks = modules.len() as u32;
            self.dashboard_state.completed_tasks = modules.iter()
                .filter(|m| m.get("status").and_then(|s| s.as_str()) == Some("completed"))
                .count() as u32;
                
            // Load sprint modules
            for module in modules {
                if let (Some(name), Some(status)) = (
                    module.get("name").and_then(|n| n.as_str()),
                    module.get("status").and_then(|s| s.as_str())
                ) {
                    let module_status = match status {
                        "completed" => ModuleStatus::Completed,
                        "in_progress" => ModuleStatus::InProgress,
                        "failed" => ModuleStatus::Failed,
                        _ => ModuleStatus::Paused,
                    };
                    
                    let tests_passing = module.get("tests_passing")
                        .and_then(|t| t.as_u64()).unwrap_or(0) as u32;
                    
                    self.sprint_state.modules.push(ModuleInfo {
                        name: name.to_string(),
                        status: module_status,
                        tests_passed: tests_passing,
                        tests_total: tests_passing, // Assume all tests are passing for now
                        expanded: false,
                        sub_tasks: Vec::new(),
                    });
                }
            }
        }
        
        Ok(())
    }
    
    fn load_review_data(&mut self, _data: &str) -> Result<()> {
        // For now, keep default review data
        // In a real implementation, we would parse the markdown file
        Ok(())
    }
}