// MCP Implementation Architecture for Forge

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

// Core MCP Server Structure
pub struct ForgeMCPServer {
    // Track active sessions and their state
    sessions: Arc<Mutex<HashMap<String, SessionState>>>,
    
    // Reference to Forge's core components
    project_manager: Arc<crate::project_config::ProjectConfigManager>,
    block_manager: Arc<crate::block_config::BlockConfigManager>,
    task_executor: Arc<crate::task_executor::TaskExecutor>,
    
    // Tool registry
    tools: HashMap<String, Box<dyn MCPTool>>,
}

// Session state tracking - Forge knows exactly what's happening
#[derive(Debug, Clone)]
pub struct SessionState {
    pub session_id: String,
    pub current_task: Option<String>,
    pub files_modified: Vec<String>,
    pub git_branch: Option<String>,
    pub progress: TaskProgress,
    pub last_activity: std::time::Instant,
}

#[derive(Debug, Clone)]
pub struct TaskProgress {
    pub stage: TaskStage,
    pub percentage: u8,
    pub description: String,
    pub actions_completed: Vec<ActionCompleted>,
}

#[derive(Debug, Clone)]
pub enum TaskStage {
    Planning,
    AnalyzingCode,
    Implementing,
    Testing,
    Committing,
    Completed,
    Failed(String),
}

#[derive(Debug, Clone)]
pub struct ActionCompleted {
    pub action_type: String,
    pub target: String,
    pub result: ActionResult,
    pub timestamp: std::time::SystemTime,
}

#[derive(Debug, Clone)]
pub enum ActionResult {
    Success(Value),
    Failed(String),
}

// MCP Tool trait - each tool provides specific functionality
pub trait MCPTool: Send + Sync {
    fn name(&self) -> &str;
    fn description(&self) -> &str;
    fn input_schema(&self) -> Value;
    fn execute(&self, params: Value, session: &mut SessionState) -> Result<MCPResponse, MCPError>;
}

// MCP Response structure - Forge always gets structured data back
#[derive(Debug, Serialize, Deserialize)]
pub struct MCPResponse {
    pub success: bool,
    pub content: Vec<ResponseContent>,
    pub session_updates: Option<SessionUpdate>,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum ResponseContent {
    Text { text: String },
    Data { data: Value },
    Error { error: String },
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SessionUpdate {
    pub progress: Option<TaskProgress>,
    pub files_modified: Option<Vec<String>>,
    pub git_status: Option<String>,
}

// Example Tool Implementation: File Operations
pub struct ForgeFileOperationsTool {
    project_manager: Arc<crate::project_config::ProjectConfigManager>,
}

impl MCPTool for ForgeFileOperationsTool {
    fn name(&self) -> &str {
        "forge_write_file"
    }

    fn description(&self) -> &str {
        "Create or modify a file in the project with automatic tracking"
    }

    fn input_schema(&self) -> Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "file_path": {"type": "string"},
                "content": {"type": "string"},
                "create_if_not_exists": {"type": "boolean", "default": true},
                "backup_original": {"type": "boolean", "default": true}
            },
            "required": ["file_path", "content"]
        })
    }

    fn execute(&self, params: Value, session: &mut SessionState) -> Result<MCPResponse, MCPError> {
        let file_path: String = params["file_path"].as_str()
            .ok_or_else(|| MCPError::InvalidParams("file_path required".to_string()))?
            .to_string();
        
        let content: String = params["content"].as_str()
            .ok_or_else(|| MCPError::InvalidParams("content required".to_string()))?
            .to_string();

        // Get project directory
        let project_config = self.project_manager.get_config()
            .map_err(|e| MCPError::Internal(format!("Failed to get project config: {}", e)))?;
        
        let full_path = std::path::Path::new(&project_config.project_home_directory).join(&file_path);

        // Backup original if it exists
        if full_path.exists() && params.get("backup_original").and_then(|v| v.as_bool()).unwrap_or(true) {
            let backup_path = format!("{}.backup", full_path.display());
            std::fs::copy(&full_path, &backup_path)
                .map_err(|e| MCPError::Internal(format!("Failed to backup file: {}", e)))?;
        }

        // Write the file
        std::fs::write(&full_path, &content)
            .map_err(|e| MCPError::Internal(format!("Failed to write file: {}", e)))?;

        // Update session state - Forge tracks exactly what happened
        session.files_modified.push(file_path.clone());
        session.progress.actions_completed.push(ActionCompleted {
            action_type: "file_write".to_string(),
            target: file_path.clone(),
            result: ActionResult::Success(serde_json::json!({
                "lines_written": content.lines().count(),
                "bytes_written": content.len(),
                "file_exists": full_path.exists()
            })),
            timestamp: std::time::SystemTime::now(),
        });

        // Update progress
        session.progress.percentage = std::cmp::min(session.progress.percentage + 10, 90);
        session.progress.description = format!("Modified file: {}", file_path);

        Ok(MCPResponse {
            success: true,
            content: vec![ResponseContent::Data { 
                data: serde_json::json!({
                    "file_path": file_path,
                    "action": if full_path.exists() { "modified" } else { "created" },
                    "lines": content.lines().count(),
                    "size_bytes": content.len()
                })
            }],
            session_updates: Some(SessionUpdate {
                progress: Some(session.progress.clone()),
                files_modified: Some(session.files_modified.clone()),
                git_status: None,
            }),
        })
    }
}

