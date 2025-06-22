use serde::{Deserialize, Serialize};
// Enhanced task executor with better Claude integration
use std::io::{BufRead, BufReader, Write};
use std::process::{Command, Stdio};
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

use crate::log_stream;
use crate::models::Task;

#[derive(Debug, Serialize, Deserialize)]
pub struct EnhancedTaskExecution {
    pub task_id: String,
    pub success: bool,
    pub files_modified: Vec<String>,
    pub summary: String,
    pub error_message: Option<String>,
    pub execution_log: Vec<String>,
}

impl crate::task_executor::TaskExecutor {
    /// Enhanced version of execute_git_task with better Claude integration
    pub fn execute_git_task_enhanced(
        &self,
        block_id: &String,
        task_id: &String,
    ) -> Result<(String, String), String> {
        
        // Create a unique task ID for logging
        let log_task_id = format!("{}:{}", block_id, task_id);

        // Get the project configuration and task details (existing code)
        let project_config = match self.project_manager.get_config() {
            Ok(config) => config,
            Err(_) => return Err("Failed to get project configuration".to_string()),
        };

        let main_branch = &project_config.main_branch.unwrap_or("main".to_string());
        let project_dir = project_config.project_home_directory.clone();
        
        if project_dir.is_empty() {
            return Err("Project home directory is not set. Please configure it in the project settings.".to_string());
        }

        let mut blocks = self.block_manager.get_blocks()
            .map_err(|e| format!("Failed to get blocks: {}", e))?;

        let block = blocks.iter_mut()
            .find(|b| b.block_id == *block_id)
            .ok_or("Block not found")?;

        let task = block.todo_list.get(task_id)
            .ok_or("Task not found")?;

        // Clear any existing logs for this task
        log_stream::clear_logs(&log_task_id);

        // Step 1: Setup Git branch (existing code)
        self.setup_git_branch(&log_task_id, task_id, main_branch, &project_dir)?;

        // Step 2: Execute task with enhanced Claude integration
        let execution_result = self.execute_claude_task_enhanced(task, &project_dir, &log_task_id)?;

        // Step 3: Process results and commit if successful
        if execution_result.success {
            let commit_id = self.commit_changes(&log_task_id, task, &project_dir)?;
            self.merge_and_cleanup(&log_task_id, task_id, main_branch, &project_dir)?;
            
            // Update task with results
            self.update_task_with_results(block_id, task_id, &execution_result, &commit_id)?;
            
            Ok((execution_result.summary, commit_id))
        } else {
            let error_msg = execution_result.error_message
                .unwrap_or_else(|| "Task execution failed".to_string());
            Err(error_msg)
        }
    }

