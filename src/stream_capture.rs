use crate::log_manager::{LogManager, LogManagerConfig};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fs::{File, OpenOptions};
use std::io::{BufWriter, Write};
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use thiserror::Error;
use tracing::{debug, error, info, warn};

#[derive(Error, Debug)]
pub enum CaptureError {
    #[error("Stream capture is already active for task {task_id}")]
    AlreadyActive { task_id: String },
    #[error("Stream capture is not active for task {task_id}")]
    NotActive { task_id: String },
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Buffer overflow: maximum size {max_size} bytes exceeded")]
    BufferOverflow { max_size: usize },
    #[error("Failed to acquire lock: {0}")]
    LockError(String),
    #[error("Log manager error: {0}")]
    LogManager(#[from] crate::log_manager::LogManagerError),
}

/// Configuration for stream capture
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CaptureConfig {
    /// Maximum buffer size in bytes before flushing to disk
    pub max_buffer_size: usize,
    /// Maximum total capture size in bytes
    pub max_capture_size: usize,
    /// Whether to automatically flush buffer periodically
    pub auto_flush: bool,
    /// Auto-flush interval in seconds
    pub flush_interval_seconds: u64,
}

impl Default for CaptureConfig {
    fn default() -> Self {
        Self {
            max_buffer_size: 1_048_576,    // 1MB buffer
            max_capture_size: 100_1048_576, // 100MB max capture
            auto_flush: true,
            flush_interval_seconds: 30,     // Flush every 30 seconds
        }
    }
}

/// Stream capture metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CaptureMetadata {
    pub task_id: String,
    pub block_id: String,
    pub started_at: DateTime<Utc>,
    pub ended_at: Option<DateTime<Utc>>,
    pub total_bytes: usize,
    pub log_file_path: PathBuf,
}

/// Stream capture instance for a specific task
pub struct StreamCapture {
    pub task_id: String,
    pub block_id: String,
    buffer: Arc<Mutex<Vec<u8>>>,
    active: Arc<AtomicBool>,
    file_writer: Arc<Mutex<Option<BufWriter<File>>>>,
    config: CaptureConfig,
    metadata: Arc<Mutex<CaptureMetadata>>,
    total_bytes_written: Arc<Mutex<usize>>,
}

impl std::fmt::Debug for StreamCapture {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("StreamCapture")
            .field("task_id", &self.task_id)
            .field("block_id", &self.block_id)
            .field("active", &self.active.load(Ordering::Acquire))
            .field("config", &self.config)
            .field("buffer_size", &self.get_buffer_size())
            .field("total_bytes_written", &self.get_total_bytes_written())
            .finish()
    }
}

impl StreamCapture {
    /// Create a new stream capture instance
    pub fn new(task_id: String, block_id: String, log_file_path: PathBuf, config: CaptureConfig) -> Result<Self, CaptureError> {
        // Create the log file
        let file = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(&log_file_path)?;

        let writer = BufWriter::new(file);

        let metadata = CaptureMetadata {
            task_id: task_id.clone(),
            block_id: block_id.clone(),
            started_at: Utc::now(),
            ended_at: None,
            total_bytes: 0,
            log_file_path: log_file_path.clone(),
        };

        Ok(Self {
            task_id,
            block_id,
            buffer: Arc::new(Mutex::new(Vec::new())),
            active: Arc::new(AtomicBool::new(false)),
            file_writer: Arc::new(Mutex::new(Some(writer))),
            config,
            metadata: Arc::new(Mutex::new(metadata)),
            total_bytes_written: Arc::new(Mutex::new(0)),
        })
    }

    /// Start stream capture
    pub fn start_capture(&self) -> Result<(), CaptureError> {
        if self.active.load(Ordering::Acquire) {
            return Err(CaptureError::AlreadyActive {
                task_id: self.task_id.clone(),
            });
        }

        self.active.store(true, Ordering::Release);
        info!("Started stream capture for task: {}", self.task_id);

        // Start auto-flush task if enabled
        if self.config.auto_flush {
            self.start_auto_flush_task();
        }

        Ok(())
    }

