use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use std::fs::{self, OpenOptions};
use std::io::{Error as IoError, ErrorKind, Write};
use std::path::{Path, PathBuf};
use thiserror::Error;
use tracing::{debug, error, info, warn};

#[derive(Error, Debug)]
pub enum LogManagerError {
    #[error("IO error: {0}")]
    Io(#[from] IoError),
    #[error("Invalid path: {0}")]
    InvalidPath(String),
    #[error("Permission denied: {0}")]
    PermissionDenied(String),
    #[error("Disk space insufficient")]
    DiskSpaceInsufficient,
    #[error("Log file not found: {0}")]
    LogFileNotFound(String),
}

/// Configuration for log management
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogManagerConfig {
    /// Base directory for all logs
    pub base_log_dir: PathBuf,
    /// Maximum number of days to retain logs
    pub retention_days: u32,
    /// Maximum size per log file in bytes
    pub max_file_size: u64,
    /// Maximum total size for all logs in bytes
    pub max_total_size: u64,
    /// Whether to compress old log files
    pub compress_old_files: bool,
    /// File extension for log files
    pub log_file_extension: String,
}

impl Default for LogManagerConfig {
    fn default() -> Self {
        Self {
            base_log_dir: PathBuf::from("./logs"),
            retention_days: 30,                    // Keep logs for 30 days
            max_file_size: 100 * 1024 * 1024,     // 100MB per file
            max_total_size: 1024 * 1024 * 1024,   // 1GB total
            compress_old_files: false,             // Don't compress by default
            log_file_extension: "log".to_string(),
        }
    }
}

/// Metadata for a log file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogFileMetadata {
    pub block_id: String,
    pub task_id: String,
    pub file_path: PathBuf,
    pub created_at: DateTime<Utc>,
    pub last_modified: DateTime<Utc>,
    pub file_size: u64,
    pub is_compressed: bool,
}

/// Log manager for handling directory structure and file operations
#[derive(Debug)]
pub struct LogManager {
    config: LogManagerConfig,
}

impl LogManager {
    /// Create a new log manager with the given configuration
    pub fn new(config: LogManagerConfig) -> Self {
        Self { config }
    }

    /// Create a new log manager with default configuration
    pub fn with_default_config() -> Self {
        Self::new(LogManagerConfig::default())
    }

    /// Create the log directory structure for a specific block and task
    pub fn create_log_directory(&self, block_id: &str, task_id: &str) -> Result<PathBuf, LogManagerError> {
        // Validate input parameters
        if block_id.is_empty() || task_id.is_empty() {
            return Err(LogManagerError::InvalidPath(
                "Block ID and Task ID cannot be empty".to_string(),
            ));
        }

        // Sanitize IDs to prevent path traversal attacks
        let sanitized_block_id = sanitize_path_component(block_id);
        let sanitized_task_id = sanitize_path_component(task_id);

        // Create the directory path: ./logs/[block_id]/[task_id]
        let log_dir = self.config.base_log_dir
            .join(&sanitized_block_id)
            .join(&sanitized_task_id);

        // Create the directory structure
        fs::create_dir_all(&log_dir).map_err(|e| {
            error!("Failed to create log directory {:?}: {}", log_dir, e);
            LogManagerError::Io(e)
        })?;

        info!("Created log directory: {:?}", log_dir);
        Ok(log_dir)
    }

    /// Generate a log file path with timestamp
    pub fn generate_log_file_path(&self, block_id: &str, task_id: &str) -> Result<PathBuf, LogManagerError> {
        let log_dir = self.create_log_directory(block_id, task_id)?;

        // Generate timestamp for unique filename
        let timestamp = Utc::now().format("%Y%m%d_%H%M%S_%3f");
        let filename = format!("task_{}_{}.{}",
                               sanitize_path_component(task_id),
                               timestamp,
                               self.config.log_file_extension
        );

        let file_path = log_dir.join(filename);
        Ok(file_path)
    }

