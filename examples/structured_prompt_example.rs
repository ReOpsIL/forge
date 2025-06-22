// Example of enhanced task prompts for better Claude output parsing
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct StructuredTaskPrompt {
    pub task: TaskDetails,
    pub context: ProjectContext,
    pub requirements: ExecutionRequirements,
    pub output_format: OutputFormat,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TaskDetails {
    pub id: String,
    pub name: String,
    pub description: String,
    pub acceptance_criteria: Vec<String>,
    pub files_to_modify: Vec<String>,
    pub dependencies: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ProjectContext {
    pub project_name: String,
    pub project_type: String, // "rust", "javascript", etc.
    pub architecture_patterns: Vec<String>,
    pub existing_modules: Vec<String>,
    pub code_style_guide: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ExecutionRequirements {
    pub create_tests: bool,
    pub follow_existing_patterns: bool,
    pub update_documentation: bool,
    pub run_quality_checks: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OutputFormat {
    pub structured_response: bool,
    pub include_file_list: bool,
    pub include_summary: bool,
    pub response_markers: ResponseMarkers,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ResponseMarkers {
    pub start_marker: String,
    pub end_marker: String,
    pub success_marker: String,
    pub error_marker: String,
    pub files_marker: String,
    pub summary_marker: String,
}

impl Default for ResponseMarkers {
    fn default() -> Self {
        Self {
            start_marker: "🚀 FORGE_TASK_START".to_string(),
            end_marker: "🏁 FORGE_TASK_END".to_string(),
            success_marker: "✅ FORGE_TASK_SUCCESS".to_string(),
            error_marker: "❌ FORGE_TASK_ERROR".to_string(),
            files_marker: "📁 FORGE_FILES_MODIFIED".to_string(),
            summary_marker: "📋 FORGE_TASK_SUMMARY".to_string(),
        }
    }
}

impl StructuredTaskPrompt {
    pub fn to_claude_prompt(&self) -> String {
        format!(
            r#"
# Forge IDE Task Execution

You are working within Forge IDE, a visual development platform. Please execute this task with structured output for proper integration.

## Task Information
**Task ID**: {task_id}
**Task Name**: {task_name}
**Description**: {description}

## Acceptance Criteria
{acceptance_criteria}

## Project Context
- **Project**: {project_name} ({project_type})
- **Architecture Patterns**: {architecture_patterns}
- **Existing Modules**: {existing_modules}

## Files to Modify
{files_to_modify}

## Dependencies
{dependencies}

## Execution Requirements
- Create Tests: {create_tests}
- Follow Existing Patterns: {follow_existing_patterns}
- Update Documentation: {update_documentation}
- Run Quality Checks: {run_quality_checks}

## IMPORTANT: Output Format Requirements

Please structure your response with these exact markers:

1. Start your work with: `{start_marker}`
2. When you successfully complete the task, use: `{success_marker}`
3. If you encounter errors, use: `{error_marker}: [error description]`
4. List modified files with: `{files_marker}: [file1.rs, file2.js, ...]`
5. Provide a summary with: `{summary_marker}: [brief summary of what was accomplished]`
6. End your response with: `{end_marker}`

## Example Output Format:
```
{start_marker}
[Your implementation work here...]

{success_marker}
{files_marker}: src/main.rs, src/models.rs, tests/integration_test.rs
{summary_marker}: Successfully implemented user authentication module with JWT support, added comprehensive tests, and updated documentation.
{end_marker}
```

Please begin the task implementation now.
"#,
            task_id = self.task.id,
            task_name = self.task.name,
            description = self.task.description,
            acceptance_criteria = self.task.acceptance_criteria.iter()
                .enumerate()
                .map(|(i, criteria)| format!("{}. {}", i + 1, criteria))
                .collect::<Vec<_>>()
                .join("\n"),
            project_name = self.context.project_name,
            project_type = self.context.project_type,
            architecture_patterns = self.context.architecture_patterns.join(", "),
            existing_modules = self.context.existing_modules.join(", "),
            files_to_modify = if self.task.files_to_modify.is_empty() {
                "No specific files specified - determine based on task requirements".to_string()
            } else {
                self.task.files_to_modify.iter()
                    .map(|f| format!("- {}", f))
                    .collect::<Vec<_>>()
                    .join("\n")
            },
            dependencies = if self.task.dependencies.is_empty() {
                "No dependencies specified".to_string()
            } else {
                self.task.dependencies.iter()
                    .map(|d| format!("- {}", d))
                    .collect::<Vec<_>>()
                    .join("\n")
            },
            create_tests = self.requirements.create_tests,
            follow_existing_patterns = self.requirements.follow_existing_patterns,
            update_documentation = self.requirements.update_documentation,
            run_quality_checks = self.requirements.run_quality_checks,
            start_marker = self.output_format.response_markers.start_marker,
            success_marker = self.output_format.response_markers.success_marker,
            error_marker = self.output_format.response_markers.error_marker,
            files_marker = self.output_format.response_markers.files_marker,
            summary_marker = self.output_format.response_markers.summary_marker,
            end_marker = self.output_format.response_markers.end_marker,
        )
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ClaudeTaskResponse {
    pub success: bool,
    pub error_message: Option<String>,
    pub files_modified: Vec<String>,
    pub summary: String,
    pub raw_output: String,
    pub execution_time: std::time::Duration,
}

pub fn parse_claude_response(
    output: &str,
    markers: &ResponseMarkers,
) -> Result<ClaudeTaskResponse, String> {
    let start_time = std::time::Instant::now();
    
    // Check for task completion markers
    let has_start = output.contains(&markers.start_marker);
    let has_end = output.contains(&markers.end_marker);
    let has_success = output.contains(&markers.success_marker);
    let has_error = output.contains(&markers.error_marker);
    
    if !has_start || !has_end {
        return Err("Claude response missing start/end markers - task may not have completed properly".to_string());
    }
    
    let success = has_success && !has_error;
    
    // Extract error message if present
    let error_message = if has_error {
        extract_after_marker(output, &markers.error_marker)
    } else {
        None
    };
    
    // Extract modified files
    let files_modified = if let Some(files_str) = extract_after_marker(output, &markers.files_marker) {
        parse_file_list(&files_str)
    } else {
        Vec::new()
    };
    
    // Extract summary
    let summary = extract_after_marker(output, &markers.summary_marker)
        .unwrap_or_else(|| "No summary provided".to_string());
    
    Ok(ClaudeTaskResponse {
        success,
        error_message,
        files_modified,
        summary,
        raw_output: output.to_string(),
        execution_time: start_time.elapsed(),
    })
}

fn extract_after_marker(text: &str, marker: &str) -> Option<String> {
    if let Some(start) = text.find(marker) {
        let after_marker = &text[start + marker.len()..];
        if let Some(end) = after_marker.find('\n') {
            Some(after_marker[..end].trim().to_string())
        } else {
            Some(after_marker.trim().to_string())
        }
    } else {
        None
    }
}

fn parse_file_list(files_str: &str) -> Vec<String> {
    // Handle different file list formats
    let cleaned = files_str.trim_start_matches(':').trim();
    
    if cleaned.starts_with('[') && cleaned.ends_with(']') {
        // JSON array format: [file1.rs, file2.js, ...]
        cleaned[1..cleaned.len()-1]
            .split(',')
            .map(|f| f.trim().trim_matches('"').trim_matches('\'').to_string())
            .filter(|f| !f.is_empty())
            .collect()
    } else {
        // Comma-separated format: file1.rs, file2.js, ...
        cleaned
            .split(',')
            .map(|f| f.trim().to_string())
            .filter(|f| !f.is_empty())
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_successful_response() {
        let output = r#"
🚀 FORGE_TASK_START
Working on implementing user authentication...
[... implementation details ...]
✅ FORGE_TASK_SUCCESS
📁 FORGE_FILES_MODIFIED: src/auth.rs, src/models/user.rs, tests/auth_test.rs
📋 FORGE_TASK_SUMMARY: Successfully implemented JWT authentication with user registration and login endpoints
🏁 FORGE_TASK_END
        "#;

        let markers = ResponseMarkers::default();
        let result = parse_claude_response(output, &markers).unwrap();
        
        assert!(result.success);
        assert_eq!(result.files_modified.len(), 3);
        assert!(result.files_modified.contains(&"src/auth.rs".to_string()));
        assert!(result.summary.contains("JWT authentication"));
    }

    #[test]
    fn test_parse_error_response() {
        let output = r#"
🚀 FORGE_TASK_START
Attempting to implement the feature...
❌ FORGE_TASK_ERROR: Unable to compile due to missing dependency
📋 FORGE_TASK_SUMMARY: Task failed due to compilation errors
🏁 FORGE_TASK_END
        "#;

        let markers = ResponseMarkers::default();
        let result = parse_claude_response(output, &markers).unwrap();
        
        assert!(!result.success);
        assert!(result.error_message.is_some());
        assert!(result.error_message.unwrap().contains("missing dependency"));
    }
}