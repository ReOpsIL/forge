use async_trait::async_trait;
use serde_json::{Value, json};
use std::path::{Path, PathBuf};
use tokio::fs;
use tracing::debug;

use crate::mcp::tools::{
    Content, ContextUpdate, ExecutionContext, MCPTool, Permission, ToolCategory, ToolError,
    ToolResult, ToolResultBuilder,
};

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

    async fn execute(
        &self,
        params: Value,
        context: &mut ExecutionContext,
    ) -> Result<ToolResult, ToolError> {
        let path = params["path"].as_str().unwrap_or(".");
        let recursive = params["recursive"].as_bool().unwrap_or(false);
        let include_hidden = params["include_hidden"].as_bool().unwrap_or(false);
        let max_depth = params["max_depth"].as_u64().unwrap_or(5) as usize;
        let filter_pattern = params["filter"].as_str();

        let dir_path = PathBuf::from(path);
        let canonical_path = dir_path.canonicalize().map_err(|e| {
            ToolError::FileSystem(format!("Failed to resolve directory path: {}", e))
        })?;

        // Security check
        if !canonical_path.starts_with(&context.working_directory) {
            return Err(ToolError::PermissionDenied(format!(
                "Access denied: path outside working directory: {}",
                path
            )));
        }

        let entries = if recursive {
            list_directory_recursive(&canonical_path, max_depth, include_hidden, filter_pattern)
                .await?
        } else {
            list_directory_single(&canonical_path, include_hidden, filter_pattern).await?
        };

        let context_update = ContextUpdate {
            files_accessed: Some(vec![canonical_path.to_string_lossy().to_string()]),
            files_modified: None,
            git_status: None,
            task_updates: None,
            performance_metrics: None,
            custom_data: Some(
                [
                    ("entries_count".to_string(), json!(entries.len())),
                    ("recursive".to_string(), json!(recursive)),
                ]
                .into_iter()
                .collect(),
            ),
        };

        debug!("Listed directory: {} ({} entries)", path, entries.len());

        // Convert entries to a formatted text representation
        let formatted_entries = serde_json::to_string_pretty(&entries).map_err(|e| {
            ToolError::Internal(format!("Failed to format directory entries: {}", e))
        })?;

        Ok(ToolResult::success()
            .with_content(Content::Text {
                text: formatted_entries,
            })
            .with_context_update(context_update))
    }

    fn required_permissions(&self) -> Vec<Permission> {
        vec![Permission::FileRead]
    }

    fn category(&self) -> ToolCategory {
        ToolCategory::FileSystem
    }
}

pub fn glob_match(pattern: &str, text: &str) -> bool {
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

pub async fn list_directory_single(
    path: &Path,
    include_hidden: bool,
    filter_pattern: Option<&str>,
) -> Result<Vec<Value>, ToolError> {
    use tokio::fs;
    let mut entries = Vec::new();
    let mut dir = fs::read_dir(path)
        .await
        .map_err(|e| ToolError::FileSystem(format!("Failed to read directory: {}", e)))?;

    while let Some(entry) = dir
        .next_entry()
        .await
        .map_err(|e| ToolError::FileSystem(format!("Failed to read directory entry: {}", e)))?
    {
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

        let metadata = entry
            .metadata()
            .await
            .map_err(|e| ToolError::FileSystem(format!("Failed to read metadata: {}", e)))?;

        let entry_info = serde_json::json!({
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

pub async fn list_directory_recursive(
    path: &Path,
    max_depth: usize,
    include_hidden: bool,
    filter_pattern: Option<&str>,
) -> Result<Vec<Value>, ToolError> {
    fn collect_entries(
        entries: &mut Vec<Value>,
        path: &Path,
        current_depth: usize,
        max_depth: usize,
        include_hidden: bool,
        filter_pattern: Option<&str>,
    ) -> Result<(), ToolError> {
        if current_depth > max_depth {
            return Ok(());
        }

        let dir = std::fs::read_dir(path)
            .map_err(|e| ToolError::FileSystem(format!("Failed to read directory: {}", e)))?;

        for entry in dir {
            let entry = entry.map_err(|e| {
                ToolError::FileSystem(format!("Failed to read directory entry: {}", e))
            })?;

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

            let metadata = entry
                .metadata()
                .map_err(|e| ToolError::FileSystem(format!("Failed to read metadata: {}", e)))?;

            let entry_info = serde_json::json!({
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
                collect_entries(
                    entries,
                    &entry.path(),
                    current_depth + 1,
                    max_depth,
                    include_hidden,
                    filter_pattern,
                )?;
            }
        }

        Ok(())
    }

    let mut entries = Vec::new();
    collect_entries(
        &mut entries,
        path,
        0,
        max_depth,
        include_hidden,
        filter_pattern,
    )?;
    Ok(entries)
}