    /// Write stream data to a log file
    pub fn write_stream_log(&self, file_path: &Path, data: &[u8]) -> Result<(), LogManagerError> {
        // Check if file exists and is within size limits
        if file_path.exists() {
            let metadata = fs::metadata(file_path)?;
            if metadata.len() + data.len() as u64 > self.config.max_file_size {
                warn!("Log file {:?} would exceed size limit, rotation needed", file_path);
                return Err(LogManagerError::DiskSpaceInsufficient);
            }
        }

        // Open file in append mode
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(file_path)
            .map_err(|e| {
                error!("Failed to open log file {:?}: {}", file_path, e);
                LogManagerError::Io(e)
            })?;

        // Write data to file
        file.write_all(data).map_err(|e| {
            error!("Failed to write to log file {:?}: {}", file_path, e);
            LogManagerError::Io(e)
        })?;

        file.flush().map_err(|e| {
            error!("Failed to flush log file {:?}: {}", file_path, e);
            LogManagerError::Io(e)
        })?;

        debug!("Wrote {} bytes to log file: {:?}", data.len(), file_path);
        Ok(())
    }

    /// Read the contents of a log file
    pub fn read_log_file(&self, block_id: &str, task_id: &str) -> Result<Vec<u8>, LogManagerError> {
        let log_dir = self.config.base_log_dir
            .join(sanitize_path_component(block_id))
            .join(sanitize_path_component(task_id));

        if !log_dir.exists() {
            return Err(LogManagerError::LogFileNotFound(format!(
                "Log directory not found for block_id: {}, task_id: {}", block_id, task_id
            )));
        }

        // Find the most recent log file for this task
        let log_file = self.find_latest_log_file(&log_dir, task_id)?;

        let contents = fs::read(&log_file).map_err(|e| {
            error!("Failed to read log file {:?}: {}", log_file, e);
            LogManagerError::Io(e)
        })?;

        debug!("Read {} bytes from log file: {:?}", contents.len(), log_file);
        Ok(contents)
    }

    /// Find the latest log file for a given task in a directory
    fn find_latest_log_file(&self, dir: &Path, task_id: &str) -> Result<PathBuf, LogManagerError> {
        let entries = fs::read_dir(dir).map_err(LogManagerError::Io)?;

        let mut latest_file: Option<PathBuf> = None;
        let mut latest_time: Option<std::time::SystemTime> = None;

        let task_prefix = format!("task_{}_", sanitize_path_component(task_id));

        for entry in entries {
            let entry = entry.map_err(LogManagerError::Io)?;
            let path = entry.path();

            if path.is_file() {
                if let Some(filename) = path.file_name().and_then(|n| n.to_str()) {
                    if filename.starts_with(&task_prefix) && filename.ends_with(&format!(".{}", self.config.log_file_extension)) {
                        let metadata = entry.metadata().map_err(LogManagerError::Io)?;
                        let modified_time = metadata.modified().map_err(LogManagerError::Io)?;

                        if latest_time.is_none() || Some(modified_time) > latest_time {
                            latest_file = Some(path);
                            latest_time = Some(modified_time);
                        }
                    }
                }
            }
        }

        latest_file.ok_or_else(|| LogManagerError::LogFileNotFound(format!(
            "No log files found for task_id: {}", task_id
        )))
    }

    /// List all log files for a specific block
    pub fn list_log_files(&self, block_id: &str) -> Result<Vec<LogFileMetadata>, LogManagerError> {
        let block_dir = self.config.base_log_dir.join(sanitize_path_component(block_id));

        if !block_dir.exists() {
            return Ok(Vec::new());
        }

        let mut log_files = Vec::new();

        // Iterate through task directories
        let task_dirs = fs::read_dir(&block_dir).map_err(LogManagerError::Io)?;

        for task_dir_entry in task_dirs {
            let task_dir_entry = task_dir_entry.map_err(LogManagerError::Io)?;
            let task_dir_path = task_dir_entry.path();

            if task_dir_path.is_dir() {
                if let Some(task_id) = task_dir_path.file_name().and_then(|n| n.to_str()) {
                    // List log files in this task directory
                    let task_log_files = self.list_task_log_files(block_id, task_id)?;
                    log_files.extend(task_log_files);
                }
            }
        }

        // Sort by creation time (newest first)
        log_files.sort_by(|a, b| b.created_at.cmp(&a.created_at));

        Ok(log_files)
    }

