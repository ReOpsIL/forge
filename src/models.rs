use crate::llm_handler::BlockConnection;
use portable_pty::Child;
use rand::{Rng, distributions::Alphanumeric};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tokio::sync::mpsc;
use tracing::info;

// Define the structure for a task

// Define the structure for task response from LLM
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Task {
    pub task_id: String,
    pub task_name: String,
    pub description: String,
    pub acceptance_criteria: Vec<String>,
    pub dependencies: Vec<String>,
    pub estimated_effort: String,
    pub files_affected: Vec<String>,
    pub function_signatures: Vec<String>,
    pub testing_requirements: Vec<String>,
    pub log: String,
    pub commit_id: String,
    pub status: String,
}

impl Task {
    pub fn new(description: String) -> Self {
        let unique_id: String = rand::thread_rng()
            .sample_iter(&Alphanumeric)
            .take(6)
            .map(char::from)
            .collect();
        Self {
            task_id: unique_id,
            task_name: "".to_string(),
            description,
            acceptance_criteria: Vec::new(),
            dependencies: Vec::new(),
            estimated_effort: "".to_string(),
            files_affected: Vec::new(),
            function_signatures: Vec::new(),
            testing_requirements: Vec::new(),
            log: String::new(),
            commit_id: "".to_string(),
            status: "".to_string(),
        }
    }

    /// Converts the Task attributes into a markdown-formatted prompt for LLM execution
    pub fn to_prompt(&self) -> String {
        let mut p = String::new();

        // Add the title (task name)
        p.push_str(&format!("# {} Task\n\n", self.task_name));

        // Add the objective (description)
        p.push_str("## Objective\n");
        p.push_str(&format!("{}\n\n", self.description));

        // Add task details section
        p.push_str("## Task Details\n\n");

        // Add primary requirements (acceptance criteria)
        if !self.acceptance_criteria.is_empty() {
            p.push_str("### Primary Requirements\n");
            for (i, criterion) in self.acceptance_criteria.iter().enumerate() {
                p.push_str(&format!("{}. {}\n", i + 1, criterion));
            }
            p.push_str("\n");
        }

        // Add function signatures if available
        if !self.function_signatures.is_empty() {
            p.push_str("### Function Signatures\n");
            p.push_str("```rust\n");
            for signature in &self.function_signatures {
                p.push_str(&format!("{}\n", signature));
            }
            p.push_str("```\n\n");
        }

        // Add acceptance criteria
        if !self.acceptance_criteria.is_empty() {
            p.push_str("### Acceptance Criteria\n");
            for criterion in &self.acceptance_criteria {
                p.push_str(&format!("- {}\n", criterion));
            }
            p.push_str("\n");
        }

        // Add files to modify
        if !self.files_affected.is_empty() {
            p.push_str("### Files to Modify\n");
            for file in &self.files_affected {
                p.push_str(&format!("- {}\n", file));
            }
            p.push_str("\n");
        }

        // Add dependencies
        if !self.dependencies.is_empty() {
            p.push_str("### Dependencies\n");
            for dependency in &self.dependencies {
                p.push_str(&format!("- {}\n", dependency));
            }
            p.push_str("\n");
        }

        // Add testing requirements
        if !self.testing_requirements.is_empty() {
            p.push_str("### Testing Requirements\n");
            for (i, requirement) in self.testing_requirements.iter().enumerate() {
                p.push_str(&format!("{}. {}\n", i + 1, requirement));
            }
            p.push_str("\n");
        }

        // Add implementation notes (if any)
        if !self.log.is_empty() {
            p.push_str("### Implementation Notes\n");
            p.push_str(&format!("{}\n\n", self.log));
        }

        // Add deliverables section
        p.push_str("## Deliverables\n");
        p.push_str("- Complete implementation of the task according to the requirements\n");
        if !self.testing_requirements.is_empty() {
            p.push_str("- Unit tests covering all acceptance criteria\n");
        }
        if !self.files_affected.is_empty() {
            p.push_str("- Updated files with the necessary changes\n");
        }

        p
    }
}

// Define the structure for module connections
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct InputConnection {
    pub from_module: String,
    pub output_type: String,
    pub input_id: String,
}

