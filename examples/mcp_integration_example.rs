// Simplified MCP integration example for immediate improvement
use serde_json::{json, Value};
use std::process::{Command, Stdio};
use std::io::{Write, BufRead, BufReader};
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

pub struct ClaudeMCPSession {
    project_dir: String,
    session_id: String,
}

impl ClaudeMCPSession {
    pub fn new(project_dir: String) -> Self {
        let session_id = uuid::Uuid::new_v4().to_string();
        Self { project_dir, session_id }
    }

    pub fn execute_task_with_mcp_context(
        &self,
        task: &crate::models::Task,
        project_context: &ProjectContext,
    ) -> Result<TaskExecutionResult, Box<dyn std::error::Error>> {
        
        // Create MCP-style context file
        let context_file = self.create_context_file(task, project_context)?;
        
        // Use Claude with MCP context
        let mut cmd = Command::new("claude");
        cmd.args(&[
            "--dangerously-skip-permissions",
            "--mcp-server", "forge://localhost:8081", // Point to our MCP server
            "--context-file", &context_file,
            "--output-format", "structured",
        ]);
        cmd.current_dir(&self.project_dir);
        cmd.stdin(Stdio::piped());
        cmd.stdout(Stdio::piped());
        cmd.stderr(Stdio::piped());

        let mut child = cmd.spawn()?;
        
        // Enhanced prompt with structured output requirements
        let enhanced_prompt = self.create_enhanced_prompt(task, project_context);
        
        // Send prompt
        if let Some(stdin) = child.stdin.as_mut() {
            stdin.write_all(enhanced_prompt.as_bytes())?;
            stdin.flush()?;
        }

        // Collect output with timeout and parsing
        let (tx, rx) = mpsc::channel();
        
        // Read stdout
        if let Some(stdout) = child.stdout.take() {
            let tx_clone = tx.clone();
            thread::spawn(move || {
                let reader = BufReader::new(stdout);
                for line in reader.lines() {
                    if let Ok(line) = line {
                        let _ = tx_clone.send(OutputLine::Stdout(line));
                    }
                }
            });
        }

        // Read stderr
        if let Some(stderr) = child.stderr.take() {
            let tx_clone = tx.clone();
            thread::spawn(move || {
                let reader = BufReader::new(stderr);
                for line in reader.lines() {
                    if let Ok(line) = line {
                        let _ = tx_clone.send(OutputLine::Stderr(line));
                    }
                }
            });
        }

        // Collect output with intelligent parsing
        let mut result = TaskExecutionResult::default();
        let timeout = Duration::from_secs(600); // 10 minutes
        let start_time = std::time::Instant::now();
        
        while start_time.elapsed() < timeout {
            match rx.recv_timeout(Duration::from_secs(1)) {
                Ok(OutputLine::Stdout(line)) => {
                    result.add_output_line(&line);
                    
                    // Check for completion indicators
                    if let Some(status) = self.parse_completion_status(&line) {
                        result.completion_status = Some(status);
                        if status.is_final() {
                            break;
                        }
                    }
                    
                    // Check for file modifications
                    if let Some(files) = self.parse_file_modifications(&line) {
                        result.files_modified.extend(files);
                    }
                }
                Ok(OutputLine::Stderr(line)) => {
                    result.add_error_line(&line);
                }
                Err(mpsc::RecvTimeoutError::Timeout) => {
                    // Check if process is still running
                    match child.try_wait()? {
                        Some(status) => {
                            result.exit_status = Some(status);
                            break;
                        }
                        None => {
                            // Process still running, continue waiting
                            continue;
                        }
                    }
                }
                Err(_) => break,
            }
        }

        // Wait for process completion
        if let Ok(status) = child.wait() {
            result.exit_status = Some(status);
        }

        // Clean up context file
        let _ = std::fs::remove_file(&context_file);

        Ok(result)
    }

    fn create_context_file(
        &self,
        task: &crate::models::Task,
        project_context: &ProjectContext,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let context = json!({
            "session_id": self.session_id,
            "task": {
                "id": task.task_id,
                "name": task.task_name,
                "description": task.description,
                "acceptance_criteria": task.acceptance_criteria,
                "dependencies": task.dependencies,
                "files_affected": task.files_affected,
                "function_signatures": task.function_signatures,
                "testing_requirements": task.testing_requirements
            },
            "project": {
                "name": project_context.name,
                "root_directory": self.project_dir,
                "language": project_context.primary_language,
                "framework": project_context.framework,
                "existing_files": project_context.relevant_files,
                "architecture_patterns": project_context.patterns
            },
            "forge_integration": {
                "block_id": project_context.current_block_id,
                "connected_blocks": project_context.connected_blocks,
                "task_dependencies": task.dependencies,
                "expected_outputs": project_context.expected_outputs
            },
            "output_requirements": {
                "structured_response": true,
                "file_modification_tracking": true,
                "progress_reporting": true,
                "error_handling": true
            }
        });

        let context_file = format!("/tmp/forge_context_{}.json", self.session_id);
        std::fs::write(&context_file, serde_json::to_string_pretty(&context)?)?;
        
        Ok(context_file)
    }

