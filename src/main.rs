use actix_web::{web, App, HttpServer, Responder, HttpResponse};
use actix_files as fs;

// Import models from the models module
mod models;
use models::get_blocks;

// API endpoint to get blocks
async fn get_blocks_handler() -> impl Responder {
    let blocks = get_blocks();
    HttpResponse::Ok().json(blocks)
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
            .service(web::resource("/api/blocks").route(web::get().to(get_blocks_handler)))

            // Serve static files from the frontend/dist directory
            .service(fs::Files::new("/assets", "./frontend/dist/assets"))

            // Serve the index.html for all other routes
            .default_service(web::get().to(index))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
