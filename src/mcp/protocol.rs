use crate::mcp::errors::{JsonRpcError, MCPError, MCPResult, ProtocolError};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

/// JSON-RPC 2.0 message structure for MCP
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MCPMessage {
    pub jsonrpc: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub method: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub params: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<JsonRpcError>,
}

/// Request message structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MCPRequest {
    pub jsonrpc: String,
    pub id: Value,
    pub method: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub params: Option<Value>,
}

/// Response message structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MCPResponse {
    pub jsonrpc: String,
    pub id: Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<JsonRpcError>,
}

/// Notification message structure (no id, no response expected)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MCPNotification {
    pub jsonrpc: String,
    pub method: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub params: Option<Value>,
}

impl MCPMessage {
    const JSONRPC_VERSION: &'static str = "2.0";

    /// Create a new request message
    pub fn request(method: impl Into<String>, params: Option<Value>) -> Self {
        Self {
            jsonrpc: Self::JSONRPC_VERSION.to_string(),
            id: Some(Value::String(Uuid::new_v4().to_string())),
            method: Some(method.into()),
            params,
            result: None,
            error: None,
        }
    }

    /// Create a new response message
    pub fn response(id: Value, result: Option<Value>) -> Self {
        Self {
            jsonrpc: Self::JSONRPC_VERSION.to_string(),
            id: Some(id),
            method: None,
            params: None,
            result,
            error: None,
        }
    }

    /// Create a new error response message
    pub fn error_response(id: Value, error: JsonRpcError) -> Self {
        Self {
            jsonrpc: Self::JSONRPC_VERSION.to_string(),
            id: Some(id),
            method: None,
            params: None,
            result: None,
            error: Some(error),
        }
    }

    /// Create a new notification message
    pub fn notification(method: impl Into<String>, params: Option<Value>) -> Self {
        Self {
            jsonrpc: Self::JSONRPC_VERSION.to_string(),
            id: None,
            method: Some(method.into()),
            params,
            result: None,
            error: None,
        }
    }

    /// Check if this is a request message
    pub fn is_request(&self) -> bool {
        self.method.is_some() && self.id.is_some()
    }

    /// Check if this is a response message
    pub fn is_response(&self) -> bool {
        self.id.is_some()
            && self.method.is_none()
            && (self.result.is_some() || self.error.is_some())
    }

    /// Check if this is a notification message
    pub fn is_notification(&self) -> bool {
        self.method.is_some() && self.id.is_none()
    }

    /// Validate the message structure
    pub fn validate(&self) -> MCPResult<()> {
        // Check JSON-RPC version
        if self.jsonrpc != Self::JSONRPC_VERSION {
            return Err(MCPError::Protocol(ProtocolError::InvalidMessage(format!(
                "Invalid JSON-RPC version: {}",
                self.jsonrpc
            ))));
        }

        // Validate message type consistency
        if self.is_request() {
            if self.result.is_some() || self.error.is_some() {
                return Err(MCPError::Protocol(ProtocolError::InvalidMessage(
                    "Request message cannot have result or error fields".to_string(),
                )));
            }
        } else if self.is_response() {
            if self.method.is_some() || self.params.is_some() {
                return Err(MCPError::Protocol(ProtocolError::InvalidMessage(
                    "Response message cannot have method or params fields".to_string(),
                )));
            }

            // Response must have either result or error, but not both
            match (self.result.is_some(), self.error.is_some()) {
                (true, true) => {
                    return Err(MCPError::Protocol(ProtocolError::InvalidMessage(
                        "Response cannot have both result and error".to_string(),
                    )));
                }
                (false, false) => {
                    return Err(MCPError::Protocol(ProtocolError::InvalidMessage(
                        "Response must have either result or error".to_string(),
                    )));
                }
                _ => {} // Valid
            }
        } else if self.is_notification() {
            if self.result.is_some() || self.error.is_some() {
                return Err(MCPError::Protocol(ProtocolError::InvalidMessage(
                    "Notification message cannot have result or error fields".to_string(),
                )));
            }
        } else {
            return Err(MCPError::Protocol(ProtocolError::InvalidMessage(
                "Message does not match any valid type (request, response, notification)"
                    .to_string(),
            )));
        }

        Ok(())
    }

    /// Convert to typed request
    pub fn as_request(&self) -> MCPResult<MCPRequest> {
        if !self.is_request() {
            return Err(MCPError::Protocol(ProtocolError::InvalidMessage(
                "Message is not a request".to_string(),
            )));
        }

        Ok(MCPRequest {
            jsonrpc: self.jsonrpc.clone(),
            id: self.id.clone().unwrap(),
            method: self.method.clone().unwrap(),
            params: self.params.clone(),
        })
    }

    /// Convert to typed response
    pub fn as_response(&self) -> MCPResult<MCPResponse> {
        if !self.is_response() {
            return Err(MCPError::Protocol(ProtocolError::InvalidMessage(
                "Message is not a response".to_string(),
            )));
        }

        Ok(MCPResponse {
            jsonrpc: self.jsonrpc.clone(),
            id: self.id.clone().unwrap(),
            result: self.result.clone(),
            error: self.error.clone(),
        })
    }

