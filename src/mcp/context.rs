/// Context management for MCP - handles shared state and data between tools
///
/// This module provides a centralized context store that enables tools to share
/// data, maintain state, and coordinate activities across the MCP session.
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use std::time::{Duration, SystemTime};
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

use crate::mcp::errors::{ContextError, MCPError, MCPResult};

/// Context store for managing shared data across tools
#[derive(Debug)]
pub struct ContextStore {
    /// Shared data accessible to all tools
    shared_data: HashMap<String, ContextEntry>,

    /// Session-specific data
    session_data: HashMap<String, HashMap<String, ContextEntry>>,

    /// Event history for debugging and rollback
    event_history: VecDeque<ContextEvent>,

    /// Configuration for the context store
    config: ContextConfig,

    /// Cache for frequently accessed data
    cache: HashMap<String, CachedEntry>,
}

/// Context entry with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextEntry {
    /// The actual data value
    pub value: Value,

    /// When this entry was created
    pub created_at: SystemTime,

    /// When this entry was last modified
    pub last_modified: SystemTime,

    /// Who last modified this entry (tool name)
    pub modified_by: String,

    /// Access count for optimization
    pub access_count: u64,

    /// Entry expiration time (optional)
    pub expires_at: Option<SystemTime>,

    /// Entry tags for categorization
    pub tags: Vec<String>,

    /// Whether this entry is read-only
    pub read_only: bool,
}

/// Context event for audit trail
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextEvent {
    /// Event identifier
    pub id: String,

    /// Type of event
    pub event_type: ContextEventType,

    /// Key that was affected
    pub key: String,

    /// Session ID (if applicable)
    pub session_id: Option<String>,

    /// Tool that triggered the event
    pub tool_name: String,

    /// Previous value (for updates/deletes)
    pub previous_value: Option<Value>,

    /// New value (for creates/updates)
    pub new_value: Option<Value>,

    /// Event timestamp
    pub timestamp: SystemTime,
}

/// Types of context events
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum ContextEventType {
    Create,
    Read,
    Update,
    Delete,
    Expire,
    Cache,
}

/// Context store configuration
#[derive(Debug, Clone)]
pub struct ContextConfig {
    /// Maximum number of entries in shared data
    pub max_shared_entries: usize,

    /// Maximum number of entries per session
    pub max_session_entries: usize,

    /// Maximum size of event history
    pub max_event_history: usize,

    /// Default expiration time for entries
    pub default_expiration: Option<Duration>,

    /// Cache size limit
    pub max_cache_size: usize,

    /// Cache TTL
    pub cache_ttl: Duration,

    /// Whether to enable event history
    pub enable_event_history: bool,
}

impl Default for ContextConfig {
    fn default() -> Self {
        Self {
            max_shared_entries: 10000,
            max_session_entries: 1000,
            max_event_history: 5000,
            default_expiration: Some(Duration::from_secs(86400)), // 24 hours
            max_cache_size: 500,
            cache_ttl: Duration::from_secs(3600), // 1 hour
            enable_event_history: true,
        }
    }
}

/// Cached entry with metadata
#[derive(Debug, Clone)]
struct CachedEntry {
    value: Value,
    created_at: SystemTime,
    access_count: u64,
}

impl ContextStore {
    /// Create a new context store
    pub fn new() -> Self {
        Self::with_config(ContextConfig::default())
    }

    /// Create a context store with custom configuration
    pub fn with_config(config: ContextConfig) -> Self {
        Self {
            shared_data: HashMap::new(),
            session_data: HashMap::new(),
            event_history: VecDeque::new(),
            config,
            cache: HashMap::new(),
        }
    }

