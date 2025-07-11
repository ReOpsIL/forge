use crate::block_config::BlockConfigManager;
use crate::log_stream;
use crate::log_stream::get_logs_str;
use crate::models::Task;
use crate::project_config::ProjectConfigManager;
use crate::task_queue::QueuedTask;
use std::collections::{HashMap, HashSet, VecDeque};
use std::io::{BufRead, BufReader, Write};
use std::path::Path;
use std::process::{Command, Stdio};
use std::sync::{Arc, Mutex, RwLock};
use std::thread;
use std::time::Duration;

// Singleton task executor that manages a global execution queue
pub struct TaskExecutor {
    queue: Mutex<VecDeque<QueuedTask>>,
    in_progress: RwLock<HashSet<String>>, // Set of task IDs currently in the queue or being processed
    project_manager: Arc<ProjectConfigManager>,
    block_manager: Arc<BlockConfigManager>,
}

impl TaskExecutor {
    // Create a new TaskExecutor instance
    pub fn new(project_manager: Arc< ProjectConfigManager>, block_manager: Arc<BlockConfigManager>) -> Arc<Self> {
        let executor = Arc::new(Self {
            queue: Mutex::new(VecDeque::new()),
            in_progress: RwLock::new(HashSet::new()),
            project_manager,
            block_manager,
        });

        // Start the background thread for processing the queue
        TaskExecutor::start_background_thread(executor.clone());

        executor
    }

    // Start a background thread to process the task queue
    fn start_background_thread(executor: Arc<TaskExecutor>) {
        thread::spawn(move || {
            loop {
                // Process any tasks in the queue
                if let Some(task) = executor.get_next_task() {
                    println!("Processing task: {}:{}", task.block_id, task.task_id);

                    // Execute the task
                    executor.execute_task(task.clone());

                    // Remove the task from the in_progress set
                    let task_id = task.get_unique_id();
                    if let Ok(mut in_progress) = executor.in_progress.write() {
                        in_progress.remove(&task_id);
                    }
                }

                // Sleep for a short time before checking the queue again
                thread::sleep(Duration::from_millis(100));
            }
        });
    }

