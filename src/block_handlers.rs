use actix_web::{web, Responder, HttpResponse};
use std::sync::Arc;
use serde::{Serialize, Deserialize};
use std::process::{Command, Stdio};
use std::io::Write;
use std::path::Path;
use tokio::task;

use crate::block_config::{BlockConfigManager, generate_sample_config};
use crate::models::Block;
use crate::llm_handler::{auto_complete_description, enhance_description, generate_tasks, process_markdown_file, process_markdown_spec, GeneratedBlock};
use crate::project_config::ProjectConfigManager;

// Define a response type for auto-complete suggestions
#[derive(Serialize)]
pub struct AutoCompleteResponse {
    pub suggestion: String,
}

// Define request and response types for task execution
#[derive(Deserialize)]
pub struct ExecuteTaskRequest {
    pub block_name: String,
    pub task_index: usize,
    pub task_description: String,
}

#[derive(Serialize)]
pub struct ExecuteTaskResponse {
    pub status: String,
    pub message: String,
}

// Define request and response types for markdown file processing
#[derive(Deserialize)]
pub struct ProcessMarkdownRequest {
    pub block_name: String,
    pub markdown_content: String,
}

#[derive(Serialize)]
pub struct ProcessMarkdownResponse {
    pub status: String,
    pub message: String,
    pub tasks: Vec<String>,
}

// Define request and response types for markdown specification processing
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
async fn enhance_block_with_llm(mut block: Block) -> Result<Block, String> {
    // Enhance the description using LLM
    let enhanced_description = enhance_description(&block.description).await?;

    // Update the block with the enhanced description
    block.description = enhanced_description;

    Ok(block)
}

async fn generate_tasks_with_llm(mut block: Block) -> Result<Block, String> {

    // Generate tasks based on the enhanced description
    let generated_tasks = generate_tasks(&block.description).await?;

    // Add the generated tasks to the block's todo list
    for task_description in generated_tasks {
        let task = crate::models::Task::new(task_description);
        block.todo_list.push(task);
    }

    Ok(block)
}