    /// Set a value in shared context
    pub fn set_shared(
        &mut self,
        key: impl Into<String>,
        value: Value,
        tool_name: impl Into<String>,
    ) -> MCPResult<()> {
        let key = key.into();
        let tool_name = tool_name.into();

        // Check limits
        if self.shared_data.len() >= self.config.max_shared_entries
            && !self.shared_data.contains_key(&key)
        {
            return Err(MCPError::Context(ContextError::StorageLimit(
                "Shared data limit exceeded".to_string(),
            )));
        }

        let now = SystemTime::now();
        let previous_value = self.shared_data.get(&key).map(|entry| entry.value.clone());

        let entry = ContextEntry {
            value: value.clone(),
            created_at: if previous_value.is_some() {
                self.shared_data[&key].created_at
            } else {
                now
            },
            last_modified: now,
            modified_by: tool_name.clone(),
            access_count: if previous_value.is_some() {
                self.shared_data[&key].access_count
            } else {
                0
            },
            expires_at: self.config.default_expiration.map(|d| now + d),
            tags: vec![],
            read_only: false,
        };

        self.shared_data.insert(key.clone(), entry);

        // Record event
        if self.config.enable_event_history {
            self.record_event(ContextEvent {
                id: uuid::Uuid::new_v4().to_string(),
                event_type: if previous_value.is_some() {
                    ContextEventType::Update
                } else {
                    ContextEventType::Create
                },
                key: key.clone(),
                session_id: None,
                tool_name,
                previous_value,
                new_value: Some(value),
                timestamp: now,
            });
        }

        // Invalidate cache
        self.cache.remove(&key);

        debug!("Set shared context value: {}", key);
        Ok(())
    }

    /// Get a value from shared context
    pub fn get_shared(&mut self, key: &str) -> Option<Value> {
        // Check cache first
        if let Some(cached) = self.cache.get(key) {
            if cached.created_at.elapsed().unwrap_or(Duration::MAX) < self.config.cache_ttl {
                return Some(cached.value.clone());
            } else {
                self.cache.remove(key);
            }
        }

        if let Some(entry) = self.shared_data.get_mut(key) {
            // Check expiration
            if let Some(expires_at) = entry.expires_at {
                if SystemTime::now() > expires_at {
                    self.shared_data.remove(key);
                    return None;
                }
            }

            entry.access_count += 1;
            let value = entry.value.clone();

            // Update cache
            if self.cache.len() < self.config.max_cache_size {
                self.cache.insert(
                    key.to_string(),
                    CachedEntry {
                        value: value.clone(),
                        created_at: SystemTime::now(),
                        access_count: 1,
                    },
                );
            }

            // Record event
            if self.config.enable_event_history {
                self.record_event(ContextEvent {
                    id: uuid::Uuid::new_v4().to_string(),
                    event_type: ContextEventType::Read,
                    key: key.to_string(),
                    session_id: None,
                    tool_name: "unknown".to_string(),
                    previous_value: None,
                    new_value: Some(value.clone()),
                    timestamp: SystemTime::now(),
                });
            }

            Some(value)
        } else {
            None
        }
    }

    /// Set a value in session context
    pub fn set_session(
        &mut self,
        session_id: impl Into<String>,
        key: impl Into<String>,
        value: Value,
        tool_name: impl Into<String>,
    ) -> MCPResult<()> {
        let session_id = session_id.into();
        let key = key.into();
        let tool_name = tool_name.into();

        let session_data = self
            .session_data
            .entry(session_id.clone())
            .or_insert_with(HashMap::new);

        // Check limits
        if session_data.len() >= self.config.max_session_entries && !session_data.contains_key(&key)
        {
            return Err(MCPError::Context(ContextError::StorageLimit(
                "Session data limit exceeded".to_string(),
            )));
        }

        let now = SystemTime::now();
        let previous_value = session_data.get(&key).map(|entry| entry.value.clone());

        let entry = ContextEntry {
            value: value.clone(),
            created_at: if previous_value.is_some() {
                session_data[&key].created_at
            } else {
                now
            },
            last_modified: now,
            modified_by: tool_name.clone(),
            access_count: if previous_value.is_some() {
                session_data[&key].access_count
            } else {
                0
            },
            expires_at: self.config.default_expiration.map(|d| now + d),
            tags: vec![],
            read_only: false,
        };

        session_data.insert(key.clone(), entry);

        // Record event
        if self.config.enable_event_history {
            self.record_event(ContextEvent {
                id: uuid::Uuid::new_v4().to_string(),
                event_type: if previous_value.is_some() {
                    ContextEventType::Update
                } else {
                    ContextEventType::Create
                },
                key: key.clone(),
                session_id: Some(session_id.clone()),
                tool_name,
                previous_value,
                new_value: Some(value),
                timestamp: now,
            });
        }

        debug!(
            "Set session context value: {} in session {}",
            key, session_id
        );
        Ok(())
    }