    /// Stop stream capture and return captured data
    pub fn stop_capture(&self) -> Result<Vec<u8>, CaptureError> {
        if !self.active.load(Ordering::Acquire) {
            return Err(CaptureError::NotActive {
                task_id: self.task_id.clone(),
            });
        }

        self.active.store(false, Ordering::Release);

        // Flush any remaining buffer data to file
        self.flush_buffer()?;

        // Close the file writer
        if let Ok(mut writer_opt) = self.file_writer.lock() {
            if let Some(writer) = writer_opt.take() {
                writer.into_inner().map_err(|e| CaptureError::Io(e.into_error()))?;
            }
        }

        // Update metadata
        if let Ok(mut metadata) = self.metadata.lock() {
            metadata.ended_at = Some(Utc::now());
            if let Ok(total_bytes) = self.total_bytes_written.lock() {
                metadata.total_bytes = *total_bytes;
            }
        }

        // Return the buffer contents
        let buffer = self.buffer.lock()
            .map_err(|e| CaptureError::LockError(e.to_string()))?;

        info!("Stopped stream capture for task: {} (captured {} bytes)", 
              self.task_id, buffer.len());

        Ok(buffer.clone())
    }

    /// Write data to the capture buffer
    pub fn write_to_buffer(&self, data: &[u8]) -> Result<(), CaptureError> {
        if !self.active.load(Ordering::Acquire) {
            return Ok(()); // Silently ignore writes when not active
        }

        let mut buffer = self.buffer.lock()
            .map_err(|e| CaptureError::LockError(e.to_string()))?;

        // Check total capture size limit
        let current_total = if let Ok(total) = self.total_bytes_written.lock() {
            *total
        } else {
            0
        };

        if current_total + buffer.len() + data.len() > self.config.max_capture_size {
            return Err(CaptureError::BufferOverflow {
                max_size: self.config.max_capture_size,
            });
        }

        buffer.extend_from_slice(data);

        debug!("Added {} bytes to capture buffer for task: {} (buffer size: {})", 
               data.len(), self.task_id, buffer.len());

        // Auto-flush if buffer is too large
        if buffer.len() >= self.config.max_buffer_size {
            drop(buffer); // Release the lock before flushing
            self.flush_buffer()?;
        }

        Ok(())
    }

    /// Flush buffer contents to file
    pub fn flush_buffer(&self) -> Result<(), CaptureError> {
        let mut buffer = self.buffer.lock()
            .map_err(|e| CaptureError::LockError(e.to_string()))?;

        if buffer.is_empty() {
            return Ok(());
        }

        let mut writer_opt = self.file_writer.lock()
            .map_err(|e| CaptureError::LockError(e.to_string()))?;

        if let Some(writer) = writer_opt.as_mut() {
            writer.write_all(&buffer)?;
            writer.flush()?;

            // Update total bytes written
            if let Ok(mut total) = self.total_bytes_written.lock() {
                *total += buffer.len();
            }

            debug!("Flushed {} bytes to log file for task: {}", buffer.len(), self.task_id);
            buffer.clear();
        } else {
            warn!("No file writer available for task: {}", self.task_id);
        }

        Ok(())
    }

    /// Get capture metadata
    pub fn get_metadata(&self) -> Result<CaptureMetadata, CaptureError> {
        let metadata = self.metadata.lock()
            .map_err(|e| CaptureError::LockError(e.to_string()))?;
        Ok(metadata.clone())
    }

    /// Check if capture is currently active
    pub fn is_active(&self) -> bool {
        self.active.load(Ordering::Acquire)
    }

    /// Get current buffer size
    pub fn get_buffer_size(&self) -> usize {
        if let Ok(buffer) = self.buffer.lock() {
            buffer.len()
        } else {
            0
        }
    }

    /// Get total bytes written to file
    pub fn get_total_bytes_written(&self) -> usize {
        if let Ok(total) = self.total_bytes_written.lock() {
            *total
        } else {
            0
        }
    }

