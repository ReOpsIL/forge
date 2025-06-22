use actix_files as fs;
use actix_web::{web, App, HttpServer, Responder};
use dotenv::dotenv;
use std::sync::Arc;

// Import models from the models module
mod models;
mod block_config;
mod block_handlers;
mod llm_handler;
mod profession_prompts;
mod project_config;
mod project_handlers;
mod git_handlers;
pub mod task_executor;
mod task_executor_wrapper;
mod task_queue;
mod log_stream;
mod chat_handlers;
mod claude_mcp_server;
mod mcp;
use crate::block_handlers::{generate_tasks_block_handler, process_spec_handler};
use crate::git_handlers::pull_handler;
use block_config::{generate_sample_config, BlockConfigManager};
use block_handlers::{
    add_block_handler, add_task_handler, auto_complete_handler, delete_block_handler, enhance_block_handler,
    generate_sample_config_handler, get_block_dependencies_handler, get_blocks_handler, process_markdown_handler, remove_task_handler,
    update_block_handler, AppState, BLOCK_CONFIG_FILE
};
use git_handlers::{
    build_handler, commit_handler, create_branch_handler, execute_git_task_handler, get_branches_handler, get_task_diff_handler,
    merge_branch_handler, push_handler, GitAppState
};
use project_config::{ProjectConfigManager, PROJECT_CONFIG_FILE};
use project_handlers::{
    check_project_config_handler, get_profession_prompts_handler, get_professions_handler, get_project_config_handler,
    test_git_connection_handler, update_project_config_handler, ProjectAppState
};

use crate::chat_handlers::{chat_websocket, ChatAppState};
use crate::claude_mcp_server::{claude_chat_handler, claude_models_handler, ClaudeMCPAppState};
use crate::log_stream::{get_task_ids, stream_logs};
use crate::mcp::{server::MCPServerConfig, MCPServer};
use crate::task_executor_wrapper::initialize as init_task_executor;

// Index handler to serve the frontend
async fn index() -> impl Responder {
    fs::NamedFile::open_async("./frontend/dist/index.html").await
}