    pub fn execute_git_task(&self, block_id: &String, task_id: &String) -> Result<(String, String), String> {
        // Create a unique task ID for logging
        let log_task_id = format!("{}:{}", block_id, task_id);
        // Clear any existing logs for this task
        log_stream::clear_logs(&log_task_id);

        // Get the project home directory from the project config
        let project_config = match self.project_manager.get_config() {
            Ok(config) => config,
            Err(_) => return Err("Failed to get project configuration".to_string()),
        };

        let main_branch = &project_config.main_branch.unwrap_or("main".to_string());

        let project_dir = project_config.project_home_directory.clone();
        if project_dir.is_empty() {
            let task_id = task_id.clone();
            let error_msg = format!("Project home directory is not set. Please configure it in the project settings, task: {} project dir: {}", task_id, project_dir);
            log_stream::add_log(&log_task_id, error_msg.clone());
            return Err(error_msg)
        }

        // Check if the project directory exists
        if !Path::new(&project_dir).exists() {
            let task_id = task_id.clone();
            let error_msg = format!("Project home directory does not exist, task: {} project dir: {}", task_id, project_dir);
            log_stream::add_log(&log_task_id, error_msg.clone());
            return Err(error_msg)
        }

        let mut blocks = self.block_manager.get_blocks()
            .map_err(|e| format!("Failed to get blocks: {}", e))?;

        let block = blocks.iter_mut()
            .find(|b| b.block_id == *block_id)
            .ok_or("Block not found")?;

        let task_opt = block.todo_list.get(task_id).unwrap();

        if task_opt.description.is_empty() {
            let task_id = task_id.clone();
            let error_msg = format!("Task description cannot be empty, task: {}", task_id);
            log_stream::add_log(&log_task_id, error_msg.clone());
            return Err(error_msg)
        }

        // Get the task prompt
        let task_prompt = task_opt.to_prompt();

        // Step 1: Pull latest main branch
        println!("Step 1: Pulling latest main branch");
        let msg = format!("Step 1: Pulling latest main branch {}",  task_id);
        log_stream::add_log(&task_id, msg.clone());

        let pull_output = Command::new("git")
            .arg("checkout")
            .arg(main_branch)
            .current_dir(&project_dir)
            .output();

        if let Err(e) = pull_output {
            let task_id = task_id.clone();
            let error_msg = format!("Failed to checkout {} branch, task: {} error: {}", main_branch, task_id, e);
            log_stream::add_log(&log_task_id, error_msg.clone());
            return Err(error_msg)
        }

        let pull_output = Command::new("git")
            .arg("pull")
            .current_dir(&project_dir)
            .output();

        if let Err(e) = pull_output {
            let task_id = task_id.clone();
            let error_msg = format!("Failed to pull latest changes from git. task: {}, error: {}", task_id, e);
            log_stream::add_log(&log_task_id, error_msg.clone());
            return Err(error_msg);
        }

        // Step 2: Create a task-specific branch
        println!("Step 2: Creating task-specific branch using task ID: {}", task_id);
        let msg = format!("Step 2: Creating task-specific branch using task ID {}",  task_id);
        log_stream::add_log(&task_id, msg.clone());

        let branch_output = Command::new("git")
            .arg("checkout")
            .arg("-b")
            .arg(&task_id)
            .current_dir(&project_dir)
            .output();

        if let Err(e) = branch_output {
            let task_id = task_id.clone();
            return Err(format!("Failed to create task branch: {}", e));
        }

        // Step 3: Execute the task using Claude CLI
        println!("Step 3: Executing task");
        let msg = format!("Step 3: Executing task {}",  task_id);
        log_stream::add_log(&task_id, msg.clone());

        // Log the start of the task
        log_stream::add_log(&log_task_id, "Starting Claude execution...".to_string());

        let result = Command::new("claude")
            .arg("--dangerously-skip-permissions")
            .current_dir(&project_dir)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn();

        let mut child = match result {
            Ok(child) => child,
            Err(e) => {
                let error_msg = format!("Failed to execute Claude CLI: {}", e);
                log_stream::add_log(&log_task_id, error_msg.clone());
                return Err(error_msg);
            }
        };


        // Write the task description to the command's stdin
        if let Some(mut stdin) = child.stdin.take() {
            if let Err(e) = stdin.write_all(task_prompt.as_bytes()) {
                let error_msg = format!("Failed to write to Claude CLI stdin: {}", e);
                log_stream::add_log(&log_task_id, error_msg.clone());
                return Err(error_msg);
            }

        }

        // Stream stdout in real-time
        if let Some(stdout) = child.stdout.take() {
            let reader = BufReader::new(stdout);
            let log_task_id_clone = log_task_id.clone();

            // Spawn a thread to read stdout line by line
            thread::spawn(move || {
                for line in reader.lines() {
                    if let Ok(line) = line {
                        // Add the line to the log storage
                        log_stream::add_log(&log_task_id_clone, line.clone());
                        println!("Claude: {}", line);
                    }
                }
            });
        }

        // Stream stderr in real-time
        if let Some(stderr) = child.stderr.take() {
            let reader = BufReader::new(stderr);
            let log_task_id_clone = log_task_id.clone();

            // Spawn a thread to read stderr line by line
            thread::spawn(move || {
                for line in reader.lines() {
                    if let Ok(line) = line {
                        // Add the line to the log storage with an error prefix
                        log_stream::add_log(&log_task_id_clone, format!("ERROR: {}", line));
                        println!("Claude ERROR: {}", line);
                    }
                }
            });
        }

        // Wait for the command to complete
        let status = match child.wait() {
            Ok(status) => status,
            Err(e) => {
                let error_msg = format!("Failed to wait for Claude CLI command: {}", e);
                log_stream::add_log(&log_task_id, error_msg.clone());
                return Err(error_msg);
            }
        };

        let task_success = status.success();


        if !task_success {
            let error_msg = format!("Claude CLI command failed with exit code: {:?}", status.code());
            log_stream::add_log(&log_task_id, error_msg.clone());
            return Err(get_logs_str(task_id));
        }

        log_stream::add_log(&log_task_id, "Claude CLI command completed successfully".to_string());
        println!("Claude CLI command completed successfully");

        // Step 4: Commit changes
        println!("Step 4: Committing changes");
        let msg = format!("Step 4: Committing changes {}",  task_id);
        log_stream::add_log(&log_task_id, msg.clone());

        let add_output = Command::new("git")
            .arg("add")
            .arg(".")
            .current_dir(&project_dir)
            .output();

        if let Err(e) = add_output {
            let error_msg = format!("Failed to stage changes into git (add): {}", e);
            log_stream::add_log(&log_task_id, error_msg.clone());
            return Err(get_logs_str(task_id));
        }

        // Use the task description as a commit message
        let commit_message = task_opt.description.lines().next().unwrap_or("Task execution").to_string();
        let commit_output = Command::new("git")
            .arg("commit")
            .arg("-m")
            .arg(&commit_message)
            .current_dir(&project_dir)
            .output();

        if let Err(e) = commit_output {
            let error_msg = format!("Failed to commit changes into git: {}", e);
            log_stream::add_log(&log_task_id, error_msg.clone());
            return Err(get_logs_str(task_id));
        }

        // Get the commit ID
        let commit_id_output = Command::new("git")
            .arg("rev-parse")
            .arg("HEAD")
            .current_dir(&project_dir)
            .output();

        let commit_id = match commit_id_output {
            Ok(output) => {
                if output.status.success() {
                    Some(String::from_utf8_lossy(&output.stdout).trim().to_string())
                } else {
                    let error_msg = format!("Failed to get commit id: {:?}, {}", status.code(),  String::from_utf8_lossy(&output.stderr));
                    log_stream::add_log(&log_task_id, error_msg.clone());
                    return Err(get_logs_str(task_id));
                }
            }
            Err(e) => {
                let error_msg = format!("Failed to get commit id: {:?}, {}", status.code(), e);
                log_stream::add_log(&log_task_id, error_msg.clone());
                return Err(get_logs_str(task_id));
            }
        };

        let commit_id = commit_id.unwrap_or("No commit id".to_string());
        self.update_task_commit_id(&block_id, &task_id, commit_id.as_str());

        let msg = format!("Commit id: {}, {}",  task_id, commit_id);
        log_stream::add_log(&log_task_id, msg.clone());

        // Step 5: Merge back to main
        let msg = format!("Step 5: Merging back to main {}",  task_id);
        log_stream::add_log(&log_task_id, msg.clone());

        let checkout_output = Command::new("git")
            .arg("checkout")
            .arg(main_branch)
            .current_dir(&project_dir)
            .output();

        if let Err(e) = checkout_output {
            let error_msg = format!("Failed to checkout {} branch: {}",  main_branch, e);
            log_stream::add_log(&log_task_id, error_msg.clone());
            return Err(get_logs_str(task_id));
        }

        let merge_output = Command::new("git")
            .arg("merge")
            .arg("--ff-only")
            .arg(&task_id)
            .current_dir(&project_dir)
            .output();

        if let Err(e) = merge_output {
            let error_msg = format!("Failed to merge task branch: {}",  e);
            log_stream::add_log(&log_task_id, error_msg.clone());
            return Err(get_logs_str(task_id));
        }

        // Step 6: Clean up (delete the task branch)
        let msg = format!("Step 6: Cleaning up {}",  task_id);
        log_stream::add_log(&log_task_id, msg.clone());

        let delete_output = Command::new("git")
            .arg("branch")
            .arg("-d")
            .arg(&task_id)
            .current_dir(&project_dir)
            .output();

        if let Err(e) = delete_output {
            let error_msg = format!("Failed to delete task branch: {}",  e);
            log_stream::add_log(&log_task_id, error_msg.clone());
        }

        let msg = format!("Task ended: {}",  task_id);
        log_stream::add_log(&log_task_id, msg.clone());

        Ok((get_logs_str(task_id), commit_id))
    }

