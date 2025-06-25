use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime};
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};
use uuid::Uuid;

use crate::mcp::errors::{MCPError, MCPResult};
use crate::mcp::tools::{
    Content, ExecutionContext, MCPTool, PerformanceTracker, Permission, ToolCategory,
    ToolError, ToolExecution, ToolResult, ToolStatistics,
};

/// Tool registry for managing and executing MCP tools
pub struct ToolRegistry {
    /// Registered tools indexed by name
    tools: Arc<RwLock<HashMap<String, Box<dyn MCPTool>>>>,

    /// Tool metadata and configuration
    metadata: Arc<RwLock<HashMap<String, ToolMetadata>>>,

    /// Tool execution statistics
    statistics: Arc<RwLock<HashMap<String, ToolStatistics>>>,

    /// Configuration for the registry
    config: ToolRegistryConfig,

    /// Performance tracker
    performance_tracker: Arc<RwLock<PerformanceTracker>>,
}

/// Tool metadata for registration and discovery
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolMetadata {
    pub name: String,
    pub description: String,
    pub version: String,
    pub category: ToolCategory,
    pub input_schema: Value,
    pub required_permissions: Vec<Permission>,
    pub supports_parallel: bool,
    pub estimated_duration: Duration,
    pub tags: Vec<String>,
    pub author: Option<String>,
    pub documentation_url: Option<String>,
    pub examples: Vec<ToolExample>,
    pub registration_time: SystemTime,
    pub last_used: Option<SystemTime>,
    pub usage_count: u64,
}

/// Tool example for documentation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolExample {
    pub name: String,
    pub description: String,
    pub parameters: Value,
    pub expected_result: Option<Value>,
}

/// Tool information for MCP client discovery
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolInfo {
    pub name: String,
    pub description: String,
    #[serde(rename = "inputSchema")]
    pub input_schema: Value,
}

/// Tool registry configuration
#[derive(Debug, Clone)]
pub struct ToolRegistryConfig {
    /// Maximum number of concurrent tool executions
    pub max_concurrent_executions: usize,

    /// Default timeout for tool execution
    pub default_timeout: Duration,

    /// Whether to enable performance tracking
    pub enable_performance_tracking: bool,

    /// Whether to enable caching of tool results
    pub enable_result_caching: bool,

    /// Maximum size of result cache
    pub max_cache_size: usize,

    /// Cache TTL
    pub cache_ttl: Duration,

    /// Whether to validate tool parameters against schema
    pub validate_parameters: bool,
}

impl Default for ToolRegistryConfig {
    fn default() -> Self {
        Self {
            max_concurrent_executions: 8,
            default_timeout: Duration::from_secs(300), // 5 minutes
            enable_performance_tracking: true,
            enable_result_caching: true,
            max_cache_size: 1000,
            cache_ttl: Duration::from_secs(3600), // 1 hour
            validate_parameters: true,
        }
    }
}

/// Result cache entry
#[derive(Debug, Clone)]
struct CacheEntry {
    result: ToolResult,
    created_at: Instant,
    access_count: u64,
}

/// Tool registry implementation
impl ToolRegistry {
    /// Create a new tool registry with default configuration
    pub fn new() -> Self {
        Self::with_config(ToolRegistryConfig::default())
    }

    /// Create a new tool registry with custom configuration
    pub fn with_config(config: ToolRegistryConfig) -> Self {
        Self {
            tools: Arc::new(RwLock::new(HashMap::new())),
            metadata: Arc::new(RwLock::new(HashMap::new())),
            statistics: Arc::new(RwLock::new(HashMap::new())),
            config,
            performance_tracker: Arc::new(RwLock::new(PerformanceTracker::default())),
        }
    }

    /// Register a new tool
    pub async fn register_tool(&self, tool: Box<dyn MCPTool>) -> MCPResult<()> {
        let name = tool.name().to_string();

        info!("Registering tool: {}", name);

        // Validate tool
        self.validate_tool(&*tool).await?;

        // Create metadata
        let metadata = ToolMetadata {
            name: name.clone(),
            description: tool.description().to_string(),
            version: tool.version().to_string(),
            category: tool.category(),
            input_schema: tool.input_schema(),
            required_permissions: tool.required_permissions(),
            supports_parallel: tool.supports_parallel_execution(),
            estimated_duration: tool.estimated_execution_time(),
            tags: vec![], // TODO: Extract from tool or configuration
            author: None, // TODO: Extract from tool or configuration
            documentation_url: None, // TODO: Extract from tool or configuration
            examples: vec![], // TODO: Extract from tool or configuration
            registration_time: SystemTime::now(),
            last_used: None,
            usage_count: 0,
        };

        // Register tool and metadata
        {
            let mut tools = self.tools.write().await;
            let mut metadata_map = self.metadata.write().await;

            tools.insert(name.clone(), tool);
            metadata_map.insert(name.clone(), metadata);
        }

        debug!("Tool registered successfully: {}", name);
        Ok(())
    }

