use actix_web::{web, HttpResponse, Responder, Error};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use crate::project_config::{ProjectConfig, ProjectConfigManager, test_git_connection};
use crate::profession_prompts::{self, Profession, ProfessionCategory};

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

// Response for getting all professions
#[derive(Debug, Serialize)]
pub struct ProfessionResponse {
    pub id: String,
    pub name: String,
    pub category: String,
}

// Response for getting all profession categories
#[derive(Debug, Serialize)]
pub struct ProfessionCategoryResponse {
    pub name: String,
    pub professions: Vec<ProfessionResponse>,
}

// Response for getting all professions grouped by category
#[derive(Debug, Serialize)]
pub struct AllProfessionsResponse {
    pub categories: Vec<ProfessionCategoryResponse>,
}

// Response for getting profession-specific prompts
#[derive(Debug, Serialize)]
pub struct ProfessionPromptsResponse {
    pub auto_complete_system_prompt: String,
    pub auto_complete_user_prompt: String,
    pub enhance_description_system_prompt: String,
    pub enhance_description_user_prompt: String,
    pub generate_tasks_system_prompt: String,
    pub generate_tasks_user_prompt: String,
    pub process_markdown_spec_system_prompt: String,
    pub process_markdown_spec_user_prompt: String,
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

// Handler to get all available professions
pub async fn get_professions_handler() -> impl Responder {
    // Get all professions
    let all_professions = profession_prompts::get_all_professions();

    // Group professions by category
    let mut categories_map: std::collections::HashMap<ProfessionCategory, Vec<ProfessionResponse>> = std::collections::HashMap::new();

    // Process each profession
    for profession in all_professions {
        let profession_response = ProfessionResponse {
            id: profession.id,
            name: profession.name,
            category: profession.category.display_name().to_string(),
        };

        categories_map.entry(profession.category)
            .or_insert_with(Vec::new)
            .push(profession_response);
    }

    // Convert the map to a vector of category responses
    let mut categories = Vec::new();
    for category in ProfessionCategory::all_categories() {
        let professions = categories_map.remove(&category).unwrap_or_default();
        categories.push(ProfessionCategoryResponse {
            name: category.display_name().to_string(),
            professions,
        });
    }

    // Return the response
    HttpResponse::Ok().json(AllProfessionsResponse { categories })
}

// Handler to get profession-specific prompts
pub async fn get_profession_prompts_handler(path: web::Path<String>) -> impl Responder {
    let profession_id = path.into_inner();

    // Get the profession by ID
    if let Some(profession) = profession_prompts::get_profession_by_id(&profession_id) {
        // Convert the prompts to a response
        let prompts_response = ProfessionPromptsResponse {
            auto_complete_system_prompt: profession.prompts.auto_complete_system_prompt,
            auto_complete_user_prompt: profession.prompts.auto_complete_user_prompt,
            enhance_description_system_prompt: profession.prompts.enhance_description_system_prompt,
            enhance_description_user_prompt: profession.prompts.enhance_description_user_prompt,
            generate_tasks_system_prompt: profession.prompts.generate_tasks_system_prompt,
            generate_tasks_user_prompt: profession.prompts.generate_tasks_user_prompt,
            process_markdown_spec_system_prompt: profession.prompts.process_markdown_spec_system_prompt,
            process_markdown_spec_user_prompt: profession.prompts.process_markdown_spec_user_prompt,
        };

        HttpResponse::Ok().json(prompts_response)
    } else {
        HttpResponse::NotFound().body(format!("Profession with ID '{}' not found", profession_id))
    }
}