    /// Start auto-flush background task
    fn start_auto_flush_task(&self) {
        let task_id = self.task_id.clone();
        let active = Arc::clone(&self.active);
        let buffer = Arc::clone(&self.buffer);
        let file_writer = Arc::clone(&self.file_writer);
        let total_bytes_written = Arc::clone(&self.total_bytes_written);
        let flush_interval = self.config.flush_interval_seconds;

        tokio::spawn(async move {
            let mut interval = tokio::time::interval(
                tokio::time::Duration::from_secs(flush_interval)
            );

            while active.load(Ordering::Acquire) {
                interval.tick().await;

                // Flush buffer if it has content
                if let Ok(buffer_guard) = buffer.lock() {
                    if !buffer_guard.is_empty() {
                        drop(buffer_guard); // Release lock before flushing

                        // Perform flush (similar to flush_buffer but without self reference)
                        if let (Ok(mut buffer_guard), Ok(mut writer_opt)) =
                            (buffer.lock(), file_writer.lock()) {
                            if let Some(writer) = writer_opt.as_mut() {
                                if let Err(e) = writer.write_all(&buffer_guard) {
                                    error!("Auto-flush write error for task {}: {}", task_id, e);
                                    continue;
                                }
                                if let Err(e) = writer.flush() {
                                    error!("Auto-flush flush error for task {}: {}", task_id, e);
                                    continue;
                                }

                                // Update total bytes written
                                if let Ok(mut total) = total_bytes_written.lock() {
                                    *total += buffer_guard.len();
                                }

                                debug!("Auto-flushed {} bytes for task: {}", buffer_guard.len(), task_id);
                                buffer_guard.clear();
                            }
                        }
                    }
                }
            }

            debug!("Auto-flush task ended for task: {}", task_id);
        });
    }
}

/// Stream capture manager for handling multiple captures
#[derive(Debug)]
pub struct StreamCaptureManager {
    captures: Arc<Mutex<std::collections::HashMap<String, Arc<StreamCapture>>>>,
    config: CaptureConfig,
    log_manager: Arc<LogManager>,
}

impl StreamCaptureManager {
    /// Create a new stream capture manager
    pub fn new(config: CaptureConfig) -> Self {
        let log_manager_config = LogManagerConfig::default();
        let log_manager = Arc::new(LogManager::new(log_manager_config));

        Self {
            captures: Arc::new(Mutex::new(std::collections::HashMap::new())),
            config,
            log_manager,
        }
    }

    /// Create a new stream capture manager with custom log manager
    pub fn with_log_manager(config: CaptureConfig, log_manager: Arc<LogManager>) -> Self {
        Self {
            captures: Arc::new(Mutex::new(std::collections::HashMap::new())),
            config,
            log_manager,
        }
    }

    /// Start stream capture for a task
    pub fn start_capture(&self, task_id: &str, block_id: &str) -> Result<(), CaptureError> {
        let mut captures = self.captures.lock()
            .map_err(|e| CaptureError::LockError(e.to_string()))?;

        if captures.contains_key(task_id) {
            return Err(CaptureError::AlreadyActive {
                task_id: task_id.to_string(),
            });
        }

        // Generate log file path using log manager
        let log_file_path = self.log_manager.generate_log_file_path(block_id, task_id)?;

        let capture = Arc::new(StreamCapture::new(
            task_id.to_string(),
            block_id.to_string(),
            log_file_path,
            self.config.clone(),
        )?);

        capture.start_capture()?;
        captures.insert(task_id.to_string(), capture);

        Ok(())
    }

    /// Stop stream capture for a task
    pub fn stop_capture(&self, task_id: &str) -> Result<Vec<u8>, CaptureError> {
        let mut captures = self.captures.lock()
            .map_err(|e| CaptureError::LockError(e.to_string()))?;

        if let Some(capture) = captures.remove(task_id) {
            capture.stop_capture()
        } else {
            Err(CaptureError::NotActive {
                task_id: task_id.to_string(),
            })
        }
    }

    /// Write data to a specific task's capture
    pub fn write_to_capture(&self, task_id: &str, data: &[u8]) -> Result<(), CaptureError> {
        let captures = self.captures.lock()
            .map_err(|e| CaptureError::LockError(e.to_string()))?;

        if let Some(capture) = captures.get(task_id) {
            capture.write_to_buffer(data)
        } else {
            // Silently ignore writes for non-existent captures
            Ok(())
        }
    }

