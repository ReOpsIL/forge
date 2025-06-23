use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
/// MCP Server orchestrator - coordinates all MCP components
/// 
/// This module provides the main MCP server that orchestrates transport layers,
/// tool registry, session management, and state management to provide a unified
/// interface for Claude Code integration.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, SystemTime};
use tokio::sync::{mpsc, Mutex, RwLock};
use tokio::time::interval;
use tracing::{debug, error, info, warn};

use crate::mcp::{
    context::{ContextManager, ContextStore},
    errors::{MCPError, MCPResult, ServerError},
    protocol::{
        ClientCapabilities, InitializeParams, InitializeResult, MCPMessage, MCPRequest,
        MCPResponse, ServerCapabilities, ServerInfo, ToolsCapability,
    },
    session::{ClientInfo, SessionCleanupService, SessionId, SessionManager},
    state::{StateConfig, UnifiedStateManager},
    tools::{
        blocks::ListBlocksTool,
        filesystem::{CreateDirectoryTool, DeleteTool, ListDirectoryTool, ReadFileTool, WriteFileTool}, 
        ExecutionContext, MCPTool, ToolError, ToolRegistry, ToolResult,
    },
    transport::{MCPTransport, TransportType},
    MCP_PROTOCOL_VERSION, SERVER_NAME, SERVER_VERSION,
};

/// MCP Server configuration
#[derive(Debug, Clone)]
pub struct MCPServerConfig {
    /// Maximum number of concurrent sessions
    pub max_sessions: usize,

    /// Session timeout duration
    pub session_timeout: Duration,

    /// Maximum number of concurrent tool executions per session
    pub max_concurrent_tools: usize,

    /// Tool execution timeout
    pub tool_timeout: Duration,

    /// Whether to enable performance monitoring
    pub enable_monitoring: bool,

    /// Monitoring interval
    pub monitoring_interval: Duration,

    /// Whether to enable automatic cleanup
    pub enable_cleanup: bool,

    /// Cleanup interval
    pub cleanup_interval: Duration,

    /// Working directory for tool executions
    pub working_directory: std::path::PathBuf,
}

impl Default for MCPServerConfig {
    fn default() -> Self {
        Self {
            max_sessions: 25,
            session_timeout: Duration::from_secs(7200), // 2 hours
            max_concurrent_tools: 8,
            tool_timeout: Duration::from_secs(300), // 5 minutes
            enable_monitoring: true,
            monitoring_interval: Duration::from_secs(60), // 1 minute
            enable_cleanup: true,
            cleanup_interval: Duration::from_secs(300), // 5 minutes
            working_directory: std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("/")),
        }
    }
}

/// MCP Server implementation
pub struct MCPServer {
    /// Server configuration
    config: MCPServerConfig,

    /// Session manager
    session_manager: Arc<SessionManager>,

    /// Tool registry
    tool_registry: Arc<ToolRegistry>,

    /// Context manager
    context_manager: Arc<ContextManager>,

    /// Unified state manager
    state_manager: Arc<UnifiedStateManager>,

    /// Project configuration manager
    project_config: Arc<crate::project_config::ProjectConfigManager>,

    /// Block configuration manager
    block_manager: Arc<crate::block_config::BlockConfigManager>,

    /// Active connections
    connections: Arc<RwLock<HashMap<String, ConnectionInfo>>>,

    /// Server statistics
    stats: Arc<Mutex<ServerStatistics>>,

    /// Shutdown signal
    shutdown_tx: Option<mpsc::Sender<()>>,
}

/// Connection information
#[derive(Debug, Clone)]
struct ConnectionInfo {
    pub session_id: SessionId,
    pub transport_type: TransportType,
    pub connected_at: SystemTime,
    pub last_activity: SystemTime,
    pub message_count: u64,
}

/// Server statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerStatistics {
    pub total_connections: u64,
    pub active_connections: usize,
    pub total_messages: u64,
    pub total_tool_executions: u64,
    pub average_response_time: Duration,
    pub error_count: u64,
    pub uptime: Duration,
    pub started_at: SystemTime,
}

impl Default for ServerStatistics {
    fn default() -> Self {
        Self {
            total_connections: 0,
            active_connections: 0,
            total_messages: 0,
            total_tool_executions: 0,
            average_response_time: Duration::from_secs(0),
            error_count: 0,
            uptime: Duration::from_secs(0),
            started_at: SystemTime::now(),
        }
    }
}

