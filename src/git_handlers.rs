use actix_web::{web, HttpResponse, Responder};
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::Write;
use std::path::Path;
use std::process::Command;
use std::process::Stdio;
use std::sync::Arc;
use std::collections::{HashMap, HashSet, VecDeque};

use crate::block_config::BlockConfigManager;
use crate::project_config::ProjectConfigManager;
use crate::models::Task;

// AppState for git handlers
pub struct GitAppState {
    pub project_manager: Arc<ProjectConfigManager>,
    pub block_manager: Arc<BlockConfigManager>,
}

// Request body for executing a task with Git integration
#[derive(Debug, Deserialize)]
pub struct ExecuteGitTaskRequest {
    pub block_id: String,
    pub task_id: String,
    pub task_description: String,
    pub resolve_dependencies: Option<bool>,
}

// Request body for creating a branch
#[derive(Debug, Deserialize)]
pub struct CreateBranchRequest {
    pub branch_name: String,
}

// Request body for committing changes
#[derive(Debug, Deserialize)]
pub struct CommitRequest {
    pub commit_message: String,
}

// Request body for merging branches
#[derive(Debug, Deserialize)]
pub struct MergeBranchRequest {
    pub source_branch: String,
    pub target_branch: String,
}

// Response for Git operations
#[derive(Debug, Serialize)]
pub struct GitResponse {
    pub success: bool,
    pub message: String,
}

// Response for task execution
#[derive(Debug, Serialize)]
pub struct ExecuteGitTaskResponse {
    pub status: String,
    pub message: String,
}

// Response for Build operation
#[derive(Debug, Serialize)]
pub struct BuildResponse {
    pub success: bool,
    pub message: String,
    pub output: String,
}

// Request body for getting a task's diff
#[derive(Debug, Deserialize)]
pub struct GetTaskDiffRequest {
    pub block_id: String,
    pub task_id: String,
}

// Struct to hold original and modified content for a file
#[derive(Debug, Serialize)]
pub struct CommitFiles {
    pub file_path: String,
    pub original_content: Option<String>,
    pub modified_content: Option<String>,
}

// Response for getting a task's diff
#[derive(Debug, Serialize)]
pub struct GetTaskDiffResponse {
    pub success: bool,
    pub message: String,
    pub commit_id: Option<String>,
    pub files_diff: Vec<CommitFiles>,
}

// Handler to create a new Git branch
pub async fn create_branch_handler(
    data: web::Data<GitAppState>,
    request: web::Json<CreateBranchRequest>,
) -> impl Responder {
    // Get the project home directory
    let project_config = match data.project_manager.get_config() {
        Ok(config) => config,
        Err(e) => {
            return HttpResponse::InternalServerError().json(GitResponse {
                success: false,
                message: format!("Failed to get project configuration: {}", e),
            });
        }
    };

    let project_dir = project_config.project_home_directory.clone();
    if project_dir.is_empty() {
        return HttpResponse::BadRequest().json(GitResponse {
            success: false,
            message: "Project home directory is not set".to_string(),
        });
    }

    // Check if the project directory exists
    if !Path::new(&project_dir).exists() {
        return HttpResponse::BadRequest().json(GitResponse {
            success: false,
            message: format!("Project home directory does not exist: {}", project_dir),
        });
    }

    // Create a new branch
    let output = Command::new("git")
        .arg("checkout")
        .arg("-b")
        .arg(&request.branch_name)
        .current_dir(&project_dir)
        .output();

    match output {
        Ok(output) => {
            if output.status.success() {
                HttpResponse::Ok().json(GitResponse {
                    success: true,
                    message: format!("Branch '{}' created successfully", request.branch_name),
                })
            } else {
                let error_message = String::from_utf8_lossy(&output.stderr).to_string();
                HttpResponse::BadRequest().json(GitResponse {
                    success: false,
                    message: format!("Failed to create branch: {}", error_message),
                })
            }
        }
        Err(e) => {
            HttpResponse::InternalServerError().json(GitResponse {
                success: false,
                message: format!("Failed to execute git command: {}", e),
            })
        }
    }
}