    // Get the next task from the queue
    fn get_next_task(&self) -> Option<QueuedTask> {
        if let Ok(mut queue) = self.queue.lock() {
            queue.pop_front()
        } else {
            None
        }
    }

    // Execute a task
    fn execute_task(&self, task: QueuedTask) {
        // This is a placeholder for the actual task execution logic
        // In a real implementation, this would call the appropriate handler based on the task type
        self.update_task_status(&task.block_id, &task.task_id, "[IN-PROGRESS]");

        println!("Executing task: {}:{}", task.block_id, task.task_id);
        match self.execute_git_task(&task.block_id, &task.task_id) {
            Ok((log, commit_id)) => {
                // Update the task status in the block config
                self.update_task_status_with_log_and_commit_id(task.block_id, task.task_id, "[COMPLETED]".to_string(), log, commit_id );
            } ,
            Err(err_str) => {
                self.update_task_status_with_log_and_commit_id(task.block_id, task.task_id, "[FAILED]".to_string(), err_str, "No commit id".to_string() );
            },
        }
    }

    // Helper function to update task status, log, and commit ID
    fn update_task_status_with_log_and_commit_id(&self,
        block_id: String,
        task_id: String,
        status: String,
        log: String,
        commit_id: String,
    ) {
        // Usage (replaces the original code):
        if let Err(e) = self.update_task_and_save(&block_id, &task_id, &status, &log, commit_id) {
            println!("Failed to update task: {}", e);
        }
    }

