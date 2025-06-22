/// File system tools for MCP - comprehensive file operations
/// 
/// This module provides a comprehensive set of file system tools that enable
/// Claude Code to interact with the project file system through structured
/// MCP calls instead of brittle CLI commands.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::path::{Path, PathBuf};
use tokio::fs;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tracing::{debug, info, warn};


use crate::mcp::tools::{
    Content, ContextUpdate, ExecutionContext, MCPTool, Notification, Permission,
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
    
    async fn execute(&self, params: Value, context: &mut ExecutionContext) -> Result<ToolResult, ToolError> {
        let path = params["path"].as_str()
            .ok_or_else(|| ToolError::InvalidParams("Missing 'path' parameter".to_string()))?;
        let content = params["content"].as_str()
            .ok_or_else(|| ToolError::InvalidParams("Missing 'content' parameter".to_string()))?;
        let encoding = params["encoding"].as_str().unwrap_or("utf8");
        let create_dirs = params["create_dirs"].as_bool().unwrap_or(true);
        let backup = params["backup"].as_bool().unwrap_or(false);
        
        let file_path = PathBuf::from(path);
        
        // Security check: ensure path is within working directory
        let parent_dir = file_path.parent()
            .ok_or_else(|| ToolError::FileSystem("Invalid file path".to_string()))?;
        
        let canonical_parent = if parent_dir.exists() {
            parent_dir.canonicalize()
                .map_err(|e| ToolError::FileSystem(format!("Failed to resolve parent directory: {}", e)))?
        } else if create_dirs {
            context.working_directory.join(parent_dir)
        } else {
            return Err(ToolError::FileSystem("Parent directory does not exist".to_string()));
        };
        
        if !canonical_parent.starts_with(&context.working_directory) {
            return Err(ToolError::PermissionDenied(
                format!("Access denied: path outside working directory: {}", path)
            ));
        }
        
        let canonical_path = canonical_parent.join(file_path.file_name().unwrap());
        
        // Create parent directories if needed
        if create_dirs {
            if let Some(parent) = canonical_path.parent() {
                fs::create_dir_all(parent).await
                    .map_err(|e| ToolError::FileSystem(format!("Failed to create directories: {}", e)))?;
            }
        }
        
        // Create backup if requested and file exists
        let mut backup_path = None;
        if backup && canonical_path.exists() {
            let backup_file = format!("{}.backup.{}", 
                canonical_path.to_string_lossy(), 
                chrono::Utc::now().format("%Y%m%d_%H%M%S")
            );
            backup_path = Some(PathBuf::from(&backup_file));
            fs::copy(&canonical_path, &backup_file).await
                .map_err(|e| ToolError::FileSystem(format!("Failed to create backup: {}", e)))?;
        }
        
        // Write content based on encoding
        let bytes_written = match encoding {
            "utf8" => {
                fs::write(&canonical_path, content).await
                    .map_err(|e| ToolError::FileSystem(format!("Failed to write file: {}", e)))?;
                content.len()
            }
            "base64" => {
                let decoded = base64::decode(content)
                    .map_err(|e| ToolError::InvalidParams(format!("Invalid base64 content: {}", e)))?;
                let len = decoded.len();
                fs::write(&canonical_path, decoded).await
                    .map_err(|e| ToolError::FileSystem(format!("Failed to write file: {}", e)))?;
                len
            }
            _ => return Err(ToolError::InvalidParams(format!("Unsupported encoding: {}", encoding))),
        };
        
        // Create context update
        let mut context_update = ContextUpdate {
            files_accessed: None,
            files_modified: Some(vec![canonical_path.to_string_lossy().to_string()]),
            git_status: None,
            task_updates: None,
            performance_metrics: None,
            custom_data: Some([
                ("bytes_written".to_string(), json!(bytes_written)),
                ("encoding".to_string(), json!(encoding)),
            ].into_iter().collect()),
        };
        
        if let Some(backup_path) = backup_path {
            context_update.custom_data.as_mut().unwrap().insert(
                "backup_created".to_string(), 
                json!(backup_path.to_string_lossy())
            );
        }
        
        let notification = Notification::FileChanged {
            path: canonical_path.to_string_lossy().to_string(),
            change_type: "write".to_string(),
        };
        
        info!("Wrote file: {} ({} bytes)", path, bytes_written);
        
        Ok(ToolResult::success()
            .with_content(Content::Text { 
                text: format!("Successfully wrote {} bytes to {}", bytes_written, path) 
            })
            .with_context_update(context_update)
            .with_notification(notification))
    }
}

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
    
    fn required_permissions(&self) -> Vec<Permission> {
        vec![Permission::FileRead]
    }
    
    fn category(&self) -> ToolCategory {
        ToolCategory::FileSystem
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
        
        Ok(ToolResult::success()
            .with_content(Content::Data { data: json!(entries) })
            .with_context_update(context_update))
    }
}

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

// Helper functions

