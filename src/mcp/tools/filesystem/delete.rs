use async_trait::async_trait;
use serde_json::{json, Value};
use std::path::PathBuf;
use tokio::fs;
use tracing::info;

use crate::mcp::tools::{
    Content, ContextUpdate, ExecutionContext, MCPTool, Notification, Permission,
    ToolCategory, ToolError, ToolResult, ToolResultBuilder,
};

/// Delete file or directory tool
pub struct DeleteTool;

#[async_trait]
impl MCPTool for DeleteTool {
    fn name(&self) -> &str {
        "delete"
    }

    fn description(&self) -> &str {
        "Delete a file or directory"
    }

    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "path": {
                    "type": "string",
                    "description": "Path to the file or directory to delete"
                },
                "recursive": {
                    "type": "boolean",
                    "description": "Delete directories recursively",
                    "default": false
                },
                "force": {
                    "type": "boolean",
                    "description": "Force deletion without confirmation",
                    "default": false
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
        let recursive = params["recursive"].as_bool().unwrap_or(false);
        let force = params["force"].as_bool().unwrap_or(false);

        let file_path = PathBuf::from(path);
        let canonical_path = file_path.canonicalize()
            .map_err(|e| ToolError::FileSystem(format!("Failed to resolve path: {}", e)))?;

        // Security check
        if !canonical_path.starts_with(&context.working_directory) {
            return Err(ToolError::PermissionDenied(
                format!("Access denied: path outside working directory: {}", path)
            ));
        }

        // Additional safety check for important directories
        let important_dirs = [".git", "node_modules", "target", "build", "dist"];
        if let Some(file_name) = canonical_path.file_name() {
            if important_dirs.contains(&file_name.to_string_lossy().as_ref()) && !force {
                return Err(ToolError::PermissionDenied(
                    format!("Deletion of '{}' requires force=true for safety", file_name.to_string_lossy())
                ));
            }
        }

        let metadata = fs::metadata(&canonical_path).await
            .map_err(|e| ToolError::FileSystem(format!("Failed to get file metadata: {}", e)))?;

        let is_dir = metadata.is_dir();
        let file_size = metadata.len();

        if is_dir {
            if recursive {
                fs::remove_dir_all(&canonical_path).await
                    .map_err(|e| ToolError::FileSystem(format!("Failed to delete directory: {}", e)))?;
            } else {
                fs::remove_dir(&canonical_path).await
                    .map_err(|e| ToolError::FileSystem(format!("Failed to delete directory: {}", e)))?;
            }
        } else {
            fs::remove_file(&canonical_path).await
                .map_err(|e| ToolError::FileSystem(format!("Failed to delete file: {}", e)))?;
        }

        let context_update = ContextUpdate {
            files_accessed: None,
            files_modified: Some(vec![canonical_path.to_string_lossy().to_string()]),
            git_status: None,
            task_updates: None,
            performance_metrics: None,
            custom_data: Some([
                ("deleted_type".to_string(), json!(if is_dir { "directory" } else { "file" })),
                ("file_size".to_string(), json!(file_size)),
                ("recursive".to_string(), json!(recursive)),
            ].into_iter().collect()),
        };

        let notification = Notification::FileChanged {
            path: canonical_path.to_string_lossy().to_string(),
            change_type: "delete".to_string(),
        };

        info!("Deleted {}: {}", if is_dir { "directory" } else { "file" }, path);

        Ok(ToolResult::success()
            .with_content(Content::Text { 
                text: format!("Successfully deleted {}: {}", if is_dir { "directory" } else { "file" }, path)
            })
            .with_context_update(context_update)
            .with_notification(notification))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_delete_tool() {
        let temp_dir = TempDir::new().unwrap();
        
        // Create a test file to delete
        let test_file = temp_dir.path().join("test.txt");
        fs::write(&test_file, "test content").await.unwrap();
        
        let tool = DeleteTool;
        let params = json!({
            "path": test_file.to_string_lossy()
        });

        // This would need a proper ExecutionContext for a full test
        // For now, we just verify the tool structure
        assert_eq!(tool.name(), "delete");
        assert_eq!(tool.category(), ToolCategory::FileSystem);
        assert!(tool.required_permissions().contains(&Permission::FileWrite));
    }
}