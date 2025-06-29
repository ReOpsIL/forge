use async_trait::async_trait;
use futures_util::{SinkExt, StreamExt};
use serde_json::Value;
use std::sync::Arc;
use tokio::sync::{RwLock, mpsc};
use tokio_tungstenite::{accept_async, tungstenite::Message as WsMessage};
use tracing::{debug, error, info, warn};

use crate::mcp::{
    errors::{MCPError, MCPResult, TransportError},
    protocol::{MCPMessage, MessageParser},
};

/// Transport types supported by the MCP server
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TransportType {
    WebSocket,
    Http,
    Stdio,
}

/// Abstract transport trait for MCP communication
#[async_trait]
pub trait MCPTransport: Send + Sync {
    /// Send a message through the transport
    async fn send(&mut self, message: MCPMessage) -> MCPResult<()>;

    /// Receive a message from the transport
    async fn receive(&mut self) -> MCPResult<MCPMessage>;

    /// Close the transport connection
    async fn close(&mut self) -> MCPResult<()>;

    /// Check if the transport is still connected
    fn is_connected(&self) -> bool;

    /// Get the transport type
    fn transport_type(&self) -> TransportType;
}

/// WebSocket transport implementation
pub struct WebSocketTransport {
    sender: mpsc::UnboundedSender<MCPMessage>,
    receiver: mpsc::UnboundedReceiver<MCPMessage>,
    is_connected: Arc<RwLock<bool>>,
}

impl WebSocketTransport {
    /// Create a new WebSocket transport from a TCP stream
    pub async fn new(stream: tokio::net::TcpStream) -> MCPResult<Self> {
        let ws_stream = accept_async(stream)
            .await
            .map_err(|e| MCPError::Transport(TransportError::WebSocket(e)))?;

        let (ws_sender, ws_receiver) = ws_stream.split();
        let (msg_sender, msg_receiver) = mpsc::unbounded_channel();
        let (response_sender, response_receiver) = mpsc::unbounded_channel();

        let is_connected = Arc::new(RwLock::new(true));
        let is_connected_clone = is_connected.clone();

        // Spawn task to handle outgoing messages
        let mut ws_sender = ws_sender;
        tokio::spawn(async move {
            let mut response_receiver = response_receiver;
            while let Some(message) = response_receiver.recv().await {
                let json_data = match MessageParser::serialize_message(&message) {
                    Ok(data) => data,
                    Err(e) => {
                        error!("Failed to serialize message: {}", e);
                        continue;
                    }
                };

                let ws_message = WsMessage::Text(String::from_utf8_lossy(&json_data).to_string());
                if let Err(e) = ws_sender.send(ws_message).await {
                    error!("Failed to send WebSocket message: {}", e);
                    *is_connected_clone.write().await = false;
                    break;
                }
            }
        });

        // Spawn task to handle incoming messages
        let msg_sender_clone = msg_sender.clone();
        let is_connected_clone = is_connected.clone();
        tokio::spawn(async move {
            let mut ws_receiver = ws_receiver;
            while let Some(message) = ws_receiver.next().await {
                match message {
                    Ok(WsMessage::Text(text)) => {
                        match MessageParser::parse_message(text.as_bytes()) {
                            Ok(mcp_message) => {
                                if let Err(_) = msg_sender_clone.send(mcp_message) {
                                    warn!("Receiver dropped, closing WebSocket connection");
                                    break;
                                }
                            }
                            Err(e) => {
                                error!("Failed to parse MCP message: {}", e);
                            }
                        }
                    }
                    Ok(WsMessage::Binary(data)) => match MessageParser::parse_message(&data) {
                        Ok(mcp_message) => {
                            if let Err(_) = msg_sender_clone.send(mcp_message) {
                                warn!("Receiver dropped, closing WebSocket connection");
                                break;
                            }
                        }
                        Err(e) => {
                            error!("Failed to parse MCP message from binary: {}", e);
                        }
                    },
                    Ok(WsMessage::Close(_)) => {
                        info!("WebSocket connection closed by client");
                        break;
                    }
                    Ok(WsMessage::Ping(data)) => {
                        debug!("Received ping, sending pong");
                        // Pong is handled automatically by tungstenite
                    }
                    Ok(WsMessage::Pong(_)) => {
                        debug!("Received pong");
                    }
                    Ok(WsMessage::Frame(_)) => {
                        debug!("Received raw frame");
                    }
                    Err(e) => {
                        error!("WebSocket error: {}", e);
                        break;
                    }
                }
            }
            *is_connected_clone.write().await = false;
        });

        Ok(Self {
            sender: response_sender,
            receiver: msg_receiver,
            is_connected,
        })
    }
}

