use actix_web::{web, App, HttpServer, Responder, Error};
use actix_files as fs;
use std::sync::Arc;
use dotenv::dotenv;

// Import models from the models module
mod models;
mod block_config;
mod block_handlers;
mod llm_handler;
mod project_config;
mod project_handlers;
use block_config::{BlockConfigManager, load_blocks_from_file, generate_sample_config};
use block_handlers::{
    AppState, CONFIG_FILE, get_blocks_handler, add_block_handler, update_block_handler,
    delete_block_handler, add_todo_handler, remove_todo_handler, generate_sample_config_handler
};
use project_config::{ProjectConfigManager, PROJECT_CONFIG_FILE};
use project_handlers::{
    ProjectAppState, get_project_config_handler, update_project_config_handler, test_git_connection_handler
};

// Index handler to serve the frontend
async fn index() -> impl Responder {
    fs::NamedFile::open_async("./frontend/dist/index.html").await
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Load environment variables from .env file
    dotenv().ok();

    println!("Starting server at http://127.0.0.1:8080");

    // Create a BlockConfigManager instance
    let block_manager = Arc::new(BlockConfigManager::new(CONFIG_FILE));

    // Load blocks from the config file
    match block_manager.load_blocks_from_file() {
        Ok(_) => println!("Blocks loaded successfully from {}", CONFIG_FILE),
        Err(e) => {
            println!("Failed to load blocks from {}: {}", CONFIG_FILE, e);
            println!("Generating a sample config file...");
            if let Err(e) = generate_sample_config(CONFIG_FILE) {
                println!("Failed to generate sample config: {}", e);
            } else {
                println!("Sample config generated successfully");
                // Try loading again
                if let Err(e) = block_manager.load_blocks_from_file() {
                    println!("Failed to load blocks from the generated config: {}", e);
                } else {
                    println!("Blocks loaded successfully from the generated config");
                }
            }
        }
    }

    // Create a ProjectConfigManager instance
    let project_manager = Arc::new(ProjectConfigManager::new(PROJECT_CONFIG_FILE));

    // Load project configuration
    match project_manager.load_config() {
        Ok(_) => println!("Project configuration loaded successfully from {}", PROJECT_CONFIG_FILE),
        Err(e) => {
            println!("Failed to load project configuration from {}: {}", PROJECT_CONFIG_FILE, e);
            println!("A default configuration will be created when saved for the first time.");
        }
    }

    // Create the app states
    let block_app_state = web::Data::new(AppState {
        block_manager: block_manager.clone(),
    });

    let project_app_state = web::Data::new(ProjectAppState {
        project_manager: project_manager.clone(),
    });

    HttpServer::new(move || {
        App::new()
            .app_data(block_app_state.clone())
            .app_data(project_app_state.clone())
            // API routes
            .service(
                web::scope("/api")
                    // Block routes
                    .route("/blocks", web::get().to(get_blocks_handler))
                    .route("/blocks", web::post().to(add_block_handler))
                    .route("/blocks", web::put().to(update_block_handler))
                    .route("/blocks/{name}", web::delete().to(delete_block_handler))
                    .route("/blocks/{name}/todo", web::post().to(add_todo_handler))
                    .route("/blocks/{name}/todo/{index}", web::delete().to(remove_todo_handler))
                    .route("/generate-sample", web::post().to(generate_sample_config_handler))
                    // Project routes
                    .route("/project", web::get().to(get_project_config_handler))
                    .route("/project", web::put().to(update_project_config_handler))
                    .route("/project/test-git-connection", web::post().to(test_git_connection_handler))
            )

            // Serve static files from the frontend/dist directory
            .service(fs::Files::new("/assets", "./frontend/dist/assets"))

            // Serve the index.html for all other routes
            .default_service(web::get().to(index))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
