use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, SystemTime};
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::mcp::errors::{MCPError, MCPResult, SessionError};
use crate::mcp::tools::{ExecutionContext, SessionPermissions, ToolExecution, UserPreferences};

/// Session identifier type
pub type SessionId = String;

/// Session management for MCP connections
pub struct SessionManager {
    /// Active sessions indexed by session ID
    sessions: Arc<RwLock<HashMap<SessionId, Session>>>,
    
    /// Session configuration
    config: SessionConfig,
}

/// Individual session state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    /// Unique session identifier
    pub id: SessionId,
    
    /// Client information
    pub client_info: ClientInfo,
    
    /// Session context and state
    pub context: SessionContext,
    
    /// Currently active tasks
    pub active_tasks: Vec<String>,
    
    /// Tool execution history
    pub tool_history: Vec<ToolExecution>,
    
    /// Session permissions
    pub permissions: SessionPermissions,
    
    /// Collaboration state
    pub collaboration_state: CollaborationState,
    
    /// Session creation time
    pub created_at: SystemTime,
    
    /// Last activity timestamp
    pub last_activity: SystemTime,
    
    /// Session status
    pub status: SessionStatus,
}

/// Session context containing working state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionContext {
    /// Current project identifier
    pub current_project: Option<String>,
    
    /// Current block being worked on
    pub current_block: Option<String>,
    
    /// Working directory
    pub working_directory: std::path::PathBuf,
    
    /// Environment variables for this session
    pub environment_variables: HashMap<String, String>,
    
    /// User preferences
    pub user_preferences: UserPreferences,
    
    /// Cached data for performance
    pub cached_data: HashMap<String, serde_json::Value>,
}

/// Client information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientInfo {
    pub client_name: String,
    pub client_version: String,
    pub user_id: Option<String>,
    pub capabilities: Vec<String>,
    pub connection_time: SystemTime,
}

/// Session status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SessionStatus {
    Active,
    Idle,
    Suspended,
    Terminated,
}

/// Collaboration state for multi-user sessions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollaborationState {
    /// Whether this session is participating in collaboration
    pub is_collaborative: bool,
    
    /// Other sessions collaborating on the same project
    pub collaborators: Vec<SessionId>,
    
    /// Shared context data
    pub shared_context: HashMap<String, serde_json::Value>,
    
    /// Conflict resolution strategy
    pub conflict_resolution: ConflictResolutionStrategy,
}

/// Conflict resolution strategies
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum ConflictResolutionStrategy {
    Manual,
    AutoMerge,
    LastWriteWins,
    Vote,
}

/// Session configuration
#[derive(Debug, Clone)]
pub struct SessionConfig {
    /// Maximum number of concurrent sessions
    pub max_sessions: usize,
    
    /// Session timeout duration
    pub session_timeout: Duration,
    
    /// Whether to persist sessions across server restarts
    pub enable_persistence: bool,
    
    /// Maximum tool history size per session
    pub max_tool_history: usize,
    
    /// Default session permissions
    pub default_permissions: SessionPermissions,
}

impl Default for SessionConfig {
    fn default() -> Self {
        Self {
            max_sessions: 25,
            session_timeout: Duration::from_secs(7200), // 2 hours
            enable_persistence: true,
            max_tool_history: 1000,
            default_permissions: SessionPermissions::default(),
        }
    }
}

impl SessionManager {
    /// Create a new session manager
    pub fn new() -> Self {
        Self::with_config(SessionConfig::default())
    }
    
    /// Create a session manager with custom configuration
    pub fn with_config(config: SessionConfig) -> Self {
        Self {
            sessions: Arc::new(RwLock::new(HashMap::new())),
            config,
        }
    }
    
    /// Create a new session
    pub async fn create_session(&self, client_info: ClientInfo) -> MCPResult<SessionId> {
        let sessions = self.sessions.read().await;
        
        // Check session limit
        if sessions.len() >= self.config.max_sessions {
            return Err(MCPError::Session(SessionError::LimitExceeded));
        }
        
        drop(sessions); // Release read lock
        
        let session_id = Uuid::new_v4().to_string();
        let now = SystemTime::now();
        
        let session = Session {
            id: session_id.clone(),
            client_info,
            context: SessionContext {
                current_project: None,
                current_block: None,
                working_directory: std::env::current_dir()
                    .unwrap_or_else(|_| std::path::PathBuf::from("/")),
                environment_variables: HashMap::new(),
                user_preferences: UserPreferences::default(),
                cached_data: HashMap::new(),
            },
            active_tasks: Vec::new(),
            tool_history: Vec::new(),
            permissions: self.config.default_permissions.clone(),
            collaboration_state: CollaborationState {
                is_collaborative: false,
                collaborators: Vec::new(),
                shared_context: HashMap::new(),
                conflict_resolution: ConflictResolutionStrategy::Manual,
            },
            created_at: now,
            last_activity: now,
            status: SessionStatus::Active,
        };
        
        let mut sessions = self.sessions.write().await;
        sessions.insert(session_id.clone(), session);
        
        Ok(session_id)
    }
    
