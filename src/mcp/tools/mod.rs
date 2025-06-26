/// MCP Tools module - Comprehensive tool registry and execution system
///
/// This module provides the foundation for all MCP tools in Forge, including
/// tool registration, validation, execution, and result handling.

pub mod registry;
pub mod blocks;
pub mod filesystem;

// Re-export core tool types
pub use self::registry::ToolRegistry;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::time::{Duration, SystemTime};
use uuid::Uuid;

/// Core trait that all MCP tools must implement
#[async_trait]
pub trait MCPTool: Send + Sync {
    /// Get the tool name (unique identifier)
    fn name(&self) -> &str;

    /// Get the tool description for documentation
    fn description(&self) -> &str;

    /// Get the JSON schema for input parameters
    fn input_schema(&self) -> Value;

    /// Execute the tool with given parameters and context
    async fn execute(&self, params: Value, context: &mut ExecutionContext) -> Result<ToolResult, ToolError>;

    /// Get required permissions for this tool
    fn required_permissions(&self) -> Vec<Permission> {
        vec![]
    }

    /// Get tool category for organization
    fn category(&self) -> ToolCategory {
        ToolCategory::General
    }

    /// Get tool version
    fn version(&self) -> &str {
        "1.0.0"
    }

    /// Whether this tool supports parallel execution
    fn supports_parallel_execution(&self) -> bool {
        true
    }

    /// Estimated execution time (used for optimization)
    fn estimated_execution_time(&self) -> Duration {
        Duration::from_secs(5)
    }

    /// Validate parameters before execution
    fn validate_params(&self, params: &Value) -> Result<(), ToolError> {
        // Default implementation uses JSON schema validation
        // Individual tools can override for custom validation
        Ok(())
    }
}

/// Tool execution context containing all necessary information
#[derive(Debug, Clone)]
pub struct ExecutionContext {
    /// Unique session identifier
    pub session_id: String,

    /// Project configuration manager
    pub project_config: std::sync::Arc<crate::project_config::ProjectConfigManager>,

    /// Block configuration manager  
    pub block_manager: std::sync::Arc<crate::block_config::BlockConfigManager>,

    /// Current working directory
    pub working_directory: std::path::PathBuf,

    /// Context store for cross-tool data sharing
    pub context_store: std::sync::Arc<tokio::sync::RwLock<crate::mcp::context::ContextStore>>,

    /// Tool execution history for this session
    pub execution_history: Vec<ToolExecution>,

    /// User preferences and settings
    pub user_preferences: UserPreferences,

    /// Security permissions for this session
    pub permissions: SessionPermissions,

    /// Performance tracking
    pub performance_tracker: std::sync::Arc<tokio::sync::Mutex<PerformanceTracker>>,
}

/// Tool execution result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolResult {
    /// Whether the tool execution was successful
    pub success: bool,

    /// Content returned by the tool
    pub content: Vec<Content>,

    /// Optional context updates
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context_updates: Option<ContextUpdate>,

    /// Notifications to send to client
    #[serde(default)]
    pub notifications: Vec<Notification>,

    /// Execution metadata
    pub metadata: ExecutionMetadata,
}

/// Content types that tools can return
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Content {
    /// Plain text content
    #[serde(rename = "text")]
    Text { text: String },

    /// Structured data content
    #[serde(rename = "data")]
    Data { data: Value },

    /// Binary content (base64 encoded)
    #[serde(rename = "binary")]
    Binary {
        data: String,
        content_type: String,
    },

    /// Error content
    #[serde(rename = "error")]
    Error {
        error: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        details: Option<Value>,
    },

    /// File reference
    #[serde(rename = "file")]
    File {
        path: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        content_type: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        size: Option<u64>,
    },

    /// Progress update
    #[serde(rename = "progress")]
    Progress {
        percentage: f32,
        message: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        stage: Option<String>,
    },
}

/// Context updates that tools can make
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextUpdate {
    /// Files that were accessed or modified
    #[serde(skip_serializing_if = "Option::is_none")]
    pub files_accessed: Option<Vec<String>>,

    /// Files that were modified
    #[serde(skip_serializing_if = "Option::is_none")]
    pub files_modified: Option<Vec<String>>,

    /// Git status changes
    #[serde(skip_serializing_if = "Option::is_none")]
    pub git_status: Option<GitStatusUpdate>,

    /// Task status changes
    #[serde(skip_serializing_if = "Option::is_none")]
    pub task_updates: Option<Vec<TaskUpdate>>,

    /// Performance metrics
    #[serde(skip_serializing_if = "Option::is_none")]
    pub performance_metrics: Option<PerformanceMetrics>,

    /// Custom context data
    #[serde(skip_serializing_if = "Option::is_none")]
    pub custom_data: Option<HashMap<String, Value>>,
}

