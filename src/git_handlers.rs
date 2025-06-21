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
use crate::task_executor;
use crate::task_executor::get_task_executor;
use crate::task_executor_wrapper::enqueue_task;

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
    pub resolve_dependencies: bool,
    pub force_completed: bool,
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

// Response for getting local branches
#[derive(Debug, Serialize)]
pub struct BranchesResponse {
    pub success: bool,
    pub message: String,
    pub branches: Vec<String>,
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

// Handler to execute a task with Git integration
pub async fn execute_git_task_handler(
    data: web::Data<GitAppState>,
    request: web::Json<ExecuteGitTaskRequest>,
) -> impl Responder {
    let request = request.into_inner();
    let resolve_dependencies = request.resolve_dependencies;
    let force_completed = request.force_completed;
    println!("force_completed: {}",force_completed);

    let result= enqueue_task(&*request.block_id, &*request.task_id, &*request.task_description, resolve_dependencies, force_completed);
    match result {
        Ok(_) => {
            HttpResponse::Ok().json(GitResponse {
                success: true,
                message: "Task execution queued successfully".to_string(),
            })
        },
        Err(e) => {
            HttpResponse::InternalServerError().json(GitResponse {
                success: false,
                message: format!("Failed to enqueue task: {}", e),
            })
        }
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
                let err_message = String::from_utf8_lossy(&output.stderr).to_string();

                HttpResponse::Ok().json(BuildResponse {
                    success: true,
                    message: "Build completed successfully".to_string(),
                    output: format!("Output:\n{}\n\nError:\n{}",output_message,err_message)
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

// Handler to get local Git branches
pub async fn get_branches_handler(data: web::Data<GitAppState>) -> impl Responder {
    // Get the project home directory
    let project_config = match data.project_manager.get_config() {
        Ok(config) => config,
        Err(e) => {
            return HttpResponse::InternalServerError().json(BranchesResponse {
                success: false,
                message: format!("Failed to get project configuration: {}", e),
                branches: Vec::new(),
            });
        }
    };

    let project_dir = project_config.project_home_directory.clone();
    if project_dir.is_empty() {
        return HttpResponse::BadRequest().json(BranchesResponse {
            success: false,
            message: "Project home directory is not set".to_string(),
            branches: Vec::new(),
        });
    }

    // Check if the project directory exists
    if !Path::new(&project_dir).exists() {
        return HttpResponse::BadRequest().json(BranchesResponse {
            success: false,
            message: format!("Project home directory does not exist: {}", project_dir),
            branches: Vec::new(),
        });
    }

    // Get local branches using git branch command
    let output = Command::new("git")
        .arg("branch")
        .arg("--format=%(refname:short)")
        .current_dir(&project_dir)
        .output();

    match output {
        Ok(output) => {
            if output.status.success() {
                let branches_string = String::from_utf8_lossy(&output.stdout);
                let branches: Vec<String> = branches_string
                    .lines()
                    .map(|line| line.trim().to_string())
                    .filter(|line| !line.is_empty())
                    .collect();

                HttpResponse::Ok().json(BranchesResponse {
                    success: true,
                    message: "Branches retrieved successfully".to_string(),
                    branches,
                })
            } else {
                let error_message = String::from_utf8_lossy(&output.stderr).to_string();
                HttpResponse::BadRequest().json(BranchesResponse {
                    success: false,
                    message: format!("Failed to get branches: {}", error_message),
                    branches: Vec::new(),
                })
            }
        }
        Err(e) => {
            HttpResponse::InternalServerError().json(BranchesResponse {
                success: false,
                message: format!("Failed to execute git command: {}", e),
                branches: Vec::new(),
            })
        }
    }
}
