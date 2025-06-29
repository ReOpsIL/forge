use std::collections::HashMap;
use std::sync::{Arc, Mutex};

// Structure to represent a task in the execution queue
#[derive(Debug, Clone)]
pub struct QueuedTask {
    pub block_id: String,
    pub task_id: String,
    pub task_description: String,
    pub status: String,
}

impl QueuedTask {
    pub fn new(block_id: String, task_id: String, task_description: String) -> Self {
        Self {
            block_id,
            task_id,
            task_description,
            status: "queued".to_string(),
        }
    }

    // Create a unique identifier for the task to check for duplicates
    pub fn get_unique_id(&self) -> String {
        format!("{}:{}", self.block_id, self.task_id)
    }
}

// Global task queue singleton
pub struct TaskQueue {
    tasks: Mutex<HashMap<String, QueuedTask>>,
}

// Singleton instance using lazy_static
lazy_static::lazy_static! {
    static ref TASK_QUEUE_INSTANCE: Mutex<Arc<TaskQueue>> = Mutex::new(Arc::new(TaskQueue {
        tasks: Mutex::new(HashMap::new()),
    }));
}

impl TaskQueue {
    // Get the singleton instance
    pub fn instance() -> Arc<TaskQueue> {
        let instance = TASK_QUEUE_INSTANCE.lock().unwrap().clone();
        instance
    }

    // Add a task to the queue
    pub fn enqueue_task(
        &self,
        block_id: &str,
        task_id: &str,
        task_description: &str,
    ) -> Result<String, String> {
        let task_unique_id = format!("{}:{}", block_id, task_id);

        // Check if the task is already in the queue
        let mut tasks = self.tasks.lock().unwrap();
        if tasks.contains_key(&task_unique_id) {
            return Ok(format!(
                "Task {}:{} is already in the queue",
                block_id, task_id
            ));
        }

        // Add the task to the queue
        let task = QueuedTask::new(
            block_id.to_string(),
            task_id.to_string(),
            task_description.to_string(),
        );

        tasks.insert(task_unique_id.clone(), task);

        Ok(format!("Added task {}:{} to the queue", block_id, task_id))
    }

    // Check if a task is in the queue
    pub fn is_task_in_queue(&self, block_id: &str, task_id: &str) -> bool {
        let task_unique_id = format!("{}:{}", block_id, task_id);
        let tasks = self.tasks.lock().unwrap();
        tasks.contains_key(&task_unique_id)
    }

    // Get the status of a task
    pub fn get_task_status(&self, block_id: &str, task_id: &str) -> Option<String> {
        let task_unique_id = format!("{}:{}", block_id, task_id);
        let tasks = self.tasks.lock().unwrap();
        tasks.get(&task_unique_id).map(|task| task.status.clone())
    }
}

// Helper functions for easier access
pub fn enqueue_task(
    block_id: &str,
    task_id: &str,
    task_description: &str,
) -> Result<String, String> {
    TaskQueue::instance().enqueue_task(block_id, task_id, task_description)
}

pub fn is_task_in_queue(block_id: &str, task_id: &str) -> bool {
    TaskQueue::instance().is_task_in_queue(block_id, task_id)
}

pub fn get_task_status(block_id: &str, task_id: &str) -> Option<String> {
    TaskQueue::instance().get_task_status(block_id, task_id)
}
