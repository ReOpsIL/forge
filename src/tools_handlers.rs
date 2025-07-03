use actix_web::{web, HttpResponse, Responder};
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::process::Command;
use std::sync::Arc;

use crate::block_config::BlockConfigManager;
use crate::block_handlers::AppState;
use crate::project_config::ProjectConfigManager;
use crate::task_executor_wrapper::enqueue_task;

// AppState for git handlers
pub struct ToolsAppState {
    pub project_manager: Arc<ProjectConfigManager>,
    pub block_manager: Arc<BlockConfigManager>,
}

// Request body for executing a task with Git integration
#[derive(Debug, Deserialize)]
pub struct ExecuteBlockTaskRequest {
    pub block_id: String,
    pub task_id: String,
    pub task_description: String,
    pub resolve_dependencies: bool,
    pub force_completed: bool,
}


// Response for Git operations
#[derive(Debug, Serialize)]
pub struct ExecuteBlockTaskResponse {
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


// Response for getting local branches
#[derive(Debug, Serialize)]
pub struct BranchesResponse {
    pub success: bool,
    pub message: String,
    pub branches: Vec<String>,
}

pub async fn get_branches_handler(data: web::Data<AppState>) -> impl Responder {
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
        Err(e) => HttpResponse::InternalServerError().json(BranchesResponse {
            success: false,
            message: format!("Failed to execute git command: {}", e),
            branches: Vec::new(),
        }),
    }
}

pub async fn execute_block_task_handler(
    data: web::Data<ToolsAppState>,
    request: web::Json<ExecuteBlockTaskRequest>,
) -> impl Responder {
    let request = request.into_inner();
    let resolve_dependencies = request.resolve_dependencies;
    let force_completed = request.force_completed;

    let result = enqueue_task(
        &*request.block_id,
        &*request.task_id,
        &*request.task_description,
        resolve_dependencies,
        force_completed,
    );
    match result {
        Ok(_) => HttpResponse::Ok().json(ExecuteBlockTaskResponse {
            success: true,
            message: "Task execution queued successfully".to_string(),
        }),
        Err(e) => HttpResponse::InternalServerError().json(ExecuteBlockTaskResponse {
            success: false,
            message: format!("Failed to enqueue task: {}", e),
        }),
    }
}
