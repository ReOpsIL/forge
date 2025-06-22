use actix_web::{web, HttpResponse, Error};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::env;
use reqwest::Client;
use crate::project_config::{ProjectConfigManager, PROJECT_CONFIG_FILE};

// Constants for Anthropic API
const ANTHROPIC_API_URL: &str = "https://api.anthropic.com/v1/messages";
const DEFAULT_ANTHROPIC_MODEL: &str = "claude-sonnet-4-20250514";

// App state for Claude MCP server
pub struct ClaudeMCPAppState {
    pub project_manager: Arc<ProjectConfigManager>,
    pub client: Client,
}

impl ClaudeMCPAppState {
    pub fn new(project_manager: Arc<ProjectConfigManager>) -> Self {
        Self {
            project_manager,
            client: Client::new(),
        }
    }
}

// Request and response structs for Claude chat completion
#[derive(Debug, Deserialize)]
pub struct ClaudeChatRequest {
    pub system_prompt: Option<String>,
    pub messages: Vec<ClaudeMessage>,
    pub model: Option<String>,
    pub max_tokens: Option<u32>,
    pub temperature: Option<f32>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ClaudeMessage {
    pub role: String,
    pub content: String,
}

#[derive(Debug, Serialize)]
pub struct ClaudeChatResponse {
    pub content: String,
    pub model: String,
}

// Anthropic API response structs
#[derive(Debug, Deserialize)]
struct AnthropicResponse {
    content: Vec<AnthropicContent>,
    model: String,
}

#[derive(Debug, Deserialize)]
struct AnthropicContent {
    text: String,
    #[serde(rename = "type")]
    content_type: String,
}

// Handler for Claude chat completion
pub async fn claude_chat_handler(
    data: web::Data<ClaudeMCPAppState>,
    request: web::Json<ClaudeChatRequest>,
) -> Result<HttpResponse, Error> {
    // Get API key from environment
    let api_key = match env::var("ANTHROPIC_API_KEY") {
        Ok(key) => key,
        Err(_) => {
            return Ok(HttpResponse::InternalServerError()
                .json(serde_json::json!({
                    "error": "ANTHROPIC_API_KEY environment variable not set"
                })));
        }
    };

    // Load project configuration to get model preference
    let config = match data.project_manager.load_config() {
        Ok(config) => config,
        Err(e) => {
            return Ok(HttpResponse::InternalServerError()
                .json(serde_json::json!({
                    "error": format!("Failed to load project config: {}", e)
                })));
        }
    };

    // Determine which model to use
    let model = match &request.model {
        Some(model) => model.clone(),
        None => config.anthropic_model.unwrap_or_else(|| DEFAULT_ANTHROPIC_MODEL.to_string()),
    };

    // Prepare the request payload
    let mut payload = serde_json::json!({
        "model": model,
        "messages": request.messages,
        "max_tokens": request.max_tokens.unwrap_or(4096)
    });

    // Add system prompt if provided
    if let Some(system_prompt) = &request.system_prompt {
        payload["system"] = serde_json::Value::String(system_prompt.clone());
    }

    // Add temperature if provided
    if let Some(temperature) = request.temperature {
        payload["temperature"] = serde_json::Value::Number(serde_json::Number::from_f64(temperature as f64).unwrap());
    }

    // Send the request to Anthropic
    let response = match data.client.post(ANTHROPIC_API_URL)
        .header("x-api-key", api_key)
        .header("anthropic-version", "2023-06-01")
        .header("Content-Type", "application/json")
        .json(&payload)
        .send()
        .await {
            Ok(response) => response,
            Err(e) => {
                return Ok(HttpResponse::InternalServerError()
                    .json(serde_json::json!({
                        "error": format!("Failed to send request to Anthropic: {}", e)
                    })));
            }
        };

    // Parse the response
    let response_body = match response.json::<AnthropicResponse>().await {
        Ok(body) => body,
        Err(e) => {
            return Ok(HttpResponse::InternalServerError()
                .json(serde_json::json!({
                    "error": format!("Failed to parse Anthropic response: {}", e)
                })));
        }
    };

    // Extract the content
    let mut content = String::new();
    for item in response_body.content {
        if item.content_type == "text" {
            content = item.text;
            break;
        }
    }

    // Return the response
    Ok(HttpResponse::Ok().json(ClaudeChatResponse {
        content,
        model: response_body.model,
    }))
}

// Handler for Claude model info
pub async fn claude_models_handler(
    data: web::Data<ClaudeMCPAppState>,
) -> Result<HttpResponse, Error> {
    // For now, just return a static list of available Claude models
    let models = vec![
        "claude-sonnet-4-20250514",
        "claude-3-opus-20240229",
        "claude-3-sonnet-20240229",
        "claude-3-haiku-20240307",
    ];

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "models": models
    })))
}