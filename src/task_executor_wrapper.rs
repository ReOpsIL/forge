use crate::block_config::BlockConfigManager;
use crate::project_config::ProjectConfigManager;
use crate::task_executor::{get_task_executor, init_task_executor, TaskExecutor};
use std::sync::Arc;

// Initialize the task executor
pub fn initialize(product_manager: Arc<ProjectConfigManager>, block_manager: Arc<BlockConfigManager>) -> Arc<TaskExecutor> {
    init_task_executor(product_manager, block_manager)
}

// Enqueue a task for execution
pub fn enqueue_task(
    block_id: &str,
    task_id: &str,
    task_description: &str,
    resolve_dependencies: bool,
    force_completed: bool
) -> Result<String, String> {
    let executor = get_task_executor()?;
    executor.enqueue_task(block_id, task_id, task_description, resolve_dependencies, force_completed)
}