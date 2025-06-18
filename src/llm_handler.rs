use std::env;
use std::sync::Arc;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::json;
use crate::project_config::{
    ProjectConfigManager, PROJECT_CONFIG_FILE,
    DEFAULT_AUTO_COMPLETE_SYSTEM_PROMPT, DEFAULT_AUTO_COMPLETE_USER_PROMPT,
    DEFAULT_ENHANCE_DESCRIPTION_SYSTEM_PROMPT, DEFAULT_ENHANCE_DESCRIPTION_USER_PROMPT,
    DEFAULT_GENERATE_TASKS_SYSTEM_PROMPT, DEFAULT_GENERATE_TASKS_USER_PROMPT,
    DEFAULT_PROCESS_MARKDOWN_SPEC_SYSTEM_PROMPT, DEFAULT_PROCESS_MARKDOWN_SPEC_USER_PROMPT
};

// LLM Provider enum
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum LLMProvider {
    OpenRouter,
    Gemini,
    Anthropic,
}

impl Default for LLMProvider {
    fn default() -> Self {
        LLMProvider::OpenRouter
    }
}

// OpenRouter API configuration
const OPENROUTER_API_URL: &str = "https://openrouter.ai/api/v1/chat/completions";
const DEFAULT_OPENROUTER_MODEL: &str = "google/gemini-2.5-flash-preview-05-20";

// Gemini API configuration
const GEMINI_API_URL: &str = "https://generativelanguage.googleapis.com/v1beta/models/gemini-pro:generateContent";
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
        let project_manager = Arc::new(ProjectConfigManager::new(PROJECT_CONFIG_FILE));

        let config = project_manager.load_config();
        // Load project configuration
        match config {
            Ok(config) => {
                println!("Project configuration loaded successfully from {}", PROJECT_CONFIG_FILE);

                let openrouter_model: Option<String>  = config.openrouter_model;
                let gemini_model: Option<String>  = config.gemini_model;
                let anthropic_model: Option<String>  = config.anthropic_model;

                Self {
                    provider_type,
                    client: Client::new(),
                    openrouter_model,
                    gemini_model,
                    anthropic_model,
                }

            },
            Err(e) => {
                println!("Failed to load project configuration from {}: {}", PROJECT_CONFIG_FILE, e);
                println!("A default configuration will be created when saved for the first time.");

                Self {
                    provider_type,
                    client: Client::new(),
                    openrouter_model: None,
                    gemini_model: None,
                    anthropic_model: None
                }
            }
        }

    }

    pub async fn send_prompt(&self, system_prompt: &str, user_prompt: &str) -> Result<String, String> {
        match self.provider_type {
            LLMProvider::OpenRouter => self.send_openrouter_prompt(system_prompt, user_prompt).await,
            LLMProvider::Gemini => self.send_gemini_prompt(system_prompt, user_prompt).await,
            LLMProvider::Anthropic => self.send_anthropic_prompt(system_prompt, user_prompt).await,
        }
    }

    async fn send_openrouter_prompt(&self, system_prompt: &str, user_prompt: &str) -> Result<String, String> {
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
        let response = self.client.post(OPENROUTER_API_URL)
            .header("Authorization", format!("Bearer {}", api_key))
            .header("Content-Type", "application/json")
            .json(&payload)
            .send()
            .await
            .map_err(|e| format!("Failed to send request to OpenRouter: {}", e))?;

        // Parse the response
        let response_body = response.json::<OpenRouterResponse>()
            .await
            .map_err(|e| format!("Failed to parse OpenRouter response: {}", e))?;

        // Extract the content
        if let Some(choice) = response_body.choices.first() {
            Ok(choice.message.content.clone())
        } else {
            Err("No response from OpenRouter".to_string())
        }
    }

    async fn send_gemini_prompt(&self, system_prompt: &str, user_prompt: &str) -> Result<String, String> {
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
        let url = format!("{}?key={}", GEMINI_API_URL, api_key);
        let response = self.client.post(url)
            .header("Content-Type", "application/json")
            .json(&payload)
            .send()
            .await
            .map_err(|e| format!("Failed to send request to Gemini: {}", e))?;

        // Parse the response
        let response_body = response.json::<GeminiResponse>()
            .await
            .map_err(|e| format!("Failed to parse Gemini response: {}", e))?;

        // Extract the content
        if let Some(candidate) = response_body.candidates.first() {
            if let Some(part) = candidate.content.parts.first() {
                return Ok(part.text.clone());
            }
        }

        Err("No response from Gemini".to_string())
    }

    async fn send_anthropic_prompt(&self, system_prompt: &str, user_prompt: &str) -> Result<String, String> {
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
        let response = self.client.post(ANTHROPIC_API_URL)
            .header("x-api-key", api_key)
            .header("anthropic-version", "2023-06-01")
            .header("Content-Type", "application/json")
            .json(&payload)
            .send()
            .await
            .map_err(|e| format!("Failed to send request to Anthropic: {}", e))?;

        // Parse the response
        let response_body = response.json::<AnthropicResponse>()
            .await
            .map_err(|e| format!("Failed to parse Anthropic response: {}", e))?;

        // Extract the content
        if let Some(content) = response_body.content.first() {
            if content.content_type == "text" {
                return Ok(content.text.clone());
            }
        }

        Err("No response from Anthropic".to_string())
    }
}


