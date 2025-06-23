use async_trait::async_trait;
use serde_json::{json, Value};
use std::path::PathBuf;
use tokio::fs;
use tracing::debug;

use crate::mcp::tools::{
    Content, ContextUpdate, ExecutionContext, MCPTool, Permission,
    ToolCategory, ToolError, ToolResult, ToolResultBuilder,
};

/// Read file contents tool
pub struct ReadFileTool;

#[async_trait]
impl MCPTool for ReadFileTool {
    fn name(&self) -> &str {
        "read_file"
    }

    fn description(&self) -> &str {
        "Read the contents of a file from the file system"
    }

    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "path": {
                    "type": "string",
                    "description": "Path to the file to read"
                },
                "encoding": {
                    "type": "string",
                    "enum": ["utf8", "base64", "binary"],
                    "description": "Encoding to use when reading the file",
                    "default": "utf8"
                },
                "max_size": {
                    "type": "integer",
                    "description": "Maximum file size to read in bytes",
                    "default": 10485760
                }
            },
            "required": ["path"]
        })
    }

    fn required_permissions(&self) -> Vec<Permission> {
        vec![Permission::FileRead]
    }

    fn category(&self) -> ToolCategory {
        ToolCategory::FileSystem
    }

    async fn execute(&self, params: Value, context: &mut ExecutionContext) -> Result<ToolResult, ToolError> {
        let path = params["path"].as_str()
            .ok_or_else(|| ToolError::InvalidParams("Missing 'path' parameter".to_string()))?;
        let encoding = params["encoding"].as_str().unwrap_or("utf8");
        let max_size = params["max_size"].as_u64().unwrap_or(10 * 1024 * 1024); // 10MB default

        let file_path = PathBuf::from(path);

        // Security check: ensure path is within working directory
        let canonical_path = file_path.canonicalize()
            .map_err(|e| ToolError::FileSystem(format!("Failed to resolve path: {}", e)))?;

        if !canonical_path.starts_with(&context.working_directory) {
            return Err(ToolError::PermissionDenied(
                format!("Access denied: path outside working directory: {}", path)
            ));
        }

        // Check file size
        let metadata = fs::metadata(&canonical_path).await
            .map_err(|e| ToolError::FileSystem(format!("Failed to read file metadata: {}", e)))?;

        if metadata.len() > max_size {
            return Err(ToolError::ResourceLimit(
                format!("File size {} exceeds maximum allowed size {}", metadata.len(), max_size)
            ));
        }

        // Read file based on encoding
        let content = match encoding {
            "utf8" => {
                let contents = fs::read_to_string(&canonical_path).await
                    .map_err(|e| ToolError::FileSystem(format!("Failed to read file: {}", e)))?;
                Content::Text { text: contents }
            }
            "base64" => {
                let contents = fs::read(&canonical_path).await
                    .map_err(|e| ToolError::FileSystem(format!("Failed to read file: {}", e)))?;
                let encoded = base64::encode(contents);
                Content::Binary { 
                    data: encoded, 
                    content_type: "application/octet-stream".to_string() 
                }
            }
            "binary" => {
                let contents = fs::read(&canonical_path).await
                    .map_err(|e| ToolError::FileSystem(format!("Failed to read file: {}", e)))?;
                let encoded = base64::encode(contents);
                Content::Binary { 
                    data: encoded, 
                    content_type: mime_guess::from_path(&canonical_path)
                        .first_or_octet_stream()
                        .to_string()
                }
            }
            _ => return Err(ToolError::InvalidParams(format!("Unsupported encoding: {}", encoding))),
        };

        // Create context update
        let context_update = ContextUpdate {
            files_accessed: Some(vec![canonical_path.to_string_lossy().to_string()]),
            files_modified: None,
            git_status: None,
            task_updates: None,
            performance_metrics: None,
            custom_data: Some([
                ("file_size".to_string(), json!(metadata.len())),
                ("file_type".to_string(), json!(encoding)),
            ].into_iter().collect()),
        };

        debug!("Read file: {} ({} bytes)", path, metadata.len());

        Ok(ToolResult::success()
            .with_content(content)
            .with_context_update(context_update))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_read_file_tool() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.txt");
        let content = "Hello, world!";

        fs::write(&file_path, content).await.unwrap();

        let tool = ReadFileTool;
        let params = json!({
            "path": file_path.to_string_lossy()
        });

        // This would need a proper ExecutionContext for a full test
        // For now, we just verify the tool structure
        assert_eq!(tool.name(), "read_file");
        assert_eq!(tool.category(), ToolCategory::FileSystem);
        assert!(tool.required_permissions().contains(&Permission::FileRead));
    }
}