    /// Convert to typed notification
    pub fn as_notification(&self) -> MCPResult<MCPNotification> {
        if !self.is_notification() {
            return Err(MCPError::Protocol(ProtocolError::InvalidMessage(
                "Message is not a notification".to_string(),
            )));
        }

        Ok(MCPNotification {
            jsonrpc: self.jsonrpc.clone(),
            method: self.method.clone().unwrap(),
            params: self.params.clone(),
        })
    }
}

impl From<MCPRequest> for MCPMessage {
    fn from(request: MCPRequest) -> Self {
        Self {
            jsonrpc: request.jsonrpc,
            id: Some(request.id),
            method: Some(request.method),
            params: request.params,
            result: None,
            error: None,
        }
    }
}

impl From<MCPResponse> for MCPMessage {
    fn from(response: MCPResponse) -> Self {
        Self {
            jsonrpc: response.jsonrpc,
            id: Some(response.id),
            method: None,
            params: None,
            result: response.result,
            error: response.error,
        }
    }
}

impl From<MCPNotification> for MCPMessage {
    fn from(notification: MCPNotification) -> Self {
        Self {
            jsonrpc: notification.jsonrpc,
            id: None,
            method: Some(notification.method),
            params: notification.params,
            result: None,
            error: None,
        }
    }
}

/// MCP protocol initialization parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InitializeParams {
    #[serde(rename = "protocolVersion")]
    pub protocol_version: String,
    pub capabilities: ClientCapabilities,
    #[serde(rename = "clientInfo")]
    pub client_info: ClientInfo,
}

/// Client capabilities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientCapabilities {
    #[serde(default)]
    pub tools: ToolsCapability,
    #[serde(default)]
    pub prompts: PromptsCapability,
    #[serde(default)]
    pub resources: ResourcesCapability,
    #[serde(default)]
    pub logging: LoggingCapability,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ToolsCapability {
    #[serde(rename = "listChanged", default)]
    pub list_changed: bool,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PromptsCapability {
    #[serde(rename = "listChanged", default)]
    pub list_changed: bool,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ResourcesCapability {
    #[serde(rename = "listChanged", default)]
    pub list_changed: bool,
    #[serde(default)]
    pub subscribe: bool,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct LoggingCapability {
    // Future extension point
}

/// Client information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientInfo {
    pub name: String,
    pub version: String,
}

/// Server capabilities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerCapabilities {
    #[serde(default)]
    pub tools: ToolsCapability,
    #[serde(default)]
    pub prompts: PromptsCapability,
    #[serde(default)]
    pub resources: ResourcesCapability,
    #[serde(default)]
    pub logging: LoggingCapability,
}

/// Server information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerInfo {
    pub name: String,
    pub version: String,
}

/// Initialize response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InitializeResult {
    #[serde(rename = "protocolVersion")]
    pub protocol_version: String,
    pub capabilities: ServerCapabilities,
    #[serde(rename = "serverInfo")]
    pub server_info: ServerInfo,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub instructions: Option<String>,
}

/// Protocol message parser
pub struct MessageParser;

impl MessageParser {
    /// Parse a message from JSON bytes
    pub fn parse_message(data: &[u8]) -> MCPResult<MCPMessage> {
        let message: MCPMessage = serde_json::from_slice(data)
            .map_err(|e| MCPError::Protocol(ProtocolError::ParseError(e.to_string())))?;

        message.validate()?;
        Ok(message)
    }

    /// Serialize a message to JSON bytes
    pub fn serialize_message(message: &MCPMessage) -> MCPResult<Vec<u8>> {
        message.validate()?;
        serde_json::to_vec(message)
            .map_err(|e| MCPError::Protocol(ProtocolError::InternalError(e.to_string())))
    }

    /// Parse multiple messages from a buffer (for stream parsing)
    pub fn parse_messages(buffer: &str) -> Vec<MCPResult<MCPMessage>> {
        let mut results = Vec::new();

        for line in buffer.lines() {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }

            let result = Self::parse_message(line.as_bytes());
            results.push(result);
        }

        results
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_request_message() {
        let msg = MCPMessage::request("test_method", Some(json!({"param": "value"})));
        assert!(msg.is_request());
        assert!(!msg.is_response());
        assert!(!msg.is_notification());
        assert!(msg.validate().is_ok());
    }

    #[test]
    fn test_response_message() {
        let id = json!("test-id");
        let msg = MCPMessage::response(id, Some(json!({"result": "success"})));
        assert!(!msg.is_request());
        assert!(msg.is_response());
        assert!(!msg.is_notification());
        assert!(msg.validate().is_ok());
    }

    #[test]
    fn test_notification_message() {
        let msg = MCPMessage::notification("test_notification", Some(json!({"data": "value"})));
        assert!(!msg.is_request());
        assert!(!msg.is_response());
        assert!(msg.is_notification());
        assert!(msg.validate().is_ok());
    }

    #[test]
    fn test_invalid_message() {
        let mut msg = MCPMessage::request("test", None);
        msg.result = Some(json!("invalid")); // Request shouldn't have result
        assert!(msg.validate().is_err());
    }

    #[test]
    fn test_message_parsing() {
        let json_data = r#"{"jsonrpc":"2.0","id":"1","method":"test","params":{"key":"value"}}"#;
        let result = MessageParser::parse_message(json_data.as_bytes());
        assert!(result.is_ok());

        let message = result.unwrap();
        assert!(message.is_request());
        assert_eq!(message.method.unwrap(), "test");
    }
}