pub async fn enhance_block_handler(block: web::Json<Block>, data: web::Data<AppState>) -> impl Responder {
    let mut block = block.into_inner();

    match enhance_block_with_llm(block.clone()).await {
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

    match generate_tasks_with_llm(block.clone()).await {
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

// API endpoint to add a todo item to a block
pub async fn add_todo_handler(path: web::Path<String>, todo: web::Json<String>, data: web::Data<AppState>) -> impl Responder {
    let block_id = path.into_inner();
    match data.block_manager.add_todo_item(&block_id, &todo.into_inner()) {
        Ok(_) => {
            // Save the updated blocks to the file
            if let Err(e) = data.block_manager.save_blocks_to_file() {
                return HttpResponse::InternalServerError().body(e);
            }
            HttpResponse::Ok().body("Todo item added successfully")
        },
        Err(e) => HttpResponse::BadRequest().body(e),
    }
}

// API endpoint to remove a todo item from a block
pub async fn remove_todo_handler(path: web::Path<(String, usize)>, data: web::Data<AppState>) -> impl Responder {
    let (block_id, todo_index) = path.into_inner();
    match data.block_manager.remove_todo_item(&block_id, todo_index) {
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
pub async fn auto_complete_handler(description: web::Json<String>) -> impl Responder {
    let description = description.into_inner();

    match auto_complete_description(&description).await {
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

    let block_index = blocks.iter().position(|b| b.name == request.block_name);
    if block_index.is_none() {
        return HttpResponse::BadRequest().body(format!("Block '{}' not found", request.block_name));
    }

    // Process the markdown file and generate tasks
    match process_markdown_file(&request.markdown_content).await {
        Ok(tasks) => {
            // Add the generated tasks to the block's todo list
            let block = &mut blocks[block_index.unwrap()];
            for task_description in &tasks {
                let task = crate::models::Task::new(task_description.clone());
                block.todo_list.push(task);
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
                        message: format!("Successfully processed markdown file and added {} tasks to block '{}'", tasks.len(), request.block_name),
                        tasks,
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

// API endpoint to process a markdown specification and generate blocks
pub async fn process_spec_handler(request: web::Json<ProcessSpecRequest>, data: web::Data<AppState>) -> impl Responder {
    let request = request.into_inner();

    // Process the markdown specification and generate blocks
    match process_markdown_spec(&request.markdown_content).await {
        Ok(generated_blocks) => {
            let mut created_blocks = Vec::new();

            // Create blocks from the generated blocks
            for generated_block in generated_blocks {
                // Store the name for error reporting
                let block_name = generated_block.name.clone();

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
                        return HttpResponse::BadRequest().body(format!("Failed to add block '{}': {}", block_name, e));
                    }
                }
            }

            // Save the updated blocks to the file
            if let Err(e) = data.block_manager.save_blocks_to_file() {
                return HttpResponse::InternalServerError().body(e);
            }

            // Return the response with the created blocks
            let response = ProcessSpecResponse {
                status: "success".to_string(),
                message: format!("Successfully processed markdown specification and created {} blocks", created_blocks.len()),
                blocks: created_blocks,
            };
            HttpResponse::Ok().json(response)
        },
        Err(e) => {
            HttpResponse::InternalServerError().body(format!("Failed to process markdown specification: {}", e))
        }
    }
}


// API endpoint to execute a task using Claude CLI
pub async fn execute_task_handler(
    request: web::Json<ExecuteTaskRequest>,
    data: web::Data<AppState>
) -> impl Responder {
    let request = request.into_inner();

    // Get the project home directory from the project config
    let project_config = match data.project_manager.get_config() {
        Ok(config) => config,
        Err(e) => {
            return HttpResponse::InternalServerError().body(
                format!("Failed to get project configuration: {}", e)
            );
        }
    };

    let project_dir = project_config.project_home_directory.clone();
    if project_dir.is_empty() {
        return HttpResponse::BadRequest().body(
            "Project home directory is not set. Please configure it in the project settings."
        );
    }

    // Check if the project directory exists
    if !Path::new(&project_dir).exists() {
        return HttpResponse::BadRequest().body(
            format!("Project home directory does not exist: {}", project_dir)
        );
    }

    // Get the task description
    let task_description = request.task_description.clone();
    if task_description.is_empty() {
        return HttpResponse::BadRequest().body("Task description cannot be empty");
    }

    // Clone the data for use in the background task
    let block_manager = data.block_manager.clone();
    let block_name = request.block_name.clone();
    let task_index = request.task_index;

    // Spawn a background task to execute the Claude CLI command
    task::spawn(async move {
        // Execute the Claude CLI command with --allowedTools option
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
                return;
            }
        };

        // Write the task description to the command's stdin
        if let Some(mut stdin) = child.stdin.take() {
            if let Err(e) = stdin.write_all(task_description.as_bytes()) {
                println!("Failed to write to Claude CLI stdin: {}", e);
                return;
            }
        }

        // Wait for the command to complete
        match child.wait_with_output() {
            Ok(output) => {
                if output.status.success() {
                    println!("Claude CLI command completed successfully");
                    println!("Claude output:\n-----------------\n{}", String::from_utf8_lossy(&output.stdout));
                } else {
                    println!("Claude CLI command failed with exit code: {:?}", output.status.code());
                    println!("Claude stderr:\n-----------------\n{}", String::from_utf8_lossy(&output.stderr));
                }

                // Update the task status and log in the block config
                if let Ok(mut blocks) = block_manager.get_blocks() {
                    if let Some(block) = blocks.iter_mut().find(|b| b.name == block_name) {
                        if let Some(task) = block.todo_list.get_mut(task_index) {
                            // Append a completion marker to the task description based on success/failure
                            let status_marker = if output.status.success() {
                                "[COMPLETED]"
                            } else {
                                "[FAILED]"
                            };
                            task.description = format!("{} {}", task.description, status_marker);

                            // Store the stdout output in the task's log field
                            let log_output = String::from_utf8_lossy(&output.stdout).to_string();
                            task.log = Some(log_output);

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
            },
            Err(e) => {
                println!("Failed to wait for Claude CLI command: {}", e);
            }
        }
    });

    // Return a response indicating the task has been started
    let response = ExecuteTaskResponse {
        status: "started".to_string(),
        message: "Task execution has been started in the background with tools enabled".to_string(),
    };

    HttpResponse::Ok().json(response)
}