    /// List log files for a specific task
    pub fn list_task_log_files(&self, block_id: &str, task_id: &str) -> Result<Vec<LogFileMetadata>, LogManagerError> {
        let task_dir = self.config.base_log_dir
            .join(sanitize_path_component(block_id))
            .join(sanitize_path_component(task_id));

        if !task_dir.exists() {
            return Ok(Vec::new());
        }

        let mut log_files = Vec::new();
        let entries = fs::read_dir(&task_dir).map_err(LogManagerError::Io)?;

        for entry in entries {
            let entry = entry.map_err(LogManagerError::Io)?;
            let path = entry.path();

            if path.is_file() {
                if let Some(filename) = path.file_name().and_then(|n| n.to_str()) {
                    if filename.ends_with(&format!(".{}", self.config.log_file_extension)) {
                        let metadata = entry.metadata().map_err(LogManagerError::Io)?;

                        let created_at = metadata.created()
                            .map_err(LogManagerError::Io)?
                            .into();

                        let last_modified = metadata.modified()
                            .map_err(LogManagerError::Io)?
                            .into();

                        let log_file_meta = LogFileMetadata {
                            block_id: block_id.to_string(),
                            task_id: task_id.to_string(),
                            file_path: path.clone(),
                            created_at,
                            last_modified,
                            file_size: metadata.len(),
                            is_compressed: filename.ends_with(".gz"),
                        };

                        log_files.push(log_file_meta);
                    }
                }
            }
        }

        // Sort by creation time (newest first)
        log_files.sort_by(|a, b| b.created_at.cmp(&a.created_at));

        Ok(log_files)
    }

    /// Clean up old log files based on retention policy
    pub fn cleanup_old_logs(&self) -> Result<usize, LogManagerError> {
        let cutoff_date = Utc::now() - Duration::days(self.config.retention_days as i64);
        let mut cleaned_count = 0;

        if !self.config.base_log_dir.exists() {
            return Ok(0);
        }

        cleaned_count += self.cleanup_directory(&self.config.base_log_dir, cutoff_date)?;

        info!("Cleaned up {} old log files", cleaned_count);
        Ok(cleaned_count)
    }

    /// Recursively clean up files in a directory
    fn cleanup_directory(&self, dir: &Path, cutoff_date: DateTime<Utc>) -> Result<usize, LogManagerError> {
        let mut cleaned_count = 0;
        let entries = fs::read_dir(dir).map_err(LogManagerError::Io)?;

        for entry in entries {
            let entry = entry.map_err(LogManagerError::Io)?;
            let path = entry.path();

            if path.is_dir() {
                // Recursively clean subdirectories
                cleaned_count += self.cleanup_directory(&path, cutoff_date)?;

                // Remove empty directories
                if let Ok(mut dir_entries) = fs::read_dir(&path) {
                    if dir_entries.next().is_none() {
                        if let Err(e) = fs::remove_dir(&path) {
                            warn!("Failed to remove empty directory {:?}: {}", path, e);
                        } else {
                            debug!("Removed empty directory: {:?}", path);
                        }
                    }
                }
            } else if path.is_file() {
                // Check if file should be cleaned up
                if let Ok(metadata) = entry.metadata() {
                    if let Ok(created_time) = metadata.created() {
                        let created_datetime: DateTime<Utc> = created_time.into();

                        if created_datetime < cutoff_date {
                            if let Err(e) = fs::remove_file(&path) {
                                warn!("Failed to remove old log file {:?}: {}", path, e);
                            } else {
                                debug!("Removed old log file: {:?}", path);
                                cleaned_count += 1;
                            }
                        }
                    }
                }
            }
        }

        Ok(cleaned_count)
    }

    /// Get total disk usage of all log files
    pub fn get_total_log_size(&self) -> Result<u64, LogManagerError> {
        if !self.config.base_log_dir.exists() {
            return Ok(0);
        }

        let total_size = self.calculate_directory_size(&self.config.base_log_dir)?;
        Ok(total_size)
    }

    /// Calculate the total size of files in a directory recursively
    fn calculate_directory_size(&self, dir: &Path) -> Result<u64, LogManagerError> {
        let mut total_size = 0;
        let entries = fs::read_dir(dir).map_err(LogManagerError::Io)?;

        for entry in entries {
            let entry = entry.map_err(LogManagerError::Io)?;
            let path = entry.path();

            if path.is_dir() {
                total_size += self.calculate_directory_size(&path)?;
            } else if path.is_file() {
                if let Ok(metadata) = entry.metadata() {
                    total_size += metadata.len();
                }
            }
        }

        Ok(total_size)
    }