    fn execute_claude_task_enhanced(
        &self,
        task: &Task,
        project_dir: &str,
        log_task_id: &str,
    ) -> Result<EnhancedTaskExecution, String> {
        
        // Create enhanced prompt with structured output requirements
        let enhanced_prompt = self.create_enhanced_task_prompt(task, project_dir)?;
        
        log_stream::add_log(log_task_id, "Starting enhanced Claude execution...".to_string());

        // Setup Claude command with better configuration
        let mut cmd = Command::new("claude");
        cmd.args(&["--dangerously-skip-permissions"]);
        cmd.current_dir(project_dir);
        cmd.stdin(Stdio::piped());
        cmd.stdout(Stdio::piped());
        cmd.stderr(Stdio::piped());
        
        // Set environment variables for better output
        cmd.env("TERM", "xterm-256color");
        cmd.env("COLUMNS", "120");
        cmd.env("LINES", "40");

        let mut child = cmd.spawn()
            .map_err(|e| format!("Failed to spawn Claude process: {}", e))?;

        // Send the enhanced prompt
        if let Some(mut stdin) = child.stdin.take() {
            stdin.write_all(enhanced_prompt.as_bytes())
                .map_err(|e| format!("Failed to write prompt to Claude: {}", e))?;
            stdin.flush()
                .map_err(|e| format!("Failed to flush stdin: {}", e))?;
        }

        // Collect output with intelligent parsing
        let (tx, rx) = mpsc::channel();
        
        // Read stdout in a separate thread
        if let Some(stdout) = child.stdout.take() {
            let tx_clone = tx.clone();
            let log_task_id_clone = log_task_id.to_string();
            thread::spawn(move || {
                let reader = BufReader::new(stdout);
                for line in reader.lines() {
                    if let Ok(line) = line {
                        log_stream::add_log(&log_task_id_clone, format!("Claude: {}", line));
                        let _ = tx_clone.send(ProcessOutput::Stdout(line));
                    }
                }
            });
        }

        // Read stderr in a separate thread
        if let Some(stderr) = child.stderr.take() {
            let tx_clone = tx.clone();
            let log_task_id_clone = log_task_id.to_string();
            thread::spawn(move || {
                let reader = BufReader::new(stderr);
                for line in reader.lines() {
                    if let Ok(line) = line {
                        log_stream::add_log(&log_task_id_clone, format!("Claude Error: {}", line));
                        let _ = tx_clone.send(ProcessOutput::Stderr(line));
                    }
                }
            });
        }

        // Parse output with timeout and intelligence
        let mut parser = ClaudeOutputParser::new(task.task_id.clone());
        let timeout = Duration::from_secs(600); // 10 minutes
        let start_time = std::time::Instant::now();

        while start_time.elapsed() < timeout {
            match rx.recv_timeout(Duration::from_secs(1)) {
                Ok(ProcessOutput::Stdout(line)) => {
                    parser.process_line(&line);
                    
                    // Check if we have a completion signal
                    if parser.is_complete() {
                        break;
                    }
                }
                Ok(ProcessOutput::Stderr(line)) => {
                    parser.add_error(&line);
                }
                Err(mpsc::RecvTimeoutError::Timeout) => {
                    // Check if process is still running
                    match child.try_wait() {
                        Ok(Some(status)) => {
                            parser.set_exit_status(status);
                            break;
                        }
                        Ok(None) => {
                            // Process still running, check for inactivity
                            if parser.last_activity_elapsed() > Duration::from_secs(60) {
                                log_stream::add_log(log_task_id, "Claude seems inactive, sending prompt...".to_string());
                                // Could send a gentle prompt here if we had stdin access
                            }
                            continue;
                        }
                        Err(e) => {
                            return Err(format!("Error checking process status: {}", e));
                        }
                    }
                }
                Err(_) => break,
            }
        }

        // Wait for process to finish
        let exit_status = child.wait()
            .map_err(|e| format!("Failed to wait for Claude process: {}", e))?;
        
        parser.set_exit_status(exit_status);

        // Generate execution result
        let result = parser.into_result();
        
        log_stream::add_log(log_task_id, format!("Claude execution completed: success={}", result.success));
        
        Ok(result)
    }

