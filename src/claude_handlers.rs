use crate::models::ClaudeSessionManager;
use actix::{Actor, AsyncContext, Handler, Message, StreamHandler};
use actix_web::{Error, HttpRequest, HttpResponse, web};
use actix_web_actors::ws;
use portable_pty::{CommandBuilder, PtySize, native_pty_system};
use std::io::{Read, Write};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::sync::mpsc;
use tracing::{debug, error, info, warn};
use crate::project_handlers::ProjectAppState;

/// WebSocket actor for handling Claude CLI communication
pub struct ClaudeWebSocket {
    session_id: String,
    session_manager: Arc<ClaudeSessionManager>,
    project_app_state: web::Data<ProjectAppState>,
}

impl Actor for ClaudeWebSocket {
    type Context = ws::WebsocketContext<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        info!(
            "Claude WebSocket connection established for session: {}",
            self.session_id
        );

        // Create or get existing session
        match self.session_manager.create_session(self.session_id.clone()) {
            Ok(_) => {
                info!("Session {} ready", self.session_id);

                // Check if session already has an active process
                let needs_new_process = if let Some(session) = self.session_manager.get_session(&self.session_id) {
                    // Increment connection count for this session
                    session.increment_connections();
                    
                    // Check if we need to spawn a new process
                    !session.is_active()
                } else {
                    true
                };

                if needs_new_process {
                    // Spawn Claude CLI process with PTY
                    match self.spawn_claude_process(ctx) {
                        Ok(_) => {
                            info!(
                                "Claude CLI process spawned successfully for session {}",
                                self.session_id
                            );
                        }
                        Err(e) => {
                            error!(
                                "Failed to spawn Claude CLI process for session {}: {}",
                                self.session_id, e
                            );
                            let _ = self.session_manager.cleanup_session(&self.session_id);
                            ctx.close(Some(ws::CloseCode::Error.into()));
                        }
                    }
                } else {
                    info!(
                        "Reconnecting to existing Claude CLI process for session {}",
                        self.session_id
                    );
                    // Set up output streaming for the reconnected session
                    self.setup_output_streaming(ctx);
                }
            }
            Err(e) => {
                error!("Failed to create session {}: {}", self.session_id, e);
                ctx.close(Some(ws::CloseCode::Error.into()));
            }
        }
    }

    fn stopped(&mut self, _ctx: &mut Self::Context) {
        info!(
            "Claude WebSocket connection closed for session: {}",
            self.session_id
        );

        // Decrement connection count but DO NOT cleanup the session
        // This allows the Claude process to keep running when switching tabs
        if let Some(session) = self.session_manager.get_session(&self.session_id) {
            let remaining_connections = session.decrement_connections();
            info!(
                "Session {} has {} remaining connections (keeping session alive)",
                self.session_id, remaining_connections
            );

            // Note: We intentionally do NOT cleanup the session here
            // The Claude process will continue running until the server shuts down
            // This allows seamless reconnection when switching back to the terminal tab
        }
    }
}

/// Message types for WebSocket communication
#[derive(Message)]
#[rtype(result = "()")]
pub struct WsMessage(pub String);

impl Handler<WsMessage> for ClaudeWebSocket {
    type Result = ();

    fn handle(&mut self, msg: WsMessage, ctx: &mut Self::Context) {
        ctx.text(msg.0);
    }
}

impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for ClaudeWebSocket {
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        match msg {
            Ok(ws::Message::Text(text)) => {
                debug!(
                    "Received WebSocket message for session {}: {}",
                    self.session_id, text
                );

                // Update session activity and send text to PTY stdin
                if let Some(session) = self.session_manager.get_session(&self.session_id) {
                    session.update_activity();

                    // Send text to PTY stdin if available
                    if let Ok(stdin_opt) = session.stdin_tx.lock() {
                        if let Some(ref tx) = stdin_opt.as_ref() {
                            if let Err(e) = tx.send(format!("{}", text)) {
                                error!(
                                    "Failed to send message to PTY for session {}: {}",
                                    self.session_id, e
                                );
                                ctx.close(Some(ws::CloseCode::Error.into()));
                            }
                        } else {
                            warn!("No stdin channel available for session {}", self.session_id);
                        }
                    } else {
                        warn!(
                            "Failed to acquire stdin lock for session {}",
                            self.session_id
                        );
                    }
                } else {
                    error!("Session {} not found", self.session_id);
                    ctx.close(Some(ws::CloseCode::Error.into()));
                }
            }
            Ok(ws::Message::Binary(bin)) => {
                // Convert binary to string and send to PTY
                if let Ok(text) = String::from_utf8(bin.to_vec()) {
                    if let Some(session) = self.session_manager.get_session(&self.session_id) {
                        session.update_activity();

                        if let Ok(stdin_opt) = session.stdin_tx.lock() {
                            if let Some(ref tx) = stdin_opt.as_ref() {
                                if let Err(e) = tx.send(format!("{}", text)) {
                                    error!(
                                        "Failed to send binary message to PTY for session {}: {}",
                                        self.session_id, e
                                    );
                                    ctx.close(Some(ws::CloseCode::Error.into()));
                                }
                            }
                        }
                    }
                }
            }
            Ok(ws::Message::Close(reason)) => {
                info!("WebSocket close received: {:?}", reason);
                ctx.close(reason);
            }
            Ok(ws::Message::Ping(msg)) => {
                debug!("WebSocket ping received");
                ctx.pong(&msg);
            }
            Ok(ws::Message::Pong(_)) => {
                debug!("WebSocket pong received");
            }
            Err(e) => {
                error!("WebSocket protocol error: {}", e);
                ctx.close(Some(ws::CloseCode::Error.into()));
            }
            _ => {
                debug!("Unhandled WebSocket message type");
            }
        }
    }
}

impl ClaudeWebSocket {
    pub fn new(session_manager: Arc<ClaudeSessionManager>, project_app_state: web::Data<ProjectAppState>) -> Self {
        // Use a fixed session ID for persistent connection
        let session_id = "default-claude-session".to_string();

        Self {
            session_id,
            session_manager,
            project_app_state,
        }
    }

    pub fn with_session_id(session_id: String, session_manager: Arc<ClaudeSessionManager>, project_app_state: web::Data<ProjectAppState>) -> Self {
        Self {
            session_id,
            session_manager,
            project_app_state,
        }
    }