    /// Get a session by ID
    pub async fn get_session(&self, session_id: &str) -> Option<Session> {
        self.sessions.read().await.get(session_id).cloned()
    }
    
    /// Update a session
    pub async fn update_session(&self, session: Session) -> MCPResult<()> {
        let mut sessions = self.sessions.write().await;
        
        if !sessions.contains_key(&session.id) {
            return Err(MCPError::Session(SessionError::NotFound(session.id.clone())));
        }
        
        sessions.insert(session.id.clone(), session);
        Ok(())
    }
    
    /// Update session activity timestamp
    pub async fn update_activity(&self, session_id: &str) -> MCPResult<()> {
        let mut sessions = self.sessions.write().await;
        
        if let Some(session) = sessions.get_mut(session_id) {
            session.last_activity = SystemTime::now();
            Ok(())
        } else {
            Err(MCPError::Session(SessionError::NotFound(session_id.to_string())))
        }
    }
    
    /// Terminate a session
    pub async fn terminate_session(&self, session_id: &str) -> MCPResult<()> {
        let mut sessions = self.sessions.write().await;
        
        if let Some(mut session) = sessions.remove(session_id) {
            session.status = SessionStatus::Terminated;
            // Could store in terminated sessions for audit purposes
            Ok(())
        } else {
            Err(MCPError::Session(SessionError::NotFound(session_id.to_string())))
        }
    }
    
    /// List active sessions
    pub async fn list_active_sessions(&self) -> Vec<SessionId> {
        let sessions = self.sessions.read().await;
        sessions
            .values()
            .filter(|s| s.status == SessionStatus::Active)
            .map(|s| s.id.clone())
            .collect()
    }
    
    /// Clean up expired sessions
    pub async fn cleanup_expired_sessions(&self) -> usize {
        let mut sessions = self.sessions.write().await;
        let now = SystemTime::now();
        let mut expired_sessions = Vec::new();
        
        for (id, session) in sessions.iter() {
            if let Ok(elapsed) = now.duration_since(session.last_activity) {
                if elapsed > self.config.session_timeout {
                    expired_sessions.push(id.clone());
                }
            }
        }
        
        let count = expired_sessions.len();
        for id in expired_sessions {
            sessions.remove(&id);
        }
        
        count
    }
    
    /// Get session statistics
    pub async fn get_statistics(&self) -> SessionStatistics {
        let sessions = self.sessions.read().await;
        
        let total_sessions = sessions.len();
        let active_sessions = sessions
            .values()
            .filter(|s| s.status == SessionStatus::Active)
            .count();
        
        let idle_sessions = sessions
            .values()
            .filter(|s| s.status == SessionStatus::Idle)
            .count();
        
        let average_tools_per_session = if total_sessions > 0 {
            sessions
                .values()
                .map(|s| s.tool_history.len())
                .sum::<usize>() as f64 / total_sessions as f64
        } else {
            0.0
        };
        
        SessionStatistics {
            total_sessions,
            active_sessions,
            idle_sessions,
            average_tools_per_session,
            oldest_session: sessions
                .values()
                .min_by_key(|s| s.created_at)
                .map(|s| s.created_at),
        }
    }
    
    /// Create execution context from session
    pub async fn create_execution_context(
        &self,
        session_id: &str,
        project_config: Arc<crate::project_config::ProjectConfigManager>,
        block_manager: Arc<crate::block_config::BlockConfigManager>,
        context_store: Arc<tokio::sync::RwLock<crate::mcp::context::ContextStore>>,
    ) -> MCPResult<ExecutionContext> {
        let session = self.get_session(session_id).await
            .ok_or_else(|| MCPError::Session(SessionError::NotFound(session_id.to_string())))?;
        
        Ok(ExecutionContext {
            session_id: session_id.to_string(),
            project_config,
            block_manager,
            working_directory: session.context.working_directory,
            context_store,
            execution_history: session.tool_history,
            user_preferences: session.context.user_preferences,
            permissions: session.permissions,
            performance_tracker: Arc::new(tokio::sync::Mutex::new(
                crate::mcp::tools::PerformanceTracker::default()
            )),
        })
    }
    
