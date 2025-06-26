use actix_web::{web, HttpResponse, Responder};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::sync::Arc;
use tracing::{error, info};
use crate::block_config::{generate_sample_config, BlockConfigManager};
use crate::llm_handler::{auto_complete_description, enhance_description, generate_tasks, process_specification, GeneratedBlock, LLMProvider};
use crate::models::{Block, Task};
use crate::project_config::ProjectConfigManager;

// Define a response type for block dependencies
#[derive(Serialize)]
pub struct BlockDependenciesResponse {
    pub tasks: Vec<TaskDependency>,
}

#[derive(Serialize)]
pub struct TaskDependency {
    pub task_id: String,
    pub description: String,
    pub dependencies: Vec<String>,
}

// Define a response type for auto-complete suggestions
#[derive(Serialize)]
pub struct AutoCompleteResponse {
    pub suggestion: String,
}

#[derive(Serialize)]
pub struct ExecuteTaskResponse {
    pub status: String,
    pub message: String,
}

// Define request and response types for markdown file processing
#[derive(Deserialize)]
pub struct ProcessMarkdownRequest {
    pub block_id: String,
    pub markdown_content: String,
}

#[derive(Serialize)]
pub struct ProcessMarkdownResponse {
    pub status: String,
    pub message: String,
}

// Define request and response types for specification processing
#[derive(Deserialize)]
pub struct ProcessSpecRequest {
    pub markdown_content: String,
}

#[derive(Serialize)]
pub struct ProcessSpecResponse {
    pub status: String,
    pub message: String,
    pub blocks: Vec<Block>,
}

// Define the config file path
pub const BLOCK_CONFIG_FILE: &str = "blocks_config.json";

// Create a data structure to hold the BlockConfigManager and ProjectConfigManager
pub struct AppState {
    pub block_manager: Arc<BlockConfigManager>,
    pub project_manager: Arc<ProjectConfigManager>,
}

// API endpoint to get blocks
pub async fn get_blocks_handler(data: web::Data<AppState>) -> impl Responder {
    match data.block_manager.get_blocks() {
        Ok(blocks) => HttpResponse::Ok().json(blocks),
        Err(e) => HttpResponse::InternalServerError().body(e),
    }
}

// API endpoint to add a new block
pub async fn add_block_handler(block: web::Json<Block>, data: web::Data<AppState>) -> impl Responder {
    match data.block_manager.add_block(block.into_inner()) {
        Ok(_) => {
            // Save the updated blocks to the file
            if let Err(e) = data.block_manager.save_blocks_to_file() {
                return HttpResponse::InternalServerError().body(e);
            }
            HttpResponse::Ok().body("Block added successfully")
        },
        Err(e) => HttpResponse::BadRequest().body(e),
    }
}

// Function to enhance block description and generate tasks using LLM
async fn enhance_block_with_llm(mut block: Block, data: &web::Data<AppState>) -> Result<Block, String> {
    // Get the project configuration to get the LLM provider setting
    let project_config = data.project_manager.get_config()
        .map_err(|e| format!("Failed to get project config: {}", e))?;

    // Enhance the description using LLM
    let enhanced_description = enhance_description(
        &block.description, 
        project_config.llm_provider
    ).await?;

    // Update the block with the enhanced description
    block.description = enhanced_description;

    Ok(block)
}

async fn generate_tasks_with_llm(mut block: Block, data: &web::Data<AppState>) -> Result<Block, String> {
    // Get the project configuration to get the LLM provider setting
    let project_config = data.project_manager.get_config()
        .map_err(|e| format!("Failed to get project config: {}", e))?;

    // Generate tasks based on the enhanced description
    let generated_tasks = generate_tasks(
        &block.description, 
        project_config.llm_provider
    ).await?;

    // Add the generated tasks to the block's todo list
    for task in generated_tasks {
        let task_id = task.task_id.clone();
        block.todo_list.insert(task_id, task);
    }

    Ok(block)
}


pub async fn enhance_block_handler(block: web::Json<Block>, data: web::Data<AppState>) -> impl Responder {
    let mut block = block.into_inner();

    match enhance_block_with_llm(block.clone(), &data).await {
        Ok(enhanced_block) => {
            block = enhanced_block;
        },
        Err(e) => {
            println!("Failed to enhance block with LLM: {}", e);
            // Continue with the original update even if LLM enhancement fails
        }
    }


    // Update the block in the database
    match data.block_manager.update_block(block) {
        Ok(_) => {
            // Save the updated blocks to the file
            if let Err(e) = data.block_manager.save_blocks_to_file() {
                return HttpResponse::InternalServerError().body(e);
            }
            HttpResponse::Ok().body("Block updated successfully")
        },
        Err(e) => HttpResponse::BadRequest().body(e),
    }
}