impl MCPServer {
    /// Create a new MCP server
    pub async fn new(
        config: MCPServerConfig,
        project_config: Arc<crate::project_config::ProjectConfigManager>,
        block_manager: Arc<crate::block_config::BlockConfigManager>,
    ) -> MCPResult<Self> {
        // Create session manager
        let session_manager = Arc::new(SessionManager::with_config(
            crate::mcp::session::SessionConfig {
                max_sessions: config.max_sessions,
                session_timeout: config.session_timeout,
                enable_persistence: true,
                max_tool_history: 1000,
                default_permissions: crate::mcp::tools::SessionPermissions::default(),
            }
        ));

        // Create tool registry and register built-in tools
        let tool_registry = Arc::new(ToolRegistry::new());
        Self::register_builtin_tools(&tool_registry).await?;

        // Create context store and manager
        let context_store = ContextStore::new();
        let context_manager = Arc::new(ContextManager::new(context_store));

        // Create unified state manager
        let state_manager = Arc::new(UnifiedStateManager::with_config(StateConfig::default()));

        let server = Self {
            config,
            session_manager,
            tool_registry,
            context_manager,
            state_manager,
            project_config,
            block_manager,
            connections: Arc::new(RwLock::new(HashMap::new())),
            stats: Arc::new(Mutex::new(ServerStatistics::default())),
            shutdown_tx: None,
        };

        info!("MCP Server created with working directory: {}", server.config.working_directory.display());
        Ok(server)
    }

    /// Start the MCP server
    pub async fn start(&mut self) -> MCPResult<()> {
        info!("Starting MCP Server v{}", SERVER_VERSION);

        let (shutdown_tx, mut shutdown_rx) = mpsc::channel(1);
        self.shutdown_tx = Some(shutdown_tx);

        // Start cleanup services
        if self.config.enable_cleanup {
            self.start_cleanup_services().await;
        }

        // Start monitoring service
        if self.config.enable_monitoring {
            self.start_monitoring_service().await;
        }

        // Start context cleanup
        self.context_manager.start_cleanup_service().await;

        info!("MCP Server started successfully");

        // Wait for shutdown signal
        let _ = shutdown_rx.recv().await;

        info!("MCP Server shutting down...");
        Ok(())
    }

    /// Handle a new connection
    pub async fn handle_connection(
        &self,
        mut transport: Box<dyn MCPTransport>,
        connection_id: String,
    ) -> MCPResult<()> {
        info!("New connection: {} ({})", connection_id, transport.transport_type() as u8);

        // Update stats
        {
            let mut stats = self.stats.lock().await;
            stats.total_connections += 1;
            stats.active_connections += 1;
        }

        let mut session_id = None;
        let start_time = SystemTime::now();

        // Main message loop
        loop {
            match transport.receive().await {
                Ok(message) => {
                    if let Err(e) = self.handle_message(message, &mut transport, &connection_id, &mut session_id).await {
                        error!("Error handling message: {}", e);

                        // Update error stats
                        let mut stats = self.stats.lock().await;
                        stats.error_count += 1;
                    }
                }
                Err(MCPError::Transport(_)) => {
                    debug!("Connection closed: {}", connection_id);
                    break;
                }
                Err(e) => {
                    error!("Transport error: {}", e);
                    break;
                }
            }

            if !transport.is_connected() {
                break;
            }
        }

        // Cleanup connection
        self.cleanup_connection(&connection_id, session_id.as_deref()).await;

        // Update stats
        {
            let mut stats = self.stats.lock().await;
            stats.active_connections = stats.active_connections.saturating_sub(1);
        }

        info!("Connection closed: {} (duration: {:?})", connection_id, start_time.elapsed().unwrap_or_default());
        Ok(())
    }

    /// Handle a single message
    async fn handle_message(
        &self,
        message: MCPMessage,
        transport: &mut Box<dyn MCPTransport>,
        connection_id: &str,
        session_id: &mut Option<SessionId>,
    ) -> MCPResult<()> {
        // Update message stats
        {
            let mut stats = self.stats.lock().await;
            stats.total_messages += 1;
        }

        // Update connection activity
        self.update_connection_activity(connection_id).await;

        if message.is_request() {
            let request = message.as_request()?;
            let response = self.handle_request(request, session_id).await;
            transport.send(response).await?;
        } else if message.is_notification() {
            let notification = message.as_notification()?;
            self.handle_notification(notification, session_id).await?;
        } else {
            warn!("Received unexpected message type");
        }

        Ok(())
    }

