use actix_web::{web, App, HttpServer, Responder, HttpResponse};
use actix_files as fs;
use serde::{Serialize, Deserialize};
use std::collections::HashMap;

// Define the structure for module connections
#[derive(Serialize, Deserialize, Clone)]
struct InputConnection {
    from_module: String,
    output_type: String,
    unique_id: String,
}

#[derive(Serialize, Deserialize, Clone)]
struct OutputConnection {
    to_module: String,
    input_type: String,
    unique_id: String,
}

#[derive(Serialize, Deserialize, Clone)]
struct Connections {
    input_connections: Vec<InputConnection>,
    output_connections: Vec<OutputConnection>,
}

// Define the structure for a software module
#[derive(Serialize, Deserialize, Clone)]
struct Module {
    name: String,
    description: String,
    inputs: Vec<String>,
    outputs: Vec<String>,
    connections: Connections,
    todo_list: Vec<String>,
}

// Function to get a list of modules (in a real application, this would likely fetch from a database)
fn get_modules() -> Vec<Module> {
    vec![
        Module {
            name: "DataIngestion".to_string(),
            description: "Handles the ingestion of raw data from various sources".to_string(),
            inputs: vec!["RawData".to_string()],
            outputs: vec!["ParsedData".to_string()],
            connections: Connections {
                input_connections: vec![],
                output_connections: vec![
                    OutputConnection {
                        to_module: "DataProcessing".to_string(),
                        input_type: "ParsedData".to_string(),
                        unique_id: "conn-1".to_string(),
                    }
                ],
            },
            todo_list: vec![
                "Add support for CSV files".to_string(),
                "Improve error handling".to_string(),
            ],
        },
        Module {
            name: "DataProcessing".to_string(),
            description: "Processes and transforms the parsed data".to_string(),
            inputs: vec!["ParsedData".to_string()],
            outputs: vec!["ProcessedData".to_string()],
            connections: Connections {
                input_connections: vec![
                    InputConnection {
                        from_module: "DataIngestion".to_string(),
                        output_type: "ParsedData".to_string(),
                        unique_id: "conn-1".to_string(),
                    }
                ],
                output_connections: vec![
                    OutputConnection {
                        to_module: "DataVisualization".to_string(),
                        input_type: "ProcessedData".to_string(),
                        unique_id: "conn-2".to_string(),
                    }
                ],
            },
            todo_list: vec![
                "Implement data normalization".to_string(),
                "Add support for filtering".to_string(),
            ],
        },
        Module {
            name: "DataVisualization".to_string(),
            description: "Visualizes the processed data".to_string(),
            inputs: vec!["ProcessedData".to_string()],
            outputs: vec!["Visualization".to_string()],
            connections: Connections {
                input_connections: vec![
                    InputConnection {
                        from_module: "DataProcessing".to_string(),
                        output_type: "ProcessedData".to_string(),
                        unique_id: "conn-2".to_string(),
                    }
                ],
                output_connections: vec![],
            },
            todo_list: vec![
                "Add more chart types".to_string(),
                "Implement interactive visualizations".to_string(),
            ],
        },
    ]
}

// API endpoint to get modules
async fn get_modules_handler() -> impl Responder {
    let modules = get_modules();
    HttpResponse::Ok().json(modules)
}

// Index handler to serve the frontend
async fn index() -> impl Responder {
    fs::NamedFile::open_async("./frontend/dist/index.html").await
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    println!("Starting server at http://127.0.0.1:8080");

    HttpServer::new(|| {
        App::new()
            // API routes
            .service(web::resource("/api/modules").route(web::get().to(get_modules_handler)))

            // Serve static files from the frontend/dist directory
            .service(fs::Files::new("/assets", "./frontend/dist/assets"))

            // Serve the index.html for all other routes
            .default_service(web::get().to(index))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