    /// Unregister a tool
    pub async fn unregister_tool(&self, name: &str) -> MCPResult<()> {
        info!("Unregistering tool: {}", name);

        let mut tools = self.tools.write().await;
        let mut metadata_map = self.metadata.write().await;

        if tools.remove(name).is_none() {
            return Err(MCPError::ToolExecution(crate::mcp::errors::ToolError::NotFound(name.to_string())));
        }

        metadata_map.remove(name);

        debug!("Tool unregistered successfully: {}", name);
        Ok(())
    }

    /// Get a tool by name
    pub async fn get_tool(&self, name: &str) -> Option<Box<dyn MCPTool + '_>> {
        // This is challenging due to borrowing rules
        // In practice, we'd need a different approach or use Arc<dyn MCPTool>
        None
    }

    /// List all available tools
    pub async fn list_tools(&self) -> Vec<ToolInfo> {
        let metadata = self.metadata.read().await;

        metadata
            .values()
            .map(|meta| ToolInfo {
                name: meta.name.clone(),
                description: meta.description.clone(),
                input_schema: meta.input_schema.clone(),
            })
            .collect()
    }

    /// Get tool metadata
    pub async fn get_tool_metadata(&self, name: &str) -> Option<ToolMetadata> {
        self.metadata.read().await.get(name).cloned()
    }

    /// Get tools by category
    pub async fn get_tools_by_category(&self, category: ToolCategory) -> Vec<ToolInfo> {
        let metadata = self.metadata.read().await;

        metadata
            .values()
            .filter(|meta| meta.category == category)
            .map(|meta| ToolInfo {
                name: meta.name.clone(),
                description: meta.description.clone(),
                input_schema: meta.input_schema.clone(),
            })
            .collect()
    }

    /// Search tools by name or description
    pub async fn search_tools(&self, query: &str) -> Vec<ToolInfo> {
        let metadata = self.metadata.read().await;
        let query_lower = query.to_lowercase();

        metadata
            .values()
            .filter(|meta| {
                meta.name.to_lowercase().contains(&query_lower)
                    || meta.description.to_lowercase().contains(&query_lower)
                    || meta.tags.iter().any(|tag| tag.to_lowercase().contains(&query_lower))
            })
            .map(|meta| ToolInfo {
                name: meta.name.clone(),
                description: meta.description.clone(),
                input_schema: meta.input_schema.clone(),
            })
            .collect()
    }

    /// Execute a tool with given parameters and context
    pub async fn execute_tool(
        &self,
        name: &str,
        params: Value,
        context: &mut ExecutionContext,
    ) -> Result<ToolResult, ToolError> {
        let start_time = Instant::now();
        let execution_id = Uuid::new_v4().to_string();

        debug!("Executing tool: {} with ID: {}", name, execution_id);

        // Check if tool exists
        let tools = self.tools.read().await;
        let tool = tools.get(name).ok_or_else(|| ToolError::NotFound(name.to_string()))?;

        // Validate parameters if enabled
        if self.config.validate_parameters {
            tool.validate_params(&params)?;
        }

        // Check permissions
        self.check_permissions(tool.as_ref(), &context.permissions).await?;

        // Create tool execution record
        let mut execution = ToolExecution {
            id: execution_id.clone(),
            tool_name: name.to_string(),
            parameters: params.clone(),
            result: None,
            error: None,
            start_time: SystemTime::now(),
            end_time: None,
            duration: None,
        };

        // Execute the tool
        let result = match tokio::time::timeout(
            self.config.default_timeout,
            tool.execute(params, context),
        ).await {
            Ok(Ok(mut result)) => {
                // Update execution metadata
                result.metadata.tool_name = name.to_string();
                result.metadata.execution_id = execution_id;
                result.metadata.start_time = execution.start_time;
                result.metadata.end_time = SystemTime::now();
                result.metadata.duration = start_time.elapsed();
                result.metadata.session_id = context.session_id.clone();
                result.metadata.parameters_hash = self.hash_parameters(&execution.parameters);
                result.metadata.result_size = self.calculate_result_size(&result);

                execution.result = Some(result.clone());
                execution.end_time = Some(SystemTime::now());
                execution.duration = Some(start_time.elapsed());

                Ok(result)
            }
            Ok(Err(e)) => {
                execution.error = Some(e.to_string());
                execution.end_time = Some(SystemTime::now());
                execution.duration = Some(start_time.elapsed());

                Err(e)
            }
            Err(_) => {
                let timeout_error = ToolError::Timeout {
                    timeout_ms: self.config.default_timeout.as_millis() as u64,
                };
                execution.error = Some(timeout_error.to_string());
                execution.end_time = Some(SystemTime::now());
                execution.duration = Some(start_time.elapsed());

                Err(timeout_error)
            }
        };

        // Update statistics
        self.update_statistics(name, &execution).await;

        // Record execution in context
        context.execution_history.push(execution.clone());

        // Record in performance tracker
        if self.config.enable_performance_tracking {
            let mut tracker = context.performance_tracker.lock().await;
            tracker.record_execution(execution);
        }

        // Update tool metadata usage
        self.update_tool_usage(name).await;

        result
    }

    /// Execute multiple tools in parallel
    pub async fn execute_tools_parallel(
        &self,
        tool_calls: Vec<(String, Value)>,
        context: &mut ExecutionContext,
    ) -> Vec<Result<ToolResult, ToolError>> {
        let max_parallel = self.config.max_concurrent_executions.min(tool_calls.len());
        let semaphore = Arc::new(tokio::sync::Semaphore::new(max_parallel));

        let mut handles = Vec::new();

        for (tool_name, params) in tool_calls {
            let registry = self.clone_for_parallel(); // We'd need to implement this
            let semaphore = semaphore.clone();
            let mut context_clone = context.clone();

            let handle = tokio::spawn(async move {
                let _permit = semaphore.acquire().await.unwrap();
                registry.execute_tool(&tool_name, params, &mut context_clone).await
            });

            handles.push(handle);
        }

        let mut results = Vec::new();
        for handle in handles {
            match handle.await {
                Ok(result) => results.push(result),
                Err(e) => results.push(Err(ToolError::Internal(e.to_string()))),
            }
        }

        results
    }

    /// Get tool statistics
    pub async fn get_tool_statistics(&self, name: &str) -> Option<ToolStatistics> {
        self.statistics.read().await.get(name).cloned()
    }

    /// Get all tool statistics
    pub async fn get_all_statistics(&self) -> HashMap<String, ToolStatistics> {
        self.statistics.read().await.clone()
    }

    /// Validate a tool before registration
    async fn validate_tool(&self, tool: &dyn MCPTool) -> MCPResult<()> {
        // Validate tool name
        let name = tool.name();
        if name.is_empty() {
            return Err(MCPError::Validation("Tool name cannot be empty".to_string()));
        }

        // Check if tool is already registered
        let tools = self.tools.read().await;
        if tools.contains_key(name) {
            return Err(MCPError::Validation(format!("Tool '{}' is already registered", name)));
        }

        // Validate input schema
        let schema = tool.input_schema();
        if !schema.is_object() {
            return Err(MCPError::Validation("Tool input schema must be a JSON object".to_string()));
        }

        Ok(())
    }

    /// Check if session has required permissions for tool
    async fn check_permissions(
        &self,
        tool: &dyn MCPTool,
        session_permissions: &crate::mcp::tools::SessionPermissions,
    ) -> Result<(), ToolError> {
        let required = tool.required_permissions();

        for permission in required {
            if !session_permissions.granted_permissions.contains(&permission) {
                return Err(ToolError::PermissionDenied(
                    format!("Missing required permission: {:?}", permission)
                ));
            }
        }

        Ok(())
    }

    /// Update tool statistics
    async fn update_statistics(&self, tool_name: &str, execution: &ToolExecution) {
        let mut stats = self.statistics.write().await;

        let tool_stats = stats.entry(tool_name.to_string()).or_insert_with(|| {
            ToolStatistics {
                tool_name: tool_name.to_string(),
                total_executions: 0,
                successful_executions: 0,
                failure_rate: 0.0,
                average_execution_time: Duration::from_secs(0),
                total_execution_time: Duration::from_secs(0),
            }
        });

        tool_stats.total_executions += 1;

        if execution.result.is_some() {
            tool_stats.successful_executions += 1;
        }

        if let Some(duration) = execution.duration {
            tool_stats.total_execution_time += duration;
            tool_stats.average_execution_time =
                tool_stats.total_execution_time / tool_stats.total_executions as u32;
        }

        tool_stats.failure_rate =
            (tool_stats.total_executions - tool_stats.successful_executions) as f32
                / tool_stats.total_executions as f32;
    }

    /// Update tool usage metadata
    async fn update_tool_usage(&self, tool_name: &str) {
        let mut metadata = self.metadata.write().await;

        if let Some(meta) = metadata.get_mut(tool_name) {
            meta.last_used = Some(SystemTime::now());
            meta.usage_count += 1;
        }
    }

    /// Calculate hash of parameters for caching
    fn hash_parameters(&self, params: &Value) -> String {
        use std::hash::{Hash, Hasher};
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        params.to_string().hash(&mut hasher);
        format!("{:x}", hasher.finish())
    }

    /// Calculate approximate size of tool result
    fn calculate_result_size(&self, result: &ToolResult) -> usize {
        serde_json::to_string(result).map(|s| s.len()).unwrap_or(0)
    }

    /// Clone registry for parallel execution (simplified)
    fn clone_for_parallel(&self) -> Self {
        Self {
            tools: self.tools.clone(),
            metadata: self.metadata.clone(),
            statistics: self.statistics.clone(),
            config: self.config.clone(),
            performance_tracker: self.performance_tracker.clone(),
        }
    }
}

