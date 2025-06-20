use serde::{Deserialize, Serialize};
use std::{env, fs};
use std::io::{self, ErrorKind};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

pub const PROJECT_CONFIG_FILE: &str = "project_config.json";

// Default prompts for LLM
// Improved prompts for LLM with enhanced specificity and structure

pub const DEFAULT_AUTO_COMPLETE_SYSTEM_PROMPT: &str = "You are a senior software architect specializing in system design and component specification. Your role is to complete partial software component descriptions with technical precision while maintaining clarity and implementability.";

pub const DEFAULT_AUTO_COMPLETE_USER_PROMPT: &str = "Complete the following partial software component description by adding 2-3 sentences that:
1. Clarify the technical implementation approach
2. Specify key interfaces or data structures involved
3. Highlight any important constraints or considerations

Maintain the original intent and technical level. Be specific about technologies, patterns, or frameworks when relevant.

Partial description:
{}

Complete description:";

pub const DEFAULT_ENHANCE_DESCRIPTION_SYSTEM_PROMPT: &str = "You are a technical writing expert specializing in software architecture documentation. Transform brief component descriptions into comprehensive, implementation-ready specifications that developers can directly use for coding.";

pub const DEFAULT_ENHANCE_DESCRIPTION_USER_PROMPT: &str = "Transform the following component description into a detailed, professional specification that includes:

**Required elements:**
- Clear purpose and scope
- Technical implementation approach
- Key interfaces, APIs, or data structures
- Input/output specifications
- Important constraints, dependencies, or assumptions
- Success criteria or acceptance conditions

**Guidelines:**
- Use precise technical language
- Include specific technologies/frameworks when applicable
- Ensure the description is actionable for developers
- Maintain focus on implementation details

Original description:
{}

Enhanced specification:";

pub const DEFAULT_GENERATE_TASKS_SYSTEM_PROMPT: &str = "You are a senior software developer and project manager expert at breaking down software components into granular, executable development tasks. Focus on creating tasks that are specific, measurable, and can be directly implemented by developers.";

pub const DEFAULT_GENERATE_TASKS_USER_PROMPT: &str = "
# JSON Task Generation Prompt
**IMPORTANT: You must respond with valid JSON only. No additional text, explanations, or markdown formatting.**
Based on the software component description below, generate a prioritized list of concrete implementation tasks

**JSON Schema:**
```json
{
  \"component_name\": \"string\",
  \"total_tasks\": number,
  \"tasks\": [
    {
      \"task_id\": string,
      \"task_name\": \"string\",
      \"description\": \"string\", 
      \"acceptance_criteria\": [
        \"string\"
      ],
      \"dependencies\": [
        \"string or task_id\"
      ],
      \"estimated_effort\": \"S|M|L\",
      \"files_affected\": [
        \"string\"
      ],
      \"function_signatures\": [
        \"string\"
      ],
      \"testing_requirements\": [
        \"string\"
      ],
      \"log\": \"\",
      \"commit_id\": \"\",
      \"status\": \"[TODO]\", 
    }
  ]
}
```

**Task Requirements:**
- Specific and actionable (avoid vague terms)
- Estimable in scope (typically 1-8 hours of work)
- Include relevant file names, function signatures, or code locations
- Specify testing requirements where applicable
- Indicate dependencies between tasks using task IDs or descriptive names
- Use effort indicators: S (Simple, 1-3 hours), M (Medium, 3-6 hours), L (Large, 6-8 hours)
- Task ID: task_id should be a random alpha numeric string of 6 characters.

**Component Description:**
{}

**Output Requirements:**
- Return ONLY valid JSON
- No explanatory text before or after the JSON
- Ensure all JSON syntax is correct
- Include 5-15 prioritized tasks
- Tasks should be ordered by implementation priority";

pub const DEFAULT_PROCESS_MARKDOWN_SPEC_SYSTEM_PROMPT: &str = "You are a software architecture analyst expert at parsing technical specifications and extracting structured implementation components. Your output must be valid JSON that can be directly consumed by automated development tools.";

pub const DEFAULT_PROCESS_MARKDOWN_SPEC_USER_PROMPT: &str = "Analyze the following technical specification markdown and extract structured implementation blocks. 

**Output Requirements:**
- Valid JSON array format
- Each block must have clear, implementable descriptions
- Inputs/outputs should specify data types and formats
- Include error handling and validation requirements
- Ensure naming follows consistent conventions
- Block ID: block_id should be a random alpha numeric string of 6 characters.

**JSON Schema:**
```json
{
  \"name\": \"CamelCaseBlockName\",
  \"block_id\": \"sg3gf6\",
  \"description\": \"Detailed implementation description with technical specifics\",
  \"inputs\": [
    {\"name\": \"inputName\", \"ctype\": \"dataType\", \"description\": \"purpose and format\"}
  ],
  \"outputs\": [
    {\"name\": \"outputName\", \"ctype\": \"dataType\", \"description\": \"expected result format\"}
  ],
  \"dependencies\": [\"RequiredComponent1\", \"RequiredComponent2\"]
}
```

**Analysis Guidelines:**
- Extract only implementable components (ignore documentation sections)
- Infer missing technical details from context
- Group related functionality into logical blocks
- Ensure each block is self-contained where possible

Specification document:
{}
";

// Additional helper prompts for common scenarios
pub const DEFAULT_CODE_REVIEW_SYSTEM_PROMPT: &str = "You are a senior code reviewer with expertise in software quality, security, and maintainability. Provide constructive feedback focused on improvements that enhance code reliability and developer productivity.";

pub const DEFAULT_CODE_REVIEW_USER_PROMPT: &str = "Review the following code for:
- **Functionality**: Logic correctness and edge cases
- **Security**: Common vulnerabilities and best practices
- **Performance**: Efficiency and resource usage
- **Maintainability**: Code clarity, documentation, and structure
- **Testing**: Coverage and test quality

Provide specific, actionable recommendations with examples where helpful.

Code to review:
{}

Review feedback:";

pub const DEFAULT_REFACTOR_SYSTEM_PROMPT: &str = "You are a refactoring specialist focused on improving code quality while preserving functionality. Suggest specific improvements that enhance readability, maintainability, and performance.";

pub const DEFAULT_REFACTOR_USER_PROMPT: &str = "Analyze the following code and suggest refactoring improvements:

**Focus Areas:**
- Code structure and organization
- Performance optimizations
- Error handling improvements
- Testing enhancements
- Documentation gaps

**Requirements:**
- Preserve existing functionality
- Provide before/after examples
- Explain the benefits of each suggestion
- Prioritize changes by impact

Code to refactor:
{}

Refactoring suggestions:";


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectConfig {
    pub git_repository_url: String,
    pub project_home_directory: String,
    pub project_description: String,
    pub llm_provider: Option<crate::llm_handler::LLMProvider>,
    pub openrouter_model: Option<String>,
    pub gemini_model: Option<String>,
    pub anthropic_model: Option<String>,

    // Selected profession for prompts
    pub selected_profession_id: Option<String>,

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
            anthropic_model: None,

            // Default profession is software architect
            selected_profession_id: Some("software_architect".to_string()),

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
