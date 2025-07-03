/// Unified state management for the MCP server
///
/// This module provides centralized state management that replaces the fragmented
/// app states throughout Forge, enabling consistent state access and updates
/// across all MCP tools and components.
use dashmap::DashMap;
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, SystemTime};
use tokio::sync::broadcast;
use tracing::{debug, info, warn};


use crate::mcp::errors::{MCPError, MCPResult, StateError};
use crate::models::{Block, Task};

/// Unified state manager for the MCP server
pub struct UnifiedStateManager {
    /// Core application state
    core_state: Arc<RwLock<CoreState>>,

    /// Real-time state updates using DashMap for concurrent access
    live_state: Arc<DashMap<String, StateEntry>>,

    /// State change broadcaster
    state_broadcaster: broadcast::Sender<StateChangeEvent>,

    /// State configuration
    config: StateConfig,

    /// State history for debugging and rollback
    state_history: Arc<RwLock<Vec<StateChangeEvent>>>,
}

/// Core application state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoreState {
    /// Current projects and their configurations
    pub projects: HashMap<String, ProjectState>,

    /// Active tasks across all projects
    pub tasks: HashMap<String, TaskState>,

    /// Block definitions and their current state
    pub blocks: HashMap<String, BlockState>,

    /// Active sessions and their contexts
    pub sessions: HashMap<String, SessionState>,

    /// Global application configuration
    pub app_config: AppConfig,

    /// Performance metrics
    pub performance_metrics: PerformanceState,

    /// Last update timestamp
    pub last_updated: SystemTime,
}

/// Project state information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectState {
    pub id: String,
    pub name: String,
    pub description: String,
    pub working_directory: String,
    pub git_repository: Option<GitState>,
    pub dependencies: Vec<String>,
    pub build_configuration: BuildConfig,
    pub active_tasks: Vec<String>,
    pub status: ProjectStatus,
    pub created_at: SystemTime,
    pub last_modified: SystemTime,
}

/// Task state information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskState {
    pub id: String,
    pub project_id: String,
    pub block_id: String,
    pub name: String,
    pub description: String,
    pub status: TaskStatus,
    pub progress: f32,
    pub created_at: SystemTime,
    pub started_at: Option<SystemTime>,
    pub completed_at: Option<SystemTime>,
    pub assigned_session: Option<String>,
    pub execution_context: Option<Value>,
    pub dependencies: Vec<String>,
    pub artifacts: Vec<TaskArtifact>,
}

/// Block state information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockState {
    pub id: String,
    pub project_id: String,
    pub name: String,
    pub block_type: String,
    pub configuration: Value,
    pub status: BlockStatus,
    pub dependencies: Vec<String>,
    pub outputs: HashMap<String, Value>,
    pub last_executed: Option<SystemTime>,
    pub execution_count: u64,
}

/// Session state information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionState {
    pub id: String,
    pub client_info: Value,
    pub current_project: Option<String>,
    pub current_block: Option<String>,
    pub working_directory: String,
    pub active_tasks: Vec<String>,
    pub tool_preferences: HashMap<String, Value>,
    pub permissions: Vec<String>,
    pub created_at: SystemTime,
    pub last_activity: SystemTime,
    pub status: SessionStatus,
}

/// Git repository state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitState {
    pub current_branch: String,
    pub has_uncommitted_changes: bool,
    pub staged_files: Vec<String>,
    pub unstaged_files: Vec<String>,
    pub untracked_files: Vec<String>,
    pub remote_url: Option<String>,
    pub last_commit_hash: Option<String>,
    pub last_sync: SystemTime,
}

/// Build configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildConfig {
    pub build_tool: String,
    pub target: String,
    pub profile: String,
    pub features: Vec<String>,
    pub environment_variables: HashMap<String, String>,
}

/// Task artifact
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskArtifact {
    pub id: String,
    pub artifact_type: String,
    pub name: String,
    pub path: String,
    pub size: u64,
    pub created_at: SystemTime,
    pub metadata: HashMap<String, Value>,
}

/// Application configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub workspace_root: String,
    pub default_project: Option<String>,
    pub tool_preferences: HashMap<String, Value>,
    pub ui_preferences: HashMap<String, Value>,
    pub feature_flags: HashMap<String, bool>,
}