// Example Tool Implementation: Task Execution
pub struct ForgeTaskExecutionTool {
    task_executor: Arc<crate::task_executor::TaskExecutor>,
    block_manager: Arc<crate::block_config::BlockConfigManager>,
}

impl MCPTool for ForgeTaskExecutionTool {
    fn name(&self) -> &str {
        "forge_execute_task"
    }

    fn description(&self) -> &str {
        "Execute a specific task with real-time progress tracking"
    }

    fn input_schema(&self) -> Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "block_id": {"type": "string"},
                "task_id": {"type": "string"},
                "options": {
                    "type": "object",
                    "properties": {
                        "create_branch": {"type": "boolean", "default": true},
                        "run_tests": {"type": "boolean", "default": true},
                        "auto_commit": {"type": "boolean", "default": true}
                    }
                }
            },
            "required": ["block_id", "task_id"]
        })
    }

    fn execute(&self, params: Value, session: &mut SessionState) -> Result<MCPResponse, MCPError> {
        let block_id = params["block_id"].as_str()
            .ok_or_else(|| MCPError::InvalidParams("block_id required".to_string()))?;
        
        let task_id = params["task_id"].as_str()
            .ok_or_else(|| MCPError::InvalidParams("task_id required".to_string()))?;

        // Update session state - task execution started
        session.current_task = Some(format!("{}:{}", block_id, task_id));
        session.progress.stage = TaskStage::Planning;
        session.progress.percentage = 5;
        session.progress.description = format!("Starting execution of task {}", task_id);

        // Get task details
        let blocks = self.block_manager.get_blocks()
            .map_err(|e| MCPError::Internal(format!("Failed to get blocks: {}", e)))?;
        
        let block = blocks.iter().find(|b| b.block_id == block_id)
            .ok_or_else(|| MCPError::NotFound("Block not found".to_string()))?;
        
        let task = block.todo_list.get(task_id)
            .ok_or_else(|| MCPError::NotFound("Task not found".to_string()))?;

        // Instead of spawning Claude CLI, we return task context for Claude to work with interactively
        Ok(MCPResponse {
            success: true,
            content: vec![
                ResponseContent::Text { 
                    text: format!("Task execution prepared: {}", task.task_name) 
                },
                ResponseContent::Data { 
                    data: serde_json::json!({
                        "task": {
                            "id": task.task_id,
                            "name": task.task_name,
                            "description": task.description,
                            "acceptance_criteria": task.acceptance_criteria,
                            "files_affected": task.files_affected,
                            "dependencies": task.dependencies
                        },
                        "execution_context": {
                            "project_ready": true,
                            "branch_created": params["options"]["create_branch"].as_bool().unwrap_or(true),
                            "available_tools": [
                                "forge_read_file",
                                "forge_write_file", 
                                "forge_run_tests",
                                "forge_commit_changes"
                            ]
                        }
                    })
                }
            ],
            session_updates: Some(SessionUpdate {
                progress: Some(session.progress.clone()),
                files_modified: None,
                git_status: Some("branch_ready".to_string()),
            }),
        })
    }
}