/// Tool discovery service for dynamic tool loading
pub struct ToolDiscovery {
    registry: Arc<ToolRegistry>,
    search_paths: Vec<std::path::PathBuf>,
}

impl ToolDiscovery {
    pub fn new(registry: Arc<ToolRegistry>) -> Self {
        Self {
            registry,
            search_paths: vec![
                std::path::PathBuf::from("./src/mcp/tools/"),
                std::path::PathBuf::from("./custom_tools/"),
            ],
        }
    }

    /// Discover and register tools from configured paths
    pub async fn discover_tools(&self) -> MCPResult<usize> {
        let mut discovered_count = 0;

        for path in &self.search_paths {
            if path.exists() && path.is_dir() {
                // TODO: Implement dynamic tool loading from compiled modules
                // This would require a plugin system or dynamic library loading
                debug!("Scanning for tools in: {}", path.display());
            }
        }

        Ok(discovered_count)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mcp::tools::Content;
    use serde_json::json;

    struct TestTool;

    #[async_trait]
    impl MCPTool for TestTool {
        fn name(&self) -> &str { "test_tool" }
        fn description(&self) -> &str { "A test tool" }
        fn input_schema(&self) -> Value {
            json!({
                "type": "object",
                "properties": {
                    "message": { "type": "string" }
                },
                "required": ["message"]
            })
        }

        async fn execute(&self, params: Value, _context: &mut ExecutionContext) -> Result<ToolResult, ToolError> {
            let message = params["message"].as_str().unwrap_or("default");

            Ok(ToolResult {
                success: true,
                content: vec![Content::Text { text: message.to_string() }],
                context_updates: None,
                notifications: vec![],
                metadata: crate::mcp::tools::ExecutionMetadata {
                    tool_name: self.name().to_string(),
                    execution_id: Uuid::new_v4().to_string(),
                    start_time: SystemTime::now(),
                    end_time: SystemTime::now(),
                    duration: Duration::from_millis(1),
                    session_id: "test".to_string(),
                    parameters_hash: "test".to_string(),
                    result_size: 0,
                },
            })
        }
    }

    #[tokio::test]
    async fn test_tool_registration() {
        let registry = ToolRegistry::new();
        let tool = Box::new(TestTool);

        assert!(registry.register_tool(tool).await.is_ok());

        let tools = registry.list_tools().await;
        assert_eq!(tools.len(), 1);
        assert_eq!(tools[0].name, "test_tool");
    }

    #[tokio::test]
    async fn test_tool_search() {
        let registry = ToolRegistry::new();
        let tool = Box::new(TestTool);

        registry.register_tool(tool).await.unwrap();

        let results = registry.search_tools("test").await;
        assert_eq!(results.len(), 1);

        let results = registry.search_tools("nonexistent").await;
        assert_eq!(results.len(), 0);
    }
}