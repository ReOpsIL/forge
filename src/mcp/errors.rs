use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fmt;

/// Comprehensive error handling for MCP server operations
pub type MCPResult<T> = Result<T, MCPError>;

/// Main error type for all MCP operations
#[derive(Debug, thiserror::Error)]
pub enum MCPError {
    #[error("Transport error: {0}")]
    Transport(#[from] TransportError),

    #[error("Protocol error: {0}")]
    Protocol(#[from] ProtocolError),

    #[error("Tool execution error: {0}")]
    ToolExecution(#[from] ToolError),

    #[error("Session error: {0}")]
    Session(#[from] SessionError),

    #[error("State management error: {0}")]
    State(#[from] StateError),

    #[error("Context error: {0}")]
    Context(#[from] ContextError),

    #[error("Server error: {0}")]
    Server(#[from] ServerError),

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Permission denied: {0}")]
    PermissionDenied(String),

    #[error("Resource not found: {0}")]
    NotFound(String),

    #[error("Timeout: {0}")]
    Timeout(String),

    #[error("Internal server error: {0}")]
    Internal(String),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Recovery attempted: {recovery_action}")]
    RecoverableError {
        source: Box<dyn std::error::Error + Send + Sync>,
        recovery_action: String,
        retry_count: usize,
    },
}

/// Transport layer errors
#[derive(Debug, thiserror::Error)]
pub enum TransportError {
    #[error("Connection failed: {0}")]
    ConnectionFailed(String),

    #[error("Connection lost: {0}")]
    ConnectionLost(String),

    #[error("WebSocket error: {0}")]
    WebSocket(#[from] tokio_tungstenite::tungstenite::Error),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Invalid message format: {0}")]
    InvalidMessage(String),

    #[error("Transport closed")]
    Closed,
}

/// Protocol-level errors
#[derive(Debug, thiserror::Error)]
pub enum ProtocolError {
    #[error("Invalid JSON-RPC message: {0}")]
    InvalidMessage(String),

    #[error("Method not found: {0}")]
    MethodNotFound(String),

    #[error("Invalid parameters: {0}")]
    InvalidParams(String),

    #[error("Parse error: {0}")]
    ParseError(String),

    #[error("Internal error: {0}")]
    InternalError(String),

    #[error("Server error: {0}")]
    ServerError(String),
}

/// Tool execution errors
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
}

/// Session management errors
#[derive(Debug, thiserror::Error)]
pub enum SessionError {
    #[error("Session not found: {0}")]
    NotFound(String),

    #[error("Session expired: {0}")]
    Expired(String),

    #[error("Session limit exceeded")]
    LimitExceeded,

    #[error("Authentication failed: {0}")]
    AuthenticationFailed(String),

    #[error("Concurrent access violation: {0}")]
    ConcurrentAccess(String),

    #[error("Session corrupted: {0}")]
    Corrupted(String),
}

/// State management errors
#[derive(Debug, thiserror::Error)]
pub enum StateError {
    #[error("Transaction failed: {0}")]
    TransactionFailed(String),

    #[error("Consistency violation: {0}")]
    ConsistencyViolation(String),

    #[error("Lock acquisition failed: {0}")]
    LockFailed(String),

    #[error("Rollback failed: {0}")]
    RollbackFailed(String),

    #[error("Persistence error: {0}")]
    Persistence(String),

    #[error("Conflict detected: {0}")]
    Conflict(String),
}

/// Context management errors
#[derive(Debug, thiserror::Error)]
pub enum ContextError {
    #[error("Context not found: {0}")]
    NotFound(String),

    #[error("Context expired: {0}")]
    Expired(String),

    #[error("Serialization failed: {0}")]
    Serialization(String),

    #[error("Size limit exceeded: current {current}, limit {limit}")]
    SizeLimit { current: usize, limit: usize },

    #[error("Context locked: {0}")]
    Locked(String),

    #[error("Storage limit exceeded: {0}")]
    StorageLimit(String),

    #[error("Entry is read-only: {0}")]
    ReadOnly(String),

    #[error("Concurrent access error: {0}")]
    ConcurrentAccess(String),
}

/// Server operation errors
#[derive(Debug, thiserror::Error)]
pub enum ServerError {
    #[error("Method not found: {0}")]
    MethodNotFound(String),

    #[error("Invalid parameters: {0}")]
    InvalidParams(String),

    #[error("Session required")]
    SessionRequired,

    #[error("Session not found: {0}")]
    SessionNotFound(String),

    #[error("Tool execution failed: {0}")]
    ToolExecutionFailed(String),

    #[error("Server startup failed: {0}")]
    StartupFailed(String),

    #[error("Server shutdown failed: {0}")]
    ShutdownFailed(String),
}

/// JSON-RPC error codes following the spec
#[derive(Debug, Clone, Copy)]
pub enum JsonRpcErrorCode {
    ParseError = -32700,
    InvalidRequest = -32600,
    MethodNotFound = -32601,
    InvalidParams = -32602,
    InternalError = -32603,
    ServerError = -32000, // -32000 to -32099 are reserved for implementation-defined server-errors
}

/// JSON-RPC error response structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcError {
    pub code: i32,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<Value>,
}

impl JsonRpcError {
    pub fn new(code: JsonRpcErrorCode, message: impl Into<String>) -> Self {
        Self {
            code: code as i32,
            message: message.into(),
            data: None,
        }
    }

    pub fn with_data(mut self, data: Value) -> Self {
        self.data = Some(data);
        self
    }
}

impl From<MCPError> for JsonRpcError {
    fn from(error: MCPError) -> Self {
        match error {
            MCPError::Protocol(ProtocolError::MethodNotFound(msg)) => {
                JsonRpcError::new(JsonRpcErrorCode::MethodNotFound, msg)
            }
            MCPError::Protocol(ProtocolError::InvalidParams(msg)) => {
                JsonRpcError::new(JsonRpcErrorCode::InvalidParams, msg)
            }
            MCPError::Protocol(ProtocolError::ParseError(msg)) => {
                JsonRpcError::new(JsonRpcErrorCode::ParseError, msg)
            }
            MCPError::Validation(msg) => JsonRpcError::new(JsonRpcErrorCode::InvalidParams, msg),
            _ => JsonRpcError::new(JsonRpcErrorCode::InternalError, error.to_string()),
        }
    }
}

impl From<ProtocolError> for JsonRpcError {
    fn from(error: ProtocolError) -> Self {
        MCPError::Protocol(error).into()
    }
}

impl From<ToolError> for JsonRpcError {
    fn from(error: ToolError) -> Self {
        MCPError::ToolExecution(error).into()
    }
}

/// Error recovery system for handling recoverable errors
pub struct ErrorRecoverySystem {
    max_retry_attempts: usize,
    retry_delay: std::time::Duration,
}

impl ErrorRecoverySystem {
    pub fn new(max_retry_attempts: usize, retry_delay: std::time::Duration) -> Self {
        Self {
            max_retry_attempts,
            retry_delay,
        }
    }

    pub async fn attempt_recovery<F, T>(&self, operation: F) -> MCPResult<T>
    where
        F: Fn() -> MCPResult<T>,
    {
        let mut attempts = 0;
        let mut last_error = None;

        while attempts < self.max_retry_attempts {
            match operation() {
                Ok(result) => return Ok(result),
                Err(error) => {
                    attempts += 1;
                    last_error = Some(error);

                    if attempts < self.max_retry_attempts {
                        tracing::warn!(
                            "Operation failed (attempt {}/{}), retrying in {:?}",
                            attempts,
                            self.max_retry_attempts,
                            self.retry_delay
                        );
                        tokio::time::sleep(self.retry_delay).await;
                    }
                }
            }
        }

        Err(last_error
            .unwrap_or_else(|| MCPError::Internal("Unknown error during recovery".to_string())))
    }
}

/// Helper trait for adding context to errors
pub trait ErrorContext<T> {
    fn with_context(self, context: impl Into<String>) -> MCPResult<T>;
}

impl<T, E> ErrorContext<T> for Result<T, E>
where
    E: Into<MCPError>,
{
    fn with_context(self, context: impl Into<String>) -> MCPResult<T> {
        self.map_err(|e| {
            let original_error = e.into();
            MCPError::Internal(format!("{}: {}", context.into(), original_error))
        })
    }
}