    /// Handle a request message
    async fn handle_request(&self, request: MCPRequest, session_id: &mut Option<SessionId>) -> MCPMessage {
        let start_time = SystemTime::now();

        let response = match request.method.as_str() {
            "initialize" => self.handle_initialize(request.params).await,
            "tools/list" => self.handle_list_tools().await,
            "tools/call" => self.handle_tool_call(request.params, session_id).await,
            "session/create" => self.handle_create_session(request.params, session_id).await,
            "session/info" => self.handle_session_info(session_id).await,
            "server/stats" => self.handle_server_stats().await,
            "prompts/list" => self.handle_list_prompts().await,
            "resources/list" => self.handle_list_resources().await,
            _ => Err(MCPError::Server(ServerError::MethodNotFound(request.method.clone()))),
        };

        // Update response time stats
        if let Ok(elapsed) = start_time.elapsed() {
            let mut stats = self.stats.lock().await;
            let total_time = stats.average_response_time * stats.total_messages as u32 + elapsed;
            stats.average_response_time = total_time / (stats.total_messages + 1) as u32;
        }

        match response {
            Ok(result) => MCPMessage::response(request.id, Some(result)),
            Err(error) => {
                let json_rpc_error = crate::mcp::errors::JsonRpcError {
                    code: -32603, // Internal error
                    message: error.to_string(),
                    data: None,
                };
                MCPMessage::error_response(request.id, json_rpc_error)
            }
        }
    }

    /// Handle a notification message
    async fn handle_notification(
        &self,
        _notification: crate::mcp::protocol::MCPNotification,
        _session_id: &Option<SessionId>,
    ) -> MCPResult<()> {
        // Handle notifications (currently none defined)
        Ok(())
    }

    /// Handle initialize request
    async fn handle_initialize(&self, params: Option<Value>) -> MCPResult<Value> {
        if let Some(params) = params {
            let _init_params: InitializeParams = serde_json::from_value(params)
                .map_err(|e| MCPError::Server(ServerError::InvalidParams(e.to_string())))?;
        }

        let result = InitializeResult {
            protocol_version: MCP_PROTOCOL_VERSION.to_string(),
            capabilities: ServerCapabilities {
                tools: ToolsCapability { list_changed: true },
                prompts: crate::mcp::protocol::PromptsCapability { list_changed: true },
                resources: crate::mcp::protocol::ResourcesCapability { list_changed: true, subscribe: false },
                logging: Default::default(),
            },
            server_info: ServerInfo {
                name: SERVER_NAME.to_string(),
                version: SERVER_VERSION.to_string(),
            },
            instructions: Some("Forge MCP Server ready for tool execution".to_string()),
        };

        Ok(serde_json::to_value(result)?)
    }

    /// Handle list tools request
    async fn handle_list_tools(&self) -> MCPResult<Value> {
        let tools = self.tool_registry.list_tools().await;
        Ok(json!({ "tools": tools }))
    }

    /// Handle tool call request
    async fn handle_tool_call(&self, params: Option<Value>, session_id: &Option<SessionId>) -> MCPResult<Value> {
        let params = params.ok_or_else(|| MCPError::Server(ServerError::InvalidParams("Missing parameters".to_string())))?;

        let tool_name = params["name"].as_str()
            .ok_or_else(|| MCPError::Server(ServerError::InvalidParams("Missing tool name".to_string())))?;
        let tool_params = params.get("arguments").cloned().unwrap_or(json!({}));

        // If no session exists, create one automatically
        let session_id_str = if let Some(id) = session_id {
            id.clone()
        } else {
            info!("No session found for tool '{}', creating a default session automatically", tool_name);
            let client_info = ClientInfo {
                client_name: "Default".to_string(),
                client_version: "1.0.0".to_string(),
                user_id: None,
                capabilities: vec![],
                connection_time: SystemTime::now(),
            };

            let new_session_id = self.session_manager.create_session(client_info).await?;
            info!("Created default session with ID: {}", new_session_id);
            new_session_id
        };

        // Create execution context
        let mut context = self.session_manager.create_execution_context(
            &session_id_str,
            self.project_config.clone(),
            self.block_manager.clone(),
            self.context_manager.get_store(),
        ).await?;

        // Execute tool
        let result = self.tool_registry.execute_tool(tool_name, tool_params, &mut context).await
            .map_err(|e| MCPError::Server(ServerError::ToolExecutionFailed(e.to_string())))?;

        // Update stats
        {
            let mut stats = self.stats.lock().await;
            stats.total_tool_executions += 1;
        }

        Ok(serde_json::to_value(result)?)
    }

    /// Handle create session request
    async fn handle_create_session(&self, params: Option<Value>, session_id: &mut Option<SessionId>) -> MCPResult<Value> {
        let client_info = if let Some(params) = params {
            serde_json::from_value(params)
                .map_err(|e| MCPError::Server(ServerError::InvalidParams(e.to_string())))?
        } else {
            ClientInfo {
                client_name: "Unknown".to_string(),
                client_version: "1.0.0".to_string(),
                user_id: None,
                capabilities: vec![],
                connection_time: SystemTime::now(),
            }
        };

        let new_session_id = self.session_manager.create_session(client_info).await?;
        *session_id = Some(new_session_id.clone());

        Ok(json!({ "session_id": new_session_id }))
    }