/// Git status update
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitStatusUpdate {
    pub current_branch: String,
    pub has_changes: bool,
    pub staged_files: Vec<String>,
    pub unstaged_files: Vec<String>,
    pub untracked_files: Vec<String>,
}

/// Task update information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskUpdate {
    pub task_id: String,
    pub block_id: String,
    pub status: String,
    pub progress: f32,
    pub message: String,
}

/// Performance metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    pub execution_time: Duration,
    pub memory_usage: u64,
    pub cpu_usage: f32,
    pub network_requests: u32,
    pub file_operations: u32,
}

/// Notifications that tools can send
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Notification {
    /// Information notification
    #[serde(rename = "info")]
    Info { message: String },

    /// Warning notification
    #[serde(rename = "warning")]
    Warning { message: String },

    /// Error notification
    #[serde(rename = "error")]
    Error { message: String },

    /// Progress notification
    #[serde(rename = "progress")]
    Progress {
        task_id: String,
        percentage: f32,
        message: String,
    },

    /// File change notification
    #[serde(rename = "file_changed")]
    FileChanged {
        path: String,
        change_type: String,
    },

    /// Custom notification
    #[serde(rename = "custom")]
    Custom {
        event: String,
        data: Value,
    },
}

/// Execution metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionMetadata {
    pub tool_name: String,
    pub execution_id: String,
    pub start_time: SystemTime,
    pub end_time: SystemTime,
    pub duration: Duration,
    pub session_id: String,
    pub parameters_hash: String,
    pub result_size: usize,
}

/// Tool execution record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolExecution {
    pub id: String,
    pub tool_name: String,
    pub parameters: Value,
    pub result: Option<ToolResult>,
    pub error: Option<String>,
    pub start_time: SystemTime,
    pub end_time: Option<SystemTime>,
    pub duration: Option<Duration>,
}

/// Tool error types
#[derive(Debug, thiserror::Error)]
pub enum ToolError {
    #[error("Tool not found: {0}")]
    NotFound(String),

    #[error("Invalid parameters: {0}")]
    InvalidParams(String),

    #[error("Execution failed: {0}")]
    ExecutionFailed(String),

    #[error("Timeout: tool execution exceeded {timeout_ms}ms")]
    Timeout { timeout_ms: u64 },

    #[error("Permission denied: {0}")]
    PermissionDenied(String),

    #[error("Resource limit exceeded: {0}")]
    ResourceLimit(String),

    #[error("Dependency error: {0}")]
    Dependency(String),

    #[error("File system error: {0}")]
    FileSystem(String),

    #[error("Git operation error: {0}")]
    Git(String),

    #[error("Network error: {0}")]
    Network(String),

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Internal error: {0}")]
    Internal(String),
}

/// Tool categories for organization
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ToolCategory {
    General,
    FileSystem,
    Git,
    Project,
    Tasks,
    Testing,
    CodeGeneration,
    Analysis,
    Collaboration,
    Monitoring,
    Performance,
}

/// Permission types for tool access control
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Permission {
    /// Read files from filesystem
    FileRead,
    /// Write files to filesystem
    FileWrite,
    /// Execute system commands
    Execute,
    /// Access network resources
    Network,
    /// Modify git repository
    Git,
    /// Modify project configuration
    ProjectConfig,
    /// Create/modify tasks
    TaskManagement,
    /// Access sensitive data
    Sensitive,
    /// Administrative privileges
    Admin,
}

/// User preferences
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserPreferences {
    pub preferred_editor: Option<String>,
    pub code_style: Option<String>,
    pub notification_level: NotificationLevel,
    pub auto_save: bool,
    pub parallel_execution: bool,
    pub max_parallel_tools: u32,
    pub custom_settings: HashMap<String, Value>,
}

impl Default for UserPreferences {
    fn default() -> Self {
        Self {
            preferred_editor: None,
            code_style: None,
            notification_level: NotificationLevel::Standard,
            auto_save: true,
            parallel_execution: true,
            max_parallel_tools: 4,
            custom_settings: HashMap::new(),
        }
    }
}

/// Notification levels
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum NotificationLevel {
    Minimal,
    Standard,
    Verbose,
}

/// Session permissions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionPermissions {
    pub granted_permissions: std::collections::HashSet<Permission>,
    pub restricted_paths: Vec<String>,
    pub max_file_size: u64,
    pub max_execution_time: Duration,
    pub max_memory_usage: u64,
}