    fn spawn_claude_process(
        &mut self,
        ctx: &mut ws::WebsocketContext<Self>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // Create PTY system
        let pty_system = native_pty_system();

        // Set up PTY size (standard terminal size)
        let pty_size = PtySize {
            rows: 24,
            cols: 80,
            pixel_width: 0,
            pixel_height: 0,
        };

        let project_config = match self.project_app_state.project_manager.get_config() {
            Ok(config) => config,
            Err(_) => return Err("Failed to get project configuration".to_string().into()),
        };
        let project_dir = project_config.project_home_directory.clone();

        // Create PTY pair
        let pty_pair = pty_system.openpty(pty_size)?;

        // Create command for Claude CLI
        let mut cmd = CommandBuilder::new("claude");
        cmd.args(&["--dangerously-skip-permissions"]);
        cmd.cwd(project_dir);

        // Spawn the process
        let child = pty_pair.slave.spawn_command(cmd)?;

        // Get master PTY for I/O
        let master = pty_pair.master;

        // Create channels for communication
        let (stdin_tx, stdin_rx) = mpsc::unbounded_channel::<String>();
        
        // Create broadcast channel for output (capacity 1000 messages)
        let (output_tx, _) = tokio::sync::broadcast::channel::<String>(1000);

        // Store channels and child process in the session
        if let Some(session) = self.session_manager.get_session(&self.session_id) {
            session.set_child_process_box(child);
            session.set_stdin_tx(stdin_tx);
            session.set_output_tx(output_tx.clone());
            session.set_active(true);
        } else {
            return Err("Session not found when setting up process".into());
        }

        // Get reader and writer from master PTY
        let master_reader = master.try_clone_reader()?;
        let master_writer = master.take_writer()?;
        let master_writer = Arc::new(Mutex::new(master_writer));

        // Spawn blocking task to handle PTY input (from WebSocket to Claude)
        let writer_handle = master_writer.clone();
        tokio::task::spawn_blocking(move || {
            let mut stdin_rx = stdin_rx;
            loop {
                match stdin_rx.blocking_recv() {
                    Some(input) => {
                        if let Ok(mut writer) = writer_handle.lock() {
                            if let Err(e) = writer.write_all(input.as_bytes()) {
                                error!("Failed to write to PTY: {}", e);
                                break;
                            }
                            if let Err(e) = writer.flush() {
                                error!("Failed to flush PTY writer: {}", e);
                                break;
                            }
                        }
                    }
                    None => break,
                }
            }
        });

        // Spawn blocking task to handle PTY output (from Claude to broadcast channel)
        tokio::task::spawn_blocking(move || {
            let mut reader = master_reader;
            let mut buffer = [0u8; 8192];

            loop {
                match reader.read(&mut buffer) {
                    Ok(0) => {
                        // EOF - process has terminated
                        debug!("PTY reader reached EOF");
                        break;
                    }
                    Ok(n) => {
                        let output = String::from_utf8_lossy(&buffer[..n]).to_string();
                        debug!("PTY output: {}", output);

                        // Send output to broadcast channel (all connected WebSockets will receive it)
                        if let Err(e) = output_tx.send(output) {
                            warn!("Failed to send PTY output to broadcast channel: {}", e);
                            // Continue reading even if no receivers are listening
                        }
                    }
                    Err(e) => {
                        error!("Failed to read from PTY: {}", e);
                        break;
                    }
                }
            }
        });

        // Spawn task to monitor child process
        let session_manager_clone = Arc::clone(&self.session_manager);
        let session_id_clone = self.session_id.clone();
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(1));
            loop {
                interval.tick().await;

                if let Some(session) = session_manager_clone.get_session(&session_id_clone) {
                    if let Ok(mut child_opt) = session.child_process.lock() {
                        if let Some(ref mut child) = child_opt.as_mut() {
                            match child.try_wait() {
                                Ok(Some(status)) => {
                                    info!(
                                        "Claude CLI process for session {} exited with status: {:?}",
                                        session_id_clone, status
                                    );
                                    session.set_active(false);
                                    break;
                                }
                                Ok(None) => {
                                    // Process still running
                                    continue;
                                }
                                Err(e) => {
                                    error!(
                                        "Error checking child process status for session {}: {}",
                                        session_id_clone, e
                                    );
                                    session.set_active(false);
                                    break;
                                }
                            }
                        } else {
                            // No child process
                            break;
                        }
                    } else {
                        error!(
                            "Failed to acquire child process lock for session {}",
                            session_id_clone
                        );
                        break;
                    }
                } else {
                    // Session no longer exists
                    break;
                }
            }
        });

        // Set up output streaming for this WebSocket connection
        self.setup_output_streaming(ctx);

        Ok(())
    }

    /// Set up output streaming for reconnecting to an existing session
    fn setup_output_streaming(&self, ctx: &mut ws::WebsocketContext<Self>) {
        // Get the session to access existing broadcast receiver
        if let Some(session) = self.session_manager.get_session(&self.session_id) {
            // Get WebSocket address for sending messages back
            let addr = ctx.address();
            let session_id_clone = self.session_id.clone();

            // Subscribe to the broadcast channel for output
            if let Some(mut output_rx) = session.get_output_rx() {
                info!(
                    "WebSocket reconnected to existing session {}. Setting up output streaming.",
                    session_id_clone
                );

                // Send a reconnection message to the client
                //addr.do_send(WsMessage("Reconnected to claude CLI session. Ready for commands ...\n".to_string()));

                // Spawn task to forward broadcast messages to this WebSocket
                tokio::spawn(async move {
                    while let Ok(output) = output_rx.recv().await {
                        if addr.try_send(WsMessage(output)).is_err() {
                            warn!(
                                "Failed to send broadcast output to WebSocket for session {} - connection may be closed",
                                session_id_clone
                            );
                            break;
                        }
                    }
                    debug!("Output streaming task ended for session {}", session_id_clone);
                });
            } else {
                warn!("No output broadcast channel available for session {}", self.session_id);
            }
        } else {
            error!("Failed to find session {} for output streaming setup", self.session_id);
        }
    }
}

/// WebSocket handler function for Claude CLI
pub async fn claude_ws_handler(
    req: HttpRequest,
    stream: web::Payload,
    session_manager: web::Data<Arc<ClaudeSessionManager>>,
    project_app_state: web::Data<ProjectAppState>,
) -> Result<HttpResponse, Error> {
    info!(
        "New Claude WebSocket connection request from: {:?}",
        req.connection_info().realip_remote_addr()
    );

    let actor = ClaudeWebSocket::new(Arc::clone(&session_manager), project_app_state);
    let resp = ws::start(actor, &req, stream)?;

    Ok(resp)
}

/// Alternative implementation using lower-level WebSocket handling
pub async fn handle_claude_socket(
    _socket: actix_web_actors::ws::WebsocketContext<ClaudeWebSocket>,
) -> Result<(), std::io::Error> {
    // This function signature matches the requirement but with actix-web,
    // the WebSocket handling is done through the Actor pattern above.
    // This function serves as a placeholder for the required signature.
    Ok(())
}
