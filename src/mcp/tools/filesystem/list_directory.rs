use async_trait::async_trait;
use serde_json::{json, Value};
use std::path::PathBuf;
use tokio::fs;
use tracing::debug;

use crate::mcp::tools::{
    Content, ContextUpdate, ExecutionContext, MCPTool, Permission,
    ToolCategory, ToolError, ToolResult, ToolResultBuilder,
};

use super::helpers::{list_directory_recursive, list_directory_single};

/// List directory contents tool
pub struct ListDirectoryTool;

#[async_trait]
impl MCPTool for ListDirectoryTool {
    fn name(&self) -> &str {
        "list_directory"
    }

    fn description(&self) -> &str {
        "List the contents of a directory"
    }

    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "path": {
                    "type": "string",
                    "description": "Path to the directory to list",
                    "default": "."
                },
                "recursive": {
                    "type": "boolean",
                    "description": "Whether to list recursively",
                    "default": false
                },
                "include_hidden": {
                    "type": "boolean",
                    "description": "Whether to include hidden files",
                    "default": false
                },
                "max_depth": {
                    "type": "integer",
                    "description": "Maximum recursion depth",
                    "default": 5
                },
                "filter": {
                    "type": "string",
                    "description": "Glob pattern to filter files (e.g., '*.rs')"
                }
            }
        })
    }

    async fn execute(&self, params: Value, context: &mut ExecutionContext) -> Result<ToolResult, ToolError> {
        let path = params["path"].as_str().unwrap_or(".");
        let recursive = params["recursive"].as_bool().unwrap_or(false);
        let include_hidden = params["include_hidden"].as_bool().unwrap_or(false);
        let max_depth = params["max_depth"].as_u64().unwrap_or(5) as usize;
        let filter_pattern = params["filter"].as_str();

        let dir_path = PathBuf::from(path);
        let canonical_path = dir_path.canonicalize()
            .map_err(|e| ToolError::FileSystem(format!("Failed to resolve directory path: {}", e)))?;

        // Security check
        if !canonical_path.starts_with(&context.working_directory) {
            return Err(ToolError::PermissionDenied(
                format!("Access denied: path outside working directory: {}", path)
            ));
        }

        let entries = if recursive {
            list_directory_recursive(&canonical_path, max_depth, include_hidden, filter_pattern).await?
        } else {
            list_directory_single(&canonical_path, include_hidden, filter_pattern).await?
        };

        let context_update = ContextUpdate {
            files_accessed: Some(vec![canonical_path.to_string_lossy().to_string()]),
            files_modified: None,
            git_status: None,
            task_updates: None,
            performance_metrics: None,
            custom_data: Some([
                ("entries_count".to_string(), json!(entries.len())),
                ("recursive".to_string(), json!(recursive)),
            ].into_iter().collect()),
        };

        debug!("Listed directory: {} ({} entries)", path, entries.len());

        // Convert entries to a formatted text representation
        let formatted_entries = serde_json::to_string_pretty(&entries)
            .map_err(|e| ToolError::Internal(format!("Failed to format directory entries: {}", e)))?;

        Ok(ToolResult::success()
            .with_content(Content::Text { text: formatted_entries })
            .with_context_update(context_update))
    }

    fn required_permissions(&self) -> Vec<Permission> {
        vec![Permission::FileRead]
    }

    fn category(&self) -> ToolCategory {
        ToolCategory::FileSystem
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_list_directory_tool() {
        let temp_dir = TempDir::new().unwrap();
        
        // Create a test file
        let test_file = temp_dir.path().join("test.txt");
        fs::write(&test_file, "test content").await.unwrap();
        
        let tool = ListDirectoryTool;
        let params = json!({
            "path": temp_dir.path().to_string_lossy()
        });

        // This would need a proper ExecutionContext for a full test
        // For now, we just verify the tool structure
        assert_eq!(tool.name(), "list_directory");
        assert_eq!(tool.category(), ToolCategory::FileSystem);
        assert!(tool.required_permissions().contains(&Permission::FileRead));
    }
}