// Handler to commit changes
pub async fn commit_handler(
    data: web::Data<GitAppState>,
    request: web::Json<CommitRequest>,
) -> impl Responder {
    // Get the project home directory
    let project_config = match data.project_manager.get_config() {
        Ok(config) => config,
        Err(e) => {
            return HttpResponse::InternalServerError().json(GitResponse {
                success: false,
                message: format!("Failed to get project configuration: {}", e),
            });
        }
    };

    let project_dir = project_config.project_home_directory.clone();
    if project_dir.is_empty() {
        return HttpResponse::BadRequest().json(GitResponse {
            success: false,
            message: "Project home directory is not set".to_string(),
        });
    }

    // Check if the project directory exists
    if !Path::new(&project_dir).exists() {
        return HttpResponse::BadRequest().json(GitResponse {
            success: false,
            message: format!("Project home directory does not exist: {}", project_dir),
        });
    }

    // Add all changes
    let add_output = Command::new("git")
        .arg("add")
        .arg(".")
        .current_dir(&project_dir)
        .output();

    match add_output {
        Ok(output) => {
            if !output.status.success() {
                let error_message = String::from_utf8_lossy(&output.stderr).to_string();
                return HttpResponse::BadRequest().json(GitResponse {
                    success: false,
                    message: format!("Failed to add changes: {}", error_message),
                });
            }
        }
        Err(e) => {
            return HttpResponse::InternalServerError().json(GitResponse {
                success: false,
                message: format!("Failed to execute git add command: {}", e),
            });
        }
    }

    // Commit changes
    let commit_output = Command::new("git")
        .arg("commit")
        .arg("-m")
        .arg(&request.commit_message)
        .current_dir(&project_dir)
        .output();

    match commit_output {
        Ok(output) => {
            if output.status.success() {
                HttpResponse::Ok().json(GitResponse {
                    success: true,
                    message: "Changes committed successfully".to_string(),
                })
            } else {
                let error_message = String::from_utf8_lossy(&output.stderr).to_string();
                HttpResponse::BadRequest().json(GitResponse {
                    success: false,
                    message: format!("Failed to commit changes: {}", error_message),
                })
            }
        }
        Err(e) => {
            HttpResponse::InternalServerError().json(GitResponse {
                success: false,
                message: format!("Failed to execute git commit command: {}", e),
            })
        }
    }
}

// Handler to merge branches
pub async fn merge_branch_handler(
    data: web::Data<GitAppState>,
    request: web::Json<MergeBranchRequest>,
) -> impl Responder {
    // Get the project home directory
    let project_config = match data.project_manager.get_config() {
        Ok(config) => config,
        Err(e) => {
            return HttpResponse::InternalServerError().json(GitResponse {
                success: false,
                message: format!("Failed to get project configuration: {}", e),
            });
        }
    };

    let project_dir = project_config.project_home_directory.clone();
    if project_dir.is_empty() {
        return HttpResponse::BadRequest().json(GitResponse {
            success: false,
            message: "Project home directory is not set".to_string(),
        });
    }

    // Check if the project directory exists
    if !Path::new(&project_dir).exists() {
        return HttpResponse::BadRequest().json(GitResponse {
            success: false,
            message: format!("Project home directory does not exist: {}", project_dir),
        });
    }

    // Checkout the target branch
    let checkout_output = Command::new("git")
        .arg("checkout")
        .arg(&request.target_branch)
        .current_dir(&project_dir)
        .output();

    match checkout_output {
        Ok(output) => {
            if !output.status.success() {
                let error_message = String::from_utf8_lossy(&output.stderr).to_string();
                return HttpResponse::BadRequest().json(GitResponse {
                    success: false,
                    message: format!("Failed to checkout target branch: {}", error_message),
                });
            }
        }
        Err(e) => {
            return HttpResponse::InternalServerError().json(GitResponse {
                success: false,
                message: format!("Failed to execute git checkout command: {}", e),
            });
        }
    }

    // Merge the source branch
    let merge_output = Command::new("git")
        .arg("merge")
        .arg(&request.source_branch)
        .current_dir(&project_dir)
        .output();

    match merge_output {
        Ok(output) => {
            if output.status.success() {
                HttpResponse::Ok().json(GitResponse {
                    success: true,
                    message: format!(
                        "Branch '{}' merged into '{}' successfully",
                        request.source_branch, request.target_branch
                    ),
                })
            } else {
                let error_message = String::from_utf8_lossy(&output.stderr).to_string();
                HttpResponse::BadRequest().json(GitResponse {
                    success: false,
                    message: format!("Failed to merge branch: {}", error_message),
                })
            }
        }
        Err(e) => {
            HttpResponse::InternalServerError().json(GitResponse {
                success: false,
                message: format!("Failed to execute git merge command: {}", e),
            })
        }
    }
}

