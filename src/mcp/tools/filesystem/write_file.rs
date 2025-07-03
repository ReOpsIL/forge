use async_trait::async_trait;
use serde_json::{json, Value};
use std::path::PathBuf;
use tokio::fs;
use tracing::info;

use crate::mcp::tools::{
    Content, ContextUpdate, ExecutionContext, MCPTool, Notification, Permission, ToolCategory,
    ToolError, ToolResult, ToolResultBuilder,
};

/// Write file contents tool
pub struct WriteFileTool;

#[async_trait]
impl MCPTool for WriteFileTool {
    fn name(&self) -> &str {
        "write_file"
    }

    fn description(&self) -> &str {
        "Write content to a file in the file system"
    }

    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "path": {
                    "type": "string",
                    "description": "Path to the file to write"
                },
                "content": {
                    "type": "string",
                    "description": "Content to write to the file"
                },
                "encoding": {
                    "type": "string",
                    "enum": ["utf8", "base64"],
                    "description": "Encoding of the content",
                    "default": "utf8"
                },
                "create_dirs": {
                    "type": "boolean",
                    "description": "Create parent directories if they don't exist",
                    "default": true
                },
                "backup": {
                    "type": "boolean",
                    "description": "Create a backup of existing file",
                    "default": false
                }
            },
            "required": ["path", "content"]
        })
    }

    fn required_permissions(&self) -> Vec<Permission> {
        vec![Permission::FileWrite]
    }

    fn category(&self) -> ToolCategory {
        ToolCategory::FileSystem
    }

    async fn execute(
        &self,
        params: Value,
        context: &mut ExecutionContext,
    ) -> Result<ToolResult, ToolError> {
        let path = params["path"]
            .as_str()
            .ok_or_else(|| ToolError::InvalidParams("Missing 'path' parameter".to_string()))?;
        let content = params["content"]
            .as_str()
            .ok_or_else(|| ToolError::InvalidParams("Missing 'content' parameter".to_string()))?;
        let encoding = params["encoding"].as_str().unwrap_or("utf8");
        let create_dirs = params["create_dirs"].as_bool().unwrap_or(true);
        let backup = params["backup"].as_bool().unwrap_or(false);

        let file_path = PathBuf::from(path);

        // Security check: ensure path is within working directory
        let parent_dir = file_path
            .parent()
            .ok_or_else(|| ToolError::FileSystem("Invalid file path".to_string()))?;

        let canonical_parent = if parent_dir.exists() {
            parent_dir.canonicalize().map_err(|e| {
                ToolError::FileSystem(format!("Failed to resolve parent directory: {}", e))
            })?
        } else if create_dirs {
            context.working_directory.join(parent_dir)
        } else {
            return Err(ToolError::FileSystem(
                "Parent directory does not exist".to_string(),
            ));
        };

        if !canonical_parent.starts_with(&context.working_directory) {
            return Err(ToolError::PermissionDenied(format!(
                "Access denied: path outside working directory: {}",
                path
            )));
        }

        let canonical_path = canonical_parent.join(file_path.file_name().unwrap());

        // Create parent directories if needed
        if create_dirs {
            if let Some(parent) = canonical_path.parent() {
                fs::create_dir_all(parent).await.map_err(|e| {
                    ToolError::FileSystem(format!("Failed to create directories: {}", e))
                })?;
            }
        }

        // Create backup if requested and file exists
        let mut backup_path = None;
        if backup && canonical_path.exists() {
            let backup_file = format!(
                "{}.backup.{}",
                canonical_path.to_string_lossy(),
                chrono::Utc::now().format("%Y%m%d_%H%M%S")
            );
            backup_path = Some(PathBuf::from(&backup_file));
            fs::copy(&canonical_path, &backup_file)
                .await
                .map_err(|e| ToolError::FileSystem(format!("Failed to create backup: {}", e)))?;
        }

        // Write content based on encoding
        let bytes_written = match encoding {
            "utf8" => {
                fs::write(&canonical_path, content)
                    .await
                    .map_err(|e| ToolError::FileSystem(format!("Failed to write file: {}", e)))?;
                content.len()
            }
            "base64" => {
                let decoded = base64::decode(content).map_err(|e| {
                    ToolError::InvalidParams(format!("Invalid base64 content: {}", e))
                })?;
                let len = decoded.len();
                fs::write(&canonical_path, decoded)
                    .await
                    .map_err(|e| ToolError::FileSystem(format!("Failed to write file: {}", e)))?;
                len
            }
            _ => {
                return Err(ToolError::InvalidParams(format!(
                    "Unsupported encoding: {}",
                    encoding
                )));
            }
        };

        // Create context update
        let mut context_update = ContextUpdate {
            files_accessed: None,
            files_modified: Some(vec![canonical_path.to_string_lossy().to_string()]),
            git_status: None,
            task_updates: None,
            performance_metrics: None,
            custom_data: Some(
                [
                    ("bytes_written".to_string(), json!(bytes_written)),
                    ("encoding".to_string(), json!(encoding)),
                ]
                    .into_iter()
                    .collect(),
            ),
        };

        if let Some(backup_path) = backup_path {
            context_update.custom_data.as_mut().unwrap().insert(
                "backup_created".to_string(),
                json!(backup_path.to_string_lossy()),
            );
        }

        let notification = Notification::FileChanged {
            path: canonical_path.to_string_lossy().to_string(),
            change_type: "write".to_string(),
        };

        info!("Wrote file: {} ({} bytes)", path, bytes_written);

        Ok(ToolResult::success()
            .with_content(Content::Text {
                text: format!("Successfully wrote {} bytes to {}", bytes_written, path),
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
    async fn test_write_file_tool() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.txt");
        let content = "Hello, world!";

        let tool = WriteFileTool;
        let params = json!({
            "path": file_path.to_string_lossy(),
            "content": content
        });

        // This would need a proper ExecutionContext for a full test
        // For now, we just verify the tool structure
        assert_eq!(tool.name(), "write_file");
        assert_eq!(tool.category(), ToolCategory::FileSystem);
        assert!(tool.required_permissions().contains(&Permission::FileWrite));
    }
}