    /// Check if log storage is approaching limits
    pub fn check_storage_limits(&self) -> Result<StorageStatus, LogManagerError> {
        let total_size = self.get_total_log_size()?;
        let usage_percentage = (total_size as f64 / self.config.max_total_size as f64) * 100.0;

        let status = if usage_percentage >= 95.0 {
            StorageStatus::Critical { usage_percentage, total_size }
        } else if usage_percentage >= 80.0 {
            StorageStatus::Warning { usage_percentage, total_size }
        } else {
            StorageStatus::Ok { usage_percentage, total_size }
        };

        Ok(status)
    }

    /// Get configuration
    pub fn get_config(&self) -> &LogManagerConfig {
        &self.config
    }
}

/// Storage status for log files
#[derive(Debug, Clone)]
pub enum StorageStatus {
    Ok { usage_percentage: f64, total_size: u64 },
    Warning { usage_percentage: f64, total_size: u64 },
    Critical { usage_percentage: f64, total_size: u64 },
}

/// Sanitize a path component to prevent path traversal attacks
fn sanitize_path_component(input: &str) -> String {
    input
        .chars()
        .filter(|c| c.is_ascii_alphanumeric() || *c == '_' || *c == '-')
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn create_test_log_manager() -> (LogManager, TempDir) {
        let temp_dir = TempDir::new().unwrap();
        let mut config = LogManagerConfig::default();
        config.base_log_dir = temp_dir.path().to_path_buf();
        let manager = LogManager::new(config);
        (manager, temp_dir)
    }

    #[test]
    fn test_create_log_directory() {
        let (manager, _temp_dir) = create_test_log_manager();

        let result = manager.create_log_directory("test_block", "test_task");
        assert!(result.is_ok());

        let log_dir = result.unwrap();
        assert!(log_dir.exists());
        assert!(log_dir.ends_with("test_block/test_task"));
    }

    #[test]
    fn test_sanitize_path_component() {
        assert_eq!(sanitize_path_component("test_block-123"), "test_block-123");
        assert_eq!(sanitize_path_component("../../../etc/passwd"), "etcpasswd");
        assert_eq!(sanitize_path_component("test block!@#"), "testblock");
    }

    #[test]
    fn test_generate_log_file_path() {
        let (manager, _temp_dir) = create_test_log_manager();

        let result = manager.generate_log_file_path("test_block", "test_task");
        assert!(result.is_ok());

        let file_path = result.unwrap();
        assert!(file_path.file_name().unwrap().to_str().unwrap().starts_with("task_test_task_"));
        assert!(file_path.extension().unwrap() == "log");
    }

    #[test]
    fn test_write_and_read_log_file() {
        let (manager, _temp_dir) = create_test_log_manager();

        let file_path = manager.generate_log_file_path("test_block", "test_task").unwrap();
        let test_data = b"Hello, log file!";

        // Write data
        let write_result = manager.write_stream_log(&file_path, test_data);
        assert!(write_result.is_ok());

        // Read data back
        let read_result = manager.read_log_file("test_block", "test_task");
        assert!(read_result.is_ok());
        assert_eq!(read_result.unwrap(), test_data);
    }

    #[test]
    fn test_list_log_files() {
        let (manager, _temp_dir) = create_test_log_manager();

        // Create a few log files
        let file_path1 = manager.generate_log_file_path("test_block", "task1").unwrap();
        let file_path2 = manager.generate_log_file_path("test_block", "task2").unwrap();

        manager.write_stream_log(&file_path1, b"data1").unwrap();
        manager.write_stream_log(&file_path2, b"data2").unwrap();

        let log_files = manager.list_log_files("test_block").unwrap();
        assert_eq!(log_files.len(), 2);
    }

    #[test]
    fn test_storage_status() {
        let (manager, _temp_dir) = create_test_log_manager();

        let status = manager.check_storage_limits().unwrap();
        match status {
            StorageStatus::Ok { usage_percentage, total_size } => {
                assert!(usage_percentage < 80.0);
                assert_eq!(total_size, 0); // No files created yet
            }
            _ => panic!("Expected Ok status for empty log directory"),
        }
    }
}