// Error handling
#[derive(Debug)]
pub enum MCPError {
    InvalidParams(String),
    NotFound(String),
    Internal(String),
    PermissionDenied(String),
}

impl ForgeMCPServer {
    pub fn new(
        project_manager: Arc<crate::project_config::ProjectConfigManager>,
        block_manager: Arc<crate::block_config::BlockConfigManager>,
        task_executor: Arc<crate::task_executor::TaskExecutor>,
    ) -> Self {
        let mut server = Self {
            sessions: Arc::new(Mutex::new(HashMap::new())),
            project_manager: project_manager.clone(),
            block_manager: block_manager.clone(),
            task_executor,
            tools: HashMap::new(),
        };

        // Register all available tools
        server.register_tool(Box::new(ForgeFileOperationsTool {
            project_manager: project_manager.clone(),
        }));
        
        server.register_tool(Box::new(ForgeTaskExecutionTool {
            task_executor: server.task_executor.clone(),
            block_manager: block_manager.clone(),
        }));

        // Add more tools...

        server
    }

    fn register_tool(&mut self, tool: Box<dyn MCPTool>) {
        self.tools.insert(tool.name().to_string(), tool);
    }

    // Handle MCP tool calls - this is where Forge "understands" Claude
    pub fn handle_tool_call(
        &self, 
        session_id: &str, 
        tool_name: &str, 
        params: Value
    ) -> Result<MCPResponse, MCPError> {
        
        // Get or create session
        let mut sessions = self.sessions.lock().unwrap();
        let session = sessions.entry(session_id.to_string())
            .or_insert_with(|| SessionState {
                session_id: session_id.to_string(),
                current_task: None,
                files_modified: Vec::new(),
                git_branch: None,
                progress: TaskProgress {
                    stage: TaskStage::Planning,
                    percentage: 0,
                    description: "Session started".to_string(),
                    actions_completed: Vec::new(),
                },
                last_activity: std::time::Instant::now(),
            });

        // Update last activity
        session.last_activity = std::time::Instant::now();

        // Execute the tool
        if let Some(tool) = self.tools.get(tool_name) {
            tool.execute(params, session)
        } else {
            Err(MCPError::NotFound(format!("Tool {} not found", tool_name)))
        }
    }

    // Get current session state - Forge knows exactly what's happening
    pub fn get_session_state(&self, session_id: &str) -> Option<SessionState> {
        self.sessions.lock().unwrap().get(session_id).cloned()
    }

    // List all available tools for Claude to discover
    pub fn list_tools(&self) -> Vec<ToolInfo> {
        self.tools.iter().map(|(name, tool)| ToolInfo {
            name: name.clone(),
            description: tool.description().to_string(),
            input_schema: tool.input_schema(),
        }).collect()
    }
}

#[derive(Debug, Serialize)]
pub struct ToolInfo {
    pub name: String,
    pub description: String,
    pub input_schema: Value,
}

// Usage Example:
/*
// Instead of this (current CLI approach):
let output = Command::new("claude")
    .arg(task_prompt)
    .output()?;
// Parse text output and guess what happened 🤷‍♂️

// We get this (MCP approach):
let response = mcp_server.handle_tool_call(
    "session_123",
    "forge_execute_task", 
    json!({"block_id": "abc", "task_id": "xyz"})
)?;
// Forge knows exactly what happened ✅

// Claude then makes specific calls:
mcp_server.handle_tool_call("session_123", "forge_write_file", json!({
    "file_path": "src/auth.rs",
    "content": "..."
}))?; // Forge knows file was created

mcp_server.handle_tool_call("session_123", "forge_run_tests", json!({
    "scope": "auth"
}))?; // Forge knows test results

mcp_server.handle_tool_call("session_123", "forge_commit_changes", json!({
    "message": "Add authentication"
}))?; // Forge knows commit was made

// Final state is completely tracked:
let session = mcp_server.get_session_state("session_123");
println!("Files modified: {:?}", session.files_modified);
println!("Current stage: {:?}", session.progress.stage);
println!("Actions completed: {}", session.progress.actions_completed.len());
*/