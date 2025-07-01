use crate::models::{Block, Task};
use crate::project_config::{
    DEFAULT_AUTO_COMPLETE_SYSTEM_PROMPT, DEFAULT_AUTO_COMPLETE_USER_PROMPT,
    DEFAULT_ENHANCE_DESCRIPTION_SYSTEM_PROMPT, DEFAULT_ENHANCE_DESCRIPTION_USER_PROMPT,
    DEFAULT_GENERATE_TASKS_SYSTEM_PROMPT, DEFAULT_GENERATE_TASKS_SYSTEM_PROMPT_MCP,
    DEFAULT_GENERATE_TASKS_USER_PROMPT, DEFAULT_GENERATE_TASKS_USER_PROMPT_MCP,
    DEFAULT_PROCESS_MARKDOWN_SPEC_SYSTEM_PROMPT_MCP,
    DEFAULT_PROCESS_MARKDOWN_SPEC_USER_PROMPT_MCP,
    PROJECT_CONFIG_FILE, ProjectConfigManager,
};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::env;
use std::process::Stdio;
use tokio::process::Command;
use tracing::{error, info};

// Define the structure for a block generated from a specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockConnection {
    pub name: String,
    pub ctype: String,
    pub description: String,
}

impl BlockConnection {
    pub fn new() -> BlockConnection {
        Self {
            name: String::new(),
            ctype: String::new(),
            description: String::new(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GeneratedBlock {
    pub name: String,
    pub block_id: String,
    pub description: String,
    pub inputs: Vec<BlockConnection>,
    pub outputs: Vec<BlockConnection>,
}

// LLM Provider enum
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum LLMProvider {
    ClaudeCode,
    GeminiCode,
    OpenRouter,
    Gemini,
    Anthropic,
}

impl Default for LLMProvider {
    fn default() -> Self {
        LLMProvider::ClaudeCode
    }
}

// OpenRouter API configuration
const OPENROUTER_API_URL: &str = "https://openrouter.ai/api/v1/chat/completions";
const DEFAULT_OPENROUTER_MODEL: &str = "google/gemini-2.5-flash-preview-05-20";

// Gemini API configuration
const GEMINI_API_URL: &str = "https://generativelanguage.googleapis.com/v1beta";
const DEFAULT_GEMINI_MODEL: &str = "gemini-2.5-flash-preview-05-20";

// Anthropic API configuration
const ANTHROPIC_API_URL: &str = "https://api.anthropic.com/v1/messages";
const DEFAULT_ANTHROPIC_MODEL: &str = "claude-sonnet-4-20250514";

// Function to get the OpenRouter model from the project configuration
fn get_openrouter_model(openrouter_model: Option<&str>) -> &str {
    openrouter_model.unwrap_or(DEFAULT_OPENROUTER_MODEL)
}

// Function to get the Gemini model from the project configuration
fn get_gemini_model(gemini_model: Option<&str>) -> &str {
    gemini_model.unwrap_or(DEFAULT_GEMINI_MODEL)
}

// Function to get the Anthropic model from the project configuration
fn get_anthropic_model(anthropic_model: Option<&str>) -> &str {
    anthropic_model.unwrap_or(DEFAULT_ANTHROPIC_MODEL)
}

// Struct to hold the OpenRouter LLM response
#[derive(Debug, Deserialize)]
struct OpenRouterResponse {
    id: String,
    choices: Vec<OpenRouterChoice>,
}

#[derive(Debug, Deserialize)]
struct OpenRouterChoice {
    message: OpenRouterMessage,
}

#[derive(Debug, Deserialize)]
struct OpenRouterMessage {
    content: String,
}

// Struct to hold the Gemini LLM response
#[derive(Debug, Deserialize)]
struct GeminiResponse {
    candidates: Vec<GeminiCandidate>,
}

#[derive(Debug, Deserialize)]
struct GeminiCandidate {
    content: GeminiContent,
}

#[derive(Debug, Deserialize)]
struct GeminiContent {
    parts: Vec<GeminiPart>,
}

#[derive(Debug, Deserialize)]
struct GeminiPart {
    text: String,
}

// Struct to hold the Anthropic LLM response
#[derive(Debug, Deserialize)]
struct AnthropicResponse {
    content: Vec<AnthropicContent>,
}

#[derive(Debug, Deserialize)]
struct AnthropicContent {
    text: String,
    #[serde(rename = "type")]
    content_type: String,
}

// LLM Provider implementation
pub struct LLMProviderImpl {
    provider_type: LLMProvider,
    client: Client,
    openrouter_model: Option<String>,
    gemini_model: Option<String>,
    anthropic_model: Option<String>,
}

impl LLMProviderImpl {
    pub fn new(provider_type: LLMProvider) -> Self {
        let project_manager = ProjectConfigManager::get_instance();

        let config = project_manager.load_config();
        // Load project configuration
        match config {
            Ok(config) => {
                println!(
                    "Project configuration loaded successfully from {}",
                    PROJECT_CONFIG_FILE
                );

                let openrouter_model: Option<String> = config.openrouter_model;
                let gemini_model: Option<String> = config.gemini_model;
                let anthropic_model: Option<String> = config.anthropic_model;

                Self {
                    provider_type,
                    client: Client::new(),
                    openrouter_model,
                    gemini_model,
                    anthropic_model,
                }
            }
            Err(e) => {
                println!(
                    "Failed to load project configuration from {}: {}",
                    PROJECT_CONFIG_FILE, e
                );
                println!("A default configuration will be created when saved for the first time.");

                Self {
                    provider_type,
                    client: Client::new(),
                    openrouter_model: None,
                    gemini_model: None,
                    anthropic_model: None,
                }
            }
        }
    }
    
    async fn send_openrouter_prompt(
        &self,
        system_prompt: &str,
        user_prompt: &str,
    ) -> Result<String, String> {
        let api_key = env::var("OPENROUTER_API_KEY")
            .map_err(|_| "OPENROUTER_API_KEY environment variable not set".to_string())?;

        // Use the provided model or fall back to the default
        let model_to_use = &self.openrouter_model;

        // Create the request payload
        let payload = json!({
            "model": model_to_use,
            "messages": [
                {
                    "role": "system",
                    "content": system_prompt
                },
                {
                    "role": "user",
                    "content": user_prompt
                }
            ]
        });

        // Send the request to OpenRouter
        let response = self
            .client
            .post(OPENROUTER_API_URL)
            .header("Authorization", format!("Bearer {}", api_key))
            .header("Content-Type", "application/json")
            .json(&payload)
            .send()
            .await
            .map_err(|e| format!("Failed to send request to OpenRouter: {}", e))?;

        // Parse the response
        let response_body = response.json::<OpenRouterResponse>().await.map_err(|e| {
            println!("Failed to parse OpenRouter response: {}", e);
            format!("Failed to parse OpenRouter response: {}", e)
        })?;

        // Extract the content
        if let Some(choice) = response_body.choices.first() {
            Ok(choice.message.content.clone())
        } else {
            println!("No response from OpenRouter");
            Err("No response from OpenRouter".to_string())
        }
    }

    async fn send_gemini_prompt(
        &self,
        system_prompt: &str,
        user_prompt: &str,
    ) -> Result<String, String> {
        let api_key = env::var("GEMINI_API_KEY")
            .map_err(|_| "GEMINI_API_KEY environment variable not set".to_string())?;

        // Use the provided model or fall back to the default
        let model_to_use = &self.gemini_model;

        // Combine system and user prompts for Gemini (as it doesn't have separate roles)
        let combined_prompt = format!("{}\n\n{}", system_prompt, user_prompt);

        // Create the request payload
        let payload = json!({
            "contents": [
                {
                    "parts": [
                        {
                            "text": combined_prompt
                        }
                    ]
                }
            ]
        });

        // Send the request to Gemini
        let url = format!(
            "{}/models/{}:generateContent?key={}",
            GEMINI_API_URL,
            model_to_use.clone().unwrap(),
            api_key
        );
        let response = self
            .client
            .post(url)
            .header("Content-Type", "application/json")
            .json(&payload)
            .send()
            .await
            .map_err(|e| {
                println!("Failed to send request to Gemini: {}", e);
                format!("Failed to send request to Gemini: {}", e)
            })?;

        //println!("{:?}", response);
        // Parse the response
        let response_body = response.json::<GeminiResponse>().await.map_err(|e| {
            println!("Failed to parse Gemini response body: {}", e);
            format!("Failed to parse Gemini response body: {}", e)
        })?;

        // Extract the content
        if let Some(candidate) = response_body.candidates.first() {
            if let Some(part) = candidate.content.parts.first() {
                return Ok(part.text.clone());
            }
        }
        println!("No response from Gemini");
        Err("No response from Gemini".to_string())
    }

    async fn send_anthropic_prompt(
        &self,
        system_prompt: &str,
        user_prompt: &str,
    ) -> Result<String, String> {
        let api_key = env::var("ANTHROPIC_API_KEY")
            .map_err(|_| "ANTHROPIC_API_KEY environment variable not set".to_string())?;

        // Use the provided model or fall back to the default
        let model = get_anthropic_model(self.anthropic_model.as_deref());

        // Create the request payload
        let payload = json!({
            "model": model,
            "messages": [
                {
                    "role": "system",
                    "content": system_prompt
                },
                {
                    "role": "user",
                    "content": user_prompt
                }
            ],
            "max_tokens": 4096
        });

        // Send the request to Anthropic
        let response = self
            .client
            .post(ANTHROPIC_API_URL)
            .header("x-api-key", api_key)
            .header("anthropic-version", "2023-06-01")
            .header("Content-Type", "application/json")
            .json(&payload)
            .send()
            .await
            .map_err(|e| {
                println!("Failed to send request to Anthropic: {}", e);
                format!("Failed to send request to Anthropic: {}", e)
            })?;

        // Parse the response
        let response_body = response.json::<AnthropicResponse>().await.map_err(|e| {
            println!("Failed to parse Anthropic response: {}", e);
            format!("Failed to parse Anthropic response: {}", e)
        })?;

        // Extract the content
        if let Some(content) = response_body.content.first() {
            if content.content_type == "text" {
                return Ok(content.text.clone());
            }
        }
        println!("No response from Anthropic");
        Err("No response from Anthropic".to_string())
    }

    async fn send_claudecode_prompt(
        &self,
        system_prompt: &str,
        user_prompt: &str,
    ) -> Result<String, String> {
        // Create a combined prompt for Claude Code
        let combined_prompt = format!("{}\n\n{}", system_prompt, user_prompt);

        // Execute the claude command with the specified arguments
        let mut command = Command::new("claude");
        command
            .arg("--print")
            .arg("--dangerously-skip-permissions")
            .arg("--output-format")
            .arg("json")
            .arg(combined_prompt)
            //.stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        println!("{:?}", command);
        // Spawn the process
        let child = command
            .spawn()
            .map_err(|e| format!("Failed to execute claude command: {}", e))?;

        // // Write the prompt to stdin
        // if let Some(stdin) = child.stdin.take() {
        //     use tokio::io::AsyncWriteExt;
        //     let mut stdin = stdin;
        //     stdin.write_all(combined_prompt.as_bytes()).await
        //         .map_err(|e| format!("Failed to write to claude stdin: {}", e))?;
        //     stdin.shutdown().await
        //         .map_err(|e| format!("Failed to close claude stdin: {}", e))?;
        // }

        // Wait for the process to complete and capture output
        let output = child
            .wait_with_output()
            .await
            .map_err(|e| format!("Failed to wait for claude process: {}", e))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(format!("Claude command failed: {}", stderr));
        }

        // Parse the stdout as the response
        let stdout = String::from_utf8_lossy(&output.stdout);

        // For ClaudeCode, we expect minimal JSON response since MCP tools handle block/task creation
        // The response should just be the claude output, not comprehensive block/task data
        Ok(stdout.to_string())
    }

    async fn send_geminicode_prompt(
        &self,
        system_prompt: &str,
        user_prompt: &str,
    ) -> Result<String, String> {
        // Create a combined prompt for Claude Code
        let combined_prompt = format!("{}\n\n{}", system_prompt, user_prompt);

        // Execute the claude command with the specified arguments
        let mut command = Command::new("gemini");
        command
            .arg("-p")
            .arg("--dangerously-skip-permissions")
            .arg(combined_prompt)
            //.stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        println!("{:?}", command);
        // Spawn the process
        let child = command
            .spawn()
            .map_err(|e| format!("Failed to execute claude command: {}", e))?;

        // // Write the prompt to stdin
        // if let Some(stdin) = child.stdin.take() {
        //     use tokio::io::AsyncWriteExt;
        //     let mut stdin = stdin;
        //     stdin.write_all(combined_prompt.as_bytes()).await
        //         .map_err(|e| format!("Failed to write to claude stdin: {}", e))?;
        //     stdin.shutdown().await
        //         .map_err(|e| format!("Failed to close claude stdin: {}", e))?;
        // }

        // Wait for the process to complete and capture output
        let output = child
            .wait_with_output()
            .await
            .map_err(|e| format!("Failed to wait for claude process: {}", e))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(format!("Claude command failed: {}", stderr));
        }

        // Parse the stdout as the response
        let stdout = String::from_utf8_lossy(&output.stdout);

        // For ClaudeCode, we expect minimal JSON response since MCP tools handle block/task creation
        // The response should just be the claude output, not comprehensive block/task data
        Ok(stdout.to_string())
    }
}

pub async fn auto_complete_description(
    description: &str,
    provider_type: Option<LLMProvider>,
) -> Result<String, String> {
    let provider = LLMProviderImpl::new(provider_type.unwrap_or_default());

    // Load project configuration to get custom prompts
    let project_manager = ProjectConfigManager::get_instance();
    let config = project_manager
        .load_config()
        .map_err(|e| format!("Failed to load project config: {}", e))?;

    // Get system prompt from config or use default
    let system_prompt = config
        .auto_complete_system_prompt
        .as_deref()
        .unwrap_or(DEFAULT_AUTO_COMPLETE_SYSTEM_PROMPT);

    // Get user prompt template from config or use default
    let user_prompt_template = config
        .auto_complete_user_prompt
        .as_deref()
        .unwrap_or(DEFAULT_AUTO_COMPLETE_USER_PROMPT);

    // Create the user prompt by formatting the template with the description
    let user_prompt = user_prompt_template.replace("{}", description);

    // Send the prompt and return the result
    match provider.provider_type {
        LLMProvider::OpenRouter => {
            provider
                .send_openrouter_prompt(system_prompt, &user_prompt)
                .await
        }
        LLMProvider::Gemini => {
            provider
                .send_gemini_prompt(system_prompt, &user_prompt)
                .await
        }
        LLMProvider::Anthropic => {
            provider
                .send_anthropic_prompt(system_prompt, &user_prompt)
                .await
        }
        LLMProvider::ClaudeCode => {
            provider
                .send_claudecode_prompt(system_prompt, &user_prompt)
                .await
        }
        LLMProvider::GeminiCode => {
            provider
                .send_geminicode_prompt(system_prompt, &user_prompt)
                .await
        }
    }
}

// Function to enhance a block description using LLM
pub async fn enhance_description(
    description: &str,
    provider_type: Option<LLMProvider>,
) -> Result<String, String> {
    let provider = LLMProviderImpl::new(provider_type.unwrap_or_default());

    // Load project configuration to get custom prompts
    let project_manager = ProjectConfigManager::get_instance();
    let config = project_manager
        .load_config()
        .map_err(|e| format!("Failed to load project config: {}", e))?;

    // Get system prompt from config or use default
    let system_prompt = config
        .enhance_description_system_prompt
        .as_deref()
        .unwrap_or(DEFAULT_ENHANCE_DESCRIPTION_SYSTEM_PROMPT);

    // Get user prompt template from config or use default
    let user_prompt_template = config
        .enhance_description_user_prompt
        .as_deref()
        .unwrap_or(DEFAULT_ENHANCE_DESCRIPTION_USER_PROMPT);

    // Create the user prompt by formatting the template with the description
    let user_prompt = user_prompt_template.replace("{}", description);

    // Send the prompt and return the result
    match provider.provider_type {
        LLMProvider::OpenRouter => {
            provider
                .send_openrouter_prompt(system_prompt, &user_prompt)
                .await
        }
        LLMProvider::Gemini => {
            provider
                .send_gemini_prompt(system_prompt, &user_prompt)
                .await
        }
        LLMProvider::Anthropic => {
            provider
                .send_anthropic_prompt(system_prompt, &user_prompt)
                .await
        }
        LLMProvider::ClaudeCode => {
            provider
                .send_claudecode_prompt(system_prompt, &user_prompt)
                .await
        }
        LLMProvider::GeminiCode => {
            provider
                .send_geminicode_prompt(system_prompt, &user_prompt)
                .await
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct TaskResponse {
    pub component_name: String,
    pub total_tasks: u32,
    pub tasks: Vec<Task>,
}

// Function to get the full task response from LLM
pub async fn generate_tasks(
    block: &Block,
    claude_session_manager: &crate::models::ClaudeSessionManager
) -> Result<String, String> {
    // Load project configuration to get custom prompts
    let project_manager = ProjectConfigManager::get_instance();
    let config = project_manager
        .load_config()
        .map_err(|e| format!("Failed to load project config: {}", e))?;


    let system_prompt = config
        .generate_tasks_system_prompt_mcp
        .as_deref()
        .unwrap_or(DEFAULT_GENERATE_TASKS_SYSTEM_PROMPT_MCP);

    // Get user prompt template from config or use default
    let user_prompt_template = config
        .generate_tasks_user_prompt_mcp
        .as_deref()
        .unwrap_or(DEFAULT_GENERATE_TASKS_USER_PROMPT_MCP);

    // Create the user prompt by formatting the template with the description
    let user_prompt = user_prompt_template.replace("{}", &block.description);

    let combined_prompt = format!("{}\n\nCreate the tasks for block id **{}**:\n\n {}", system_prompt, block.block_id, user_prompt);

    // Use claude_session_manager to inject a prompt to running Claude CLI session
    let claude_session_id = "default-claude-session";

    // Ensure Claude session exists or create it
    match claude_session_manager.create_session(claude_session_id.to_string()) {
        Ok(_) => {
            info!("Claude session {} ready for creating tasks", claude_session_id);
        }
        Err(e) => {
            error!("Failed to create/get Claude session {}: {}", claude_session_id, e);
            return Err(format!("Failed to create Claude session: {}", e));
        }
    }

    // Send the prompt to Claude CLI via stdin
    if let Some(session) = claude_session_manager.get_session(claude_session_id) {
        session.update_activity();

        // Send prompt to Claude CLI stdin
        if let Ok(stdin_opt) = session.stdin_tx.lock() {
            if let Some(ref tx) = stdin_opt.as_ref() {
                match tx.send(format!("{}
", combined_prompt)) {
                    Ok(_) => {
                        info!("Successfully sent specification processing prompt to Claude CLI session {}", claude_session_id);
                        // The user will see the output streaming through the WebSocket
                        // The actual execution will be handled by Claude CLI using its MCP tools
                        return Ok("Specification processing prompt sent to Claude CLI session. Check the terminal for output.".to_string())
                    }
                    Err(e) => {
                        error!("Failed to send prompt to Claude CLI session {}: {}", claude_session_id, e);
                        return Err(format!("Failed to send prompt to Claude session: {}", e))
                    }
                }
            } else {
                error!("No stdin channel available for Claude session {}", claude_session_id);
                return Err("No stdin channel available for Claude session".to_string())
            }
        } else {
            error!("Failed to acquire stdin lock for Claude session {}", claude_session_id);
            Err("Failed to acquire stdin lock for Claude session".to_string())
        }
    } else {
        error!("Claude session {} not found", claude_session_id);
        return Err(format!("Claude session {} not found", claude_session_id))
    }
}


// Function to process a specification and generate blocks
pub async fn process_specification(
    markdown_content: &str,
    claude_session_manager: &crate::models::ClaudeSessionManager,
) -> Result<String, String> {
    // Load project configuration to get custom prompts
    let project_manager = ProjectConfigManager::get_instance();
    let config = project_manager
        .load_config()
        .map_err(|e| format!("Failed to load project config: {}", e))?;

    // Get system and user prompts
    let system_prompt = config
        .process_specification_system_prompt_mcp
        .as_deref()
        .unwrap_or(DEFAULT_PROCESS_MARKDOWN_SPEC_SYSTEM_PROMPT_MCP);

    let user_prompt_template = config
        .process_specification_user_prompt_mcp
        .as_deref()
        .unwrap_or(DEFAULT_PROCESS_MARKDOWN_SPEC_USER_PROMPT_MCP);

    // Create the user prompt by formatting the template with the markdown content
    let user_prompt = user_prompt_template.replace("{}", markdown_content);
    
    // Combine system and user prompts
    let combined_prompt = format!("{}\n\n{}", system_prompt, user_prompt);

    // Use claude_session_manager to inject prompt to running Claude CLI session
    let claude_session_id = "default-claude-session";

    // Ensure Claude session exists or create it
    match claude_session_manager.create_session(claude_session_id.to_string()) {
        Ok(_) => {
            info!("Claude session {} ready for specification processing", claude_session_id);
        }
        Err(e) => {
            error!("Failed to create/get Claude session {}: {}", claude_session_id, e);
            return Err(format!("Failed to create Claude session: {}", e));
        }
    }

    // Send the prompt to Claude CLI via stdin
    if let Some(session) = claude_session_manager.get_session(claude_session_id) {
        session.update_activity();

        // Send prompt to Claude CLI stdin
        if let Ok(stdin_opt) = session.stdin_tx.lock() {
            if let Some(ref tx) = stdin_opt.as_ref() {
                match tx.send(format!("{}", combined_prompt)) {
                    Ok(_) => {
                        info!("Successfully sent specification processing prompt to Claude CLI session {}", claude_session_id);
                        // The user will see the output streaming through the WebSocket
                        // The actual execution will be handled by Claude CLI using its MCP tools
                        Ok("Specification processing prompt sent to Claude CLI session. Check the terminal for output.".to_string())
                    }
                    Err(e) => {
                        error!("Failed to send prompt to Claude CLI session {}: {}", claude_session_id, e);
                        Err(format!("Failed to send prompt to Claude session: {}", e))
                    }
                }
            } else {
                error!("No stdin channel available for Claude session {}", claude_session_id);
                Err("No stdin channel available for Claude session".to_string())
            }
        } else {
            error!("Failed to acquire stdin lock for Claude session {}", claude_session_id);
            Err("Failed to acquire stdin lock for Claude session".to_string())
        }
    } else {
        error!("Claude session {} not found", claude_session_id);
        Err(format!("Claude session {} not found", claude_session_id))
    }
}