pub async fn generate_tasks_block_handler(block: web::Json<Block>, data: web::Data<AppState>) -> impl Responder {
    let mut block = block.into_inner();

    match generate_tasks_with_llm(block.clone(), &data).await {
        Ok(block_with_tasks) => {
            block = block_with_tasks;
        },
        Err(e) => {
            println!("Failed to enhance block with LLM: {}", e);
            // Continue with the original update even if LLM enhancement fails
        }
    }

    // Update the block in the database
    match data.block_manager.update_block(block) {
        Ok(_) => {
            // Save the updated blocks to the file
            if let Err(e) = data.block_manager.save_blocks_to_file() {
                return HttpResponse::InternalServerError().body(e);
            }
            HttpResponse::Ok().body("Block updated successfully")
        },
        Err(e) => HttpResponse::BadRequest().body(e),
    }
}


// API endpoint to update an existing block
pub async fn update_block_handler(block: web::Json<Block>, data: web::Data<AppState>) -> impl Responder {
    let block = block.into_inner();

    // Update the block in the database
    match data.block_manager.update_block(block) {
        Ok(_) => {
            // Save the updated blocks to the file
            if let Err(e) = data.block_manager.save_blocks_to_file() {
                return HttpResponse::InternalServerError().body(e);
            }
            HttpResponse::Ok().body("Block updated successfully")
        },
        Err(e) => HttpResponse::BadRequest().body(e),
    }
}

// API endpoint to get block dependencies
pub async fn get_block_dependencies_handler(path: web::Path<String>, data: web::Data<AppState>) -> impl Responder {
    let block_id = path.into_inner();

    // Get all blocks
    let blocks = match data.block_manager.get_blocks() {
        Ok(blocks) => blocks,
        Err(e) => return HttpResponse::InternalServerError().body(e),
    };

    // Find the block with the matching ID
    let block = match blocks.iter().find(|b| b.block_id == block_id) {
        Some(block) => block,
        None => return HttpResponse::NotFound().body(format!("Block with ID {} not found", block_id)),
    };

    // Extract tasks and their dependencies from the block's todo_list
    let tasks: Vec<TaskDependency> = block.todo_list.values()
        .map(|task| TaskDependency {
            task_id: task.task_id.clone(),
            description: task.description.clone(),
            dependencies: task.dependencies.clone(),
        })
        .collect();

    // Create the response
    let response = BlockDependenciesResponse { tasks };

    // Return the response as JSON
    HttpResponse::Ok().json(response)
}


// API endpoint to delete a block
pub async fn delete_block_handler(path: web::Path<String>, data: web::Data<AppState>) -> impl Responder {
    let block_id = path.into_inner();
    match data.block_manager.delete_block(&block_id) {
        Ok(_) => {
            // Save the updated blocks to the file
            if let Err(e) = data.block_manager.save_blocks_to_file() {
                return HttpResponse::InternalServerError().body(e);
            }
            HttpResponse::Ok().body("Block deleted successfully")
        },
        Err(e) => HttpResponse::BadRequest().body(e),
    }
}

// Structure for the task request
#[derive(Deserialize)]
pub struct TaskItemRequest {
    pub task_id: String,
    pub task_name: String,
    pub description: String,
    #[serde(default)]
    pub acceptance_criteria: Vec<String>,
    #[serde(default)]
    pub dependencies: Vec<String>,
    #[serde(default)]
    pub estimated_effort: String,
    #[serde(default)]
    pub files_affected: Vec<String>,
    #[serde(default)]
    pub function_signatures: Vec<String>,
    #[serde(default)]
    pub testing_requirements: Vec<String>,
    #[serde(default)]
    pub log: String,
    #[serde(default)]
    pub commit_id: String,
    #[serde(default)]
    pub status: String,
}