#[async_trait]
impl MCPTransport for WebSocketTransport {
    async fn send(&mut self, message: MCPMessage) -> MCPResult<()> {
        if !self.is_connected() {
            return Err(MCPError::Transport(TransportError::ConnectionLost(
                "WebSocket connection is closed".to_string(),
            )));
        }

        self.sender.send(message).map_err(|_| {
            MCPError::Transport(TransportError::ConnectionLost(
                "WebSocket sender channel closed".to_string(),
            ))
        })
    }

    async fn receive(&mut self) -> MCPResult<MCPMessage> {
        self.receiver.recv().await.ok_or_else(|| {
            MCPError::Transport(TransportError::ConnectionLost(
                "WebSocket receiver channel closed".to_string(),
            ))
        })
    }

    async fn close(&mut self) -> MCPResult<()> {
        *self.is_connected.write().await = false;
        Ok(())
    }

    fn is_connected(&self) -> bool {
        // Use try_read to avoid blocking
        self.is_connected
            .try_read()
            .map(|guard| *guard)
            .unwrap_or(false)
    }

    fn transport_type(&self) -> TransportType {
        TransportType::WebSocket
    }
}

/// HTTP transport implementation (for request-response pattern)
pub struct HttpTransport {
    pending_responses: Arc<RwLock<std::collections::HashMap<String, MCPMessage>>>,
    is_connected: bool,
}

impl HttpTransport {
    pub fn new() -> Self {
        Self {
            pending_responses: Arc::new(RwLock::new(std::collections::HashMap::new())),
            is_connected: true,
        }
    }

    /// Handle an HTTP request and return the response
    pub async fn handle_request(&mut self, request_body: Vec<u8>) -> MCPResult<Vec<u8>> {
        let request = MessageParser::parse_message(&request_body)?;

        // For HTTP, we expect immediate responses
        // This would typically be handled by the MCP server
        // For now, return a placeholder response
        let response = if request.is_request() {
            let req = request.as_request()?;
            MCPMessage::response(req.id, Some(serde_json::json!({"status": "received"})))
        } else {
            return Err(MCPError::Transport(TransportError::InvalidMessage(
                "HTTP transport expects request messages".to_string(),
            )));
        };

        MessageParser::serialize_message(&response)
    }
}

#[async_trait]
impl MCPTransport for HttpTransport {
    async fn send(&mut self, message: MCPMessage) -> MCPResult<()> {
        // For HTTP, sending typically means storing a response to be returned
        if let Some(id) = &message.id {
            if let Ok(id_str) = serde_json::to_string(id) {
                self.pending_responses.write().await.insert(id_str, message);
                return Ok(());
            }
        }

        Err(MCPError::Transport(TransportError::InvalidMessage(
            "HTTP transport requires message ID for responses".to_string(),
        )))
    }

    async fn receive(&mut self) -> MCPResult<MCPMessage> {
        // HTTP is request-response, so receiving is handled differently
        // This would typically be called by the HTTP handler
        Err(MCPError::Transport(TransportError::InvalidMessage(
            "HTTP transport does not support streaming receive".to_string(),
        )))
    }

    async fn close(&mut self) -> MCPResult<()> {
        self.is_connected = false;
        Ok(())
    }

    fn is_connected(&self) -> bool {
        self.is_connected
    }

    fn transport_type(&self) -> TransportType {
        TransportType::Http
    }
}

/// Stdio transport implementation
pub struct StdioTransport {
    sender: mpsc::UnboundedSender<MCPMessage>,
    receiver: mpsc::UnboundedReceiver<MCPMessage>,
    is_connected: Arc<RwLock<bool>>,
}