#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Load environment variables from .env file
    dotenv().ok();

    println!("Starting server at http://127.0.0.1:8080");

    // Create a ProjectConfigManager instance
    let project_manager = Arc::new(ProjectConfigManager::new(PROJECT_CONFIG_FILE));

    // Load project configuration
    let project_config = match project_manager.load_config() {
        Ok(config) => {
            println!("Project configuration loaded successfully from {}", PROJECT_CONFIG_FILE);
            config
        },
        Err(e) => {
            println!("Failed to load project configuration from {}: {}", PROJECT_CONFIG_FILE, e);
            println!("A default configuration will be created when saved for the first time.");
            project_config::ProjectConfig::default()
        }
    };

    // Determine the blocks config file path based on project home directory
    let blocks_config_path = if !project_config.project_home_directory.is_empty() {
        let project_dir = std::path::Path::new(&project_config.project_home_directory);
        if project_dir.exists() {
            let blocks_config_path = project_dir.join(BLOCK_CONFIG_FILE);
            println!("Using blocks config path: {}", blocks_config_path.display());
            blocks_config_path.to_string_lossy().to_string()
        } else {
            println!("Project home directory does not exist, using default blocks config path");
            BLOCK_CONFIG_FILE.to_string()
        }
    } else {
        println!("Project home directory not set, using default blocks config path");
        BLOCK_CONFIG_FILE.to_string()
    };

    // Create a BlockConfigManager instance
    let block_manager = Arc::new(BlockConfigManager::new(&blocks_config_path));

    // Initialize the task executor
    println!("Initializing task executor");
    let _task_executor = init_task_executor(project_manager.clone(), block_manager.clone());

    // Load blocks from the config file
    match block_manager.load_blocks_from_file() {
        Ok(_) => println!("Blocks loaded successfully from {}", blocks_config_path),
        Err(e) => {
            println!("Failed to load blocks from {}: {}", blocks_config_path, e);
            println!("Generating a sample config file...");
            if let Err(e) = generate_sample_config(&blocks_config_path) {
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

    // Create the app states
    let app_state = web::Data::new(AppState {
        block_manager: block_manager.clone(),
        project_manager: project_manager.clone(),
    });

    let project_app_state = web::Data::new(ProjectAppState {
        project_manager: project_manager.clone(),
    });

    let git_app_state = web::Data::new(GitAppState {
        project_manager: project_manager.clone(),
        block_manager: block_manager.clone(),
    });

    let chat_app_state = web::Data::new(ChatAppState::new());

    let claude_mcp_app_state = web::Data::new(ClaudeMCPAppState::new(project_manager.clone()));

    // Initialize MCP Server
    println!("Initializing MCP Server...");
    let mcp_config = MCPServerConfig {
        working_directory: std::path::PathBuf::from(&project_config.project_home_directory)
            .canonicalize()
            .unwrap_or_else(|_| std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("/"))),
        max_sessions: 25,
        session_timeout: std::time::Duration::from_secs(7200), // 2 hours
        max_concurrent_tools: 8,
        tool_timeout: std::time::Duration::from_secs(300), // 5 minutes
        enable_monitoring: true,
        enable_cleanup: true,
        ..Default::default()
    };
    
    match MCPServer::new(mcp_config, project_manager.clone(), block_manager.clone()).await {
        Ok(mut mcp_server) => {
            println!("MCP Server initialized successfully");
            
            // Start MCP server in background
            tokio::spawn(async move {
                if let Err(e) = mcp_server.start().await {
                    eprintln!("MCP Server error: {}", e);
                }
            });
            
            println!("MCP Server started in background");
        }
        Err(e) => {
            eprintln!("Failed to initialize MCP Server: {}", e);
            eprintln!("Continuing without MCP Server...");
        }
    }

    HttpServer::new(move || {
        App::new()
            .app_data(app_state.clone())
            .app_data(project_app_state.clone())
            .app_data(git_app_state.clone())
            .app_data(chat_app_state.clone())
            .app_data(claude_mcp_app_state.clone())
            // API routes
            .service(
                web::scope("/api")
                    // Block routes
                    .route("/blocks", web::get().to(get_blocks_handler))
                    .route("/blocks", web::post().to(add_block_handler))
                    .route("/blocks", web::put().to(update_block_handler))
                    .route("/blocks/{block_id}", web::delete().to(delete_block_handler))
                    .route("/blocks/{block_id}/task", web::post().to(add_task_handler))
                    .route("/blocks/{block_id}/delete/{task_id}", web::delete().to(remove_task_handler))
                    .route("/blocks/{block_id}/enhance", web::put().to(enhance_block_handler))
                    .route("/blocks/{block_id}/generate-tasks", web::put().to(generate_tasks_block_handler))
                    .route("/blocks/auto-complete", web::post().to(auto_complete_handler))
                    .route("/blocks/process-markdown", web::post().to(process_markdown_handler))
                    .route("/blocks/process-spec", web::post().to(process_spec_handler))
                    .route("/blocks/{blockId}/dependencies", web::get().to(get_block_dependencies_handler))
                    .route("/generate-sample", web::post().to(generate_sample_config_handler))
                    // Project routes
                    .route("/project", web::get().to(get_project_config_handler))
                    .route("/project", web::put().to(update_project_config_handler))
                    .route("/project/test-git-connection", web::post().to(test_git_connection_handler))
                    .route("/project/check-config", web::get().to(check_project_config_handler))
                    .route("/project/professions", web::get().to(get_professions_handler))
                    .route("/project/professions/{profession_id}/prompts", web::get().to(get_profession_prompts_handler))
                    // Git routes
                    .route("/git/branch", web::post().to(create_branch_handler))
                    .route("/git/commit", web::post().to(commit_handler))
                    .route("/git/merge", web::post().to(merge_branch_handler))
                    .route("/git/push", web::post().to(push_handler))
                    .route("/git/pull", web::post().to(pull_handler))
                    .route("/git/build", web::post().to(build_handler))
                    .route("/git/execute-task", web::post().to(execute_git_task_handler))
                    .route("/git/task-diff", web::post().to(get_task_diff_handler))
                    .route("/git/branches", web::get().to(get_branches_handler))
                    // Log streaming routes
                    .route("/logs/stream/{task_id}", web::get().to(stream_logs))
                    .route("/logs/tasks", web::get().to(get_task_ids))
                    // Chat routes
                    .route("/chat/ws", web::get().to(chat_websocket))
                    // Claude MCP routes
                    .route("/claude/chat", web::post().to(claude_chat_handler))
                    .route("/claude/models", web::get().to(claude_models_handler))
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
