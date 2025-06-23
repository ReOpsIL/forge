use async_trait::async_trait;
use serde_json::{json, Value};
use std::path::PathBuf;
use tokio::fs;
use tracing::info;

use crate::mcp::tools::{
    Content, ContextUpdate, ExecutionContext, MCPTool, Permission,
    ToolCategory, ToolError, ToolResult, ToolResultBuilder,
};

/// Create directory tool
pub struct CreateDirectoryTool;

#[async_trait]
impl MCPTool for CreateDirectoryTool {
    fn name(&self) -> &str {
        "create_directory"
    }

    fn description(&self) -> &str {
        "Create a new directory"
    }

    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "path": {
                    "type": "string",
                    "description": "Path of the directory to create"
                },
                "recursive": {
                    "type": "boolean",
                    "description": "Create parent directories if they don't exist",
                    "default": true
                }
            },
            "required": ["path"]
        })
    }

    fn required_permissions(&self) -> Vec<Permission> {
        vec![Permission::FileWrite]
    }

    fn category(&self) -> ToolCategory {
        ToolCategory::FileSystem
    }

    async fn execute(&self, params: Value, context: &mut ExecutionContext) -> Result<ToolResult, ToolError> {
        let path = params["path"].as_str()
            .ok_or_else(|| ToolError::InvalidParams("Missing 'path' parameter".to_string()))?;
        let recursive = params["recursive"].as_bool().unwrap_or(true);

        let dir_path = context.working_directory.join(path);

        // Security check
        let canonical_path = if dir_path.exists() {
            dir_path.canonicalize()
                .map_err(|e| ToolError::FileSystem(format!("Failed to resolve directory path: {}", e)))?
        } else {
            dir_path
        };

        if !canonical_path.starts_with(&context.working_directory) {
            return Err(ToolError::PermissionDenied(
                format!("Access denied: path outside working directory: {}", path)
            ));
        }

        if recursive {
            fs::create_dir_all(&canonical_path).await
                .map_err(|e| ToolError::FileSystem(format!("Failed to create directory: {}", e)))?;
        } else {
            fs::create_dir(&canonical_path).await
                .map_err(|e| ToolError::FileSystem(format!("Failed to create directory: {}", e)))?;
        }

        let context_update = ContextUpdate {
            files_accessed: None,
            files_modified: Some(vec![canonical_path.to_string_lossy().to_string()]),
            git_status: None,
            task_updates: None,
            performance_metrics: None,
            custom_data: Some([
                ("created_path".to_string(), json!(canonical_path.to_string_lossy())),
                ("recursive".to_string(), json!(recursive)),
            ].into_iter().collect()),
        };

        info!("Created directory: {}", path);

        Ok(ToolResult::success()
            .with_content(Content::Text { 
                text: format!("Successfully created directory: {}", path) 
            })
            .with_context_update(context_update))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_create_directory_tool() {
        let temp_dir = TempDir::new().unwrap();
        let dir_path = temp_dir.path().join("new_dir");
        
        let tool = CreateDirectoryTool;
        let params = json!({
            "path": dir_path.to_string_lossy(),
            "recursive": true
        });

        // This would need a proper ExecutionContext for a full test
        // For now, we just verify the tool structure
        assert_eq!(tool.name(), "create_directory");
        assert_eq!(tool.category(), ToolCategory::FileSystem);
        assert!(tool.required_permissions().contains(&Permission::FileWrite));
    }
}