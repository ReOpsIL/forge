use actix::{Actor, ActorContext, Addr, AsyncContext, Handler, Message, StreamHandler, WrapFuture};
use actix_web::{web, Error, HttpRequest, HttpResponse, Result};
use actix_web_actors::ws;
use serde::{Deserialize, Serialize};
use std::process::Stdio;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tokio::io::{AsyncBufReadExt, AsyncReadExt};
use tokio::process::Command;
use tokio::sync::mpsc;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    #[serde(rename = "type")]
    pub message_type: String,
    pub content: String,
}

#[derive(Debug)]
pub struct ClaudeProcess {
    // Channel for receiving prompt requests
    pub prompt_tx: mpsc::UnboundedSender<(String, Addr<ChatWebSocket>)>,
    // Store the WebSocket address
    pub ws_addr: Arc<Mutex<Option<Addr<ChatWebSocket>>>>,
}

impl ClaudeProcess {
    pub fn new() -> Self {
        // Create a channel for receiving prompt requests
        let (prompt_tx, mut prompt_rx) = mpsc::unbounded_channel::<(String, Addr<ChatWebSocket>)>();

        // Spawn a tokio thread to handle prompt requests
        tokio::spawn(async move {
            while let Some((prompt, ws_addr)) = prompt_rx.recv().await {
                // Process each prompt in a separate task
                let ws_addr_clone = ws_addr.clone();
                tokio::spawn(async move {
                    eprintln!("Processing prompt: {}", prompt);

                    // Execute claude as a batch process with the prompt
                    let result = Self::execute_claude_process(&prompt, ws_addr_clone.clone()).await;

                    if let Err(e) = result {
                        eprintln!("Error executing claude process: {}", e);
                        let error_msg = ChatMessage {
                            message_type: "error".to_string(),
                            content: format!("Error executing claude process: {}", e),
                        };
                        if let Ok(json) = serde_json::to_string(&error_msg) {
                            let _ = ws_addr_clone.try_send(WsMessage(json));
                        }
                    }
                });
            }
        });

        Self { 
            prompt_tx,
            ws_addr: Arc::new(Mutex::new(None)),
        }
    }

    // Execute the claude process as a batch with the given prompt
    async fn execute_claude_process(prompt: &str, ws_addr: Addr<ChatWebSocket>) -> Result<(), Box<dyn std::error::Error>> {
        // Create a new Command for the claude process
        let mut cmd = Command::new("claude");
        cmd.args(&["--dangerously-skip-permissions", "-c", "-p", prompt]);

        // Configure the process to capture stdout
        cmd.stdout(Stdio::piped());
        cmd.stderr(Stdio::piped());

        // Spawn the process
        let mut child = cmd.spawn()?;

        // Get the stdout handle
        let stdout = child.stdout.take().ok_or("Failed to capture stdout")?;

        // Create a buffer reader for stdout
        let mut reader = tokio::io::BufReader::new(stdout).lines();

        let mut lines : Vec<String> = Vec::new();
        // Read the output line by line and send it to the WebSocket
        while let Some(line) = reader.next_line().await? {
            lines.push(line);
        }

        let msg = ChatMessage {
            message_type: "output".to_string(),
            content: lines.join("\n"),
        };

        if let Ok(json) = serde_json::to_string(&msg) {
            let _ = ws_addr.try_send(WsMessage(json));
        }

        // Wait for the process to complete
        let status = child.wait().await?;

        if !status.success() {
            return Err(format!("Claude process exited with status: {}", status).into());
        }

        Ok(())
    }

    pub async fn start(&mut self, ws_addr: Addr<ChatWebSocket>) -> Result<(), Box<dyn std::error::Error>> {
        // Store the WebSocket address
        *self.ws_addr.lock().unwrap() = Some(ws_addr);
        Ok(())
    }

    pub fn send_input(&self, input: &str) -> Result<(), Box<dyn std::error::Error>> {
        eprintln!("send_input called with: {}", input);

        // Get the WebSocket address from the stored value
        if let Some(ws_addr) = self.ws_addr.lock().unwrap().clone() {
            // Send the prompt to the processing thread
            self.prompt_tx.send((input.to_string(), ws_addr))?;
            eprintln!("Successfully sent prompt to processing thread: {}", input);
            Ok(())
        } else {
            eprintln!("WebSocket not available");
            Err("WebSocket not available".into())
        }
    }