    /// Get a value from session context
    pub fn get_session(&mut self, session_id: &str, key: &str) -> Option<Value> {
        if let Some(session_data) = self.session_data.get_mut(session_id) {
            if let Some(entry) = session_data.get_mut(key) {
                // Check expiration
                if let Some(expires_at) = entry.expires_at {
                    if SystemTime::now() > expires_at {
                        session_data.remove(key);
                        return None;
                    }
                }

                entry.access_count += 1;
                let value = entry.value.clone();

                // Record event
                if self.config.enable_event_history {
                    self.record_event(ContextEvent {
                        id: uuid::Uuid::new_v4().to_string(),
                        event_type: ContextEventType::Read,
                        key: key.to_string(),
                        session_id: Some(session_id.to_string()),
                        tool_name: "unknown".to_string(),
                        previous_value: None,
                        new_value: Some(value.clone()),
                        timestamp: SystemTime::now(),
                    });
                }

                Some(value)
            } else {
                None
            }
        } else {
            None
        }
    }

    /// Delete a value from shared context
    pub fn delete_shared(&mut self, key: &str, tool_name: impl Into<String>) -> MCPResult<bool> {
        let tool_name = tool_name.into();

        if let Some(entry) = self.shared_data.remove(key) {
            if entry.read_only {
                // Restore the entry if it was read-only
                self.shared_data.insert(key.to_string(), entry);
                return Err(MCPError::Context(ContextError::ReadOnly(key.to_string())));
            }

            // Record event
            if self.config.enable_event_history {
                self.record_event(ContextEvent {
                    id: uuid::Uuid::new_v4().to_string(),
                    event_type: ContextEventType::Delete,
                    key: key.to_string(),
                    session_id: None,
                    tool_name,
                    previous_value: Some(entry.value),
                    new_value: None,
                    timestamp: SystemTime::now(),
                });
            }

            // Remove from cache
            self.cache.remove(key);

            debug!("Deleted shared context value: {}", key);
            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// Delete a value from session context
    pub fn delete_session(
        &mut self,
        session_id: &str,
        key: &str,
        tool_name: impl Into<String>,
    ) -> MCPResult<bool> {
        let tool_name = tool_name.into();

        if let Some(session_data) = self.session_data.get_mut(session_id) {
            if let Some(entry) = session_data.remove(key) {
                if entry.read_only {
                    // Restore the entry if it was read-only
                    session_data.insert(key.to_string(), entry);
                    return Err(MCPError::Context(ContextError::ReadOnly(key.to_string())));
                }

                // Record event
                if self.config.enable_event_history {
                    self.record_event(ContextEvent {
                        id: uuid::Uuid::new_v4().to_string(),
                        event_type: ContextEventType::Delete,
                        key: key.to_string(),
                        session_id: Some(session_id.to_string()),
                        tool_name,
                        previous_value: Some(entry.value),
                        new_value: None,
                        timestamp: SystemTime::now(),
                    });
                }

                debug!(
                    "Deleted session context value: {} in session {}",
                    key, session_id
                );
                Ok(true)
            } else {
                Ok(false)
            }
        } else {
            Ok(false)
        }
    }

    /// List all keys in shared context
    pub fn list_shared_keys(&self) -> Vec<String> {
        self.shared_data.keys().cloned().collect()
    }

    /// List all keys in session context
    pub fn list_session_keys(&self, session_id: &str) -> Vec<String> {
        self.session_data
            .get(session_id)
            .map(|data| data.keys().cloned().collect())
            .unwrap_or_default()
    }

    /// Get context metadata
    pub fn get_metadata(&self, key: &str, session_id: Option<&str>) -> Option<ContextEntry> {
        if let Some(session_id) = session_id {
            self.session_data
                .get(session_id)
                .and_then(|data| data.get(key).cloned())
        } else {
            self.shared_data.get(key).cloned()
        }
    }

    /// Set entry as read-only
    pub fn set_read_only(
        &mut self,
        key: &str,
        session_id: Option<&str>,
        read_only: bool,
    ) -> MCPResult<()> {
        if let Some(session_id) = session_id {
            if let Some(session_data) = self.session_data.get_mut(session_id) {
                if let Some(entry) = session_data.get_mut(key) {
                    entry.read_only = read_only;
                    return Ok(());
                }
            }
        } else if let Some(entry) = self.shared_data.get_mut(key) {
            entry.read_only = read_only;
            return Ok(());
        }

        Err(MCPError::Context(ContextError::NotFound(key.to_string())))
    }

    /// Add tags to an entry
    pub fn add_tags(
        &mut self,
        key: &str,
        session_id: Option<&str>,
        tags: Vec<String>,
    ) -> MCPResult<()> {
        if let Some(session_id) = session_id {
            if let Some(session_data) = self.session_data.get_mut(session_id) {
                if let Some(entry) = session_data.get_mut(key) {
                    entry.tags.extend(tags);
                    return Ok(());
                }
            }
        } else if let Some(entry) = self.shared_data.get_mut(key) {
            entry.tags.extend(tags);
            return Ok(());
        }

        Err(MCPError::Context(ContextError::NotFound(key.to_string())))
    }

    /// Search entries by tags
    pub fn search_by_tags(&self, tags: &[String], session_id: Option<&str>) -> Vec<String> {
        let mut results = Vec::new();

        let data = if let Some(session_id) = session_id {
            self.session_data
                .get(session_id)
                .map(|d| d as &HashMap<String, ContextEntry>)
        } else {
            Some(&self.shared_data)
        };

        if let Some(data) = data {
            for (key, entry) in data {
                if tags.iter().any(|tag| entry.tags.contains(tag)) {
                    results.push(key.clone());
                }
            }
        }

        results
    }

    /// Clean up expired entries
    pub fn cleanup_expired(&mut self) -> usize {
        let now = SystemTime::now();
        let mut cleaned_count = 0;

        // Clean shared data
        let expired_shared: Vec<String> = self
            .shared_data
            .iter()
            .filter(|(_, entry)| {
                entry
                    .expires_at
                    .map_or(false, |expires_at| now > expires_at)
            })
            .map(|(key, _)| key.clone())
            .collect();

        for key in expired_shared {
            self.shared_data.remove(&key);
            self.cache.remove(&key);
            cleaned_count += 1;
        }

        // Clean session data
        for session_data in self.session_data.values_mut() {
            let expired_session: Vec<String> = session_data
                .iter()
                .filter(|(_, entry)| {
                    entry
                        .expires_at
                        .map_or(false, |expires_at| now > expires_at)
                })
                .map(|(key, _)| key.clone())
                .collect();

            for key in expired_session {
                session_data.remove(&key);
                cleaned_count += 1;
            }
        }

        // Clean empty sessions
        self.session_data.retain(|_, data| !data.is_empty());

        // Clean cache
        let expired_cache: Vec<String> = self
            .cache
            .iter()
            .filter(|(_, entry)| {
                entry.created_at.elapsed().unwrap_or(Duration::MAX) > self.config.cache_ttl
            })
            .map(|(key, _)| key.clone())
            .collect();

        for key in expired_cache {
            self.cache.remove(&key);
        }

        if cleaned_count > 0 {
            info!("Cleaned up {} expired context entries", cleaned_count);
        }

        cleaned_count
    }

    /// Get context statistics
    pub fn get_statistics(&self) -> ContextStatistics {
        let total_shared = self.shared_data.len();
        let total_session = self.session_data.values().map(|data| data.len()).sum();
        let total_cached = self.cache.len();
        let total_events = self.event_history.len();

        let most_accessed_shared = self
            .shared_data
            .iter()
            .max_by_key(|(_, entry)| entry.access_count)
            .map(|(key, entry)| (key.clone(), entry.access_count));

        ContextStatistics {
            total_shared_entries: total_shared,
            total_session_entries: total_session,
            total_cached_entries: total_cached,
            total_events,
            active_sessions: self.session_data.len(),
            most_accessed_shared,
        }
    }

    /// Record a context event
    fn record_event(&mut self, event: ContextEvent) {
        if self.event_history.len() >= self.config.max_event_history {
            self.event_history.pop_front();
        }
        self.event_history.push_back(event);
    }

    /// Get recent events
    pub fn get_recent_events(&self, limit: usize) -> Vec<&ContextEvent> {
        self.event_history.iter().rev().take(limit).collect()
    }

    /// Clear session data
    pub fn clear_session(&mut self, session_id: &str) -> usize {
        if let Some(session_data) = self.session_data.remove(session_id) {
            let count = session_data.len();

            // Record events for all deleted entries
            if self.config.enable_event_history {
                for (key, entry) in session_data {
                    self.record_event(ContextEvent {
                        id: uuid::Uuid::new_v4().to_string(),
                        event_type: ContextEventType::Delete,
                        key,
                        session_id: Some(session_id.to_string()),
                        tool_name: "system".to_string(),
                        previous_value: Some(entry.value),
                        new_value: None,
                        timestamp: SystemTime::now(),
                    });
                }
            }

            info!("Cleared {} entries from session {}", count, session_id);
            count
        } else {
            0
        }
    }
}

/// Context statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextStatistics {
    pub total_shared_entries: usize,
    pub total_session_entries: usize,
    pub total_cached_entries: usize,
    pub total_events: usize,
    pub active_sessions: usize,
    pub most_accessed_shared: Option<(String, u64)>,
}

/// Context store manager with automatic cleanup
pub struct ContextManager {
    store: Arc<RwLock<ContextStore>>,
    cleanup_interval: Duration,
}

impl ContextManager {
    pub fn new(store: ContextStore) -> Self {
        Self {
            store: Arc::new(RwLock::new(store)),
            cleanup_interval: Duration::from_secs(300), // 5 minutes
        }
    }