    /// Get capture metadata for a task
    pub fn get_capture_metadata(&self, task_id: &str) -> Result<CaptureMetadata, CaptureError> {
        let captures = self.captures.lock()
            .map_err(|e| CaptureError::LockError(e.to_string()))?;

        if let Some(capture) = captures.get(task_id) {
            capture.get_metadata()
        } else {
            Err(CaptureError::NotActive {
                task_id: task_id.to_string(),
            })
        }
    }

    /// Check if capture is active for a task
    pub fn is_capture_active(&self, task_id: &str) -> bool {
        if let Ok(captures) = self.captures.lock() {
            captures.get(task_id).map_or(false, |c| c.is_active())
        } else {
            false
        }
    }

    /// Get list of active capture task IDs
    pub fn get_active_captures(&self) -> Vec<String> {
        if let Ok(captures) = self.captures.lock() {
            captures.keys().cloned().collect()
        } else {
            Vec::new()
        }
    }

    /// Cleanup inactive captures
    pub fn cleanup_inactive_captures(&self) {
        if let Ok(mut captures) = self.captures.lock() {
            captures.retain(|_, capture| capture.is_active());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn create_test_log_path() -> (TempDir, PathBuf) {
        let temp_dir = TempDir::new().unwrap();
        let log_path = temp_dir.path().join("test_capture.log");
        (temp_dir, log_path)
    }

    #[test]
    fn test_stream_capture_lifecycle() {
        let (_temp_dir, log_path) = create_test_log_path();
        let config = CaptureConfig::default();

        let capture = StreamCapture::new(
            "test_task".to_string(),
            "test_block".to_string(),
            log_path,
            config,
        ).unwrap();

        // Initially not active
        assert!(!capture.is_active());

        // Start capture
        capture.start_capture().unwrap();
        assert!(capture.is_active());

        // Write some data
        let test_data = b"Hello, stream capture!";
        capture.write_to_buffer(test_data).unwrap();
        assert_eq!(capture.get_buffer_size(), test_data.len());

        // Stop capture
        let captured_data = capture.stop_capture().unwrap();
        assert!(!capture.is_active());
        assert_eq!(captured_data, test_data);
    }

    #[test]
    fn test_stream_capture_buffer_overflow() {
        let (_temp_dir, log_path) = create_test_log_path();
        let mut config = CaptureConfig::default();
        config.max_capture_size = 10; // Very small limit

        let capture = StreamCapture::new(
            "test_task".to_string(),
            "test_block".to_string(),
            log_path,
            config,
        ).unwrap();

        capture.start_capture().unwrap();

        // Try to write more data than the limit
        let large_data = vec![0u8; 20];
        let result = capture.write_to_buffer(&large_data);

        assert!(matches!(result, Err(CaptureError::BufferOverflow { .. })));
    }

    #[test]
    fn test_stream_capture_manager() {
        let config = CaptureConfig::default();
        let manager = StreamCaptureManager::new(config);

        let task_id = "test_task";
        let block_id = "test_block";

        // Start capture
        manager.start_capture(task_id, block_id).unwrap();
        assert!(manager.is_capture_active(task_id));

        // Write data
        let test_data = b"Manager test data";
        manager.write_to_capture(task_id, test_data).unwrap();

        // Stop capture
        let captured_data = manager.stop_capture(task_id).unwrap();
        assert!(!manager.is_capture_active(task_id));
        assert_eq!(captured_data, test_data);
    }

    #[test]
    fn test_capture_metadata() {
        let (_temp_dir, log_path) = create_test_log_path();
        let config = CaptureConfig::default();

        let capture = StreamCapture::new(
            "test_task".to_string(),
            "test_block".to_string(),
            log_path.clone(),
            config,
        ).unwrap();

        let metadata = capture.get_metadata().unwrap();
        assert_eq!(metadata.task_id, "test_task");
        assert_eq!(metadata.block_id, "test_block");
        assert_eq!(metadata.log_file_path, log_path);
        assert!(metadata.ended_at.is_none());
    }
}