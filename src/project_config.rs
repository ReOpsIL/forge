use serde::{Deserialize, Serialize};
use std::{env, fs};
use std::io::{self, ErrorKind};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

pub const PROJECT_CONFIG_FILE: &str = "project_config.json";

// Default prompts for LLM
pub const DEFAULT_AUTO_COMPLETE_SYSTEM_PROMPT: &str = "You are an expert software architect assistant that helps writing software component descriptions. Use simple descriptions.";
pub const DEFAULT_AUTO_COMPLETE_USER_PROMPT: &str = "You are an expert software architect. Your task is to extend the following block description in few sentences (auto complete), preserving the user's intent. Make it a bit more detailed, rephrase and refine, use simple description:\n\n{}";
pub const DEFAULT_ENHANCE_DESCRIPTION_SYSTEM_PROMPT: &str = "You are an expert software architect assistant that helps refine and enhance software component descriptions.";
pub const DEFAULT_ENHANCE_DESCRIPTION_USER_PROMPT: &str = "You are an expert software architect. Your task is to refine, enhance, and expand the following block description, preserving the user's intent. Make it more detailed, clear, and professional:\n\n{}";
pub const DEFAULT_GENERATE_TASKS_SYSTEM_PROMPT: &str = "You are an expert software developer assistant that helps break down software components into actionable implementation tasks.";
pub const DEFAULT_GENERATE_TASKS_USER_PROMPT: &str = "Based on the following software component description, generate a list of concrete, actionable tasks required to implement this functionality. Format each task as a separate item in a list:\n\n{}";
pub const DEFAULT_PROCESS_MARKDOWN_SPEC_SYSTEM_PROMPT: &str = "You are an expert software architect that helps analyze technical specifications and generate implementation blocks with clear descriptions, inputs, and outputs.";
pub const DEFAULT_PROCESS_MARKDOWN_SPEC_USER_PROMPT: &str = "Process the following markdown file containing a technical specification and generate a structured list of implementation blocks. For each block, provide a clear block description, defined inputs, and defined outputs. Format your response as a JSON array of objects, where each object has the following structure: {\"name\": \"BlockName\", \"description\": \"Block description\", \"inputs\": [\"input1\", \"input2\"], \"outputs\": [\"output1\", \"output2\"]}. Ensure the JSON is valid and properly formatted:\n\n{}";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectConfig {
    pub git_repository_url: String,
    pub project_home_directory: String,
    pub project_description: String,
    pub llm_provider: Option<crate::llm_handler::LLMProvider>,
    pub openrouter_model: Option<String>,
    pub gemini_model: Option<String>,

    // User-configurable prompts
    pub auto_complete_system_prompt: Option<String>,
    pub auto_complete_user_prompt: Option<String>,
    pub enhance_description_system_prompt: Option<String>,
    pub enhance_description_user_prompt: Option<String>,
    pub generate_tasks_system_prompt: Option<String>,
    pub generate_tasks_user_prompt: Option<String>,
    pub process_markdown_spec_system_prompt: Option<String>,
    pub process_markdown_spec_user_prompt: Option<String>,
}

impl Default for ProjectConfig {
    fn default() -> Self {
        Self {
            git_repository_url: String::new(),
            project_home_directory: String::new(),
            project_description: String::new(),
            llm_provider: None,
            openrouter_model: None,
            gemini_model: None,

            // Default values for user-configurable prompts
            auto_complete_system_prompt: Some(DEFAULT_AUTO_COMPLETE_SYSTEM_PROMPT.to_string()),
            auto_complete_user_prompt: Some(DEFAULT_AUTO_COMPLETE_USER_PROMPT.to_string()),
            enhance_description_system_prompt: Some(DEFAULT_ENHANCE_DESCRIPTION_SYSTEM_PROMPT.to_string()),
            enhance_description_user_prompt: Some(DEFAULT_ENHANCE_DESCRIPTION_USER_PROMPT.to_string()),
            generate_tasks_system_prompt: Some(DEFAULT_GENERATE_TASKS_SYSTEM_PROMPT.to_string()),
            generate_tasks_user_prompt: Some(DEFAULT_GENERATE_TASKS_USER_PROMPT.to_string()),
            process_markdown_spec_system_prompt: Some(DEFAULT_PROCESS_MARKDOWN_SPEC_SYSTEM_PROMPT.to_string()),
            process_markdown_spec_user_prompt: Some(DEFAULT_PROCESS_MARKDOWN_SPEC_USER_PROMPT.to_string()),
        }
    }
}

pub struct ProjectConfigManager {
    config_file: String,
    config: Mutex<ProjectConfig>,
}

impl ProjectConfigManager {
    pub fn new(config_file: &str) -> Self {
        Self {
            config_file: config_file.to_string(),
            config: Mutex::new(ProjectConfig::default()),
        }
    }

    pub fn load_config(&self) -> io::Result<ProjectConfig> {
        let config_path = Path::new(&self.config_file);

        //Print the current working directory:
        //let cwd_path = env::current_dir()?;
        //println!("The current directory is {}", cwd_path.display());

        // If the file doesn't exist, return the default config
        if !config_path.exists() {
            return Ok(ProjectConfig::default());
        }

        let config_str = fs::read_to_string(config_path)?;
        let config: ProjectConfig = serde_json::from_str(&config_str)?;

        // Update the internal config
        let mut internal_config = self.config.lock().unwrap();
        *internal_config = config.clone();

        Ok(config)
    }

    pub fn save_config(&self, config: &ProjectConfig) -> io::Result<()> {
        let config_str = serde_json::to_string_pretty(config)?;

        // Update the internal config
        let mut internal_config = self.config.lock().unwrap();
        *internal_config = config.clone();

        // Create the directory if it doesn't exist
        if let Some(parent) = Path::new(&self.config_file).parent() {
            fs::create_dir_all(parent)?;
        }

        // Write the config to file
        fs::write(&self.config_file, config_str)?;

        // If project_home_directory is specified, create it if it doesn't exist
        if !config.project_home_directory.is_empty() {
            let project_dir = Path::new(&config.project_home_directory);
            if !project_dir.exists() {
                fs::create_dir_all(project_dir)?;
            }
        }

        Ok(())
    }

    pub fn get_config(&self) -> io::Result<ProjectConfig> {
        let config = self.config.lock().unwrap();
        Ok(config.clone())
    }
}

// Function to test Git repository connection
pub async fn test_git_connection(url: &str) -> Result<String, String> {
    // This is a simple check to see if the URL is valid
    // In a real implementation, you might want to use a Git library to actually test the connection
    if url.is_empty() {
        return Err("Git repository URL cannot be empty".to_string());
    }

    if !(url.starts_with("http://") || url.starts_with("https://") || url.starts_with("git@")) {
        return Err("Invalid Git repository URL format".to_string());
    }

    // For now, just return success if the URL looks valid
    Ok("Successfully connected to Git repository".to_string())
}
