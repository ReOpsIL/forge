use actix_web::{web, Responder, HttpResponse};
use std::sync::Arc;

use crate::block_config::{BlockConfigManager, generate_sample_config};
use crate::models::Block;

// Define the config file path
pub const CONFIG_FILE: &str = "blocks_config.json";

// Create a data structure to hold the BlockConfigManager
pub struct AppState {
    pub block_manager: Arc<BlockConfigManager>,
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

// API endpoint to update an existing block
pub async fn update_block_handler(block: web::Json<Block>, data: web::Data<AppState>) -> impl Responder {
    match data.block_manager.update_block(block.into_inner()) {
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
    let block_name = path.into_inner();
    match data.block_manager.delete_block(&block_name) {
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
    let block_name = path.into_inner();
    match data.block_manager.add_todo_item(&block_name, &todo.into_inner()) {
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
    let (block_name, todo_index) = path.into_inner();
    match data.block_manager.remove_todo_item(&block_name, todo_index) {
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
    match generate_sample_config(CONFIG_FILE) {
        Ok(_) => HttpResponse::Ok().body("Sample config generated successfully"),
        Err(e) => HttpResponse::InternalServerError().body(format!("Failed to generate sample config: {}", e)),
    }
}