/// Performance state tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceState {
    pub total_tasks_executed: u64,
    pub total_tools_used: u64,
    pub average_task_duration: Duration,
    pub memory_usage: u64,
    pub cpu_usage: f32,
    pub network_requests: u64,
    pub file_operations: u64,
    pub last_measured: SystemTime,
}

/// State status enumerations
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProjectStatus {
    Active,
    Inactive,
    Building,
    Error,
    Archived,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TaskStatus {
    Pending,
    Running,
    Completed,
    Failed,
    Cancelled,
    Paused,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BlockStatus {
    Ready,
    Executing,
    Completed,
    Failed,
    Disabled,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SessionStatus {
    Active,
    Idle,
    Suspended,
    Terminated,
}

/// State entry for live state tracking
#[derive(Debug, Clone)]
pub struct StateEntry {
    pub value: Value,
    pub last_updated: SystemTime,
    pub update_count: u64,
    pub source: String,
}

/// State change event for broadcasting
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateChangeEvent {
    pub id: String,
    pub event_type: StateChangeType,
    pub entity_type: EntityType,
    pub entity_id: String,
    pub changes: HashMap<String, StateChange>,
    pub timestamp: SystemTime,
    pub source: String,
    pub session_id: Option<String>,
}

/// Types of state changes
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum StateChangeType {
    Create,
    Update,
    Delete,
    StatusChange,
    ConfigChange,
}

/// Entity types that can change
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum EntityType {
    Project,
    Task,
    Block,
    Session,
    Config,
    Performance,
}

/// Individual state change
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateChange {
    pub field: String,
    pub old_value: Option<Value>,
    pub new_value: Value,
}

/// State configuration
#[derive(Debug, Clone)]
pub struct StateConfig {
    /// Maximum number of state history entries to keep
    pub max_history_size: usize,

    /// Whether to enable state change broadcasting
    pub enable_broadcasting: bool,

    /// Maximum number of broadcast subscribers
    pub max_broadcast_subscribers: usize,

    /// State persistence interval
    pub persistence_interval: Duration,

    /// Whether to enable automatic state cleanup
    pub enable_cleanup: bool,

    /// Cleanup interval for expired states
    pub cleanup_interval: Duration,
}

impl Default for StateConfig {
    fn default() -> Self {
        Self {
            max_history_size: 10000,
            enable_broadcasting: true,
            max_broadcast_subscribers: 100,
            persistence_interval: Duration::from_secs(30),
            enable_cleanup: true,
            cleanup_interval: Duration::from_secs(300), // 5 minutes
        }
    }
}

impl Default for CoreState {
    fn default() -> Self {
        Self {
            projects: HashMap::new(),
            tasks: HashMap::new(),
            blocks: HashMap::new(),
            sessions: HashMap::new(),
            app_config: AppConfig {
                workspace_root: std::env::current_dir()
                    .unwrap_or_else(|_| std::path::PathBuf::from("/"))
                    .to_string_lossy()
                    .to_string(),
                default_project: None,
                tool_preferences: HashMap::new(),
                ui_preferences: HashMap::new(),
                feature_flags: HashMap::new(),
            },
            performance_metrics: PerformanceState {
                total_tasks_executed: 0,
                total_tools_used: 0,
                average_task_duration: Duration::from_secs(0),
                memory_usage: 0,
                cpu_usage: 0.0,
                network_requests: 0,
                file_operations: 0,
                last_measured: SystemTime::now(),
            },
            last_updated: SystemTime::now(),
        }
    }
}

impl UnifiedStateManager {
    /// Create a new unified state manager
    pub fn new() -> Self {
        Self::with_config(StateConfig::default())
    }

    /// Create a unified state manager with custom configuration
    pub fn with_config(config: StateConfig) -> Self {
        let (sender, _) = broadcast::channel(config.max_broadcast_subscribers);

        Self {
            core_state: Arc::new(RwLock::new(CoreState::default())),
            live_state: Arc::new(DashMap::new()),
            state_broadcaster: sender,
            config,
            state_history: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Subscribe to state changes
    pub fn subscribe_to_changes(&self) -> broadcast::Receiver<StateChangeEvent> {
        self.state_broadcaster.subscribe()
    }

    /// Get core state (read-only access)
    pub fn get_core_state(&self) -> CoreState {
        self.core_state.read().clone()
    }

    /// Update core state
    pub fn update_core_state<F>(&self, updater: F) -> MCPResult<()>
    where
        F: FnOnce(&mut CoreState),
    {
        let mut state = self.core_state.write();
        updater(&mut state);
        state.last_updated = SystemTime::now();

        debug!("Core state updated");
        Ok(())
    }

    /// Create a new project
    pub fn create_project(
        &self,
        project: ProjectState,
        source: impl Into<String>,
    ) -> MCPResult<()> {
        let source = source.into();
        let project_id = project.id.clone();

        // Update core state
        self.update_core_state(|state| {
            state.projects.insert(project_id.clone(), project.clone());
        })?;

        // Broadcast change
        self.broadcast_change(StateChangeEvent {
            id: uuid::Uuid::new_v4().to_string(),
            event_type: StateChangeType::Create,
            entity_type: EntityType::Project,
            entity_id: project_id.clone(),
            changes: HashMap::new(), // Full project creation
            timestamp: SystemTime::now(),
            source,
            session_id: None,
        });

        info!("Created project: {}", project_id);
        Ok(())
    }

    /// Update project state
    pub fn update_project(
        &self,
        project_id: &str,
        updates: HashMap<String, Value>,
        source: impl Into<String>,
    ) -> MCPResult<()> {
        let source = source.into();
        let mut changes = HashMap::new();

        self.update_core_state(|state| {
            if let Some(project) = state.projects.get_mut(project_id) {
                for (field, new_value) in updates {
                    let old_value = match field.as_str() {
                        "name" => {
                            if let Some(name) = new_value.as_str() {
                                let old = serde_json::to_value(&project.name).unwrap();
                                project.name = name.to_string();
                                Some(old)
                            } else {
                                None
                            }
                        }
                        "description" => {
                            if let Some(desc) = new_value.as_str() {
                                let old = serde_json::to_value(&project.description).unwrap();
                                project.description = desc.to_string();
                                Some(old)
                            } else {
                                None
                            }
                        }
                        "status" => {
                            if let Ok(status) =
                                serde_json::from_value::<ProjectStatus>(new_value.clone())
                            {
                                let old = serde_json::to_value(&project.status).unwrap();
                                project.status = status;
                                Some(old)
                            } else {
                                None
                            }
                        }
                        _ => None,
                    };

                    changes.insert(
                        field.clone(),
                        StateChange {
                            field: field.clone(),
                            old_value,
                            new_value: new_value.clone(),
                        },
                    );
                }
                project.last_modified = SystemTime::now();
            }
        })?;

        if !changes.is_empty() {
            self.broadcast_change(StateChangeEvent {
                id: uuid::Uuid::new_v4().to_string(),
                event_type: StateChangeType::Update,
                entity_type: EntityType::Project,
                entity_id: project_id.to_string(),
                changes,
                timestamp: SystemTime::now(),
                source,
                session_id: None,
            });
        }

        Ok(())
    }

    /// Create a new task
    pub fn create_task(&self, task: TaskState, source: impl Into<String>) -> MCPResult<()> {
        let source = source.into();
        let task_id = task.id.clone();

        // Update core state
        self.update_core_state(|state| {
            state.tasks.insert(task_id.clone(), task.clone());

            // Add task to project's active tasks
            if let Some(project) = state.projects.get_mut(&task.project_id) {
                if !project.active_tasks.contains(&task_id) {
                    project.active_tasks.push(task_id.clone());
                }
            }
        })?;

        // Broadcast change
        self.broadcast_change(StateChangeEvent {
            id: uuid::Uuid::new_v4().to_string(),
            event_type: StateChangeType::Create,
            entity_type: EntityType::Task,
            entity_id: task_id.clone(),
            changes: HashMap::new(),
            timestamp: SystemTime::now(),
            source,
            session_id: task.assigned_session,
        });

        info!("Created task: {}", task_id);
        Ok(())
    }

    /// Update task status
    pub fn update_task_status(
        &self,
        task_id: &str,
        status: TaskStatus,
        progress: Option<f32>,
        source: impl Into<String>,
    ) -> MCPResult<()> {
        let source = source.into();
        let mut changes = HashMap::new();
        let mut session_id = None;

        self.update_core_state(|state| {
            if let Some(task) = state.tasks.get_mut(task_id) {
                // Record old values
                let old_status = serde_json::to_value(&task.status).unwrap();
                let old_progress = serde_json::to_value(&task.progress).unwrap();

                // Update status
                task.status = status;
                changes.insert(
                    "status".to_string(),
                    StateChange {
                        field: "status".to_string(),
                        old_value: Some(old_status),
                        new_value: serde_json::to_value(&status).unwrap(),
                    },
                );

                // Update progress if provided
                if let Some(progress) = progress {
                    task.progress = progress;
                    changes.insert(
                        "progress".to_string(),
                        StateChange {
                            field: "progress".to_string(),
                            old_value: Some(old_progress),
                            new_value: serde_json::to_value(&progress).unwrap(),
                        },
                    );
                }

                // Update timestamps based on status
                match status {
                    TaskStatus::Running => {
                        if task.started_at.is_none() {
                            task.started_at = Some(SystemTime::now());
                        }
                    }
                    TaskStatus::Completed | TaskStatus::Failed | TaskStatus::Cancelled => {
                        task.completed_at = Some(SystemTime::now());

                        // Update performance metrics
                        state.performance_metrics.total_tasks_executed += 1;
                        if let Some(started_at) = task.started_at {
                            if let Ok(duration) = SystemTime::now().duration_since(started_at) {
                                let total_duration = state
                                    .performance_metrics
                                    .average_task_duration
                                    * (state.performance_metrics.total_tasks_executed - 1) as u32
                                    + duration;
                                state.performance_metrics.average_task_duration = total_duration
                                    / state.performance_metrics.total_tasks_executed as u32;
                            }
                        }
                    }
                    _ => {}
                }

                session_id = task.assigned_session.clone();
            }
        })?;

        if !changes.is_empty() {
            self.broadcast_change(StateChangeEvent {
                id: uuid::Uuid::new_v4().to_string(),
                event_type: StateChangeType::StatusChange,
                entity_type: EntityType::Task,
                entity_id: task_id.to_string(),
                changes,
                timestamp: SystemTime::now(),
                source,
                session_id,
            });
        }

        Ok(())
    }

    /// Update block state
    pub fn update_block(
        &self,
        block_id: &str,
        status: BlockStatus,
        outputs: Option<HashMap<String, Value>>,
        source: impl Into<String>,
    ) -> MCPResult<()> {
        let source = source.into();
        let mut changes = HashMap::new();

        self.update_core_state(|state| {
            if let Some(block) = state.blocks.get_mut(block_id) {
                // Update status
                let old_status = serde_json::to_value(&block.status).unwrap();
                block.status = status;
                changes.insert(
                    "status".to_string(),
                    StateChange {
                        field: "status".to_string(),
                        old_value: Some(old_status),
                        new_value: serde_json::to_value(&status).unwrap(),
                    },
                );

                // Update outputs if provided
                if let Some(outputs) = outputs {
                    let old_outputs = serde_json::to_value(&block.outputs).unwrap();
                    block.outputs.extend(outputs);
                    changes.insert(
                        "outputs".to_string(),
                        StateChange {
                            field: "outputs".to_string(),
                            old_value: Some(old_outputs),
                            new_value: serde_json::to_value(&block.outputs).unwrap(),
                        },
                    );
                }

                if matches!(status, BlockStatus::Executing) {
                    block.last_executed = Some(SystemTime::now());
                    block.execution_count += 1;
                }
            }
        })?;

        if !changes.is_empty() {
            self.broadcast_change(StateChangeEvent {
                id: uuid::Uuid::new_v4().to_string(),
                event_type: StateChangeType::Update,
                entity_type: EntityType::Block,
                entity_id: block_id.to_string(),
                changes,
                timestamp: SystemTime::now(),
                source,
                session_id: None,
            });
        }

        Ok(())
    }

    /// Set live state value
    pub fn set_live_state(
        &self,
        key: impl Into<String>,
        value: Value,
        source: impl Into<String>,
    ) -> MCPResult<()> {
        let key = key.into();
        let source = source.into();

        let entry = StateEntry {
            value: value.clone(),
            last_updated: SystemTime::now(),
            update_count: self
                .live_state
                .get(&key)
                .map(|e| e.update_count + 1)
                .unwrap_or(1),
            source,
        };

        self.live_state.insert(key.clone(), entry);

        debug!("Updated live state: {}", key);
        Ok(())
    }

    /// Get live state value
    pub fn get_live_state(&self, key: &str) -> Option<Value> {
        self.live_state.get(key).map(|entry| entry.value.clone())
    }

    /// Remove live state value
    pub fn remove_live_state(&self, key: &str) -> Option<Value> {
        self.live_state.remove(key).map(|(_, entry)| entry.value)
    }

    /// Get project by ID
    pub fn get_project(&self, project_id: &str) -> Option<ProjectState> {
        self.core_state.read().projects.get(project_id).cloned()
    }

    /// Get task by ID
    pub fn get_task(&self, task_id: &str) -> Option<TaskState> {
        self.core_state.read().tasks.get(task_id).cloned()
    }

    /// Get block by ID
    pub fn get_block(&self, block_id: &str) -> Option<BlockState> {
        self.core_state.read().blocks.get(block_id).cloned()
    }

    /// Get session by ID
    pub fn get_session(&self, session_id: &str) -> Option<SessionState> {
        self.core_state.read().sessions.get(session_id).cloned()
    }

    /// List projects with optional filter
    pub fn list_projects(&self, status_filter: Option<ProjectStatus>) -> Vec<ProjectState> {
        let state = self.core_state.read();
        state
            .projects
            .values()
            .filter(|project| status_filter.map_or(true, |status| project.status == status))
            .cloned()
            .collect()
    }

    /// List tasks with optional filters
    pub fn list_tasks(
        &self,
        project_id: Option<&str>,
        status_filter: Option<TaskStatus>,
    ) -> Vec<TaskState> {
        let state = self.core_state.read();
        state
            .tasks
            .values()
            .filter(|task| {
                project_id.map_or(true, |pid| task.project_id == pid)
                    && status_filter.map_or(true, |status| task.status == status)
            })
            .cloned()
            .collect()
    }

    /// Get state statistics
    pub fn get_statistics(&self) -> StateStatistics {
        let core_state = self.core_state.read();
        let live_state_count = self.live_state.len();
        let history_count = self.state_history.read().len();

        StateStatistics {
            total_projects: core_state.projects.len(),
            active_projects: core_state
                .projects
                .values()
                .filter(|p| p.status == ProjectStatus::Active)
                .count(),
            total_tasks: core_state.tasks.len(),
            active_tasks: core_state
                .tasks
                .values()
                .filter(|t| matches!(t.status, TaskStatus::Running | TaskStatus::Pending))
                .count(),
            total_blocks: core_state.blocks.len(),
            active_sessions: core_state
                .sessions
                .values()
                .filter(|s| s.status == SessionStatus::Active)
                .count(),
            live_state_entries: live_state_count,
            state_history_entries: history_count,
            performance_metrics: core_state.performance_metrics.clone(),
        }
    }

    /// Broadcast state change
    fn broadcast_change(&self, event: StateChangeEvent) {
        if self.config.enable_broadcasting {
            if let Err(_) = self.state_broadcaster.send(event.clone()) {
                warn!("No subscribers for state change event");
            }

            // Record in history
            let mut history = self.state_history.write();
            if history.len() >= self.config.max_history_size {
                history.remove(0);
            }
            history.push(event);
        }
    }

    /// Get recent state changes
    pub fn get_recent_changes(&self, limit: usize) -> Vec<StateChangeEvent> {
        let history = self.state_history.read();
        history.iter().rev().take(limit).cloned().collect()
    }

    /// Clean up expired live state entries
    pub fn cleanup_expired_state(&self) -> usize {
        let now = SystemTime::now();
        let mut removed_count = 0;

        self.live_state.retain(|_, entry| {
            let expired =
                entry.last_updated.elapsed().unwrap_or(Duration::MAX) > Duration::from_secs(3600); // 1 hour
            if expired {
                removed_count += 1;
            }
            !expired
        });

        if removed_count > 0 {
            info!("Cleaned up {} expired live state entries", removed_count);
        }

        removed_count
    }
}

/// State statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateStatistics {
    pub total_projects: usize,
    pub active_projects: usize,
    pub total_tasks: usize,
    pub active_tasks: usize,
    pub total_blocks: usize,
    pub active_sessions: usize,
    pub live_state_entries: usize,
    pub state_history_entries: usize,
    pub performance_metrics: PerformanceState,
}