impl Default for SessionPermissions {
    fn default() -> Self {
        let mut permissions = std::collections::HashSet::new();
        permissions.insert(Permission::FileRead);
        permissions.insert(Permission::FileWrite);
        permissions.insert(Permission::ProjectConfig);
        permissions.insert(Permission::TaskManagement);

        Self {
            granted_permissions: permissions,
            restricted_paths: vec![
                "/etc".to_string(),
                "/usr".to_string(),
                "/var".to_string(),
                "/root".to_string(),
            ],
            max_file_size: 100 * 1024 * 1024, // 100MB
            max_execution_time: Duration::from_secs(300), // 5 minutes
            max_memory_usage: 1024 * 1024 * 1024, // 1GB
        }
    }
}

/// Performance tracker for monitoring tool execution
#[derive(Debug)]
pub struct PerformanceTracker {
    pub executions: Vec<ToolExecution>,
    pub total_time: Duration,
    pub total_memory: u64,
    pub total_network_requests: u32,
    pub total_file_operations: u32,
}

impl Default for PerformanceTracker {
    fn default() -> Self {
        Self {
            executions: Vec::new(),
            total_time: Duration::from_secs(0),
            total_memory: 0,
            total_network_requests: 0,
            total_file_operations: 0,
        }
    }
}

impl PerformanceTracker {
    pub fn record_execution(&mut self, execution: ToolExecution) {
        if let Some(duration) = execution.duration {
            self.total_time += duration;
        }
        self.executions.push(execution);
    }

    pub fn get_average_execution_time(&self) -> Duration {
        if self.executions.is_empty() {
            return Duration::from_secs(0);
        }
        self.total_time / self.executions.len() as u32
    }

    pub fn get_tool_statistics(&self, tool_name: &str) -> ToolStatistics {
        let tool_executions: Vec<_> = self.executions
            .iter()
            .filter(|e| e.tool_name == tool_name)
            .collect();

        let total_executions = tool_executions.len();
        let successful_executions = tool_executions
            .iter()
            .filter(|e| e.result.is_some())
            .count();

        let total_time: Duration = tool_executions
            .iter()
            .filter_map(|e| e.duration)
            .sum();

        let average_time = if total_executions > 0 {
            total_time / total_executions as u32
        } else {
            Duration::from_secs(0)
        };

        ToolStatistics {
            tool_name: tool_name.to_string(),
            total_executions,
            successful_executions,
            failure_rate: if total_executions > 0 {
                (total_executions - successful_executions) as f32 / total_executions as f32
            } else {
                0.0
            },
            average_execution_time: average_time,
            total_execution_time: total_time,
        }
    }
}

/// Tool statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolStatistics {
    pub tool_name: String,
    pub total_executions: usize,
    pub successful_executions: usize,
    pub failure_rate: f32,
    pub average_execution_time: Duration,
    pub total_execution_time: Duration,
}

/// Helper trait for creating tool results
pub trait ToolResultBuilder {
    fn success() -> ToolResult;
    fn failure(error: impl Into<String>) -> ToolResult;
    fn with_content(self, content: Content) -> ToolResult;
    fn with_notification(self, notification: Notification) -> ToolResult;
    fn with_context_update(self, update: ContextUpdate) -> ToolResult;
}

impl ToolResultBuilder for ToolResult {
    fn success() -> ToolResult {
        ToolResult {
            success: true,
            content: vec![],
            context_updates: None,
            notifications: vec![],
            metadata: ExecutionMetadata {
                tool_name: "unknown".to_string(),
                execution_id: Uuid::new_v4().to_string(),
                start_time: SystemTime::now(),
                end_time: SystemTime::now(),
                duration: Duration::from_secs(0),
                session_id: "unknown".to_string(),
                parameters_hash: "unknown".to_string(),
                result_size: 0,
            },
        }
    }

    fn failure(error: impl Into<String>) -> ToolResult {
        ToolResult {
            success: false,
            content: vec![Content::Error {
                error: error.into(),
                details: None,
            }],
            context_updates: None,
            notifications: vec![],
            metadata: ExecutionMetadata {
                tool_name: "unknown".to_string(),
                execution_id: Uuid::new_v4().to_string(),
                start_time: SystemTime::now(),
                end_time: SystemTime::now(),
                duration: Duration::from_secs(0),
                session_id: "unknown".to_string(),
                parameters_hash: "unknown".to_string(),
                result_size: 0,
            },
        }
    }

    fn with_content(mut self, content: Content) -> ToolResult {
        self.content.push(content);
        self
    }

    fn with_notification(mut self, notification: Notification) -> ToolResult {
        self.notifications.push(notification);
        self
    }

    fn with_context_update(mut self, update: ContextUpdate) -> ToolResult {
        self.context_updates = Some(update);
        self
    }
}
