use actix_files as fs;
use actix_web::{web, App, HttpServer, Responder};
use clap::{Arg, Command};
use dotenv::dotenv;
use std::sync::Arc;
use std::thread;
use tracing::{debug, error, info, warn};
use tracing_subscriber::{self, fmt, prelude::*, EnvFilter};
use tracing_appender::{self, rolling};

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

mod mcp;
use crate::block_handlers::{generate_tasks_block_handler, process_spec_handler};
use crate::git_handlers::pull_handler;
use block_config::{generate_sample_config, BlockConfigManager, DEFAULT_BLOCK_CONFIG_FILE};
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

use crate::log_stream::{get_task_ids, stream_logs};
use crate::mcp::{server::MCPServerConfig, MCPServer};
use crate::mcp::transport::TransportFactory;
use crate::task_executor_wrapper::initialize as init_task_executor;

// Initialize the logger with file output
fn init_logger(mode: &str) {
    // Create a directory for logs if it doesn't exist
    std::fs::create_dir_all("logs").unwrap_or_else(|e| {
        eprintln!("Warning: Failed to create logs directory: {}", e);
    });

    // Set up rolling file appender - creates a new log file each day
    let file_appender = rolling::daily("logs", format!("forge-{}", mode));
    let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);

    // Store the guard in a static variable to keep it alive for the duration of the program
    // This is important to ensure logs are properly flushed
    lazy_static::lazy_static! {
        static ref APPENDER_GUARD: std::sync::Mutex<Option<tracing_appender::non_blocking::WorkerGuard>> = std::sync::Mutex::new(None);
    }
    *APPENDER_GUARD.lock().unwrap() = Some(_guard);

    // Initialize the subscriber with both console and file outputs
    tracing_subscriber::registry()
        .with(fmt::layer().with_writer(std::io::stdout))
        .with(fmt::layer().with_writer(non_blocking))
        .with(EnvFilter::from_default_env().add_directive(tracing::Level::INFO.into()))
        .init();

    info!("Logger initialized in {} mode", mode);
}

// Index handler to serve the frontend
async fn index() -> impl Responder {
    fs::NamedFile::open_async("./frontend/dist/index.html").await
}

// Run MCP server in stdio mode
async fn run_mcp_server(project_manager: Arc<ProjectConfigManager> , block_manager : Arc<BlockConfigManager>) -> std::io::Result<()> {
    // Initialize tracing for MCP mode

    info!("Starting Forge MCP Server in stdio mode...");

    // Load project config to get working directory
    let project_config = project_manager.load_config().unwrap_or_default();

    // Create MCP server configuration
    let mcp_config = MCPServerConfig {
        working_directory: if !project_config.project_home_directory.is_empty() {
            std::path::PathBuf::from(&project_config.project_home_directory)
                .canonicalize()
                .unwrap_or_else(|_| std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("/")))
        } else {
            std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("/"))
        },
        max_sessions: 5, // Lower for stdio mode
        session_timeout: std::time::Duration::from_secs(3600), // 1 hour
        max_concurrent_tools: 4,
        tool_timeout: std::time::Duration::from_secs(300), // 5 minutes
        enable_monitoring: false, // Disable monitoring in stdio mode
        enable_cleanup: true,
        ..Default::default()
    };

    // Create MCP server
    let mcp_server = match MCPServer::new(mcp_config, project_manager, block_manager).await {
        Ok(server) => server,
        Err(e) => {
            error!("Failed to create MCP server: {}", e);
            return Err(std::io::Error::new(std::io::ErrorKind::Other, e));
        }
    };

    // Create stdio transport
    let transport = match TransportFactory::create_stdio().await {
        Ok(transport) => transport,
        Err(e) => {
            error!("Failed to create stdio transport: {}", e);
            return Err(std::io::Error::new(std::io::ErrorKind::Other, e));
        }
    };

    info!("MCP Server ready, handling stdio connection...");

    // Handle the stdio connection
    if let Err(e) = mcp_server.handle_connection(transport, "stdio".to_string()).await {
        error!("MCP Server connection error: {}", e);
        return Err(std::io::Error::new(std::io::ErrorKind::Other, e));
    }

    info!("MCP Server connection closed");
    Ok(())
}