pub async fn auto_complete_description(description: &str, provider_type: Option<LLMProvider>,
                                       openrouter_model: Option<String>,
                                       gemini_model: Option<String>,
                                       anthropic_model: Option<String>) -> Result<String, String> {
    let provider = LLMProviderImpl::new(provider_type.unwrap_or_default());

    // Load project configuration to get custom prompts
    let project_manager = Arc::new(ProjectConfigManager::new(PROJECT_CONFIG_FILE));
    let config = project_manager.load_config().map_err(|e| format!("Failed to load project config: {}", e))?;

    // Get system prompt from config or use default
    let system_prompt = config.auto_complete_system_prompt.as_deref().unwrap_or(DEFAULT_AUTO_COMPLETE_SYSTEM_PROMPT);

    // Get user prompt template from config or use default
    let user_prompt_template = config.auto_complete_user_prompt.as_deref().unwrap_or(DEFAULT_AUTO_COMPLETE_USER_PROMPT);

    // Create the user prompt by formatting the template with the description
    let user_prompt = user_prompt_template.replace("{}", description);

    // Send the prompt and return the result
    match provider.provider_type {
        LLMProvider::OpenRouter => {
            provider.send_openrouter_prompt(system_prompt, &user_prompt).await
        },
        LLMProvider::Gemini => {
            provider.send_gemini_prompt(system_prompt, &user_prompt).await
        },
        LLMProvider::Anthropic => {
            provider.send_anthropic_prompt(system_prompt, &user_prompt).await
        },
    }
}

// Function to enhance a block description using LLM
pub async fn enhance_description(description: &str, provider_type: Option<LLMProvider>) -> Result<String, String> {
    let provider = LLMProviderImpl::new(provider_type.unwrap_or_default());

    // Load project configuration to get custom prompts
    let project_manager = Arc::new(ProjectConfigManager::new(PROJECT_CONFIG_FILE));
    let config = project_manager.load_config().map_err(|e| format!("Failed to load project config: {}", e))?;

    // Get system prompt from config or use default
    let system_prompt = config.enhance_description_system_prompt.as_deref().unwrap_or(DEFAULT_ENHANCE_DESCRIPTION_SYSTEM_PROMPT);

    // Get user prompt template from config or use default
    let user_prompt_template = config.enhance_description_user_prompt.as_deref().unwrap_or(DEFAULT_ENHANCE_DESCRIPTION_USER_PROMPT);

    // Create the user prompt by formatting the template with the description
    let user_prompt = user_prompt_template.replace("{}", description);


    // Send the prompt and return the result
    match provider.provider_type {
        LLMProvider::OpenRouter => {
            provider.send_openrouter_prompt(system_prompt, &user_prompt).await
        },
        LLMProvider::Gemini => {
            provider.send_gemini_prompt(system_prompt, &user_prompt).await
        },
        LLMProvider::Anthropic => {
            provider.send_anthropic_prompt(system_prompt, &user_prompt).await
        },
    }
}

// Function to generate tasks for a block based on its description
pub async fn generate_tasks(description: &str, provider_type: Option<LLMProvider>) -> Result<Vec<String>, String> {
    let provider = LLMProviderImpl::new(provider_type.unwrap_or_default());

    // Load project configuration to get custom prompts
    let project_manager = Arc::new(ProjectConfigManager::new(PROJECT_CONFIG_FILE));
    let config = project_manager.load_config().map_err(|e| format!("Failed to load project config: {}", e))?;

    // Get system prompt from config or use default
    let system_prompt = config.generate_tasks_system_prompt.as_deref().unwrap_or(DEFAULT_GENERATE_TASKS_SYSTEM_PROMPT);

    // Get user prompt template from config or use default
    let user_prompt_template = config.generate_tasks_user_prompt.as_deref().unwrap_or(DEFAULT_GENERATE_TASKS_USER_PROMPT);

    // Create the user prompt by formatting the template with the description
    let user_prompt = user_prompt_template.replace("{}", description);


    // Send the prompt and get the response
    let content = match provider.provider_type {
        LLMProvider::OpenRouter => {
            provider.send_openrouter_prompt(system_prompt, &user_prompt).await?
        },
        LLMProvider::Gemini => {
            provider.send_gemini_prompt(system_prompt, &user_prompt).await?
        },
        LLMProvider::Anthropic => {
            provider.send_anthropic_prompt(system_prompt, &user_prompt).await?
        },
    };

    // Parse the content into a list of tasks
    // Simple parsing: split by newlines and filter out empty lines and list markers
    let tasks: Vec<String> = content
        .lines()
        .map(|line| line.trim())
        .filter(|line| !line.is_empty())
        .map(|line| {
            // Remove list markers like "1.", "- ", "* ", etc.
            if line.starts_with(|c: char| c.is_numeric() || c == '-' || c == '*') {
                let mut chars = line.chars();
                chars.next(); // Skip the first character

                // Skip any following characters that are not letters (like ".", ")", " ")
                let mut result = chars.as_str();
                while !result.is_empty() && !result.chars().next().unwrap().is_alphabetic() {
                    result = &result[1..];
                }

                result.trim().to_string()
            } else {
                line.to_string()
            }
        })
        .filter(|line| !line.is_empty())
        .collect();

    Ok(tasks)
}