    fn create_enhanced_task_prompt(&self, task: &Task, project_dir: &str) -> Result<String, String> {
        // Analyze project structure for context
        let project_context = self.analyze_project_context(project_dir)?;
        
        let prompt = format!(
            r#"
# Forge IDE - Task Execution

You are working within Forge IDE, executing a specific development task. Please follow the structured output format for proper integration.

## Task Information
**Task ID**: {task_id}
**Task Name**: {task_name}
**Description**: {description}

## Acceptance Criteria
{acceptance_criteria}

## Project Context
**Project Directory**: {project_dir}
**Project Type**: {project_type}
**Key Files**: {key_files}
**Architecture Patterns**: {patterns}

## Files to Consider
{files_affected}

## Dependencies
{dependencies}

## Testing Requirements
{testing_requirements}

## CRITICAL: Output Format for Forge Integration

Please use these exact markers in your output for proper tracking:

1. **Start**: `🚀 FORGE_TASK_START`
2. **Progress**: `📊 FORGE_PROGRESS: [percentage]% - [description]`
3. **File Operations**: 
   - `📝 FORGE_FILE_MODIFIED: [filepath]`
   - `🆕 FORGE_FILE_CREATED: [filepath]`
   - `🗑️ FORGE_FILE_DELETED: [filepath]`
4. **Success**: `✅ FORGE_TASK_SUCCESS`
5. **Error**: `❌ FORGE_TASK_ERROR: [error description]`
6. **Summary**: `📋 FORGE_SUMMARY: [brief summary of what was accomplished]`
7. **End**: `🏁 FORGE_TASK_END`

## Example Output:
```
🚀 FORGE_TASK_START
📊 FORGE_PROGRESS: 10% - Analyzing existing code structure
📊 FORGE_PROGRESS: 30% - Implementing core functionality
📝 FORGE_FILE_MODIFIED: src/main.rs
🆕 FORGE_FILE_CREATED: src/new_module.rs
📊 FORGE_PROGRESS: 80% - Running tests
✅ FORGE_TASK_SUCCESS
📋 FORGE_SUMMARY: Successfully implemented authentication module with JWT support and comprehensive tests
🏁 FORGE_TASK_END
```

## Important Notes:
- Always include the markers exactly as shown
- Provide regular progress updates
- List every file you modify or create
- Give a clear summary of what was accomplished
- If you encounter errors, explain them clearly

Begin the task implementation now.
"#,
            task_id = task.task_id,
            task_name = task.task_name,
            description = task.description,
            acceptance_criteria = task.acceptance_criteria.iter()
                .enumerate()
                .map(|(i, criteria)| format!("{}. {}", i + 1, criteria))
                .collect::<Vec<_>>()
                .join("\n"),
            project_dir = project_dir,
            project_type = project_context.project_type,
            key_files = project_context.key_files.join(", "),
            patterns = project_context.patterns.join(", "),
            files_affected = if task.files_affected.is_empty() {
                "No specific files - determine based on requirements".to_string()
            } else {
                task.files_affected.iter()
                    .map(|f| format!("- {}", f))
                    .collect::<Vec<_>>()
                    .join("\n")
            },
            dependencies = if task.dependencies.is_empty() {
                "No specific dependencies".to_string()
            } else {
                task.dependencies.iter()
                    .map(|d| format!("- {}", d))
                    .collect::<Vec<_>>()
                    .join("\n")
            },
            testing_requirements = if task.testing_requirements.is_empty() {
                "No specific testing requirements".to_string()
            } else {
                task.testing_requirements.iter()
                    .enumerate()
                    .map(|(i, req)| format!("{}. {}", i + 1, req))
                    .collect::<Vec<_>>()
                    .join("\n")
            }
        );

        Ok(prompt)
    }

    fn analyze_project_context(&self, project_dir: &str) -> Result<ProjectContext, String> {
        // Simple project analysis - can be enhanced later
        let mut context = ProjectContext {
            project_type: "Unknown".to_string(),
            key_files: Vec::new(),
            patterns: Vec::new(),
        };

        // Check for common project files
        let project_path = std::path::Path::new(project_dir);
        
        if project_path.join("Cargo.toml").exists() {
            context.project_type = "Rust".to_string();
            context.key_files.push("Cargo.toml".to_string());
            context.patterns.push("Rust project with cargo".to_string());
        }
        
        if project_path.join("package.json").exists() {
            context.project_type = "JavaScript/Node.js".to_string();
            context.key_files.push("package.json".to_string());
            context.patterns.push("Node.js project with npm".to_string());
        }

        if project_path.join("src").exists() {
            context.key_files.push("src/".to_string());
        }

        Ok(context)
    }

    // Helper methods for git operations (simplified versions of existing code)
    fn setup_git_branch(&self, log_task_id: &str, task_id: &str, main_branch: &str, project_dir: &str) -> Result<(), String> {
        // Implementation similar to existing code but with better error handling
        log_stream::add_log(log_task_id, format!("Setting up git branch for task {}", task_id));
        // ... existing git setup code ...
        Ok(())
    }

    fn commit_changes(&self, log_task_id: &str, task: &Task, project_dir: &str) -> Result<String, String> {
        // Implementation similar to existing code
        log_stream::add_log(log_task_id, "Committing changes...".to_string());
        // ... existing commit code ...
        Ok("dummy_commit_id".to_string()) // Replace with actual implementation
    }

    fn merge_and_cleanup(&self, log_task_id: &str, task_id: &str, main_branch: &str, project_dir: &str) -> Result<(), String> {
        // Implementation similar to existing code
        log_stream::add_log(log_task_id, "Merging and cleaning up...".to_string());
        // ... existing merge code ...
        Ok(())
    }