    // Always return true as we're not tracking a specific process
    pub fn is_running(&self) -> bool {
        true
    }
}


// --- The rest of your code (ChatWebSocket, etc.) remains the same ---
// ... (I've omitted it for brevity, no changes are needed there)
pub struct ChatAppState {
    pub claude_process: Arc<Mutex<ClaudeProcess>>,
}

impl ChatAppState {
    pub fn new() -> Self {
        Self {
            claude_process: Arc::new(Mutex::new(ClaudeProcess::new())),
        }
    }
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct WsMessage(pub String);

pub struct ChatWebSocket {
    pub hb: Instant,
    pub claude_process: Arc<Mutex<ClaudeProcess>>,
    pub started: bool,
}

impl ChatWebSocket {
    pub fn new(claude_process: Arc<Mutex<ClaudeProcess>>) -> Self {
        Self {
            hb: Instant::now(),
            claude_process,
            started: false,
        }
    }

    fn hb(&self, ctx: &mut ws::WebsocketContext<Self>) {
        ctx.run_interval(Duration::from_secs(5), |act, ctx| {
            if Instant::now().duration_since(act.hb) > Duration::from_secs(10) {
                ctx.stop();
                return;
            }
            ctx.ping(b"");
        });
    }
}

impl Actor for ChatWebSocket {
    type Context = ws::WebsocketContext<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        self.hb(ctx);

        // Start Claude process when WebSocket connects
        if !self.started {
            let claude_process = self.claude_process.clone();
            let addr = ctx.address();

            // Always start the process to ensure WebSocket address is set
            let claude_process_clone = claude_process.clone();
            let addr_clone = addr.clone();

            ctx.spawn(
                async move {
                    {
                        let mut process = claude_process_clone.lock().unwrap();
                        if let Err(e) = process.start(addr_clone.clone()).await {
                            eprintln!("Failed to start Claude process: {}", e);
                            let error_msg = ChatMessage {
                                message_type: "error".to_string(),
                                content: format!("Failed to start Claude process: {}", e),
                            };
                            if let Ok(json) = serde_json::to_string(&error_msg) {
                                let _ = addr_clone.try_send(WsMessage(json));
                            }
                        }
                    } // MutexGuard dropped here
                }
                    .into_actor(self)
            );

            self.started = true;
        }
    }
}

impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for ChatWebSocket {
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        match msg {
            Ok(ws::Message::Ping(msg)) => {
                self.hb = Instant::now();
                ctx.pong(&msg);
            }
            Ok(ws::Message::Pong(_)) => {
                self.hb = Instant::now();
            }
            Ok(ws::Message::Text(text)) => {
                self.hb = Instant::now();

                if let Ok(chat_msg) = serde_json::from_str::<ChatMessage>(&text) {
                    if chat_msg.message_type == "input" {
                        // Send processing status
                        let processing_msg = serde_json::json!({
                            "type": "processing",
                            "processing": true
                        });
                        ctx.text(processing_msg.to_string());

                        // Send input to Claude
                        let process = self.claude_process.lock().unwrap();
                        if let Err(e) = process.send_input(&chat_msg.content) {
                            let error_msg = ChatMessage {
                                message_type: "error".to_string(),
                                content: format!("Failed to send to Claude: {}", e),
                            };
                            if let Ok(json) = serde_json::to_string(&error_msg) {
                                ctx.text(json);
                            }
                        }

                        // Reset processing status after a delay (Claude should respond)
                        ctx.run_later(Duration::from_secs(1), |_, ctx| {
                            let processing_msg = serde_json::json!({
                                "type": "processing",
                                "processing": false
                            });
                            ctx.text(processing_msg.to_string());
                        });
                    }
                }
            }
            Ok(ws::Message::Binary(_)) => {
                // Not handling binary messages
            }
            Ok(ws::Message::Close(reason)) => {
                ctx.close(reason);
                ctx.stop();
            }
            _ => ctx.stop(),
        }
    }
}

impl Handler<WsMessage> for ChatWebSocket {
    type Result = ();

    fn handle(&mut self, msg: WsMessage, ctx: &mut Self::Context) {
        ctx.text(msg.0);
    }
}

pub async fn chat_websocket(
    req: HttpRequest,
    stream: web::Payload,
    data: web::Data<ChatAppState>,
) -> Result<HttpResponse, Error> {
    let resp = ws::start(
        ChatWebSocket::new(data.claude_process.clone()),
        &req,
        stream,
    );
    resp
}
