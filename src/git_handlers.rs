use actix_web::{web, HttpResponse, Responder};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::process::Command;
use std::path::Path;
use std::fs;
use std::io::Write;
use std::process::Stdio;
use tokio::task;

use crate::project_config::ProjectConfigManager;
use crate::block_config::BlockConfigManager;

// AppState for git handlers
pub struct GitAppState {
    pub project_manager: Arc<ProjectConfigManager>,
    pub block_manager: Arc<BlockConfigManager>,
}

// Request body for executing a task with Git integration
#[derive(Debug, Deserialize)]
pub struct ExecuteGitTaskRequest {
    pub block_name: String,
    pub task_index: usize,
    pub task_description: String,
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



// Handler to execute a task with Git integration
pub async fn execute_git_task_handler(
    data: web::Data<GitAppState>,
    request: web::Json<ExecuteGitTaskRequest>,
) -> impl Responder {
    let request = request.into_inner();

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
    let block_name = request.block_name.clone();
    let task_index = request.task_index;
    let task_id = format!("task_{}", task_index); // Use task index as task_id for branch name

    // Spawn a background task to execute the Git task flow
    task::spawn(async move {
        // Step 1: Pull latest main branch
        println!("Step 1: Pulling latest main branch");
        let pull_output = Command::new("git")
            .arg("checkout")
            .arg("main")
            .current_dir(&project_dir)
            .output();

        if let Err(e) = pull_output {
            println!("Failed to checkout main branch: {}", e);
            update_task_status(&block_manager, &block_name, task_index, "[FAILED] Git checkout main failed");
            return;
        }

        let pull_output = Command::new("git")
            .arg("pull")
            .current_dir(&project_dir)
            .output();

        if let Err(e) = pull_output {
            println!("Failed to pull latest changes: {}", e);
            update_task_status(&block_manager, &block_name, task_index, "[FAILED] Git pull failed");
            return;
        }

        // Step 2: Create a task-specific branch
        println!("Step 2: Creating task-specific branch: {}", task_id);
        let branch_output = Command::new("git")
            .arg("checkout")
            .arg("-b")
            .arg(&task_id)
            .current_dir(&project_dir)
            .output();

        if let Err(e) = branch_output {
            println!("Failed to create task branch: {}", e);
            update_task_status(&block_manager, &block_name, task_index, "[FAILED] Git branch creation failed");
            return;
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
                update_task_status(&block_manager, &block_name, task_index, "[FAILED] Claude execution failed");
                return;
            }
        };

        // Write the task description to the command's stdin
        if let Some(mut stdin) = child.stdin.take() {
            if let Err(e) = stdin.write_all(task_description.as_bytes()) {
                println!("Failed to write to Claude CLI stdin: {}", e);
                update_task_status(&block_manager, &block_name, task_index, "[FAILED] Claude input failed");
                return;
            }
        }

        // Wait for the command to complete
        let output = match child.wait_with_output() {
            Ok(output) => output,
            Err(e) => {
                println!("Failed to wait for Claude CLI command: {}", e);
                update_task_status(&block_manager, &block_name, task_index, "[FAILED] Claude execution failed");
                return;
            }
        };

        let task_success = output.status.success();
        let log_output = String::from_utf8_lossy(&output.stdout).to_string();

        if !task_success {
            println!("Claude CLI command failed with exit code: {:?}", output.status.code());
            println!("Claude stderr:\n-----------------\n{}", String::from_utf8_lossy(&output.stderr));
            update_task_status_with_log(&block_manager, &block_name, task_index, "[FAILED] Claude execution failed", &log_output);
            return;
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
            update_task_status_with_log(&block_manager, &block_name, task_index, "[FAILED] Git add failed", &log_output);
            return;
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
            update_task_status_with_log(&block_manager, &block_name, task_index, "[FAILED] Git commit failed", &log_output);
            return;
        }

        // Step 5: Merge back to main
        println!("Step 5: Merging back to main");
        let checkout_output = Command::new("git")
            .arg("checkout")
            .arg("main")
            .current_dir(&project_dir)
            .output();

        if let Err(e) = checkout_output {
            println!("Failed to checkout main branch: {}", e);
            update_task_status_with_log(&block_manager, &block_name, task_index, "[FAILED] Git checkout main failed", &log_output);
            return;
        }

        let merge_output = Command::new("git")
            .arg("merge")
            .arg("--ff-only")
            .arg(&task_id)
            .current_dir(&project_dir)
            .output();

        if let Err(e) = merge_output {
            println!("Failed to merge task branch: {}", e);
            update_task_status_with_log(&block_manager, &block_name, task_index, "[FAILED] Git merge failed", &log_output);
            return;
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

        // Update the task status and log in the block config
        update_task_status_with_log(&block_manager, &block_name, task_index, "[COMPLETED]", &log_output);
    });

    // Return a response indicating the task has been started
    let response = ExecuteGitTaskResponse {
        status: "started".to_string(),
        message: "Git task execution has been started in the background".to_string(),
    };

    HttpResponse::Ok().json(response)
}

// Helper function to update task status
fn update_task_status(block_manager: &Arc<BlockConfigManager>, block_name: &str, task_index: usize, status: &str) {
    update_task_status_with_log(block_manager, block_name, task_index, status, "");
}

// Helper function to update task status and log
fn update_task_status_with_log(block_manager: &Arc<BlockConfigManager>, block_name: &str, task_index: usize, status: &str, log: &str) {
    if let Ok(mut blocks) = block_manager.get_blocks() {
        if let Some(block) = blocks.iter_mut().find(|b| b.name == block_name) {
            if let Some(task) = block.todo_list.get_mut(task_index) {
                // Append a status marker to the task description
                if !task.description.contains(status) {
                    task.description = format!("{} {}", task.description, status);
                }

                // Store the output in the task's log field
                if !log.is_empty() {
                    task.log = Some(log.to_string());
                }

                // Update the block in the database
                if let Err(e) = block_manager.update_block(block.clone()) {
                    println!("Failed to update block: {}", e);
                } else {
                    // Save the updated blocks to the file
                    if let Err(e) = block_manager.save_blocks_to_file() {
                        println!("Failed to save blocks to file: {}", e);
                    }
                }
            }
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
 