// API endpoint to add a task to a block
pub async fn add_task_handler(path: web::Path<String>, task_request: web::Json<TaskItemRequest>, data: web::Data<AppState>) -> impl Responder {
    let block_id = path.into_inner();
    let task_request = task_request.into_inner();

    // Find the block to update
    let mut blocks = match data.block_manager.get_blocks() {
        Ok(blocks) => blocks,
        Err(e) => return HttpResponse::InternalServerError().body(e),
    };

    let block_index = blocks.iter().position(|b| b.block_id == block_id);
    if block_index.is_none() {
        return HttpResponse::BadRequest().body(format!("Block '{}' not found", block_id));
    }

    // Create a new task with all the provided fields
    let mut task = Task::new(task_request.description.clone());

    // Update the task with the provided fields
    task.task_id = task_request.task_id;
    task.task_name = task_request.task_name;
    task.acceptance_criteria = task_request.acceptance_criteria;
    task.dependencies = task_request.dependencies;
    task.estimated_effort = task_request.estimated_effort;
    task.files_affected = task_request.files_affected;
    task.function_signatures = task_request.function_signatures;
    task.testing_requirements = task_request.testing_requirements;

    // Only set these fields if they are provided (they should be read-only during creation)
    if !task_request.log.is_empty() {
        task.log = task_request.log;
    }
    if !task_request.commit_id.is_empty() {
        task.commit_id = task_request.commit_id;
    }
    if !task_request.status.is_empty() {
        task.status = task_request.status;
    }

    // Add the task to the block
    let task_id = task.task_id.clone();
    blocks[block_index.unwrap()].todo_list.insert(task_id.clone(), task);

    // Update the block in the database
    match data.block_manager.update_block(blocks[block_index.unwrap()].clone()) {
        Ok(_) => {
            // Save the updated blocks to the file
            if let Err(e) = data.block_manager.save_blocks_to_file() {
                return HttpResponse::InternalServerError().body(e);
            }
            HttpResponse::Ok().json(json!({ "task_id": task_id }))
        },
        Err(e) => HttpResponse::BadRequest().body(e),
    }
}

// API endpoint to remove a todo item from a block
pub async fn remove_task_handler(path: web::Path<(String, String)>, data: web::Data<AppState>) -> impl Responder {
    let (block_id, task_id) = path.into_inner();
    match data.block_manager.remove_task_item(&block_id, task_id) {
        Ok(_) => {
            // Save the updated blocks to the file
            if let Err(e) = data.block_manager.save_blocks_to_file() {
                return HttpResponse::InternalServerError().body(e);
            }
            HttpResponse::Ok().body("Todo item removed successfully")
        },
        Err(e) => HttpResponse::BadRequest().body(e),
    }
}

// API endpoint to generate a new sample config
pub async fn generate_sample_config_handler() -> impl Responder {
    match generate_sample_config(BLOCK_CONFIG_FILE) {
        Ok(_) => HttpResponse::Ok().body("Sample config generated successfully"),
        Err(e) => HttpResponse::InternalServerError().body(format!("Failed to generate sample config: {}", e)),
    }
}

// API endpoint for auto-complete suggestions
pub async fn auto_complete_handler(description: web::Json<String>, data: web::Data<AppState>) -> impl Responder {
    let description = description.into_inner();

    // Get the project configuration to get the LLM provider setting
    let project_config = match data.project_manager.get_config() {
        Ok(config) => config,
        Err(e) => {
            println!("Failed to get project config: {}", e);
            return HttpResponse::InternalServerError().body(format!("Failed to get project config: {}", e));
        }
    };

    match auto_complete_description(
        &description, 
        project_config.llm_provider
    ).await {
        Ok(enhanced_description) => {
            let response = AutoCompleteResponse {
                suggestion: enhanced_description,
            };
            HttpResponse::Ok().json(response)
        },
        Err(e) => {
            println!("Failed to generate auto-complete suggestion: {}", e);
            HttpResponse::InternalServerError().body(format!("Failed to generate auto-complete suggestion: {}", e))
        }
    }
}

