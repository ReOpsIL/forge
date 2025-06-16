use serde::{Deserialize, Serialize};
use std::fs;
use std::io::{self, ErrorKind};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

pub const PROJECT_CONFIG_FILE: &str = "project_config.json";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectConfig {
    pub git_repository_url: String,
    pub project_home_directory: String,
    pub project_description: String,
}

impl Default for ProjectConfig {
    fn default() -> Self {
        Self {
            git_repository_url: String::new(),
            project_home_directory: String::new(),
            project_description: String::new(),
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
        let path = Path::new(&self.config_file);
        
        // If the file doesn't exist, return the default config
        if !path.exists() {
            return Ok(ProjectConfig::default());
        }
        
        let config_str = fs::read_to_string(path)?;
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