    fn update_task_with_results(&self, block_id: &str, task_id: &str, result: &EnhancedTaskExecution, commit_id: &str) -> Result<(), String> {
        // Update task with enhanced results
        self.update_task_and_save(
            block_id,
            task_id,
            if result.success { "[COMPLETED]" } else { "[FAILED]" },
            &result.summary,
            commit_id.to_string(),
        )
    }
}

#[derive(Debug)]
enum ProcessOutput {
    Stdout(String),
    Stderr(String),
}

struct ProjectContext {
    project_type: String,
    key_files: Vec<String>,
    patterns: Vec<String>,
}

struct ClaudeOutputParser {
    task_id: String,
    lines: Vec<String>,
    errors: Vec<String>,
    files_modified: Vec<String>,
    progress_updates: Vec<String>,
    has_started: bool,
    has_ended: bool,
    has_success: bool,
    has_error: bool,
    summary: Option<String>,
    last_activity: std::time::Instant,
    exit_status: Option<std::process::ExitStatus>,
}

impl ClaudeOutputParser {
    fn new(task_id: String) -> Self {
        Self {
            task_id,
            lines: Vec::new(),
            errors: Vec::new(),
            files_modified: Vec::new(),
            progress_updates: Vec::new(),
            has_started: false,
            has_ended: false,
            has_success: false,
            has_error: false,
            summary: None,
            last_activity: std::time::Instant::now(),
            exit_status: None,
        }
    }

    fn process_line(&mut self, line: &str) {
        self.last_activity = std::time::Instant::now();
        self.lines.push(line.to_string());

        // Parse special markers
        if line.contains("🚀 FORGE_TASK_START") {
            self.has_started = true;
        } else if line.contains("🏁 FORGE_TASK_END") {
            self.has_ended = true;
        } else if line.contains("✅ FORGE_TASK_SUCCESS") {
            self.has_success = true;
        } else if line.contains("❌ FORGE_TASK_ERROR:") {
            self.has_error = true;
        } else if line.contains("📝 FORGE_FILE_MODIFIED:") || 
                  line.contains("🆕 FORGE_FILE_CREATED:") || 
                  line.contains("🗑️ FORGE_FILE_DELETED:") {
            if let Some(file) = self.extract_file_path(line) {
                self.files_modified.push(file);
            }
        } else if line.contains("📊 FORGE_PROGRESS:") {
            self.progress_updates.push(line.to_string());
        } else if line.contains("📋 FORGE_SUMMARY:") {
            self.summary = Some(line.replace("📋 FORGE_SUMMARY:", "").trim().to_string());
        }
    }

    fn add_error(&mut self, error: &str) {
        self.errors.push(error.to_string());
    }

    fn extract_file_path(&self, line: &str) -> Option<String> {
        let markers = ["📝 FORGE_FILE_MODIFIED:", "🆕 FORGE_FILE_CREATED:", "🗑️ FORGE_FILE_DELETED:"];
        
        for marker in &markers {
            if let Some(pos) = line.find(marker) {
                let file_part = &line[pos + marker.len()..];
                return Some(file_part.trim().to_string());
            }
        }
        None
    }

    fn is_complete(&self) -> bool {
        (self.has_started && self.has_ended) || 
        (self.has_success || self.has_error)
    }

    fn last_activity_elapsed(&self) -> Duration {
        self.last_activity.elapsed()
    }

    fn set_exit_status(&mut self, status: std::process::ExitStatus) {
        self.exit_status = Some(status);
    }

    fn into_result(self) -> EnhancedTaskExecution {
        let success = self.has_success && !self.has_error && 
                     self.exit_status.as_ref().map_or(true, |s| s.success());
        
        let error_message = if self.has_error || !self.errors.is_empty() {
            Some(self.errors.join("; "))
        } else {
            None
        };

        let summary = self.summary.unwrap_or_else(|| {
            if success {
                "Task completed successfully".to_string()
            } else {
                "Task execution failed".to_string()
            }
        });

        EnhancedTaskExecution {
            task_id: self.task_id,
            success,
            files_modified: self.files_modified,
            summary,
            error_message,
            execution_log: self.lines,
        }
    }
}