// Handler to push changes to remote repository
pub async fn push_handler(data: web::Data<GitAppState>) -> impl Responder {
    // Get the project home directory
    let project_config = match data.project_manager.get_config() {
        Ok(config) => config,
        Err(e) => {
            return HttpResponse::InternalServerError().json(GitResponse {
                success: false,
                message: format!("Failed to get project configuration: {}", e),
            });
        }
    };

    let project_dir = project_config.project_home_directory.clone();
    if project_dir.is_empty() {
        return HttpResponse::BadRequest().json(GitResponse {
            success: false,
            message: "Project home directory is not set".to_string(),
        });
    }

    // Check if the project directory exists
    if !Path::new(&project_dir).exists() {
        return HttpResponse::BadRequest().json(GitResponse {
            success: false,
            message: format!("Project home directory does not exist: {}", project_dir),
        });
    }

    // Push changes to remote
    let output = Command::new("git")
        .arg("push")
        .current_dir(&project_dir)
        .output();

    match output {
        Ok(output) => {
            if output.status.success() {
                HttpResponse::Ok().json(GitResponse {
                    success: true,
                    message: "Changes pushed to remote repository successfully".to_string(),
                })
            } else {
                let error_message = String::from_utf8_lossy(&output.stderr).to_string();
                HttpResponse::BadRequest().json(GitResponse {
                    success: false,
                    message: format!("Failed to push changes: {}", error_message),
                })
            }
        }
        Err(e) => {
            HttpResponse::InternalServerError().json(GitResponse {
                success: false,
                message: format!("Failed to execute git push command: {}", e),
            })
        }
    }
}

pub async fn pull_handler(data: web::Data<GitAppState>) -> impl Responder {
    // Get the project home directory
    let project_config = match data.project_manager.get_config() {
        Ok(config) => config,
        Err(e) => {
            return HttpResponse::InternalServerError().json(GitResponse {
                success: false,
                message: format!("Failed to get project configuration: {}", e),
            });
        }
    };

    let project_dir = project_config.project_home_directory.clone();
    if project_dir.is_empty() {
        return HttpResponse::BadRequest().json(GitResponse {
            success: false,
            message: "Project home directory is not set".to_string(),
        });
    }

    // Check if the project directory exists
    if !Path::new(&project_dir).exists() {
        return HttpResponse::BadRequest().json(GitResponse {
            success: false,
            message: format!("Project home directory does not exist: {}", project_dir),
        });
    }

    // Push changes to remote
    let output = Command::new("git")
        .arg("pull")
        .current_dir(&project_dir)
        .output();

    match output {
        Ok(output) => {
            if output.status.success() {
                HttpResponse::Ok().json(GitResponse {
                    success: true,
                    message: "Changes pulled from remote repository successfully".to_string(),
                })
            } else {
                let error_message = String::from_utf8_lossy(&output.stderr).to_string();
                HttpResponse::BadRequest().json(GitResponse {
                    success: false,
                    message: format!("Failed to pull changes: {}", error_message),
                })
            }
        }
        Err(e) => {
            HttpResponse::InternalServerError().json(GitResponse {
                success: false,
                message: format!("Failed to execute git pull command: {}", e),
            })
        }
    }
}