#[tokio::main]
async fn main() -> std::io::Result<()> {
    // Parse command line arguments
    let matches = Command::new("forge")
        .version("0.1.0")
        .about("Forge - Project Management and MCP Server")
        .arg(
            Arg::new("mcp")
                .long("mcp")
                .help("Run in MCP server mode (stdio transport)")
                .action(clap::ArgAction::SetTrue)
        )
        .get_matches();

    // Load environment variables from .env file
    dotenv().ok();

    init_logger("mcp");

    // Get the singleton instance of ProjectConfigManager
    let project_manager = ProjectConfigManager::get_instance();

    // Load project configuration
    let project_config = match project_manager.load_config() {
        Ok(config) => {
            info!("Project configuration loaded successfully from {}", PROJECT_CONFIG_FILE);
            config
        },
        Err(e) => {
            warn!("Failed to load project configuration from {}: {}", PROJECT_CONFIG_FILE, e);
            info!("A default configuration will be created when saved for the first time.");
            project_config::ProjectConfig::default()
        }
    };

    // Determine the blocks config file path based on project home directory
    let blocks_config_path = if !project_config.project_home_directory.is_empty() {
        let project_dir = std::path::Path::new(&project_config.project_home_directory);
        if project_dir.exists() {
            let blocks_config_path = project_dir.join(BLOCK_CONFIG_FILE);
            info!("Using blocks config path: {}", blocks_config_path.display());
            blocks_config_path.to_string_lossy().to_string()
        } else {
            warn!("Project home directory does not exist, using default blocks config path");
            BLOCK_CONFIG_FILE.to_string()
        }
    } else {
        info!("Project home directory not set, using default blocks config path");
        BLOCK_CONFIG_FILE.to_string()
    };

    // Create a BlockConfigManager instance with the specific config file path
    let block_manager = Arc::new(BlockConfigManager::new(&blocks_config_path));

    // Initialize the task executor
    info!("Initializing task executor");
    let _task_executor = init_task_executor(project_manager.clone(), block_manager.clone());

    // // Initialize ClaudePty singleton
    // info!("Initializing ClaudePty singleton");
    // if let Err(e) = ClaudePty::initialize() {
    //     warn!("Failed to initialize ClaudePty: {}", e);
    // } else {
    //     info!("ClaudePty initialized successfully");
    // }

    // Load blocks from the config file
    match block_manager.load_blocks_from_file() {
        Ok(_) => info!("Blocks loaded successfully from {}", blocks_config_path),
        Err(e) => {
            warn!("Failed to load blocks from {}: {}", blocks_config_path, e);
            info!("Generating a sample config file...");
            if let Err(e) = generate_sample_config(&blocks_config_path) {
                error!("Failed to generate sample config: {}", e);
            } else {
                info!("Sample config generated successfully");
                // Try loading again
                if let Err(e) = block_manager.load_blocks_from_file() {
                    error!("Failed to load blocks from the generated config: {}", e);
                } else {
                    info!("Blocks loaded successfully from the generated config");
                }
            }
        }
    }

    let num_blocks = block_manager.get_blocks().unwrap_or_default().len();
    info!(">> Num blocks (init): {}",num_blocks);


    // Run HTTP server in a new thread
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

    // Create a thread for the MCP server if the flag is set
    if matches.get_flag("mcp") {
        run_mcp_server(
            project_manager,
            block_manager).await
    } else {
        // Run the HTTP server in the main thread
        info!("Starting HTTP server on 127.0.0.1:8080");
       run_http_server(
            app_state,
            project_app_state,
            git_app_state,
        ).await
    }
}

async fn run_http_server(
    app_state: web::Data<AppState>,
    project_app_state: web::Data<ProjectAppState>,
    git_app_state: web::Data<GitAppState>
) -> std::io::Result<()> {
    HttpServer::new(move || {
        App::new()
            .app_data(app_state.clone())
            .app_data(project_app_state.clone())
            .app_data(git_app_state.clone())
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