    fn update_task_and_save(&self,
        block_id: &str,
        task_id: &str,
        status: &str,
        log: &str,
        commit_id: String,
    ) -> Result<(), String> {
        let mut blocks = self.block_manager.get_blocks()
            .map_err(|e| format!("Failed to get blocks: {}", e))?;

        let block = blocks.iter_mut()
            .find(|b| b.block_id == block_id)
            .ok_or("Block not found")?;

        // Get the task, clone it, and update the clone
        let task_opt = block.todo_list.get(task_id);
        if let Some(task_original) = task_opt {
            // Clone the task
            let mut task_updated = task_original.clone();
            // Update task fields
            task_updated.status = status.to_string();
            task_updated.description = format!("{} {}", task_updated.description, status);
            task_updated.log = log.to_string();
            task_updated.commit_id = commit_id;

            // Update the task in the block's todo_list
            block.todo_list.insert(task_id.to_string(), task_updated);
        } else {
            return Err("Task not found".to_string());
        }

        match self.block_manager.update_block(block.clone()) {
            Ok(_) => {
                // Save the updated blocks to the file
                self.block_manager.save_blocks_to_file().map_err(|e| format!("Failed to save blocks to file: {}", e))?;
            },
            Err(e) => {
                println!("Failed to update block: {}", e);
                return Err( format!(
                    "Failed to update block: {}",
                    e
                ));
            }
        }

        Ok(())
    }

    // Update the status of a task in the block config
    fn update_task_status(&self, block_id: &str, task_id: &str, status: &str) {
        if let Ok(mut blocks) = self.block_manager.get_blocks() {
            if let Some(block) = blocks.iter_mut().find(|b| b.block_id == block_id) {
                if let Some(task) = block.todo_list.get_mut(task_id) {
                    task.status = status.to_string();

                    // Update the block in the database
                    if let Err(e) = self.block_manager.update_block(block.clone()) {
                        println!("Failed to update block: {}", e);
                    } else {
                        // Save the updated blocks to the file
                        if let Err(e) = self.block_manager.save_blocks_to_file() {
                            println!("Failed to save blocks to file: {}", e);
                        }
                    }
                }
            }
        }
    }