// Helper function to resolve task dependencies and create an execution queue
fn resolve_task_dependencies(
    block_manager: &Arc<BlockConfigManager>,
    block_id: &str,
    task_id: &str,
) -> Result<Vec<String>, String> {
    // Get all blocks
    let blocks = block_manager.get_blocks()
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
        task.status.contains("COMPLETED") || task.description.contains("[COMPLETED]")
    };

    // Identify completed tasks
    for (id, task) in &block.todo_list {
        if is_task_completed(task) {
            completed_tasks.insert(id.clone());
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

// Handler to execute a task with Git integration
pub async fn execute_git_task_handler(
    data: web::Data<GitAppState>,
    request: web::Json<ExecuteGitTaskRequest>,
) -> impl Responder {
    let request = request.into_inner();
    let resolve_dependencies = request.resolve_dependencies.unwrap_or(false);

    // Get the project home directory from the project config
    let project_config = match data.project_manager.get_config() {
        Ok(config) => config,
        Err(e) => {
            return HttpResponse::InternalServerError().json(ExecuteGitTaskResponse {
                status: "error".to_string(),
                message: format!("Failed to get project configuration: {}", e),
            });
        }
    };

    let project_dir = project_config.project_home_directory.clone();
    if project_dir.is_empty() {
        return HttpResponse::BadRequest().json(ExecuteGitTaskResponse {
            status: "error".to_string(),
            message: "Project home directory is not set. Please configure it in the project settings.".to_string(),
        });
    }

    // Check if the project directory exists
    if !Path::new(&project_dir).exists() {
        return HttpResponse::BadRequest().json(ExecuteGitTaskResponse {
            status: "error".to_string(),
            message: format!("Project home directory does not exist: {}", project_dir),
        });
    }

    // Get the task description
    let task_description = request.task_description.clone();
    if task_description.is_empty() {
        return HttpResponse::BadRequest().json(ExecuteGitTaskResponse {
            status: "error".to_string(),
            message: "Task description cannot be empty".to_string(),
        });
    }

    // Clone the data for use in the background task
    let block_manager = data.block_manager.clone();
    let block_id = request.block_id.clone();
    let task_id = format!("{}", request.task_id);

    // If dependency resolution is enabled, resolve dependencies and execute tasks in order
    if resolve_dependencies {
        match resolve_task_dependencies(&block_manager, &block_id, &task_id) {
            Ok(execution_queue) => {
                if execution_queue.is_empty() {
                    // If the queue is empty, it means the task and all its dependencies are already completed
                    return HttpResponse::Ok().json(ExecuteGitTaskResponse {
                        status: "success".to_string(),
                        message: "Task and all its dependencies are already completed".to_string(),
                    });
                }

                // Log the execution queue
                println!("Executing tasks in order: {:?}", execution_queue);

                // We'll execute only the requested task, but mark its dependencies as completed
                // This is because executing multiple Git tasks in sequence can be complex
                // and might require more sophisticated error handling

                // Mark all dependencies as completed
                for task_id_to_execute in &execution_queue {
                    if task_id_to_execute != &task_id {
                        println!("Marking dependency task {} as completed", task_id_to_execute);
                        update_task_status(&block_manager, &block_id, task_id_to_execute.clone(), "[COMPLETED] (Dependency)");
                    }
                }

                // Continue with executing the requested task
                // The rest of the function will handle this
            }
            Err(e) => {
                return HttpResponse::BadRequest().json(ExecuteGitTaskResponse {
                    status: "error".to_string(),
                    message: format!("Failed to resolve task dependencies: {}", e),
                });
            }
        }
    }

    // Spawn a background task to execute the Git task flow

    // Step 1: Pull latest main branch
    println!("Step 1: Pulling latest main branch");
    let pull_output = Command::new("git")
        .arg("checkout")
        .arg("main")
        .current_dir(&project_dir)
        .output();

    if let Err(e) = pull_output {
        let task_id = task_id.clone();
        println!("Failed to checkout main branch: {}", e);
        update_task_status(&block_manager, &block_id, task_id, "[FAILED] Git checkout main failed");
        return HttpResponse::InternalServerError().json(ExecuteGitTaskResponse {
            status: "error".to_string(),
            message: format!("Failed to checkout main branch: {}", e),
        });
    }

    let pull_output = Command::new("git")
        .arg("pull")
        .current_dir(&project_dir)
        .output();

    if let Err(e) = pull_output {
        let task_id = task_id.clone();
        println!("Failed to pull latest changes: {}", e);
        update_task_status(&block_manager, &block_id, task_id, "[FAILED] Git pull failed");
        return HttpResponse::InternalServerError().json(ExecuteGitTaskResponse {
            status: "error".to_string(),
            message: format!("Git pull failed: {}", e),
        });
    }

    // Step 2: Create a task-specific branch
    println!("Step 2: Creating task-specific branch using task ID: {}", task_id);
    let branch_output = Command::new("git")
        .arg("checkout")
        .arg("-b")
        .arg(&task_id)
        .current_dir(&project_dir)
        .output();

    if let Err(e) = branch_output {
        let task_id = task_id.clone();
        println!("Failed to create task branch: {}", e);
        update_task_status(&block_manager, &block_id, task_id, "[FAILED] Git branch creation failed");
        return HttpResponse::InternalServerError().json(ExecuteGitTaskResponse {
            status: "error".to_string(),
            message: format!("Failed to create task branch: {}", e),
        });
    }

    // Step 3: Execute the task using Claude CLI
    println!("Step 3: Executing task");
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
            println!("Failed to execute Claude CLI: {}", e);
            update_task_status(&block_manager, &block_id, task_id, "[FAILED] Claude execution failed");
            return HttpResponse::InternalServerError().json(ExecuteGitTaskResponse {
                status: "error".to_string(),
                message: format!("Failed to execute Claude CLI: {}", e),
            });
        }
    };

    // Write the task description to the command's stdin
    if let Some(mut stdin) = child.stdin.take() {
        if let Err(e) = stdin.write_all(task_description.as_bytes()) {
            println!("Failed to write to Claude CLI stdin: {}", e);
            update_task_status(&block_manager, &block_id, task_id, "[FAILED] Claude input failed");
            return HttpResponse::InternalServerError().json(ExecuteGitTaskResponse {
                status: "error".to_string(),
                message: format!("Failed to write to Claude CLI stdin: {}", e),
            });
        }
    }

    // Wait for the command to complete
    let output = match child.wait_with_output() {
        Ok(output) => output,
        Err(e) => {
            println!("Failed to wait for Claude CLI command: {}", e);
            update_task_status(&block_manager, &block_id, task_id, "[FAILED] Claude execution failed");
            return HttpResponse::InternalServerError().json(ExecuteGitTaskResponse {
                status: "error".to_string(),
                message: format!("Failed to wait for Claude CLI command: {}", e),
            });
        }
    };

    let task_success = output.status.success();
    let log_output = String::from_utf8_lossy(&output.stdout).to_string();

    if !task_success {
        println!("Claude CLI command failed with exit code: {:?}", output.status.code());
        println!("Claude stderr:\n-----------------\n{}", String::from_utf8_lossy(&output.stderr));
        update_task_status_with_log(&block_manager, &block_id, task_id, "[FAILED] Claude execution failed", &log_output);
        return HttpResponse::InternalServerError().json(ExecuteGitTaskResponse {
            status: "error".to_string(),
            message: format!("Claude CLI command failed with exit code: {:?}", output.status.code()),
        });
    }

    println!("Claude CLI command completed successfully");
    println!("Claude output:\n-----------------\n{}", log_output);

    // Step 4: Commit changes
    println!("Step 4: Committing changes");
    let add_output = Command::new("git")
        .arg("add")
        .arg(".")
        .current_dir(&project_dir)
        .output();

    if let Err(e) = add_output {
        println!("Failed to stage changes: {}", e);
        update_task_status_with_log(&block_manager, &block_id, task_id, "[FAILED] Git add failed", &log_output);
        return HttpResponse::InternalServerError().json(ExecuteGitTaskResponse {
            status: "error".to_string(),
            message: format!("Failed to stage changes: {}", e),
        });
    }

    // Use task description as commit message
    let commit_message = task_description.lines().next().unwrap_or("Task execution").to_string();
    let commit_output = Command::new("git")
        .arg("commit")
        .arg("-m")
        .arg(&commit_message)
        .current_dir(&project_dir)
        .output();

    if let Err(e) = commit_output {
        println!("Failed to commit changes: {}", e);
        update_task_status_with_log(&block_manager, &block_id, task_id, "[FAILED] Git commit failed", &log_output);
        return HttpResponse::InternalServerError().json(ExecuteGitTaskResponse {
            status: "error".to_string(),
            message: format!("Failed to commit changes: {}", e),
        });
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
                println!("Failed to get commit ID: {}", String::from_utf8_lossy(&output.stderr));
                return HttpResponse::InternalServerError().json(ExecuteGitTaskResponse {
                    status: "error".to_string(),
                    message: format!("Failed to commit changes: {}", String::from_utf8_lossy(&output.stderr)),
                });
            }
        }
        Err(e) => {
            println!("Failed to execute git rev-parse command: {}", e);
            return HttpResponse::InternalServerError().json(ExecuteGitTaskResponse {
                status: "error".to_string(),
                message: format!("Failed to execute git rev-parse command: {}", e),
            });
        }
    };

    // Step 5: Merge back to main
    println!("Step 5: Merging back to main");
    let checkout_output = Command::new("git")
        .arg("checkout")
        .arg("main")
        .current_dir(&project_dir)
        .output();

    if let Err(e) = checkout_output {
        println!("Failed to checkout main branch: {}", e);
        update_task_status_with_log_and_commit_id(&block_manager, &block_id, task_id, "[FAILED] Git checkout main failed", &log_output, commit_id);
        return HttpResponse::InternalServerError().json(ExecuteGitTaskResponse {
            status: "error".to_string(),
            message: format!("Failed to checkout main branch: {}", e),
        });
    }

    let merge_output = Command::new("git")
        .arg("merge")
        .arg("--ff-only")
        .arg(&task_id)
        .current_dir(&project_dir)
        .output();

    if let Err(e) = merge_output {
        println!("Failed to merge task branch: {}", e);
        update_task_status_with_log_and_commit_id(&block_manager, &block_id, task_id, "[FAILED] Git merge failed", &log_output, commit_id);
        return HttpResponse::InternalServerError().json(ExecuteGitTaskResponse {
            status: "error".to_string(),
            message: format!("Failed to merge task branch: {}", e),
        });
    }

    // Step 6: Clean up (delete the task branch)
    println!("Step 6: Cleaning up");
    let delete_output = Command::new("git")
        .arg("branch")
        .arg("-d")
        .arg(&task_id)
        .current_dir(&project_dir)
        .output();

    if let Err(e) = delete_output {
        println!("Failed to delete task branch: {}", e);
        // This is not a critical error, so we continue
    }

    let task_id_c = task_id.clone();
    let commit_id_c = commit_id.clone();

    // Update the task status, log, and commit ID in the block config
    update_task_status_with_log_and_commit_id(&block_manager, &block_id, task_id, "[COMPLETED]", &log_output, commit_id);


    // Return a response indicating the task has been started
    let response = ExecuteGitTaskResponse {
        status: "started".to_string(),
        message: "Git task execution has been started in the background".to_string(),
    };

    println!("Task ended: {} : {}", task_id_c, commit_id_c.unwrap_or("no commit id".to_string()));
    HttpResponse::Ok().json(response)
}