impl InputConnection {
    pub fn new(from_module: String, output_type: String) -> Self {
        // Generate a random 4-character alphanumeric ID
        let unique_id: String = rand::thread_rng()
            .sample_iter(&Alphanumeric)
            .take(6)
            .map(char::from)
            .collect();

        Self {
            from_module,
            output_type,
            input_id: unique_id,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct OutputConnection {
    pub to_module: String,
    pub input_type: String,
    pub output_id: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Connections {
    pub input_connections: Vec<InputConnection>,
    pub output_connections: Vec<OutputConnection>,
}

// Define the structure for a software module
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Block {
    pub name: String,
    pub block_id: String,
    pub description: String,
    pub inputs: Vec<BlockConnection>,
    pub outputs: Vec<BlockConnection>,
    pub connections: Connections,
    pub todo_list: HashMap<String, Task>,
}

impl Block {
    pub fn new(
        name: String,
        description: String,
        inputs: Vec<BlockConnection>,
        outputs: Vec<BlockConnection>,
    ) -> Self {
        // Generate a random 6-character alphanumeric ID
        let block_id: String = rand::thread_rng()
            .sample_iter(&Alphanumeric)
            .take(6)
            .map(char::from)
            .collect();

        Self {
            block_id,
            name,
            description,
            inputs,
            outputs,
            connections: Connections {
                input_connections: Vec::new(),
                output_connections: Vec::new(),
            },
            todo_list: HashMap::new(),
        }
    }
    pub fn update_task(mut self, task: Task) {
        println!("Updating task status {} {}", task.task_id, task.status);
        let task_id = task.clone().task_id;
        self.todo_list.insert(task_id, task);
    }
}

// Function to get a list of blocks (in a real application, this would likely fetch from a database)
pub fn get_blocks() -> Vec<Block> {
    vec![
        Block {
            block_id: "abc123".to_string(), // Sample block_id
            name: "DataIngestion".to_string(),
            description: "Handles the ingestion of raw data from various sources".to_string(),
            inputs: vec![BlockConnection::new()],
            outputs: vec![BlockConnection::new()],
            connections: Connections {
                input_connections: vec![],
                output_connections: vec![OutputConnection {
                    to_module: "DataProcessing".to_string(),
                    input_type: "ParsedData".to_string(),
                    output_id: "conn-1".to_string(),
                }],
            },
            todo_list: {
                let mut map = HashMap::new();
                let task1 = Task::new("Add support for CSV files".to_string());
                let task2 = Task::new("Improve error handling".to_string());
                map.insert(task1.task_id.clone(), task1);
                map.insert(task2.task_id.clone(), task2);
                map
            },
        },
        Block {
            block_id: "def456".to_string(), // Sample block_id
            name: "DataProcessing".to_string(),
            description: "Processes and transforms the parsed data".to_string(),
            inputs: vec![BlockConnection::new()],
            outputs: vec![BlockConnection::new()],
            connections: Connections {
                input_connections: vec![InputConnection::new(
                    "DataIngestion".to_string(),
                    "ParsedData".to_string(),
                )],
                output_connections: vec![OutputConnection {
                    to_module: "DataVisualization".to_string(),
                    input_type: "ProcessedData".to_string(),
                    output_id: "conn-2".to_string(),
                }],
            },
            todo_list: {
                let mut map = HashMap::new();
                let task1 = Task::new("Implement data normalization".to_string());
                let task2 = Task::new("Add support for filtering".to_string());
                map.insert(task1.task_id.clone(), task1);
                map.insert(task2.task_id.clone(), task2);
                map
            },
        },
        Block {
            block_id: "ghi789".to_string(), // Sample block_id
            name: "DataVisualization".to_string(),
            description: "Visualizes the processed data".to_string(),
            inputs: vec![BlockConnection::new()],
            outputs: vec![BlockConnection::new()],
            connections: Connections {
                input_connections: vec![InputConnection::new(
                    "DataProcessing".to_string(),
                    "ProcessedData".to_string(),
                )],
                output_connections: vec![],
            },
            todo_list: {
                let mut map = HashMap::new();
                let task1 = Task::new("Add more chart types".to_string());
                let task2 = Task::new("Implement interactive visualizations".to_string());
                map.insert(task1.task_id.clone(), task1);
                map.insert(task2.task_id.clone(), task2);
                map
            },
        },
    ]
}

/// Claude process session for managing individual Claude CLI instances
#[derive(Debug)]
pub struct ClaudeSession {
    pub session_id: String,
    pub created_at: Instant,
    pub last_activity: Arc<Mutex<Instant>>,
    pub child_process: Arc<Mutex<Option<Box<dyn Child + Send>>>>,
    pub stdin_tx: Arc<Mutex<Option<mpsc::UnboundedSender<String>>>>,
    pub is_active: Arc<Mutex<bool>>,
    pub connection_count: Arc<Mutex<u32>>,
    /// Broadcast channel for sending output to multiple WebSocket connections
    pub output_tx: Arc<Mutex<Option<tokio::sync::broadcast::Sender<String>>>>,
}

impl ClaudeSession {
    pub fn new(session_id: String) -> Self {
        let now = Instant::now();
        Self {
            session_id,
            created_at: now,
            last_activity: Arc::new(Mutex::new(now)),
            child_process: Arc::new(Mutex::new(None)),
            stdin_tx: Arc::new(Mutex::new(None)),
            is_active: Arc::new(Mutex::new(false)),
            connection_count: Arc::new(Mutex::new(0)),
            output_tx: Arc::new(Mutex::new(None)),
        }
    }

    pub fn set_child_process(&self, child: impl Child + Send + 'static) {
        if let Ok(mut process) = self.child_process.lock() {
            *process = Some(Box::new(child));
        }
    }

    pub fn set_child_process_box(&self, child: Box<dyn Child + Send>) {
        if let Ok(mut process) = self.child_process.lock() {
            *process = Some(child);
        }
    }

    pub fn set_stdin_tx(&self, tx: mpsc::UnboundedSender<String>) {
        if let Ok(mut stdin) = self.stdin_tx.lock() {
            *stdin = Some(tx);
        }
    }

    pub fn set_output_tx(&self, tx: tokio::sync::broadcast::Sender<String>) {
        if let Ok(mut output) = self.output_tx.lock() {
            *output = Some(tx);
        }
    }

    pub fn get_output_rx(&self) -> Option<tokio::sync::broadcast::Receiver<String>> {
        if let Ok(output) = self.output_tx.lock() {
            output.as_ref().map(|tx| tx.subscribe())
        } else {
            None
        }
    }

    pub fn update_activity(&self) {
        if let Ok(mut last_activity) = self.last_activity.lock() {
            *last_activity = Instant::now();
        }
    }

    pub fn is_timeout(&self, timeout_duration: Duration) -> bool {
        if let Ok(last_activity) = self.last_activity.lock() {
            last_activity.elapsed() > timeout_duration
        } else {
            true // If we can't check, assume timeout
        }
    }

    pub fn increment_connections(&self) {
        if let Ok(mut count) = self.connection_count.lock() {
            *count += 1;
        }
    }

    pub fn decrement_connections(&self) -> u32 {
        if let Ok(mut count) = self.connection_count.lock() {
            if *count > 0 {
                *count -= 1;
            }
            *count
        } else {
            0
        }
    }

    pub fn get_connection_count(&self) -> u32 {
        if let Ok(count) = self.connection_count.lock() {
            *count
        } else {
            0
        }
    }

    pub fn set_active(&self, active: bool) {
        if let Ok(mut is_active) = self.is_active.lock() {
            *is_active = active;
        }
    }

    pub fn is_active(&self) -> bool {
        if let Ok(is_active) = self.is_active.lock() {
            *is_active
        } else {
            false
        }
    }
}

/// Configuration for session management
#[derive(Debug, Clone)]
pub struct SessionConfig {
    pub max_concurrent_sessions: usize,
    pub session_timeout: Duration,
    pub cleanup_interval: Duration,
    pub max_memory_per_session: u64, // in bytes
    pub max_cpu_per_session: u32,    // percentage
}

impl Default for SessionConfig {
    fn default() -> Self {
        Self {
            max_concurrent_sessions: 10,
            session_timeout: Duration::from_secs(300), // 5 minutes
            cleanup_interval: Duration::from_secs(30), // 30 seconds
            max_memory_per_session: 100 * 1024 * 1024, // 100MB
            max_cpu_per_session: 20,                   // 20%
        }
    }
}

/// Session manager for handling multiple Claude processes
#[derive(Debug)]
pub struct ClaudeSessionManager {
    sessions: Arc<Mutex<HashMap<String, Arc<ClaudeSession>>>>,
    config: SessionConfig,
}

impl ClaudeSessionManager {
    pub fn new(config: SessionConfig) -> Self {
        Self {
            sessions: Arc::new(Mutex::new(HashMap::new())),
            config,
        }
    }

    pub fn create_session(&self, session_id: String) -> Result<(), String> {
        let mut sessions = self
            .sessions
            .lock()
            .map_err(|_| "Failed to acquire sessions lock")?;

        // Check if session already exists - if so, just return Ok (reuse existing session)
        if sessions.contains_key(&session_id) {
            info!("Reusing existing session: {}", session_id);
            return Ok(());
        }

        // Check if we've reached the maximum number of sessions
        if sessions.len() >= self.config.max_concurrent_sessions {
            return Err("Maximum number of concurrent sessions reached".to_string());
        }

        let session = Arc::new(ClaudeSession::new(session_id.clone()));
        sessions.insert(session_id, session);

        Ok(())
    }

    pub fn get_session(&self, session_id: &str) -> Option<Arc<ClaudeSession>> {
        if let Ok(sessions) = self.sessions.lock() {
            sessions.get(session_id).map(Arc::clone)
        } else {
            None
        }
    }

    pub fn cleanup_session(&self, session_id: &str) -> Result<(), String> {
        let mut sessions = self
            .sessions
            .lock()
            .map_err(|_| "Failed to acquire sessions lock")?;

        if let Some(session) = sessions.remove(session_id) {
            // Set session as inactive
            session.set_active(false);

            // Kill the process if it's still running
            if let Ok(mut child_opt) = session.child_process.lock() {
                if let Some(ref mut child) = child_opt.as_mut() {
                    let _ = child.kill().ok();
                }
            }
        }

        Ok(())
    }

    pub fn cleanup_expired_sessions(&self) {
        if let Ok(mut sessions) = self.sessions.lock() {
            let expired_sessions: Vec<String> = sessions
                .iter()
                .filter(|(_, session)| {
                    session.is_timeout(self.config.session_timeout)
                        && session.get_connection_count() == 0
                })
                .map(|(id, _)| id.clone())
                .collect();

            for session_id in expired_sessions {
                if let Some(session) = sessions.remove(&session_id) {
                    session.set_active(false);
                    if let Ok(mut child_opt) = session.child_process.lock() {
                        if let Some(ref mut child) = child_opt.as_mut() {
                            let _ = child.kill().ok();
                        }
                    }
                }
            }
        }
    }

    pub fn get_active_sessions_count(&self) -> usize {
        if let Ok(sessions) = self.sessions.lock() {
            sessions.len()
        } else {
            0
        }
    }

    pub fn get_session_ids(&self) -> Vec<String> {
        if let Ok(sessions) = self.sessions.lock() {
            sessions.keys().cloned().collect()
        } else {
            Vec::new()
        }
    }

    /// Start the cleanup task that runs periodically
    pub fn start_cleanup_task(self: Arc<Self>) {
        let session_manager = Arc::clone(&self);
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(session_manager.config.cleanup_interval);
            loop {
                interval.tick().await;
                session_manager.cleanup_expired_sessions();
            }
        });
    }
}

// Test function to verify that the random ID generation works correctly
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_input_connection_id_generation() {
        let conn = InputConnection::new("TestModule".to_string(), "TestOutput".to_string());

        // Check that the unique_id is 4 characters long
        assert_eq!(conn.input_id.len(), 4);

        // Check that multiple calls generate different IDs
        let conn2 = InputConnection::new("TestModule".to_string(), "TestOutput".to_string());
        assert_ne!(conn.input_id, conn2.input_id);
    }

    #[test]
    fn test_task_to_markdown_prompt() {
        // Create a sample task with all fields populated
        let mut task = Task::new(
            "Implement a Deck struct in Rust with standard playing card functionality".to_string(),
        );
        task.task_name = "Rust Card Deck Implementation".to_string();
        task.acceptance_criteria = vec![
            "Deck::new() must create a deck with exactly 52 unique cards".to_string(),
            "shuffle() must randomize the card order in place".to_string(),
            "deal(n) must return a Vec<Card> of size n and reduce the deck size by n".to_string(),
            "deal() should return None if insufficient cards remain".to_string(),
        ];
        task.dependencies = vec!["rand = \"0.8\"".to_string()];
        task.files_affected = vec![
            "src/game/deck.rs".to_string(),
            "src/game/mod.rs".to_string(),
            "Cargo.toml".to_string(),
        ];
        task.function_signatures = vec![
            "struct Deck {".to_string(),
            "    cards: Vec<Card>".to_string(),
            "}".to_string(),
            "".to_string(),
            "impl Deck {".to_string(),
            "    pub fn new() -> Self;".to_string(),
            "    pub fn shuffle(&mut self);".to_string(),
            "    pub fn deal(&mut self, count: usize) -> Option<Vec<Card>>;".to_string(),
            "}".to_string(),
        ];
        task.testing_requirements = vec![
            "Deck Creation Test: Verify new() creates exactly 52 unique cards".to_string(),
            "Deal Functionality Test: Verify deal() returns correct number of cards and updates deck state".to_string(),
            "Shuffle Test: Verify shuffled deck order differs from newly created ordered deck".to_string(),
            "Edge Case Test: Test deal() behavior when requesting more cards than available".to_string(),
        ];
        task.log = "Ensure the Card type is properly imported/defined. Use rand::thread_rng() and shuffle() method from rand::seq::SliceRandom trait. Handle edge cases gracefully.".to_string();

        // Generate the markdown prompt
        let markdown = task.to_prompt();

        // Verify the markdown contains expected sections
        assert!(markdown.contains("# Rust Card Deck Implementation Task"));
        assert!(markdown.contains("## Objective"));
        assert!(markdown.contains("## Task Details"));
        assert!(markdown.contains("### Primary Requirements"));
        assert!(markdown.contains("### Function Signatures"));
        assert!(markdown.contains("### Acceptance Criteria"));
        assert!(markdown.contains("### Files to Modify"));
        assert!(markdown.contains("### Dependencies"));
        assert!(markdown.contains("### Testing Requirements"));
        assert!(markdown.contains("### Implementation Notes"));
        assert!(markdown.contains("## Deliverables"));

        // Verify specific content is included
        assert!(
            markdown.contains(
                "Implement a Deck struct in Rust with standard playing card functionality"
            )
        );
        assert!(markdown.contains("```rust"));
        assert!(markdown.contains("pub fn new() -> Self;"));
        assert!(markdown.contains("- src/game/deck.rs"));
        assert!(markdown.contains("- rand = \"0.8\""));
        assert!(
            markdown
                .contains("1. Deck Creation Test: Verify new() creates exactly 52 unique cards")
        );
    }

    #[test]
    fn test_claude_session_creation() {
        let session_id = "test_session_123".to_string();
        let session = ClaudeSession::new(session_id.clone());

        assert_eq!(session.session_id, session_id);
        assert!(!session.is_active());
        assert_eq!(session.get_connection_count(), 0);
    }

    #[test]
    fn test_claude_session_activity_update() {
        let session = ClaudeSession::new("test_session".to_string());
        let initial_time = session.created_at;

        // Wait a small amount of time
        std::thread::sleep(std::time::Duration::from_millis(10));

        session.update_activity();

        // Check that activity was updated (should be later than creation time)
        if let Ok(last_activity) = session.last_activity.lock() {
            assert!(*last_activity > initial_time);
        }
    }

    #[test]
    fn test_claude_session_connection_management() {
        let session = ClaudeSession::new("test_session".to_string());

        // Test incrementing connections
        session.increment_connections();
        assert_eq!(session.get_connection_count(), 1);

        session.increment_connections();
        assert_eq!(session.get_connection_count(), 2);

        // Test decrementing connections
        let remaining = session.decrement_connections();
        assert_eq!(remaining, 1);
        assert_eq!(session.get_connection_count(), 1);

        let remaining = session.decrement_connections();
        assert_eq!(remaining, 0);
        assert_eq!(session.get_connection_count(), 0);

        // Test that it doesn't go below zero
        let remaining = session.decrement_connections();
        assert_eq!(remaining, 0);
        assert_eq!(session.get_connection_count(), 0);
    }

    #[test]
    fn test_claude_session_timeout() {
        let session = ClaudeSession::new("test_session".to_string());
        let short_timeout = Duration::from_millis(1);

        // Should not be timed out initially
        assert!(!session.is_timeout(Duration::from_secs(60)));

        // Wait for timeout
        std::thread::sleep(Duration::from_millis(10));

        // Should be timed out with short timeout
        assert!(session.is_timeout(short_timeout));
    }

    #[test]
    fn test_claude_session_manager_creation() {
        let config = SessionConfig::default();
        let manager = ClaudeSessionManager::new(config);

        assert_eq!(manager.get_active_sessions_count(), 0);
        assert!(manager.get_session_ids().is_empty());
    }

    #[test]
    fn test_claude_session_manager_session_lifecycle() {
        let config = SessionConfig::default();
        let manager = ClaudeSessionManager::new(config);

        let session_id = "test_session_456".to_string();

        // Create session
        assert!(manager.create_session(session_id.clone()).is_ok());
        assert_eq!(manager.get_active_sessions_count(), 1);
        assert!(manager.get_session_ids().contains(&session_id));

        // Try to create duplicate session
        assert!(manager.create_session(session_id.clone()).is_err());

        // Get session
        let session = manager.get_session(&session_id);
        assert!(session.is_some());

        // Cleanup session
        assert!(manager.cleanup_session(&session_id).is_ok());
        assert_eq!(manager.get_active_sessions_count(), 0);
        assert!(!manager.get_session_ids().contains(&session_id));
    }

    #[test]
    fn test_claude_session_manager_max_sessions() {
        let mut config = SessionConfig::default();
        config.max_concurrent_sessions = 2;
        let manager = ClaudeSessionManager::new(config);

        // Create maximum number of sessions
        assert!(manager.create_session("session1".to_string()).is_ok());
        assert!(manager.create_session("session2".to_string()).is_ok());

        // Try to create one more session (should fail)
        assert!(manager.create_session("session3".to_string()).is_err());

        assert_eq!(manager.get_active_sessions_count(), 2);
    }

    #[test]
    fn test_claude_session_manager_cleanup_expired() {
        let mut config = SessionConfig::default();
        config.session_timeout = Duration::from_millis(50);
        let manager = ClaudeSessionManager::new(config);

        // Create a session
        let session_id = "expiring_session".to_string();
        assert!(manager.create_session(session_id.clone()).is_ok());

        // Wait for session to timeout
        std::thread::sleep(Duration::from_millis(100));

        // Cleanup expired sessions
        manager.cleanup_expired_sessions();

        // Session should be cleaned up
        assert_eq!(manager.get_active_sessions_count(), 0);
        assert!(manager.get_session(&session_id).is_none());
    }

    #[test]
    fn test_claude_session_manager_cleanup_with_active_connections() {
        let mut config = SessionConfig::default();
        config.session_timeout = Duration::from_millis(50);
        let manager = ClaudeSessionManager::new(config);

        // Create a session with active connections
        let session_id = "active_session".to_string();
        assert!(manager.create_session(session_id.clone()).is_ok());

        if let Some(session) = manager.get_session(&session_id) {
            session.increment_connections(); // Add active connection
        }

        // Wait for session to timeout
        std::thread::sleep(Duration::from_millis(100));

        // Cleanup expired sessions
        manager.cleanup_expired_sessions();

        // Session should NOT be cleaned up because it has active connections
        assert_eq!(manager.get_active_sessions_count(), 1);
        assert!(manager.get_session(&session_id).is_some());
    }
}
