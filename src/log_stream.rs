use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use actix_web::{web, HttpResponse, Responder};
use actix_web::http::header::{ContentType, CACHE_CONTROL};
use std::time::{Duration, Instant};
use tokio::sync::mpsc;
use tokio::time::interval;
use tokio_stream::wrappers::ReceiverStream;
use actix_web::web::Bytes;
use futures::stream::StreamExt;

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

// Handler for streaming logs for a specific task
pub async fn stream_logs(task_id: web::Path<String>) -> impl Responder {
    let task_id = task_id.into_inner();
    let log_storage = get_log_storage();

    // Create a channel for sending log updates
    let (tx, rx) = mpsc::channel(100);
    let rx_stream = ReceiverStream::new(rx);

    // Spawn a task to send log updates
    tokio::spawn(async move {
        let mut last_log_count = 0;
        let mut interval = interval(Duration::from_millis(500));

        loop {
            interval.tick().await;

            // Get the current logs
            let logs = log_storage.get_logs(&task_id);

            // If there are new logs, send them
            if logs.len() > last_log_count {
                for i in last_log_count..logs.len() {
                    if tx.send(format!("data: {}\n\n", logs[i].content)).await.is_err() {
                        // Client disconnected
                        return;
                    }
                }
                last_log_count = logs.len();
            }

            // Send a keep-alive message every 15 seconds
            if last_log_count % 30 == 0 {
                if tx.send(format!("data: keep-alive\n\n")).await.is_err() {
                    // Client disconnected
                    return;
                }
            }
        }
    });

    // Return a streaming response
    HttpResponse::Ok()
        .insert_header(ContentType::plaintext())
        .insert_header((CACHE_CONTROL, "no-cache"))
        .insert_header(("Connection", "keep-alive"))
        .insert_header(("Content-Type", "text/event-stream"))
        .streaming(rx_stream.map(|item| Ok::<Bytes, actix_web::Error>(Bytes::from(item))))
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