    fn update_task_commit_id(&self, block_id: &str, task_id: &str, commit_id: &str) {
        if let Ok(mut blocks) = self.block_manager.get_blocks() {
            if let Some(block) = blocks.iter_mut().find(|b| b.block_id == block_id) {
                if let Some(task) = block.todo_list.get_mut(task_id) {
                    task.commit_id = commit_id.to_string();

                    // Update the block in the database
                    if let Err(e) = self.block_manager.update_block(block.clone()) {
                        println!("Failed to update block: {}", e);
                    } else {
                        // Save the updated blocks to the file
                        if let Err(e) = self.block_manager.save_blocks_to_file() {
                            println!("Failed to save blocks to file: {}", e);
                        }
                    }
                }
            }
        }
    }

    // Add a task to the queue, optionally resolving dependencies
    pub fn enqueue_task(&self, block_id: &str, task_id: &str, task_description: &str, resolve_dependencies: bool, force_completed: bool) -> Result<String, String> {
        let task_unique_id = format!("{}:{}", block_id, task_id);

        // Check if the task is already in the queue
        if let Ok(in_progress) = self.in_progress.read() {
            if in_progress.contains(&task_unique_id) {
                return Ok(format!("Task {}:{} is already in the queue", block_id, task_id));
            }
        }

        // If dependency resolution is enabled, resolve dependencies and add them to the queue
        if resolve_dependencies {
            let execution_order = self.resolve_task_dependencies(block_id, task_id, force_completed)?;

            // If the execution order is empty, all tasks are already completed
            if execution_order.is_empty() {
                return Ok(format!("Task {}:{} and all its dependencies are already completed", block_id, task_id));
            }

            // Add all tasks in the execution order to the queue
            for task_id_to_execute in execution_order {
                // Get the task description
                let task_description = match self.get_task_description(block_id, &task_id_to_execute) {
                    Ok(desc) => desc,
                    Err(e) => return Err(e),
                };

                // Create a new queued task
                let queued_task = QueuedTask::new(
                    block_id.to_string(),
                    task_id_to_execute.clone(),
                    task_description,
                );

                // Add the task to the queue and mark it as in progress
                if let Ok(mut queue) = self.queue.lock() {
                    queue.push_back(queued_task);

                    if let Ok(mut in_progress) = self.in_progress.write() {
                        in_progress.insert(format!("{}:{}", block_id, task_id_to_execute));
                    }
                }
            }

            Ok(format!("Added task {}:{} and its dependencies to the queue", block_id, task_id))
        } else {
            // Just add the requested task to the queue
            let queued_task = QueuedTask::new(
                block_id.to_string(),
                task_id.to_string(),
                task_description.to_string(),
            );

            // Add the task to the queue and mark it as in progress
            if let Ok(mut queue) = self.queue.lock() {
                queue.push_back(queued_task);

                if let Ok(mut in_progress) = self.in_progress.write() {
                    in_progress.insert(task_unique_id);
                }
            }

            Ok(format!("Added task {}:{} to the queue", block_id, task_id))
        }
    }

    // Get the description of a task
    fn get_task_description(&self, block_id: &str, task_id: &str) -> Result<String, String> {
        // Get all blocks
        let blocks = self.block_manager.get_blocks()
            .map_err(|e| format!("Failed to get blocks: {}", e))?;

        // Find the block
        let block = blocks.iter()
            .find(|b| b.block_id == block_id)
            .ok_or_else(|| format!("Block {} not found", block_id))?;

        // Find the task
        let task = block.todo_list.get(task_id)
            .ok_or_else(|| format!("Task {} not found in block {}", task_id, block_id))?;

        Ok(task.description.clone())
    }