    /// Handle session info request
    async fn handle_session_info(&self, session_id: &Option<SessionId>) -> MCPResult<Value> {
        let session_id = session_id.as_ref()
            .ok_or_else(|| MCPError::Server(ServerError::SessionRequired))?;

        let session = self.session_manager.get_session(session_id).await
            .ok_or_else(|| MCPError::Server(ServerError::SessionNotFound(session_id.clone())))?;

        Ok(serde_json::to_value(session)?)
    }

    /// Handle server stats request
    async fn handle_server_stats(&self) -> MCPResult<Value> {
        let stats = self.stats.lock().await.clone();
        let session_stats = self.session_manager.get_statistics().await;
        let tool_stats = self.tool_registry.get_all_statistics().await;
        let state_stats = self.state_manager.get_statistics();
        let context_stats = {
            let context_store = self.context_manager.get_store();
            let store = context_store.read().await;
            store.get_statistics()
        };

        Ok(json!({
            "server": stats,
            "sessions": session_stats,
            "tools": tool_stats,
            "state": state_stats,
            "context": context_stats
        }))
    }

    /// Register built-in tools
    async fn register_builtin_tools(registry: &Arc<ToolRegistry>) -> MCPResult<()> {
        registry.register_tool(Box::new(ReadFileTool)).await?;
        registry.register_tool(Box::new(WriteFileTool)).await?;
        registry.register_tool(Box::new(ListDirectoryTool)).await?;
        registry.register_tool(Box::new(CreateDirectoryTool)).await?;
        registry.register_tool(Box::new(DeleteTool)).await?;
        registry.register_tool(Box::new(ListBlocksTool)).await?;

        info!("Registered {} built-in tools", 6);
        Ok(())
    }

    /// Start cleanup services
    async fn start_cleanup_services(&self) {
        let session_manager = self.session_manager.clone();
        let cleanup_interval = self.config.cleanup_interval;

        tokio::spawn(async move {
            let cleanup_service = SessionCleanupService::new(session_manager);
            cleanup_service.start().await;
        });

        // State cleanup
        let state_manager = self.state_manager.clone();
        tokio::spawn(async move {
            let mut cleanup_timer = interval(cleanup_interval);

            loop {
                cleanup_timer.tick().await;
                let cleaned = state_manager.cleanup_expired_state();
                if cleaned > 0 {
                    debug!("State cleanup: removed {} expired entries", cleaned);
                }
            }
        });
    }

    /// Start monitoring service
    async fn start_monitoring_service(&self) {
        let stats = self.stats.clone();
        let monitoring_interval = self.config.monitoring_interval;

        tokio::spawn(async move {
            let mut monitor_timer = interval(monitoring_interval);

            loop {
                monitor_timer.tick().await;

                let stats = stats.lock().await;
                debug!(
                    "Server stats - Connections: {}, Messages: {}, Tools: {}, Errors: {}",
                    stats.active_connections,
                    stats.total_messages,
                    stats.total_tool_executions,
                    stats.error_count
                );
            }
        });
    }

    /// Update connection activity
    async fn update_connection_activity(&self, connection_id: &str) {
        let mut connections = self.connections.write().await;
        if let Some(conn) = connections.get_mut(connection_id) {
            conn.last_activity = SystemTime::now();
            conn.message_count += 1;
        }
    }

    /// Cleanup connection
    async fn cleanup_connection(&self, connection_id: &str, session_id: Option<&str>) {
        // Remove connection
        self.connections.write().await.remove(connection_id);

        // Terminate session if exists
        if let Some(session_id) = session_id {
            if let Err(e) = self.session_manager.terminate_session(session_id).await {
                warn!("Failed to terminate session {}: {}", session_id, e);
            }
        }
    }

    /// Shutdown the server
    pub async fn shutdown(&self) -> MCPResult<()> {
        if let Some(shutdown_tx) = &self.shutdown_tx {
            let _ = shutdown_tx.send(()).await;
        }
        Ok(())
    }

    /// Get server statistics
    pub async fn get_statistics(&self) -> ServerStatistics {
        self.stats.lock().await.clone()
    }

    /// Handle list prompts request
    async fn handle_list_prompts(&self) -> MCPResult<Value> {
        // Return an empty list of prompts for now
        Ok(json!({ "prompts": [] }))
    }

    /// Handle list resources request
    async fn handle_list_resources(&self) -> MCPResult<Value> {
        // Return an empty list of resources for now
        Ok(json!({ "resources": [] }))
    }
}
