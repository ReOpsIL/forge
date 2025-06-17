use actix_web::{web, HttpResponse, Responder};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::process::Command;
use std::path::Path;
use std::fs;
use std::io::Write;

use crate::project_config::ProjectConfigManager;

// AppState for git handlers
pub struct GitAppState {
    pub project_manager: Arc<ProjectConfigManager>,
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