    // Resolve task dependencies and create an execution queue
    fn resolve_task_dependencies(&self, block_id: &str, task_id: &str, force_completed: bool) -> Result<Vec<String>, String> {
        // Get all blocks
        let blocks = self.block_manager.get_blocks()
            .map_err(|e| format!("Failed to get blocks: {}", e))?;

        // Find the block
        let block = blocks.iter()
            .find(|b| b.block_id == block_id)
            .ok_or_else(|| format!("Block {} not found", block_id))?;

        // Create a map of task_id to task for easy lookup
        let tasks: HashMap<&String, &Task> = block.todo_list.iter().collect();

        // Check if the task exists
        if !tasks.contains_key(&task_id.to_string()) {
            return Err(format!("Task {} not found in block {}", task_id, block_id));
        }

        // Set to track visited tasks (for cycle detection)
        let mut visited = HashSet::new();
        // Set to track tasks in the current recursion stack (for cycle detection)
        let mut rec_stack = HashSet::new();
        // Vector to store the execution order (topological sort result)
        let mut execution_order = Vec::new();
        // Set to track tasks that have already been completed
        let mut completed_tasks = HashSet::new();

        // Helper function to check if a task is completed
        let is_task_completed = |task: &Task| -> bool {
            task.status.contains("[COMPLETED]")
        };

        if !force_completed {
            // Identify completed tasks
            for (id, task) in &block.todo_list {
                if is_task_completed(task) {
                    completed_tasks.insert(id.clone());
                }
            }
        }

        // DFS function for topological sort with cycle detection
        fn dfs(
            task_id: &str,
            tasks: &HashMap<&String, &Task>,
            visited: &mut HashSet<String>,
            rec_stack: &mut HashSet<String>,
            execution_order: &mut Vec<String>,
            completed_tasks: &HashSet<String>,
        ) -> Result<(), String> {
            // Mark the current task as visited and add to recursion stack
            visited.insert(task_id.to_string());
            rec_stack.insert(task_id.to_string());

            // Skip further processing if the task is already completed
            if completed_tasks.contains(&task_id.to_string()) {
                // Remove from recursion stack before returning
                rec_stack.remove(&task_id.to_string());
                return Ok(());
            }

            // Process all dependencies
            if let Some(task) = tasks.get(&task_id.to_string()) {
                for dep_id in &task.dependencies {
                    // Check if the dependency exists
                    if !tasks.contains_key(dep_id) {
                        return Err(format!("Dependency task {} not found", dep_id));
                    }

                    // If the dependency is in the recursion stack, we have a cycle
                    if rec_stack.contains(dep_id) {
                        return Err(format!("Cycle detected in task dependencies: {} -> {}", task_id, dep_id));
                    }

                    // If the dependency hasn't been visited yet, visit it
                    if !visited.contains(dep_id) {
                        dfs(dep_id, tasks, visited, rec_stack, execution_order, completed_tasks)?;
                    }
                }
            }

            // Remove the task from recursion stack and add to execution order
            rec_stack.remove(&task_id.to_string());

            // Only add to execution order if not completed
            if !completed_tasks.contains(&task_id.to_string()) {
                execution_order.push(task_id.to_string());
            }

            Ok(())
        }

        // Start DFS from the requested task
        dfs(task_id, &tasks, &mut visited, &mut rec_stack, &mut execution_order, &completed_tasks)?;

        // Reverse the execution order to get the correct topological sort
        execution_order.reverse();

        // Log the execution order
        println!("Task execution order: {:?}", execution_order);
        println!("Skipped completed tasks: {:?}", completed_tasks);

        Ok(execution_order)
    }
}

// Lazy static instance for the global task executor
lazy_static::lazy_static! {
    static ref TASK_EXECUTOR: Mutex<Option<Arc<TaskExecutor>>> = Mutex::new(None);
}

// Initialize the global task executor
pub fn init_task_executor(product_manager: Arc<ProjectConfigManager>, block_manager: Arc<BlockConfigManager>) -> Arc<TaskExecutor> {
    let mut executor = TASK_EXECUTOR.lock().unwrap();
    if executor.is_none() {
        *executor = Some(TaskExecutor::new(product_manager, block_manager));
    }
    executor.as_ref().unwrap().clone()
}

// Get the global task executor instance
pub fn get_task_executor() -> Result<Arc<TaskExecutor>, String> {
    let executor = TASK_EXECUTOR.lock().unwrap();
    if let Some(executor) = executor.as_ref() {
        Ok(executor.clone())
    } else {
        Err("Task executor has not been initialized".to_string())
    }
}