// API endpoint to process a markdown file and generate tasks
pub async fn process_markdown_handler(request: web::Json<ProcessMarkdownRequest>, data: web::Data<AppState>) -> impl Responder {
    let request = request.into_inner();

    // Find the block to update
    let mut blocks = match data.block_manager.get_blocks() {
        Ok(blocks) => blocks,
        Err(e) => return HttpResponse::InternalServerError().body(e),
    };

    let block_index = blocks.iter().position(|b| b.block_id == request.block_id);
    if block_index.is_none() {
        return HttpResponse::BadRequest().body(format!("Block '{}' not found", request.block_id));
    }

    // Get the project configuration to get the LLM provider setting
    let project_config = match data.project_manager.get_config() {
        Ok(config) => config,
        Err(e) => {
            println!("Failed to get project config: {}", e);
            return HttpResponse::InternalServerError().body(format!("Failed to get project config: {}", e));
        }
    };

    // Process the markdown file and generate tasks
    match generate_tasks(
        &request.markdown_content, 
        project_config.llm_provider
    ).await {
        Ok(tasks) => {
            // Add the generated tasks to the block's todo list
            let block = &mut blocks[block_index.unwrap()];
            for task in &tasks {
                let task = crate::models::Task::new(task.description.clone());
                let task_id = task.task_id.clone();
                block.todo_list.insert(task_id,task);
            }

            // Update the block in the database
            match data.block_manager.update_block(block.clone()) {
                Ok(_) => {
                    // Save the updated blocks to the file
                    if let Err(e) = data.block_manager.save_blocks_to_file() {
                        return HttpResponse::InternalServerError().body(e);
                    }

                    // Return the response with the generated tasks
                    let response = ProcessMarkdownResponse {
                        status: "success".to_string(),
                        message: format!("Successfully processed markdown file and added {} tasks to block '{}'", tasks.len(), request.block_id)
                    };
                    HttpResponse::Ok().json(response)
                },
                Err(e) => HttpResponse::BadRequest().body(e),
            }
        },
        Err(e) => {
            HttpResponse::InternalServerError().body(format!("Failed to process markdown file: {}", e))
        }
    }
}

// API endpoint to process a specification and generate blocks
pub async fn process_specification_handler(request: web::Json<ProcessSpecRequest>, data: web::Data<AppState>) -> impl Responder {
    let request = request.into_inner();
    
    // Get the project configuration to get the LLM provider setting
    let project_config = match data.project_manager.get_config() {
        Ok(config) => config,
        Err(e) => {
            error!("Failed to get project config: {}", e);
            return HttpResponse::InternalServerError().body(format!("Failed to get project config: {}", e));
        }
    };

    // Process the specification and generate blocks
    match process_specification(
        &request.markdown_content, 
        project_config.llm_provider
    ).await {
        Ok(generated_blocks) => {
            match data.project_manager.get_config().unwrap().llm_provider.unwrap() {
                LLMProvider::ClaudeCode => {
                    let response = ProcessSpecResponse {
                        status: "success".to_string(),
                        message: "Successfully processed specification and created blocks using mcp.".to_string(),
                        blocks: Vec::new(),
                    };
                    HttpResponse::Ok().json(response)
                }
                _ => {
                    match create_blocks_from_llm(generated_blocks, data)  {
                        Ok(response) => {
                            HttpResponse::Ok().json(response)
                        },
                        Err(e) => {
                            error!("Failed to process specification: {}", e);
                            HttpResponse::InternalServerError().body(format!("Failed to process specification: {}", e))
                        }
                    }
                }
            }

        },
        Err(e) => {
            HttpResponse::InternalServerError().body(format!("Failed to process specification: {}", e))
        }
    }
}

pub fn create_blocks_from_llm(generated_blocks: Vec<GeneratedBlock>, data: web::Data<AppState>) -> Result<(ProcessSpecResponse), String>  {
    let mut created_blocks = Vec::new();

    // Create blocks from the generated blocks
    for generated_block in generated_blocks {

        // Store the name for error reporting
        let block_id = generated_block.block_id.clone();
        let block_name = generated_block.name.clone();
        info!("Generated block {}: {}",block_id, block_name);

        // Create a new Block from the GeneratedBlock
        let block = Block::new(
            block_name.clone(),
            generated_block.description,
            generated_block.inputs,
            generated_block.outputs
        );

        // Add the block to the database
        match data.block_manager.add_block(block.clone()) {
            Ok(_) => {
                created_blocks.push(block);
            },
            Err(e) => {
                info!("Error Failed to add block ");
                return Err(format!("Failed to add block '{}': {}", block_name, e));
            }
        }
    }

    // Save the updated blocks to the file
    if let Err(e) = data.block_manager.save_blocks_to_file() {
       return Err(e);
    }

    // Return the response with the created blocks
    let response = ProcessSpecResponse {
        status: "success".to_string(),
        message: format!("Successfully processed specification and created {} blocks", created_blocks.len()),
        blocks: created_blocks,
    };
    Ok(response)
}