    pub fn get_store(&self) -> Arc<RwLock<ContextStore>> {
        self.store.clone()
    }

    /// Start automatic cleanup service
    pub async fn start_cleanup_service(&self) {
        let store = self.store.clone();
        let interval = self.cleanup_interval;

        tokio::spawn(async move {
            let mut cleanup_timer = tokio::time::interval(interval);

            loop {
                cleanup_timer.tick().await;

                let mut store = store.write().await;
                let cleaned_count = store.cleanup_expired();

                if cleaned_count > 0 {
                    debug!("Context cleanup: removed {} expired entries", cleaned_count);
                }
            }
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_shared_context() {
        let mut store = ContextStore::new();

        // Set value
        assert!(
            store
                .set_shared("test_key", json!("test_value"), "test_tool")
                .is_ok()
        );

        // Get value
        assert_eq!(store.get_shared("test_key"), Some(json!("test_value")));

        // Delete value
        assert!(store.delete_shared("test_key", "test_tool").unwrap());
        assert_eq!(store.get_shared("test_key"), None);
    }

    #[test]
    fn test_session_context() {
        let mut store = ContextStore::new();

        // Set value
        assert!(
            store
                .set_session("session1", "test_key", json!("test_value"), "test_tool")
                .is_ok()
        );

        // Get value
        assert_eq!(
            store.get_session("session1", "test_key"),
            Some(json!("test_value"))
        );

        // Clear session
        assert_eq!(store.clear_session("session1"), 1);
        assert_eq!(store.get_session("session1", "test_key"), None);
    }

    #[test]
    fn test_read_only() {
        let mut store = ContextStore::new();

        // Set value
        store
            .set_shared("readonly_key", json!("value"), "test_tool")
            .unwrap();

        // Make read-only
        store.set_read_only("readonly_key", None, true).unwrap();

        // Try to delete (should fail)
        assert!(store.delete_shared("readonly_key", "test_tool").is_err());

        // Value should still exist
        assert_eq!(store.get_shared("readonly_key"), Some(json!("value")));
    }
}