    fn create_enhanced_prompt(
        &self,
        task: &crate::models::Task,
        _project_context: &ProjectContext,
    ) -> String {
        format!(
            r#"
# Forge IDE Task Execution

You are working as an AI assistant integrated with Forge IDE. Execute this task with proper status reporting.

## Task Details
{task_prompt}

## Critical Instructions for Forge Integration:

1. **Status Reporting**: Use these exact markers in your output:
   - `FORGE_STATUS:STARTED` - When you begin work
   - `FORGE_STATUS:ANALYZING` - When analyzing code/requirements  
   - `FORGE_STATUS:IMPLEMENTING` - When writing/modifying code
   - `FORGE_STATUS:TESTING` - When running tests
   - `FORGE_STATUS:SUCCESS` - When task completed successfully
   - `FORGE_STATUS:ERROR:reason` - If task fails

2. **File Tracking**: When you modify files, use:
   - `FORGE_FILE_MODIFIED:path/to/file.ext`
   - `FORGE_FILE_CREATED:path/to/file.ext`
   - `FORGE_FILE_DELETED:path/to/file.ext`

3. **Progress Updates**: Provide regular updates:
   - `FORGE_PROGRESS:percentage:description`
   - Example: `FORGE_PROGRESS:25:Analyzing existing code structure`

4. **Summary**: End with a structured summary:
   ```
   FORGE_SUMMARY_START
   Task: [task name]
   Status: [SUCCESS/ERROR]
   Files Modified: [list of files]
   Key Changes: [brief description]
   Next Steps: [if any]
   FORGE_SUMMARY_END
   ```

5. **Error Handling**: If you encounter issues:
   - Report specific errors with `FORGE_ERROR:description`
   - Suggest solutions where possible
   - Don't fail silently

Begin the task now and follow the integration requirements strictly.
"#,
            task_prompt = task.to_prompt()
        )
    }

    fn parse_completion_status(&self, line: &str) -> Option<CompletionStatus> {
        if line.contains("FORGE_STATUS:") {
            if line.contains("SUCCESS") {
                Some(CompletionStatus::Success)
            } else if line.contains("ERROR:") {
                let error = line.split("ERROR:").nth(1).unwrap_or("Unknown error").trim();
                Some(CompletionStatus::Error(error.to_string()))
            } else if line.contains("STARTED") {
                Some(CompletionStatus::Started)
            } else if line.contains("IMPLEMENTING") {
                Some(CompletionStatus::Implementing)
            } else if line.contains("TESTING") {
                Some(CompletionStatus::Testing)
            } else {
                None
            }
        } else {
            None
        }
    }

    fn parse_file_modifications(&self, line: &str) -> Option<Vec<String>> {
        let prefixes = ["FORGE_FILE_MODIFIED:", "FORGE_FILE_CREATED:", "FORGE_FILE_DELETED:"];
        
        for prefix in &prefixes {
            if line.contains(prefix) {
                if let Some(file_path) = line.split(prefix).nth(1) {
                    return Some(vec![file_path.trim().to_string()]);
                }
            }
        }
        None
    }
}

#[derive(Debug)]
enum OutputLine {
    Stdout(String),
    Stderr(String),
}

#[derive(Debug, Clone)]
pub enum CompletionStatus {
    Started,
    Analyzing,
    Implementing,
    Testing,
    Success,
    Error(String),
}

impl CompletionStatus {
    fn is_final(&self) -> bool {
        matches!(self, CompletionStatus::Success | CompletionStatus::Error(_))
    }
}

#[derive(Debug, Default)]
pub struct TaskExecutionResult {
    pub output_lines: Vec<String>,
    pub error_lines: Vec<String>,
    pub files_modified: Vec<String>,
    pub completion_status: Option<CompletionStatus>,
    pub exit_status: Option<std::process::ExitStatus>,
    pub progress_updates: Vec<(u32, String)>, // (percentage, description)
}

impl TaskExecutionResult {
    fn add_output_line(&mut self, line: &str) {
        self.output_lines.push(line.to_string());
        
        // Parse progress updates
        if line.contains("FORGE_PROGRESS:") {
            if let Some(progress_str) = line.split("FORGE_PROGRESS:").nth(1) {
                let parts: Vec<&str> = progress_str.splitn(2, ':').collect();
                if parts.len() == 2 {
                    if let Ok(percentage) = parts[0].trim().parse::<u32>() {
                        self.progress_updates.push((percentage, parts[1].trim().to_string()));
                    }
                }
            }
        }
    }
    
    fn add_error_line(&mut self, line: &str) {
        self.error_lines.push(line.to_string());
    }

    pub fn is_successful(&self) -> bool {
        matches!(self.completion_status, Some(CompletionStatus::Success)) ||
        (self.exit_status.as_ref().map_or(false, |s| s.success()) && 
         !matches!(self.completion_status, Some(CompletionStatus::Error(_))))
    }

    pub fn get_summary(&self) -> String {
        // Extract summary between FORGE_SUMMARY_START and FORGE_SUMMARY_END
        let full_output = self.output_lines.join("\n");
        
        if let Some(start) = full_output.find("FORGE_SUMMARY_START") {
            if let Some(end) = full_output.find("FORGE_SUMMARY_END") {
                let summary_section = &full_output[start + "FORGE_SUMMARY_START".len()..end];
                return summary_section.trim().to_string();
            }
        }
        
        // Fallback: use last few lines
        if self.output_lines.len() > 5 {
            self.output_lines[self.output_lines.len()-5..].join("\n")
        } else {
            self.output_lines.join("\n")
        }
    }
}

#[derive(Debug)]
pub struct ProjectContext {
    pub name: String,
    pub primary_language: String,
    pub framework: Option<String>,
    pub relevant_files: Vec<String>,
    pub patterns: Vec<String>,
    pub current_block_id: String,
    pub connected_blocks: Vec<String>,
    pub expected_outputs: Vec<String>,
}