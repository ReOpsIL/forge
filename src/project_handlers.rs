use actix_web::{web, HttpResponse, Responder, Error};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use crate::project_config::{ProjectConfig, ProjectConfigManager, test_git_connection};

// AppState for project handlers
pub struct ProjectAppState {
    pub project_manager: Arc<ProjectConfigManager>,
}

// Request body for testing Git connection
#[derive(Debug, Deserialize)]
pub struct TestGitConnectionRequest {
    pub url: String,
}

// Response for testing Git connection
#[derive(Debug, Serialize)]
pub struct TestGitConnectionResponse {
    pub message: String,
}

// Response for checking if project settings are configured
#[derive(Debug, Serialize)]
pub struct ProjectConfigStatusResponse {
    pub configured: bool,
    pub message: String,
}

// Handler to get project configuration
pub async fn get_project_config_handler(data: web::Data<ProjectAppState>) -> impl Responder {
    match data.project_manager.load_config() {
        Ok(config) => HttpResponse::Ok().json(config),
        Err(e) => {
            eprintln!("Error loading project config: {}", e);
            HttpResponse::InternalServerError().body(format!("Error loading project config: {}", e))
        }
    }
}

// Handler to update project configuration
pub async fn update_project_config_handler(
    data: web::Data<ProjectAppState>,
    config: web::Json<ProjectConfig>,
) -> impl Responder {
    match data.project_manager.save_config(&config) {
        Ok(_) => {
            // If project_home_directory is specified, ensure it exists
            if !config.project_home_directory.is_empty() {
                let project_dir = std::path::Path::new(&config.project_home_directory);
                if !project_dir.exists() {
                    match std::fs::create_dir_all(project_dir) {
                        Ok(_) => println!("Created project directory: {}", config.project_home_directory),
                        Err(e) => {
                            eprintln!("Error creating project directory: {}", e);
                            return HttpResponse::InternalServerError().body(
                                format!("Error creating project directory: {}", e)
                            );
                        }
                    }
                }
            }
            HttpResponse::Ok().json(config)
        },
        Err(e) => {
            eprintln!("Error saving project config: {}", e);
            HttpResponse::InternalServerError().body(format!("Error saving project config: {}", e))
        }
    }
}

// Handler to test Git connection
pub async fn test_git_connection_handler(
    request: web::Json<TestGitConnectionRequest>,
) -> impl Responder {
    match test_git_connection(&request.url).await {
        Ok(message) => HttpResponse::Ok().json(TestGitConnectionResponse { message }),
        Err(e) => {
            eprintln!("Error testing Git connection: {}", e);
            HttpResponse::BadRequest().json(TestGitConnectionResponse { message: e })
        }
    }
}

// Handler to check if project settings are configured
pub async fn check_project_config_handler(data: web::Data<ProjectAppState>) -> impl Responder {
    match data.project_manager.get_config() {
        Ok(config) => {
            // Check if project_home_directory is set
            if config.project_home_directory.is_empty() {
                HttpResponse::Ok().json(ProjectConfigStatusResponse {
                    configured: false,
                    message: "Project home directory is not set".to_string(),
                })
            } else {
                // Check if the directory exists
                let project_dir = std::path::Path::new(&config.project_home_directory);
                if !project_dir.exists() {
                    HttpResponse::Ok().json(ProjectConfigStatusResponse {
                        configured: false,
                        message: "Project home directory does not exist".to_string(),
                    })
                } else {
                    HttpResponse::Ok().json(ProjectConfigStatusResponse {
                        configured: true,
                        message: "Project settings are configured".to_string(),
                    })
                }
            }
        },
        Err(e) => {
            eprintln!("Error checking project config: {}", e);
            HttpResponse::Ok().json(ProjectConfigStatusResponse {
                configured: false,
                message: format!("Error checking project config: {}", e),
            })
        }
    }
}
