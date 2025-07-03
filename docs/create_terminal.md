Add new component for running "claude" as process and stream the output to frontend new page "Claude" , refer to the following description as example and general guideline for implementing the terminal frontend and backend functionality,
make sure the integration fits well in current backend, and frontend: // src/main.rs
use axum::{
extract::{
ws::{Message, WebSocket},
WebSocketUpgrade,
},
response::Response,
routing::get,
Router,
};
use futures::{SinkExt, StreamExt};
use portable_pty::{CommandBuilder, NativePtySystem, PtySize, PtySystem};
use std::io::{Read, Write};
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc;
use tokio::task;

#[tokio::main]
async fn main() {
let app = Router::new().route("/ws", get(ws_handler));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080")
        .await
        .unwrap();

    println!("Listening on ws://0.0.0.0:8080");

    axum::serve(listener, app).await.unwrap();

}

async fn ws_handler(ws: WebSocketUpgrade) -> Response {
ws.on_upgrade(handle_socket)
}

async fn handle_socket(mut socket: WebSocket) {
// Setup the PTY
let pty_system = NativePtySystem::default();
let pair = pty_system
.openpty(PtySize {
rows: 24,
cols: 80,
pixel_width: 0,
pixel_height: 0,
})
.expect("Failed to open PTY");

    // Spawn the shell command
    let cmd = CommandBuilder::new("bash"); // You can use "cmd.exe" on Windows
    let mut child = pair.slave.spawn_command(cmd).expect("Failed to spawn command");

    // The PTY master is the process's "virtual terminal"
    // We need to read from it and write to it.
    let master = pair.master;
    let reader = master.try_clone_reader().expect("Failed to clone reader");
    let writer = master.try_clone_writer().expect("Failed to clone writer");

    // Wrap the blocking reader/writer in Arc<Mutex> for safe sharing across async tasks
    let reader = Arc::new(Mutex::new(reader));
    let writer = Arc::new(Mutex::new(writer));

    // --- Task 1: Read from PTY and send to WebSocket ---
    let (tx, mut rx) = mpsc::unbounded_channel::<String>();
    let reader_clone = Arc::clone(&reader);

    task::spawn_blocking(move || {
        let mut buf = [0u8; 8192];
        loop {
            match reader_clone.lock().unwrap().read(&mut buf) {
                Ok(0) => break, // EOF
                Ok(n) => {
                    if let Ok(s) = String::from_utf8(buf[..n].to_vec()) {
                        tx.send(s).unwrap();
                    }
                }
                Err(_) => break,
            }
        }
    });

    let mut send_task = tokio::spawn(async move {
        while let Some(s) = rx.recv().await {
            if socket.send(Message::Text(s)).await.is_err() {
                break;
            }
        }
    });

    // --- Task 2: Read from WebSocket and send to PTY ---
    let writer_clone = Arc::clone(&writer);
    let mut recv_task = tokio::spawn(async move {
        while let Some(Ok(Message::Text(text))) = socket.next().await {
            task::spawn_blocking(move || {
                writer_clone.lock().unwrap().write_all(text.as_bytes()).unwrap();
            });
        }
    });
    
    // Wait for either task to finish
    tokio::select! {
        _ = (&mut send_task) => recv_task.abort(),
        _ = (&mut recv_task) => send_task.abort(),
    };

    // Clean up the child process
    let _ = child.kill();
    println!("WebSocket connection closed and child process killed.");

}