// // Function to process a markdown file and generate tasks
// pub async fn process_markdown_file(markdown_content: &str, provider_type: Option<LLMProvider>) -> Result<Vec<String>, String> {
//     let provider = LLMProviderImpl::new(provider_type.unwrap_or_default());
// 
//     // Create the system prompt
//     let system_prompt = "You are an expert software developer assistant that helps extract and format tasks from markdown files.";
// 
//     // Create the user prompt
//     let user_prompt = format!(
//         "Process the following markdown file and extract a list of tasks. Format each task as a separate item in a list. If the markdown already contains a list of tasks, extract and format them appropriately:\n\n{}",
//         markdown_content
//     );
// 
//     // Send the prompt and get the response
//     let content = provider.send_prompt(system_prompt, &user_prompt).await?;
// 
//     // Parse the content into a list of tasks
//     // Simple parsing: split by newlines and filter out empty lines and list markers
//     let tasks: Vec<String> = content
//         .lines()
//         .map(|line| line.trim())
//         .filter(|line| !line.is_empty())
//         .map(|line| {
//             // Remove list markers like "1.", "- ", "* ", etc.
//             if line.starts_with(|c: char| c.is_numeric() || c == '-' || c == '*') {
//                 let mut chars = line.chars();
//                 chars.next(); // Skip the first character
// 
//                 // Skip any following characters that are not letters (like ".", ")", " ")
//                 let mut result = chars.as_str();
//                 while !result.is_empty() && !result.chars().next().unwrap().is_alphabetic() {
//                     result = &result[1..];
//                 }
// 
//                 result.trim().to_string()
//             } else {
//                 line.to_string()
//             }
//         })
//         .filter(|line| !line.is_empty())
//         .collect();
// 
//     Ok(tasks)
// }

// Define the structure for a block generated from a markdown specification
#[derive(Serialize, Deserialize, Debug)]
pub struct GeneratedBlock {
    pub name: String,
    pub description: String,
    pub inputs: Vec<String>,
    pub outputs: Vec<String>,
}

// Function to process a markdown specification and generate blocks
pub async fn process_markdown_spec(markdown_content: &str, provider_type: Option<LLMProvider>) -> Result<Vec<GeneratedBlock>, String> {
    let provider = LLMProviderImpl::new(provider_type.unwrap_or_default());

    // Load project configuration to get custom prompts
    let project_manager = Arc::new(ProjectConfigManager::new(PROJECT_CONFIG_FILE));
    let config = project_manager.load_config().map_err(|e| format!("Failed to load project config: {}", e))?;

    // Get system prompt from config or use default
    let system_prompt = config.process_markdown_spec_system_prompt.as_deref().unwrap_or(DEFAULT_PROCESS_MARKDOWN_SPEC_SYSTEM_PROMPT);

    // Get user prompt template from config or use default
    let user_prompt_template = config.process_markdown_spec_user_prompt.as_deref().unwrap_or(DEFAULT_PROCESS_MARKDOWN_SPEC_USER_PROMPT);

    // Create the user prompt by formatting the template with the markdown content
    let user_prompt = user_prompt_template.replace("{}", markdown_content);

    // Send the prompt and get the response
    let content = provider.send_prompt(system_prompt, &user_prompt).await?;

    // Extract the JSON part from the response
    let json_start = content.find('[').unwrap_or(0);
    let json_end = content.rfind(']').map(|i| i + 1).unwrap_or(content.len());
    let json_str = &content[json_start..json_end];

    // Parse the JSON into a list of GeneratedBlock objects
    let blocks: Vec<GeneratedBlock> = serde_json::from_str(json_str)
        .map_err(|e| format!("Failed to parse generated blocks: {}", e))?;

    Ok(blocks)
}
