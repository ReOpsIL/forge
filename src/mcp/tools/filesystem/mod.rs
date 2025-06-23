/// File system tools for MCP - comprehensive file operations
///
/// This module provides a comprehensive set of file system tools that enable
/// Claude Code to interact with the project file system through structured
/// MCP calls instead of brittle CLI commands.

// // Re-export all tools
// pub use self::read_file::ReadFileTool;
// pub use self::write_file::WriteFileTool;
// pub use self::list_directory::ListDirectoryTool;
// pub use self::create_directory::CreateDirectoryTool;
// pub use self::delete::DeleteTool;

// Declare submodules
pub mod read_file;
pub mod write_file;
pub mod list_directory;
pub mod create_directory;
pub mod delete;

// Helper functions used by multiple tools
mod helpers {
    use std::path::Path;
    use serde_json::Value;
    use crate::mcp::tools::ToolError;
    
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
        filter_pattern: Option<&str>
    ) -> Result<Vec<Value>, ToolError> {
        use tokio::fs;
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
                    collect_entries(entries, &entry.path(), current_depth + 1, max_depth, include_hidden, filter_pattern)?;
                }
            }

            Ok(())
        }

        let mut entries = Vec::new();
        collect_entries(&mut entries, path, 0, max_depth, include_hidden, filter_pattern)?;
        Ok(entries)
    }
}

#[cfg(test)]
mod tests {
    use super::helpers::glob_match;
    
    #[test]
    fn test_glob_match() {
        assert!(glob_match("*.rs", "main.rs"));
        assert!(glob_match("test*.txt", "test123.txt"));
        assert!(glob_match("?.txt", "a.txt"));
        assert!(!glob_match("*.rs", "main.py"));
        assert!(!glob_match("test*.txt", "other.txt"));
    }
}