async fn list_directory_single(
    path: &Path, 
    include_hidden: bool, 
    filter_pattern: Option<&str>
) -> Result<Vec<Value>, ToolError> {
    let mut entries = Vec::new();
    let mut dir = fs::read_dir(path).await
        .map_err(|e| ToolError::FileSystem(format!("Failed to read directory: {}", e)))?;
    
    while let Some(entry) = dir.next_entry().await
        .map_err(|e| ToolError::FileSystem(format!("Failed to read directory entry: {}", e)))? {
        
        let file_name = entry.file_name();
        let file_name_str = file_name.to_string_lossy();
        
        // Skip hidden files if not requested
        if !include_hidden && file_name_str.starts_with('.') {
            continue;
        }
        
        // Apply filter if provided
        if let Some(pattern) = filter_pattern {
            if !glob_match(pattern, &file_name_str) {
                continue;
            }
        }
        
        let metadata = entry.metadata().await
            .map_err(|e| ToolError::FileSystem(format!("Failed to read metadata: {}", e)))?;
        
        let entry_info = json!({
            "name": file_name_str,
            "path": entry.path().to_string_lossy(),
            "type": if metadata.is_dir() { "directory" } else { "file" },
            "size": metadata.len(),
            "modified": metadata.modified()
                .map(|t| t.duration_since(std::time::UNIX_EPOCH).unwrap().as_secs())
                .unwrap_or(0),
            "permissions": format!("{:?}", metadata.permissions())
        });
        
        entries.push(entry_info);
    }
    
    Ok(entries)
}

async fn list_directory_recursive(
    path: &Path, 
    max_depth: usize, 
    include_hidden: bool, 
    filter_pattern: Option<&str>
) -> Result<Vec<Value>, ToolError> {
    fn collect_entries(
        entries: &mut Vec<Value>, 
        path: &Path, 
        current_depth: usize, 
        max_depth: usize,
        include_hidden: bool,
        filter_pattern: Option<&str>
    ) -> Result<(), ToolError> {
        if current_depth > max_depth {
            return Ok(());
        }
        
        let dir = std::fs::read_dir(path)
            .map_err(|e| ToolError::FileSystem(format!("Failed to read directory: {}", e)))?;
        
        for entry in dir {
            let entry = entry
                .map_err(|e| ToolError::FileSystem(format!("Failed to read directory entry: {}", e)))?;
            
            let file_name = entry.file_name();
            let file_name_str = file_name.to_string_lossy();
            
            // Skip hidden files if not requested
            if !include_hidden && file_name_str.starts_with('.') {
                continue;
            }
            
            // Apply filter if provided
            if let Some(pattern) = filter_pattern {
                if !glob_match(pattern, &file_name_str) && !entry.path().is_dir() {
                    continue;
                }
            }
            
            let metadata = entry.metadata()
                .map_err(|e| ToolError::FileSystem(format!("Failed to read metadata: {}", e)))?;
            
            let entry_info = json!({
                "name": file_name_str,
                "path": entry.path().to_string_lossy(),
                "type": if metadata.is_dir() { "directory" } else { "file" },
                "size": metadata.len(),
                "modified": metadata.modified()
                    .map(|t| t.duration_since(std::time::UNIX_EPOCH).unwrap().as_secs())
                    .unwrap_or(0),
                "depth": current_depth
            });
            
            entries.push(entry_info);
            
            // Recurse into directories
            if metadata.is_dir() {
                collect_entries(entries, &entry.path(), current_depth + 1, max_depth, include_hidden, filter_pattern)?;
            }
        }
        
        Ok(())
    }
    
    let mut entries = Vec::new();
    collect_entries(&mut entries, path, 0, max_depth, include_hidden, filter_pattern)?;
    Ok(entries)
}

fn glob_match(pattern: &str, text: &str) -> bool {
    // Simple glob matching - supports * and ?
    let pattern_chars: Vec<char> = pattern.chars().collect();
    let text_chars: Vec<char> = text.chars().collect();
    
    fn match_recursive(pattern: &[char], text: &[char], p_idx: usize, t_idx: usize) -> bool {
        if p_idx >= pattern.len() {
            return t_idx >= text.len();
        }
        
        match pattern[p_idx] {
            '*' => {
                // Try matching zero or more characters
                for i in t_idx..=text.len() {
                    if match_recursive(pattern, text, p_idx + 1, i) {
                        return true;
                    }
                }
                false
            }
            '?' => {
                // Match exactly one character
                if t_idx < text.len() {
                    match_recursive(pattern, text, p_idx + 1, t_idx + 1)
                } else {
                    false
                }
            }
            c => {
                // Match exact character
                if t_idx < text.len() && text[t_idx] == c {
                    match_recursive(pattern, text, p_idx + 1, t_idx + 1)
                } else {
                    false
                }
            }
        }
    }
    
    match_recursive(&pattern_chars, &text_chars, 0, 0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_glob_match() {
        assert!(glob_match("*.rs", "main.rs"));
        assert!(glob_match("test*.txt", "test123.txt"));
        assert!(glob_match("?.txt", "a.txt"));
        assert!(!glob_match("*.rs", "main.py"));
        assert!(!glob_match("test*.txt", "other.txt"));
    }

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