// Helper function to update task status
fn update_task_status(block_manager: &Arc<BlockConfigManager>, block_id: &str, task_id: String, status: &str) {
    update_task_status_with_log(block_manager, block_id, task_id, status, "");
}

// Helper function to update task status and log
fn update_task_status_with_log(block_manager: &Arc<BlockConfigManager>, block_id: &str, task_id: String, status: &str, log: &str) {
    update_task_status_with_log_and_commit_id(block_manager, block_id, task_id, status, log, None);
}

// Helper function to update task status, log, and commit ID
fn update_task_status_with_log_and_commit_id(
    block_manager: &Arc<BlockConfigManager>,
    block_id: &str,
    task_id: String,
    status: &str,
    log: &str,
    commit_id: Option<String>,
) {
    fn update_task_and_save(
        block_manager: &BlockConfigManager,
        block_id: &str,
        task_id: &str,
        status: &str,
        log: &str,
        commit_id: Option<String>,
    ) -> Result<(), String> {
        let mut blocks = block_manager.get_blocks()
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
            if let Some(id) = commit_id {
                task_updated.commit_id = id;
            }

            // Update the task in the block's todo_list
            block.todo_list.insert(task_id.to_string(), task_updated);
        } else {
            return Err("Task not found".to_string());
        }

        match block_manager.update_block(block.clone()) {
            Ok(_) => {
                // Save the updated blocks to the file
                block_manager.save_blocks_to_file().map_err(|e| format!("Failed to save blocks to file: {}", e))?;
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

    // Usage (replaces the original code):
    if let Err(e) = update_task_and_save(&block_manager, &block_id, &task_id, &status, &log, commit_id) {
        println!("Failed to update task: {}", e);
    }
}

    // Handler to get a task's diff
pub async fn get_task_diff_handler(
    data: web::Data<GitAppState>,
    request: web::Json<GetTaskDiffRequest>,
) -> impl Responder {
    // Get the project home directory from the project config
    let project_config = match data.project_manager.get_config() {
        Ok(config) => config,
        Err(e) => {
            return HttpResponse::InternalServerError().json(GetTaskDiffResponse {
                success: false,
                message: format!("Failed to get project configuration: {}", e),
                commit_id: None,
                files_diff: Vec::new(),
            });
        }
    };

    let project_dir = project_config.project_home_directory.clone();
    if project_dir.is_empty() {
        return HttpResponse::BadRequest().json(GetTaskDiffResponse {
            success: false,
            message: "Project home directory is not set. Please configure it in the project settings.".to_string(),
            commit_id: None,
            files_diff: Vec::new(),
        });
    }

    // Check if the project directory exists
    if !Path::new(&project_dir).exists() {
        return HttpResponse::BadRequest().json(GetTaskDiffResponse {
            success: false,
            message: format!("Project home directory does not exist: {}", project_dir),
            commit_id: None,
            files_diff: Vec::new(),
        });
    }

    // Get the task's commit ID from the block manager
    let blocks = match data.block_manager.get_blocks() {
        Ok(blocks) => blocks,
        Err(e) => {
            return HttpResponse::InternalServerError().json(GetTaskDiffResponse {
                success: false,
                message: format!("Failed to get blocks: {}", e),
                commit_id: None,
                files_diff: Vec::new(),
            });
        }
    };

    // Find the block and task
    let block = blocks.iter().find(|b| b.block_id == request.block_id);
    if block.is_none() {
        return HttpResponse::BadRequest().json(GetTaskDiffResponse {
            success: false,
            message: format!("Block '{}' not found", request.block_id),
            commit_id: None,
            files_diff: Vec::new(),
        });
    }

    let mut block = block.unwrap();
    if !block.todo_list.contains_key(&request.task_id) {
        return HttpResponse::BadRequest().json(GetTaskDiffResponse {
            success: false,
            message: format!("Task index {} is out of bounds for block '{}'", request.task_id, request.block_id),
            commit_id: None,
            files_diff: Vec::new(),
        });
    }

    let task = block.todo_list.get(&request.task_id).unwrap();
    let commit_id = task.commit_id.clone();

    // If there's no commit ID, return an error
    if commit_id.len() == 0 {
        return HttpResponse::BadRequest().json(GetTaskDiffResponse {
            success: false,
            message: "No commit ID associated with this task".to_string(),
            commit_id: None,
            files_diff: Vec::new(),
        });
    }

    let commit_id = commit_id;

    // Get the list of modified files in the commit
    let files_output = Command::new("git")
        .arg("diff")
        .arg("--name-only")
        .arg(format!("{}^", commit_id))
        .arg(&commit_id)
        .current_dir(&project_dir)
        .output();


    // Check if all commands were successful
    match (files_output) {
        (Ok(files)) => {
            if files.status.success() {

                // Parse the list of modified files
                let files_content = String::from_utf8_lossy(&files.stdout).to_string();
                let files_list = files_content
                    .trim()
                    .split('\n')
                    .filter(|s| !s.is_empty())
                    .collect::<Vec<&str>>();

                // For each file, get the original and modified content
                let mut files_diff = Vec::new();
                for file_path in files_list {
                    // Get the original content for this file
                    let file_original_output = Command::new("git")
                        .arg("show")
                        .arg(format!("{}^:{}", commit_id, file_path))
                        .current_dir(&project_dir)
                        .output();

                    // Get the modified content for this file
                    let file_modified_output = Command::new("git")
                        .arg("show")
                        .arg(format!("{}:{}", commit_id, file_path))
                        .current_dir(&project_dir)
                        .output();

                    match (file_original_output, file_modified_output) {
                        (Ok(file_original), Ok(file_modified)) => {
                            let file_original_content = if file_original.status.success() {
                                Some(String::from_utf8_lossy(&file_original.stdout).to_string())
                            } else {
                                None
                            };

                            let file_modified_content = if file_modified.status.success() {
                                Some(String::from_utf8_lossy(&file_modified.stdout).to_string())
                            } else {
                                None
                            };

                            files_diff.push(CommitFiles {
                                file_path: file_path.to_string(),
                                original_content: file_original_content,
                                modified_content: file_modified_content,
                            });
                        }
                        _ => {
                            // If we can't get the content for this file, skip it
                            continue;
                        }
                    }
                }

                HttpResponse::Ok().json(GetTaskDiffResponse {
                    success: true,
                    message: "File versions retrieved successfully".to_string(),
                    commit_id: Some(commit_id),
                    files_diff,
                })
            } else {
                HttpResponse::BadRequest().json(GetTaskDiffResponse {
                    success: false,
                    message: "Failed to get file versions".to_string(),
                    commit_id: Some(commit_id),
                    files_diff: Vec::new(),
                })
            }
        }
        (Err(e)) => {
            HttpResponse::InternalServerError().json(GetTaskDiffResponse {
                success: false,
                message: format!("Failed to execute git command: {}", e),
                commit_id: Some(commit_id),
                files_diff: Vec::new(),
            })
        }
    }
}

// Handler to build the project
pub async fn build_handler(data: web::Data<GitAppState>) -> impl Responder {
    // Get the project home directory
    let project_config = match data.project_manager.get_config() {
        Ok(config) => config,
        Err(e) => {
            return HttpResponse::InternalServerError().json(GitResponse {
                success: false,
                message: format!("Failed to get project configuration: {}", e),
            });
        }
    };

    let project_dir = project_config.project_home_directory.clone();
    if project_dir.is_empty() {
        return HttpResponse::BadRequest().json(GitResponse {
            success: false,
            message: "Project home directory is not set".to_string(),
        });
    }

    // Check if the project directory exists
    if !Path::new(&project_dir).exists() {
        return HttpResponse::BadRequest().json(GitResponse {
            success: false,
            message: format!("Project home directory does not exist: {}", project_dir),
        });
    }

    // Check if build script exists, create it if it doesn't
    let build_script_path = Path::new(&project_dir).join("build.sh");
    if !build_script_path.exists() {
        // Create a default build script
        let build_script_content = r#"#!/bin/bash
echo "Building project..."
# Add your build commands here
echo "Build completed successfully!"
"#;
        match fs::write(&build_script_path, build_script_content) {
            Ok(_) => {
                // Make the script executable
                #[cfg(unix)]
                {
                    use std::os::unix::fs::PermissionsExt;
                    let mut perms = fs::metadata(&build_script_path)
                        .expect("Failed to get file metadata")
                        .permissions();
                    perms.set_mode(0o755);
                    fs::set_permissions(&build_script_path, perms)
                        .expect("Failed to set file permissions");
                }
                println!("Created build script at {}", build_script_path.display());
            }
            Err(e) => {
                return HttpResponse::InternalServerError().json(GitResponse {
                    success: false,
                    message: format!("Failed to create build script: {}", e),
                });
            }
        }
    }

    // Execute the build script
    let output = Command::new("sh")
        .arg(build_script_path)
        .current_dir(&project_dir)
        .output();

    match output {
        Ok(output) => {
            if output.status.success() {
                let output_message = String::from_utf8_lossy(&output.stdout).to_string();
                HttpResponse::Ok().json(BuildResponse {
                    success: true,
                    message: "Build completed successfully".to_string(),
                    output: output_message,
                })
            } else {
                let error_message = String::from_utf8_lossy(&output.stderr).to_string();
                HttpResponse::BadRequest().json(BuildResponse {
                    success: false,
                    message: "Build failed".to_string(),
                    output: error_message,
                })
            }
        }
        Err(e) => {
            let error_message = format!("Failed to execute build script: {}", e);
            HttpResponse::InternalServerError().json(BuildResponse {
                success: false,
                message: error_message.clone(),
                output: error_message,
            })
        }
    }
}
