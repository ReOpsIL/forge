use std::env;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::json;

// OpenRouter API configuration
const OPENROUTER_API_URL: &str = "https://openrouter.ai/api/v1/chat/completions";
const MODEL: &str = "google/gemini-2.5-flash-preview-05-20";

// Struct to hold the LLM response
#[derive(Debug, Deserialize)]
struct LLMResponse {
    id: String,
    choices: Vec<Choice>,
}

#[derive(Debug, Deserialize)]
struct Choice {
    message: Message,
}

#[derive(Debug, Deserialize)]
struct Message {
    content: String,
}


pub async fn auto_complete_description(description: &str) -> Result<String, String> {
    let api_key = env::var("OPENROUTER_API_KEY")
        .map_err(|_| "OPENROUTER_API_KEY environment variable not set".to_string())?;

    let client = Client::new();

    // Create the prompt for enhancing the description
    let prompt = format!(
        "You are an expert software architect. Your task is to extend the following block description in few sentences (auto complete), preserving the user's intent. Make it a bit more detailed, rephrase and refine, use simple description:\n\n{}",
        description
    );

    // Create the request payload
    let payload = json!({
        "model": MODEL,
        "messages": [
            {
                "role": "system",
                "content": "You are an expert software architect assistant that helps writing software component descriptions. Use simple descriptions."
            },
            {
                "role": "user",
                "content": prompt
            }
        ]
    });

    // Send the request to OpenRouter
    let response = client.post(OPENROUTER_API_URL)
        .header("Authorization", format!("Bearer {}", api_key))
        .header("Content-Type", "application/json")
        .json(&payload)
        .send()
        .await
        .map_err(|e| format!("Failed to send request to OpenRouter: {}", e))?;

    // Parse the response
    let response_body = response.json::<LLMResponse>()
        .await
        .map_err(|e| format!("Failed to parse OpenRouter response: {}", e))?;

    // Extract the enhanced description
    if let Some(choice) = response_body.choices.first() {
        Ok(choice.message.content.clone())
    } else {
        Err("No response from OpenRouter".to_string())
    }
}

// Function to enhance a block description using OpenRouter LLM
pub async fn enhance_description(description: &str) -> Result<String, String> {
    let api_key = env::var("OPENROUTER_API_KEY")
        .map_err(|_| "OPENROUTER_API_KEY environment variable not set".to_string())?;

    let client = Client::new();

    // Create the prompt for enhancing the description
    let prompt = format!(
        "You are an expert software architect. Your task is to refine, enhance, and expand the following block description, preserving the user's intent. Make it more detailed, clear, and professional:\n\n{}",
        description
    );

    // Create the request payload
    let payload = json!({
        "model": MODEL,
        "messages": [
            {
                "role": "system",
                "content": "You are an expert software architect assistant that helps refine and enhance software component descriptions."
            },
            {
                "role": "user",
                "content": prompt
            }
        ]
    });

    // Send the request to OpenRouter
    let response = client.post(OPENROUTER_API_URL)
        .header("Authorization", format!("Bearer {}", api_key))
        .header("Content-Type", "application/json")
        .json(&payload)
        .send()
        .await
        .map_err(|e| format!("Failed to send request to OpenRouter: {}", e))?;

    // Parse the response
    let response_body = response.json::<LLMResponse>()
        .await
        .map_err(|e| format!("Failed to parse OpenRouter response: {}", e))?;

    // Extract the enhanced description
    if let Some(choice) = response_body.choices.first() {
        Ok(choice.message.content.clone())
    } else {
        Err("No response from OpenRouter".to_string())
    }
}

// Function to generate tasks for a block based on its description
pub async fn generate_tasks(description: &str) -> Result<Vec<String>, String> {
    let api_key = env::var("OPENROUTER_API_KEY")
        .map_err(|_| "OPENROUTER_API_KEY environment variable not set".to_string())?;

    let client = Client::new();

    // Create the prompt for generating tasks
    let prompt = format!(
        "Based on the following software component description, generate a list of concrete, actionable tasks required to implement this functionality. Format each task as a separate item in a list:\n\n{}",
        description
    );

    // Create the request payload
    let payload = json!({
        "model": MODEL,
        "messages": [
            {
                "role": "system",
                "content": "You are an expert software developer assistant that helps break down software components into actionable implementation tasks."
            },
            {
                "role": "user",
                "content": prompt
            }
        ]
    });

    // Send the request to OpenRouter
    let response = client.post(OPENROUTER_API_URL)
        .header("Authorization", format!("Bearer {}", api_key))
        .header("Content-Type", "application/json")
        .json(&payload)
        .send()
        .await
        .map_err(|e| format!("Failed to send request to OpenRouter: {}", e))?;

    // Parse the response
    let response_body = response.json::<LLMResponse>()
        .await
        .map_err(|e| format!("Failed to parse OpenRouter response: {}", e))?;

    // Extract and process the generated tasks
    if let Some(choice) = response_body.choices.first() {
        // Parse the content into a list of tasks
        let content = &choice.message.content;

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
    } else {
        Err("No response from OpenRouter".to_string())
    }
}

// Function to process a markdown file and generate tasks
pub async fn process_markdown_file(markdown_content: &str) -> Result<Vec<String>, String> {
    let api_key = env::var("OPENROUTER_API_KEY")
        .map_err(|_| "OPENROUTER_API_KEY environment variable not set".to_string())?;

    let client = Client::new();

    // Create the prompt for processing the markdown file
    let prompt = format!(
        "Process the following markdown file and extract a list of tasks. Format each task as a separate item in a list. If the markdown already contains a list of tasks, extract and format them appropriately:\n\n{}",
        markdown_content
    );

    // Create the request payload
    let payload = json!({
        "model": MODEL,
        "messages": [
            {
                "role": "system",
                "content": "You are an expert software developer assistant that helps extract and format tasks from markdown files."
            },
            {
                "role": "user",
                "content": prompt
            }
        ]
    });

    // Send the request to OpenRouter
    let response = client.post(OPENROUTER_API_URL)
        .header("Authorization", format!("Bearer {}", api_key))
        .header("Content-Type", "application/json")
        .json(&payload)
        .send()
        .await
        .map_err(|e| format!("Failed to send request to OpenRouter: {}", e))?;

    // Parse the response
    let response_body = response.json::<LLMResponse>()
        .await
        .map_err(|e| format!("Failed to parse OpenRouter response: {}", e))?;

    // Extract and process the generated tasks
    if let Some(choice) = response_body.choices.first() {
        // Parse the content into a list of tasks
        let content = &choice.message.content;

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
    } else {
        Err("No response from OpenRouter".to_string())
    }
}
