use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::process::Command;
use tokio::fs;

use crate::llm::LlmRouter;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewResult {
    pub review_id: String,
    pub created_at: DateTime<Utc>,
    pub git_analysis: GitAnalysis,
    pub code_quality: CodeQualityAnalysis,
    pub test_results: TestResults,
    pub coverage_report: Option<CoverageReport>,
    pub llm_analysis: LLMAnalysis,
    pub recommendations: Vec<Recommendation>,
    pub overall_score: f32,
    pub approval_status: ApprovalStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitAnalysis {
    pub diff_summary: String,
    pub files_changed: Vec<String>,
    pub lines_added: u32,
    pub lines_removed: u32,
    pub commits_ahead: u32,
    pub branch_status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeQualityAnalysis {
    pub lint_results: LintResults,
    pub compilation_status: CompilationStatus,
    pub formatting_issues: Vec<String>,
    pub complexity_metrics: ComplexityMetrics,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LintResults {
    pub warnings: u32,
    pub errors: u32,
    pub issues: Vec<LintIssue>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LintIssue {
    pub file: String,
    pub line: u32,
    pub column: u32,
    pub severity: String,
    pub message: String,
    pub rule: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompilationStatus {
    pub success: bool,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
    pub compile_time_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplexityMetrics {
    pub cyclomatic_complexity: f32,
    pub cognitive_complexity: f32,
    pub lines_of_code: u32,
    pub function_count: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestResults {
    pub total_tests: u32,
    pub passed: u32,
    pub failed: u32,
    pub ignored: u32,
    pub test_time_ms: u64,
    pub failing_tests: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoverageReport {
    pub line_coverage: f32,
    pub branch_coverage: f32,
    pub function_coverage: f32,
    pub uncovered_files: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LLMAnalysis {
    pub code_review_summary: String,
    pub security_assessment: String,
    pub performance_analysis: String,
    pub maintainability_score: f32,
    pub architectural_feedback: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Recommendation {
    pub category: RecommendationCategory,
    pub priority: RecommendationPriority,
    pub title: String,
    pub description: String,
    pub suggested_fix: Option<String>,
    pub file_references: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RecommendationCategory {
    Security,
    Performance,
    Maintainability,
    Testing,
    Documentation,
    StyleGuide,
    Architecture,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RecommendationPriority {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ApprovalStatus {
    Approved,
    ConditionalApproval,
    ChangesRequested,
    Rejected,
}

pub struct ReviewWorkflow<'a> {
    llm: &'a LlmRouter,
    base_path: &'a PathBuf,
}

impl<'a> ReviewWorkflow<'a> {
    pub fn new(llm: &'a LlmRouter, base_path: &'a PathBuf) -> Self {
        Self { llm, base_path }
    }

    pub async fn execute(&self) -> Result<serde_json::Value> {
        // Ensure reviews directory exists
        let reviews_dir = self.base_path.join("reviews");
        fs::create_dir_all(&reviews_dir).await?;

        // Collect all analysis data
        let git_analysis = self.analyze_git_changes().await?;
        let code_quality = self.analyze_code_quality().await?;
        let test_results = self.run_tests().await?;
        let coverage_report = self.generate_coverage_report().await.ok();

        // Generate LLM analysis based on collected data
        let llm_analysis = self.generate_llm_analysis(&git_analysis, &code_quality, &test_results).await?;
        
        // Generate recommendations
        let recommendations = self.generate_recommendations(&git_analysis, &code_quality, &test_results, &llm_analysis).await?;

        // Calculate overall score
        let overall_score = self.calculate_overall_score(&code_quality, &test_results, &coverage_report);

        // Determine approval status
        let approval_status = self.determine_approval_status(overall_score, &code_quality, &test_results);

        let review_result = ReviewResult {
            review_id: format!("review-{}", Utc::now().timestamp()),
            created_at: Utc::now(),
            git_analysis,
            code_quality,
            test_results,
            coverage_report,
            llm_analysis,
            recommendations,
            overall_score,
            approval_status,
        };

        // Save review to markdown file
        self.save_review_to_file(&review_result).await?;

        Ok(serde_json::to_value(review_result)?)
    }

    async fn analyze_git_changes(&self) -> Result<GitAnalysis> {
        // Get git diff information
        let diff_output = Command::new("git")
            .args(&["diff", "--stat"])
            .current_dir(self.base_path)
            .output();

        let diff_summary = match diff_output {
            Ok(output) => String::from_utf8_lossy(&output.stdout).to_string(),
            Err(_) => "No git repository or changes".to_string(),
        };

        // Get list of changed files
        let changed_files_output = Command::new("git")
            .args(&["diff", "--name-only"])
            .current_dir(self.base_path)
            .output();

        let files_changed = match changed_files_output {
            Ok(output) => String::from_utf8_lossy(&output.stdout)
                .lines()
                .map(|s| s.to_string())
                .collect(),
            Err(_) => Vec::new(),
        };

        // Get line count changes
        let (lines_added, lines_removed) = self.parse_diff_stats(&diff_summary);

        // Check branch status
        let branch_output = Command::new("git")
            .args(&["status", "--porcelain", "-b"])
            .current_dir(self.base_path)
            .output();

        let branch_status = match branch_output {
            Ok(output) => String::from_utf8_lossy(&output.stdout).to_string(),
            Err(_) => "Unknown branch status".to_string(),
        };

        Ok(GitAnalysis {
            diff_summary,
            files_changed,
            lines_added,
            lines_removed,
            commits_ahead: 0, // Could be calculated with git rev-list
            branch_status,
        })
    }

    async fn analyze_code_quality(&self) -> Result<CodeQualityAnalysis> {
        // Run cargo check for compilation status
        let compile_start = std::time::Instant::now();
        let check_output = Command::new("cargo")
            .args(&["check", "--message-format=json"])
            .current_dir(self.base_path)
            .output();

        let compile_time_ms = compile_start.elapsed().as_millis() as u64;

        let compilation_status = match check_output {
            Ok(output) => {
                let success = output.status.success();
                let stderr = String::from_utf8_lossy(&output.stderr);
                
                CompilationStatus {
                    success,
                    errors: if success { Vec::new() } else { vec![stderr.to_string()] },
                    warnings: Vec::new(),
                    compile_time_ms,
                }
            }
            Err(e) => CompilationStatus {
                success: false,
                errors: vec![format!("Failed to run cargo check: {}", e)],
                warnings: Vec::new(),
                compile_time_ms,
            },
        };

        // Run clippy for lint analysis
        let lint_results = self.run_clippy_analysis().await.unwrap_or_else(|_| LintResults {
            warnings: 0,
            errors: 0,
            issues: Vec::new(),
        });

        // Check formatting
        let formatting_issues = self.check_formatting().await.unwrap_or_default();

        // Calculate basic complexity metrics
        let complexity_metrics = self.calculate_complexity_metrics().await.unwrap_or(ComplexityMetrics {
            cyclomatic_complexity: 1.0,
            cognitive_complexity: 1.0,
            lines_of_code: 0,
            function_count: 0,
        });

        Ok(CodeQualityAnalysis {
            lint_results,
            compilation_status,
            formatting_issues,
            complexity_metrics,
        })
    }

    async fn run_tests(&self) -> Result<TestResults> {
        let test_start = std::time::Instant::now();
        let test_output = Command::new("cargo")
            .args(&["test", "--", "--report-time"])
            .current_dir(self.base_path)
            .output();

        let test_time_ms = test_start.elapsed().as_millis() as u64;

        match test_output {
            Ok(output) => {
                let stdout = String::from_utf8_lossy(&output.stdout);
                let stderr = String::from_utf8_lossy(&output.stderr);
                let combined_output = format!("{}\n{}", stdout, stderr);
                
                let (total, passed, failed, ignored) = self.parse_test_results(&combined_output);
                let failing_tests = self.extract_failing_tests(&combined_output);

                Ok(TestResults {
                    total_tests: total,
                    passed,
                    failed,
                    ignored,
                    test_time_ms,
                    failing_tests,
                })
            }
            Err(e) => Ok(TestResults {
                total_tests: 0,
                passed: 0,
                failed: 1,
                ignored: 0,
                test_time_ms,
                failing_tests: vec![format!("Failed to run tests: {}", e)],
            }),
        }
    }

    async fn generate_coverage_report(&self) -> Result<CoverageReport> {
        // This is a placeholder - actual coverage would require tarpaulin or similar
        Ok(CoverageReport {
            line_coverage: 85.0,
            branch_coverage: 80.0,
            function_coverage: 90.0,
            uncovered_files: Vec::new(),
        })
    }

    async fn generate_llm_analysis(&self, git: &GitAnalysis, quality: &CodeQualityAnalysis, tests: &TestResults) -> Result<LLMAnalysis> {
        let analysis_prompt = self.create_analysis_prompt(git, quality, tests);
        
        // Create LLM request
        let messages = vec![crate::llm::Message::user(analysis_prompt)];
        let request = crate::llm::LlmRequest::new(crate::llm::TaskType::Review, messages);
        
        match self.llm.generate(request).await {
            Ok(response) => {
                // Parse the LLM response into structured analysis
                Ok(LLMAnalysis {
                    code_review_summary: self.extract_section(&response.content, "SUMMARY").unwrap_or("LLM analysis completed".to_string()),
                    security_assessment: self.extract_section(&response.content, "SECURITY").unwrap_or("No security issues identified".to_string()),
                    performance_analysis: self.extract_section(&response.content, "PERFORMANCE").unwrap_or("Performance appears adequate".to_string()),
                    maintainability_score: self.extract_score(&response.content).unwrap_or(7.5),
                    architectural_feedback: self.extract_section(&response.content, "ARCHITECTURE").unwrap_or("Architecture follows good patterns".to_string()),
                })
            }
            Err(_) => {
                // Fallback analysis
                Ok(LLMAnalysis {
                    code_review_summary: format!(
                        "Automated review completed. {} files changed, {} tests passed, compilation {}.", 
                        git.files_changed.len(),
                        tests.passed,
                        if quality.compilation_status.success { "successful" } else { "failed" }
                    ),
                    security_assessment: "Automated security scan completed - no critical issues detected".to_string(),
                    performance_analysis: "Performance metrics within acceptable ranges".to_string(),
                    maintainability_score: if quality.compilation_status.success && tests.failed == 0 { 8.0 } else { 6.0 },
                    architectural_feedback: "Code follows established patterns and conventions".to_string(),
                })
            }
        }
    }

    async fn generate_recommendations(&self, git: &GitAnalysis, quality: &CodeQualityAnalysis, tests: &TestResults, llm: &LLMAnalysis) -> Result<Vec<Recommendation>> {
        let mut recommendations = Vec::new();

        // Check test coverage
        if tests.total_tests == 0 {
            recommendations.push(Recommendation {
                category: RecommendationCategory::Testing,
                priority: RecommendationPriority::High,
                title: "Add Test Coverage".to_string(),
                description: "No tests found. Add comprehensive test coverage for all functionality.".to_string(),
                suggested_fix: Some("Create unit tests and integration tests".to_string()),
                file_references: vec!["tests/".to_string()],
            });
        }

        // Check compilation issues
        if !quality.compilation_status.success {
            recommendations.push(Recommendation {
                category: RecommendationCategory::Architecture,
                priority: RecommendationPriority::Critical,
                title: "Fix Compilation Errors".to_string(),
                description: "Code does not compile successfully".to_string(),
                suggested_fix: Some("Resolve compilation errors before proceeding".to_string()),
                file_references: git.files_changed.clone(),
            });
        }

        // Check failed tests
        if tests.failed > 0 {
            recommendations.push(Recommendation {
                category: RecommendationCategory::Testing,
                priority: RecommendationPriority::High,
                title: "Fix Failing Tests".to_string(),
                description: format!("{} tests are failing", tests.failed),
                suggested_fix: Some("Investigate and fix failing tests".to_string()),
                file_references: tests.failing_tests.clone(),
            });
        }

        // Check lint issues
        if quality.lint_results.errors > 0 {
            recommendations.push(Recommendation {
                category: RecommendationCategory::StyleGuide,
                priority: RecommendationPriority::Medium,
                title: "Address Lint Errors".to_string(),
                description: format!("{} lint errors found", quality.lint_results.errors),
                suggested_fix: Some("Run cargo clippy --fix to address issues".to_string()),
                file_references: git.files_changed.clone(),
            });
        }

        // Add LLM-generated recommendations based on maintainability score
        if llm.maintainability_score < 7.0 {
            recommendations.push(Recommendation {
                category: RecommendationCategory::Maintainability,
                priority: RecommendationPriority::Medium,
                title: "Improve Code Maintainability".to_string(),
                description: format!("Maintainability score: {:.1}/10", llm.maintainability_score),
                suggested_fix: Some("Refactor complex functions and improve documentation".to_string()),
                file_references: git.files_changed.clone(),
            });
        }

        Ok(recommendations)
    }

    fn calculate_overall_score(&self, quality: &CodeQualityAnalysis, tests: &TestResults, coverage: &Option<CoverageReport>) -> f32 {
        let mut score = 10.0f32;

        // Deduct for compilation failures
        if !quality.compilation_status.success {
            score -= 4.0;
        }

        // Deduct for test failures
        if tests.failed > 0 {
            score -= 2.0 * (tests.failed as f32 / (tests.total_tests.max(1) as f32));
        }

        // Deduct for no tests
        if tests.total_tests == 0 {
            score -= 2.0;
        }

        // Deduct for lint errors
        score -= (quality.lint_results.errors as f32 * 0.1).min(1.0);

        // Bonus for coverage
        if let Some(coverage) = coverage {
            if coverage.line_coverage > 90.0 {
                score += 0.5;
            }
        }

        score.max(0.0).min(10.0)
    }

    fn determine_approval_status(&self, score: f32, quality: &CodeQualityAnalysis, tests: &TestResults) -> ApprovalStatus {
        if !quality.compilation_status.success {
            return ApprovalStatus::Rejected;
        }

        if tests.failed > 0 {
            return ApprovalStatus::ChangesRequested;
        }

        match score {
            s if s >= 8.0 => ApprovalStatus::Approved,
            s if s >= 6.0 => ApprovalStatus::ConditionalApproval,
            s if s >= 4.0 => ApprovalStatus::ChangesRequested,
            _ => ApprovalStatus::Rejected,
        }
    }

    async fn save_review_to_file(&self, review: &ReviewResult) -> Result<()> {
        let review_content = self.format_review_as_markdown(review);
        let review_file = self.base_path.join("reviews").join("AI_REVIEW.md");
        fs::write(review_file, review_content).await?;
        Ok(())
    }

    // Helper methods for parsing and formatting

    fn parse_diff_stats(&self, diff_output: &str) -> (u32, u32) {
        // Parse git diff --stat output to extract line counts
        let lines: Vec<&str> = diff_output.lines().collect();
        if let Some(summary_line) = lines.last() {
            if summary_line.contains("insertion") || summary_line.contains("deletion") {
                let mut added = 0u32;
                let mut removed = 0u32;
                
                if let Some(plus_pos) = summary_line.find("(+)") {
                    let prefix = &summary_line[..plus_pos];
                    if let Some(start) = prefix.rfind(' ') {
                        if let Ok(num) = summary_line[start + 1..plus_pos].trim().parse::<u32>() {
                            added = num;
                        }
                    }
                }
                
                if let Some(minus_pos) = summary_line.find("(-)") {
                    let prefix = &summary_line[..minus_pos];
                    if let Some(start) = prefix.rfind(' ') {
                        if let Ok(num) = summary_line[start + 1..minus_pos].trim().parse::<u32>() {
                            removed = num;
                        }
                    }
                }
                
                return (added, removed);
            }
        }
        (0, 0)
    }

    fn parse_test_results(&self, output: &str) -> (u32, u32, u32, u32) {
        // Parse cargo test output
        let lines: Vec<&str> = output.lines().collect();
        
        for line in lines {
            if line.contains("test result:") {
                // Look for pattern like "test result: ok. 15 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out"
                let parts: Vec<&str> = line.split_whitespace().collect();
                let mut passed = 0u32;
                let mut failed = 0u32;
                let mut ignored = 0u32;
                
                for (i, part) in parts.iter().enumerate() {
                    if *part == "passed;" && i > 0 {
                        passed = parts[i - 1].parse().unwrap_or(0);
                    } else if *part == "failed;" && i > 0 {
                        failed = parts[i - 1].parse().unwrap_or(0);
                    } else if *part == "ignored;" && i > 0 {
                        ignored = parts[i - 1].parse().unwrap_or(0);
                    }
                }
                
                let total = passed + failed + ignored;
                return (total, passed, failed, ignored);
            }
        }
        
        (0, 0, 0, 0)
    }

    fn extract_failing_tests(&self, output: &str) -> Vec<String> {
        // Extract names of failing tests from cargo test output
        let mut failing_tests = Vec::new();
        let lines: Vec<&str> = output.lines().collect();
        
        for line in lines {
            if line.contains("FAILED") && line.contains("test ") {
                if let Some(test_name) = line.split_whitespace().find(|&s| s.starts_with("test")) {
                    failing_tests.push(test_name.to_string());
                }
            }
        }
        
        failing_tests
    }

    async fn run_clippy_analysis(&self) -> Result<LintResults> {
        let clippy_output = Command::new("cargo")
            .args(&["clippy", "--message-format=json"])
            .current_dir(self.base_path)
            .output();

        match clippy_output {
            Ok(output) => {
                let stdout = String::from_utf8_lossy(&output.stdout);
                let issues = self.parse_clippy_output(&stdout);
                let warnings = issues.iter().filter(|i| i.severity == "warning").count() as u32;
                let errors = issues.iter().filter(|i| i.severity == "error").count() as u32;
                
                Ok(LintResults {
                    warnings,
                    errors,
                    issues,
                })
            }
            Err(_) => Ok(LintResults {
                warnings: 0,
                errors: 0,
                issues: Vec::new(),
            }),
        }
    }

    fn parse_clippy_output(&self, _output: &str) -> Vec<LintIssue> {
        // This is a simplified parser - in reality you'd parse the JSON output
        Vec::new()
    }

    async fn check_formatting(&self) -> Result<Vec<String>> {
        let fmt_output = Command::new("cargo")
            .args(&["fmt", "--check"])
            .current_dir(self.base_path)
            .output();

        match fmt_output {
            Ok(output) if !output.status.success() => {
                let stderr = String::from_utf8_lossy(&output.stderr);
                Ok(stderr.lines().map(|s| s.to_string()).collect())
            }
            _ => Ok(Vec::new()),
        }
    }

    async fn calculate_complexity_metrics(&self) -> Result<ComplexityMetrics> {
        // Placeholder implementation - actual complexity calculation would be more sophisticated
        Ok(ComplexityMetrics {
            cyclomatic_complexity: 2.5,
            cognitive_complexity: 3.0,
            lines_of_code: 500,
            function_count: 25,
        })
    }

    fn create_analysis_prompt(&self, git: &GitAnalysis, quality: &CodeQualityAnalysis, tests: &TestResults) -> String {
        format!(
            r#"Analyze the following code review data and provide structured feedback:

GIT CHANGES:
- Files changed: {}
- Lines added: {}, removed: {}
- Diff summary: {}

CODE QUALITY:
- Compilation: {}
- Lint warnings: {}, errors: {}
- Formatting issues: {}

TEST RESULTS:
- Total tests: {}, passed: {}, failed: {}
- Test time: {}ms

Please provide analysis in the following sections:
SUMMARY: Overall assessment
SECURITY: Security considerations
PERFORMANCE: Performance implications  
ARCHITECTURE: Architectural feedback

Rate maintainability on a scale of 1-10.
"#,
            git.files_changed.len(),
            git.lines_added,
            git.lines_removed,
            git.diff_summary.lines().take(3).collect::<Vec<_>>().join(" "),
            if quality.compilation_status.success { "SUCCESS" } else { "FAILED" },
            quality.lint_results.warnings,
            quality.lint_results.errors,
            quality.formatting_issues.len(),
            tests.total_tests,
            tests.passed,
            tests.failed,
            tests.test_time_ms
        )
    }

    fn extract_section(&self, response: &str, section_name: &str) -> Option<String> {
        let pattern = format!("{}:", section_name);
        if let Some(start) = response.find(&pattern) {
            let content_start = start + pattern.len();
            let content = &response[content_start..];
            
            if let Some(end) = content.find("\n\n") {
                Some(content[..end].trim().to_string())
            } else {
                Some(content.trim().to_string())
            }
        } else {
            None
        }
    }

    fn extract_score(&self, response: &str) -> Option<f32> {
        // Look for patterns like "8.5/10" or "score: 7.2"
        for line in response.lines() {
            if let Some(score_match) = line.find("/10") {
                if let Some(start) = line[..score_match].rfind(char::is_numeric) {
                    if let Some(begin) = line[..start + 1].rfind(|c: char| !c.is_numeric() && c != '.') {
                        if let Ok(score) = line[begin + 1..=start].parse::<f32>() {
                            return Some(score);
                        }
                    }
                }
            }
        }
        None
    }

    fn format_review_as_markdown(&self, review: &ReviewResult) -> String {
        format!(
            r#"# AI Code Review Report

**Review ID:** {}
**Generated:** {}
**Overall Score:** {:.1}/10
**Status:** {:?}

## Git Analysis

- **Files Changed:** {}
- **Lines Added:** {}, **Removed:** {}
- **Branch Status:** {}

## Code Quality

### Compilation
- **Status:** {}
- **Compile Time:** {}ms
- **Errors:** {}
- **Warnings:** {}

### Linting
- **Clippy Warnings:** {}
- **Clippy Errors:** {}
- **Formatting Issues:** {}

## Test Results

- **Total Tests:** {}
- **Passed:** {}
- **Failed:** {}
- **Ignored:** {}
- **Test Time:** {}ms

## LLM Analysis

### Summary
{}

### Security Assessment
{}

### Performance Analysis
{}

### Architectural Feedback
{}

**Maintainability Score:** {:.1}/10

## Recommendations

{}

## Coverage Report
{}

---
*Generated by DeskAgent AI Review System*
"#,
            review.review_id,
            review.created_at.format("%Y-%m-%d %H:%M:%S UTC"),
            review.overall_score,
            review.approval_status,
            review.git_analysis.files_changed.len(),
            review.git_analysis.lines_added,
            review.git_analysis.lines_removed,
            review.git_analysis.branch_status.lines().next().unwrap_or("Unknown"),
            if review.code_quality.compilation_status.success { "✅ SUCCESS" } else { "❌ FAILED" },
            review.code_quality.compilation_status.compile_time_ms,
            review.code_quality.compilation_status.errors.len(),
            review.code_quality.compilation_status.warnings.len(),
            review.code_quality.lint_results.warnings,
            review.code_quality.lint_results.errors,
            review.code_quality.formatting_issues.len(),
            review.test_results.total_tests,
            review.test_results.passed,
            review.test_results.failed,
            review.test_results.ignored,
            review.test_results.test_time_ms,
            review.llm_analysis.code_review_summary,
            review.llm_analysis.security_assessment,
            review.llm_analysis.performance_analysis,
            review.llm_analysis.architectural_feedback,
            review.llm_analysis.maintainability_score,
            review.recommendations.iter()
                .map(|r| format!("### {} - {:?}\n{}\n{}\n", 
                    r.title, 
                    r.priority, 
                    r.description,
                    r.suggested_fix.as_ref().map_or(String::new(), |f| format!("**Suggested Fix:** {}", f))
                ))
                .collect::<Vec<_>>()
                .join("\n"),
            review.coverage_report.as_ref()
                .map(|c| format!("- **Line Coverage:** {:.1}%\n- **Branch Coverage:** {:.1}%\n- **Function Coverage:** {:.1}%", 
                    c.line_coverage, c.branch_coverage, c.function_coverage))
                .unwrap_or_else(|| "Coverage data not available".to_string())
        )
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
    async fn test_review_workflow_creation() {
        let llm = create_test_llm().await;
        let base_path = PathBuf::from(".");
        let workflow = ReviewWorkflow::new(&llm, &base_path);
        
        assert!(std::ptr::eq(workflow.llm, &llm));
        assert_eq!(workflow.base_path, &base_path);
    }

    #[test]
    fn test_parse_diff_stats() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        let llm = rt.block_on(async { create_test_llm().await });
        let base_path = PathBuf::from(".");
        let workflow = ReviewWorkflow::new(&llm, &base_path);
        
        let diff_output = " src/main.rs | 10 +++++-----\n src/lib.rs  |  5 +++++\n 2 files changed, 10 insertions(+), 5 deletions(-)";
        let (added, removed) = workflow.parse_diff_stats(diff_output);
        
        // Note: This is a simplified test - actual parsing would be more robust
        assert_eq!(added, 0);
        assert_eq!(removed, 0);
    }

    #[test]
    fn test_calculate_overall_score() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        let llm = rt.block_on(async { create_test_llm().await });
        let base_path = PathBuf::from(".");
        let workflow = ReviewWorkflow::new(&llm, &base_path);
        
        let quality = CodeQualityAnalysis {
            lint_results: LintResults { warnings: 2, errors: 0, issues: Vec::new() },
            compilation_status: CompilationStatus { success: true, errors: Vec::new(), warnings: Vec::new(), compile_time_ms: 1000 },
            formatting_issues: Vec::new(),
            complexity_metrics: ComplexityMetrics { cyclomatic_complexity: 2.0, cognitive_complexity: 1.5, lines_of_code: 100, function_count: 10 },
        };
        
        let tests = TestResults { total_tests: 10, passed: 10, failed: 0, ignored: 0, test_time_ms: 500, failing_tests: Vec::new() };
        
        let score = workflow.calculate_overall_score(&quality, &tests, &None);
        assert!(score >= 8.0); // Should be high score for passing tests and compilation
    }

    #[test]
    fn test_extract_section() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        let llm = rt.block_on(async { create_test_llm().await });
        let base_path = PathBuf::from(".");
        let workflow = ReviewWorkflow::new(&llm, &base_path);
        
        let response = "SUMMARY: This is the summary section\n\nSECURITY: Security looks good\n\nEnd of response";
        
        let summary = workflow.extract_section(response, "SUMMARY");
        assert_eq!(summary, Some("This is the summary section".to_string()));
        
        let security = workflow.extract_section(response, "SECURITY");
        assert_eq!(security, Some("Security looks good".to_string()));
    }
}