impl StdioTransport {
    pub async fn new() -> MCPResult<Self> {
        let (msg_sender, msg_receiver) = mpsc::unbounded_channel();
        let (response_sender, response_receiver) = mpsc::unbounded_channel();
        let is_connected = Arc::new(RwLock::new(true));

        // Spawn task to handle stdout output
        let is_connected_clone = is_connected.clone();
        tokio::spawn(async move {
            let mut response_receiver = response_receiver;
            while let Some(message) = response_receiver.recv().await {
                let json_data = match MessageParser::serialize_message(&message) {
                    Ok(data) => data,
                    Err(e) => {
                        error!("Failed to serialize message: {}", e);
                        continue;
                    }
                };

                // Write to stdout with newline delimiter
                if let Err(e) =
                    tokio::io::AsyncWriteExt::write_all(&mut tokio::io::stdout(), &json_data).await
                {
                    error!("Failed to write to stdout: {}", e);
                    *is_connected_clone.write().await = false;
                    break;
                }

                if let Err(e) =
                    tokio::io::AsyncWriteExt::write_all(&mut tokio::io::stdout(), b"\n").await
                {
                    error!("Failed to write newline to stdout: {}", e);
                    *is_connected_clone.write().await = false;
                    break;
                }

                if let Err(e) = tokio::io::AsyncWriteExt::flush(&mut tokio::io::stdout()).await {
                    error!("Failed to flush stdout: {}", e);
                    *is_connected_clone.write().await = false;
                    break;
                }
            }
        });

        // Spawn task to handle stdin input
        let msg_sender_clone = msg_sender.clone();
        let is_connected_clone = is_connected.clone();
        tokio::spawn(async move {
            use tokio::io::{AsyncBufReadExt, BufReader};

            let stdin = tokio::io::stdin();
            let reader = BufReader::new(stdin);
            let mut lines = reader.lines();

            while let Ok(Some(line)) = lines.next_line().await {
                if line.trim().is_empty() {
                    continue;
                }

                match MessageParser::parse_message(line.as_bytes()) {
                    Ok(mcp_message) => {
                        if let Err(_) = msg_sender_clone.send(mcp_message) {
                            warn!("Receiver dropped, closing stdio connection");
                            break;
                        }
                    }
                    Err(e) => {
                        error!("Failed to parse MCP message from stdin: {}", e);
                    }
                }
            }

            *is_connected_clone.write().await = false;
        });

        Ok(Self {
            sender: response_sender,
            receiver: msg_receiver,
            is_connected,
        })
    }
}

#[async_trait]
impl MCPTransport for StdioTransport {
    async fn send(&mut self, message: MCPMessage) -> MCPResult<()> {
        if !self.is_connected() {
            return Err(MCPError::Transport(TransportError::ConnectionLost(
                "Stdio connection is closed".to_string(),
            )));
        }

        self.sender.send(message).map_err(|_| {
            MCPError::Transport(TransportError::ConnectionLost(
                "Stdio sender channel closed".to_string(),
            ))
        })
    }

    async fn receive(&mut self) -> MCPResult<MCPMessage> {
        self.receiver.recv().await.ok_or_else(|| {
            MCPError::Transport(TransportError::ConnectionLost(
                "Stdio receiver channel closed".to_string(),
            ))
        })
    }

    async fn close(&mut self) -> MCPResult<()> {
        *self.is_connected.write().await = false;
        Ok(())
    }

    fn is_connected(&self) -> bool {
        self.is_connected
            .try_read()
            .map(|guard| *guard)
            .unwrap_or(false)
    }

    fn transport_type(&self) -> TransportType {
        TransportType::Stdio
    }
}

/// Transport factory for creating transport instances
pub struct TransportFactory;

impl TransportFactory {
    /// Create a WebSocket transport from a TCP stream
    pub async fn create_websocket(
        stream: tokio::net::TcpStream,
    ) -> MCPResult<Box<dyn MCPTransport>> {
        let transport = WebSocketTransport::new(stream).await?;
        Ok(Box::new(transport))
    }

    /// Create an HTTP transport
    pub async fn create_http() -> MCPResult<Box<dyn MCPTransport>> {
        let transport = HttpTransport::new();
        Ok(Box::new(transport))
    }

    /// Create a stdio transport
    pub async fn create_stdio() -> MCPResult<Box<dyn MCPTransport>> {
        let transport = StdioTransport::new().await?;
        Ok(Box::new(transport))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_transport_types() {
        assert_eq!(
            TransportType::WebSocket as u8,
            TransportType::WebSocket as u8
        );
        assert_ne!(TransportType::WebSocket, TransportType::Http);
    }

    #[tokio::test]
    async fn test_http_transport() {
        let mut transport = HttpTransport::new();
        assert!(transport.is_connected());
        assert_eq!(transport.transport_type(), TransportType::Http);

        // Test handling a request
        let request = MCPMessage::request("test_method", Some(json!({"param": "value"})));
        let request_data = MessageParser::serialize_message(&request).unwrap();

        let response_data = transport.handle_request(request_data).await.unwrap();
        let response = MessageParser::parse_message(&response_data).unwrap();

        assert!(response.is_response());
    }

    #[tokio::test]
    async fn test_message_serialization() {
        let message = MCPMessage::request("test", Some(json!({"key": "value"})));
        let serialized = MessageParser::serialize_message(&message).unwrap();
        let deserialized = MessageParser::parse_message(&serialized).unwrap();

        assert_eq!(message.method, deserialized.method);
        assert_eq!(message.params, deserialized.params);
    }
}
