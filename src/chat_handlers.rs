use actix_web::{web, HttpRequest, HttpResponse, Error, Result};
use actix_web_actors::ws;
use serde::{Deserialize, Serialize};
use std::process::Stdio;
use std::sync::{Arc, Mutex};
use tokio::process::{Child, Command as TokioCommand};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::sync::mpsc;
use std::time::{Duration, Instant};
use actix::{Actor, StreamHandler, Handler, Message, ActorContext, AsyncContext, Addr, WrapFuture};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    #[serde(rename = "type")]
    pub message_type: String,
    pub content: String,
}

#[derive(Debug)]
pub struct ClaudeProcess {
    pub child: Arc<Mutex<Option<Child>>>,
    pub stdin_tx: Option<mpsc::UnboundedSender<String>>,
}

impl ClaudeProcess {
    pub fn new() -> Self {
        Self {
            child: Arc::new(Mutex::new(None)),
            stdin_tx: None,
        }
    }

    pub async fn start(&mut self, ws_addr: Addr<ChatWebSocket>) -> Result<(), Box<dyn std::error::Error>> {
        let mut cmd = TokioCommand::new("claude");
        cmd.stdin(Stdio::piped())
           .stdout(Stdio::piped())
           .stderr(Stdio::piped());

        let mut child = cmd.spawn()?;
        
        let stdin = child.stdin.take().ok_or("Failed to get stdin")?;
        let stdout = child.stdout.take().ok_or("Failed to get stdout")?;
        
        let (stdin_tx, mut stdin_rx) = mpsc::unbounded_channel::<String>();

        // Handle stdin
        tokio::spawn(async move {
            let mut stdin = stdin;
            while let Some(input) = stdin_rx.recv().await {
                if let Err(e) = stdin.write_all(input.as_bytes()).await {
                    eprintln!("Failed to write to claude stdin: {}", e);
                    break;
                }
                if let Err(e) = stdin.write_all(b"\n").await {
                    eprintln!("Failed to write newline to claude stdin: {}", e);
                    break;
                }
                if let Err(e) = stdin.flush().await {
                    eprintln!("Failed to flush claude stdin: {}", e);
                    break;
                }
            }
        });

        // Handle stdout
        tokio::spawn(async move {
            let reader = BufReader::new(stdout);
            let mut lines = reader.lines();
            while let Ok(Some(line)) = lines.next_line().await {
                let msg = ChatMessage {
                    message_type: "output".to_string(),
                    content: line,
                };
                if let Ok(json) = serde_json::to_string(&msg) {
                    let _ = ws_addr.try_send(WsMessage(json));
                }
            }
        });

        *self.child.lock().unwrap() = Some(child);
        self.stdin_tx = Some(stdin_tx);

        Ok(())
    }

    pub fn send_input(&self, input: &str) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(ref tx) = self.stdin_tx {
            tx.send(input.to_string()).map_err(|e| e.into())
        } else {
            Err("Claude process not started".into())
        }
    }

    pub fn is_running(&self) -> bool {
        self.child.lock().unwrap().is_some()
    }
}

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
            
            // Check if process is already running
            let should_start = {
                let process = claude_process.lock().unwrap();
                !process.is_running()
            };
            
            if should_start {
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
            }
            
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