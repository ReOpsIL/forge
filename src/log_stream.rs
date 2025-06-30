use actix_web::http::header::{CACHE_CONTROL, ContentType};
use actix_web::web::Bytes;
use actix_web::{HttpResponse, Responder, web};
use futures::stream::StreamExt;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Instant;
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;
use std::path::PathBuf;
use tokio::fs;
use regex::Regex;

// Structure to hold log entries for each task
#[derive(Debug, Clone)]
pub struct LogEntry {
    pub timestamp: Instant,
    pub content: String,
}

// Global log storage
pub struct LogStorage {
    logs: Mutex<HashMap<String, Vec<LogEntry>>>,
}

impl LogStorage {
    pub fn new() -> Self {
        Self {
            logs: Mutex::new(HashMap::new()),
        }
    }

    // Add a log entry for a specific task
    pub fn add_log(&self, task_id: &str, content: String) {
        let mut logs = self.logs.lock().unwrap();
        let task_logs = logs.entry(task_id.to_string()).or_insert_with(Vec::new);
        task_logs.push(LogEntry {
            timestamp: Instant::now(),
            content,
        });
    }

    // Get all logs for a specific task
    pub fn get_logs(&self, task_id: &str) -> Vec<LogEntry> {
        let logs = self.logs.lock().unwrap();
        if let Some(task_logs) = logs.get(task_id) {
            task_logs.clone()
        } else {
            Vec::new()
        }
    }

    // Clear logs for a specific task
    pub fn clear_logs(&self, task_id: &str) {
        let mut logs = self.logs.lock().unwrap();
        logs.remove(task_id);
    }

    // Get all task IDs with logs
    pub fn get_task_ids(&self) -> Vec<String> {
        let logs = self.logs.lock().unwrap();
        logs.keys().cloned().collect()
    }
}

// Initialize the global log storage
lazy_static::lazy_static! {
    static ref LOG_STORAGE: Arc<LogStorage> = Arc::new(LogStorage::new());
}

// Get the global log storage instance
pub fn get_log_storage() -> Arc<LogStorage> {
    LOG_STORAGE.clone()
}

// Function to read task logs from file system
async fn read_task_logs_from_file(block_id: &str, task_id: &str) -> Result<String, std::io::Error> {
    // Construct the log directory path: ./logs/{block_id}/{block_id}{task_id}/
    let log_dir = PathBuf::from("./logs")
        .join(block_id)
        .join(format!("{}{}", block_id, task_id));
    
    if !log_dir.exists() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "Log directory not found"
        ));
    }
    
    // Read all .log files in the directory
    let mut entries = fs::read_dir(&log_dir).await?;
    let mut log_files = Vec::new();
    
    while let Some(entry) = entries.next_entry().await? {
        let path = entry.path();
        if path.extension().and_then(|ext| ext.to_str()) == Some("log") {
            log_files.push(path);
        }
    }
    
    if log_files.is_empty() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "No log files found"
        ));
    }
    
    // Sort log files by name (timestamp in filename) to get them in chronological order
    log_files.sort();
    
    // Read the latest log file (last in sorted order)
    let latest_log_file = log_files.last().unwrap();
    let content = fs::read_to_string(latest_log_file).await?;
    
    Ok(content)
}

// Function to strip ANSI escape sequences from log content
fn strip_ansi_codes(content: &str) -> String {
    lazy_static::lazy_static! {
        static ref ANSI_REGEX: Regex = Regex::new(r"\x1b\[[0-9;]*[a-zA-Z]").unwrap();
    }
    ANSI_REGEX.replace_all(content, "").to_string()
}

// Handler for streaming logs for a specific task
pub async fn stream_logs(task_id: web::Path<String>) -> impl Responder {
    let task_id = task_id.into_inner();
    
    // Parse the task_id in format "block_id:task_id"
    let parts: Vec<&str> = task_id.split(':').collect();
    if parts.len() != 2 {
        return HttpResponse::BadRequest().json(serde_json::json!({
            "error": "Invalid task ID format. Expected 'block_id:task_id'"
        }));
    }
    
    let block_id = parts[0];
    let actual_task_id = parts[1];
    
    // Try to read from file system first
    let log_content = match read_task_logs_from_file(block_id, actual_task_id).await {
        Ok(content) => content,
        Err(_) => {
            // Fallback to in-memory storage
            let log_storage = get_log_storage();
            let logs = log_storage.get_logs(&task_id);
            logs.iter().map(|log| log.content.clone()).collect::<Vec<_>>().join("\n")
        }
    };

    if log_content.is_empty() {
        return HttpResponse::Ok()
            .insert_header(ContentType::plaintext())
            .body("No logs available for this task.");
    }

    // Return log content directly as HTTP response
    HttpResponse::Ok()
        .insert_header(ContentType::plaintext())
        .body(log_content)
}

// Handler for getting all task IDs with logs
pub async fn get_task_ids() -> impl Responder {
    let log_storage = get_log_storage();
    let task_ids = log_storage.get_task_ids();

    HttpResponse::Ok().json(task_ids)
}

// Public function to add a log entry
pub fn add_log(task_id: &str, content: String) {
    let log_storage = get_log_storage();
    log_storage.add_log(task_id, content);
}

// Public function to clear logs for a task
pub fn clear_logs(task_id: &str) {
    let log_storage = get_log_storage();
    log_storage.clear_logs(task_id);
}

pub fn get_logs_str(task_id: &str) -> String {
    let mut log_output = String::new();

    let logs = get_log_storage().get_logs(&task_id);
    for log in &logs {
        log_output.push_str(&log.content);
        log_output.push('\n');
    }

    log_output
}