    /// Enable collaboration for a session
    pub async fn enable_collaboration(
        &self,
        session_id: &str,
        collaborators: Vec<SessionId>,
    ) -> MCPResult<()> {
        let mut sessions = self.sessions.write().await;
        
        if let Some(session) = sessions.get_mut(session_id) {
            session.collaboration_state.is_collaborative = true;
            session.collaboration_state.collaborators = collaborators;
            Ok(())
        } else {
            Err(MCPError::Session(SessionError::NotFound(session_id.to_string())))
        }
    }
    
    /// Synchronize context between collaborating sessions
    pub async fn sync_collaborative_context(
        &self,
        source_session: &str,
        target_sessions: &[SessionId],
        context_data: HashMap<String, serde_json::Value>,
    ) -> MCPResult<()> {
        let mut sessions = self.sessions.write().await;
        
        // Verify source session exists and is collaborative
        let source = sessions.get(source_session)
            .ok_or_else(|| MCPError::Session(SessionError::NotFound(source_session.to_string())))?;
        
        if !source.collaboration_state.is_collaborative {
            return Err(MCPError::Session(SessionError::ConcurrentAccess(
                "Source session is not collaborative".to_string()
            )));
        }
        
        // Update target sessions
        for target_id in target_sessions {
            if let Some(target_session) = sessions.get_mut(target_id) {
                if target_session.collaboration_state.is_collaborative {
                    target_session.collaboration_state.shared_context.extend(context_data.clone());
                }
            }
        }
        
        Ok(())
    }
}

/// Session statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionStatistics {
    pub total_sessions: usize,
    pub active_sessions: usize,
    pub idle_sessions: usize,
    pub average_tools_per_session: f64,
    pub oldest_session: Option<SystemTime>,
}

/// Session cleanup service
pub struct SessionCleanupService {
    manager: Arc<SessionManager>,
    cleanup_interval: Duration,
}

impl SessionCleanupService {
    pub fn new(manager: Arc<SessionManager>) -> Self {
        Self {
            manager,
            cleanup_interval: Duration::from_secs(300), // 5 minutes
        }
    }
    
    /// Start the cleanup service
    pub async fn start(&self) {
        let manager = self.manager.clone();
        let interval = self.cleanup_interval;
        
        tokio::spawn(async move {
            let mut cleanup_timer = tokio::time::interval(interval);
            
            loop {
                cleanup_timer.tick().await;
                
                let cleaned_count = manager.cleanup_expired_sessions().await;
                if cleaned_count > 0 {
                    tracing::info!("Cleaned up {} expired sessions", cleaned_count);
                }
            }
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_session_creation() {
        let manager = SessionManager::new();
        let client_info = ClientInfo {
            client_name: "test_client".to_string(),
            client_version: "1.0.0".to_string(),
            user_id: Some("test_user".to_string()),
            capabilities: vec!["tools".to_string()],
            connection_time: SystemTime::now(),
        };
        
        let session_id = manager.create_session(client_info).await.unwrap();
        assert!(!session_id.is_empty());
        
        let session = manager.get_session(&session_id).await.unwrap();
        assert_eq!(session.id, session_id);
        assert_eq!(session.status, SessionStatus::Active);
    }

    #[tokio::test]
    async fn test_session_cleanup() {
        let mut config = SessionConfig::default();
        config.session_timeout = Duration::from_millis(1); // Very short timeout for testing
        
        let manager = SessionManager::with_config(config);
        let client_info = ClientInfo {
            client_name: "test_client".to_string(),
            client_version: "1.0.0".to_string(),
            user_id: None,
            capabilities: vec![],
            connection_time: SystemTime::now(),
        };
        
        let session_id = manager.create_session(client_info).await.unwrap();
        
        // Wait for session to expire
        tokio::time::sleep(Duration::from_millis(10)).await;
        
        let cleaned = manager.cleanup_expired_sessions().await;
        assert_eq!(cleaned, 1);
        
        let session = manager.get_session(&session_id).await;
        assert!(session.is_none());
    }
}