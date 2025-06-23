/// Block management tools for MCP
///
/// This module provides tools for managing blocks in the forge project,
/// including listing, creating, updating, and deleting blocks.

use async_trait::async_trait;
use serde_json::{json, Value};
use tracing::{debug, info, warn};

use crate::mcp::tools::{
    Content, ContextUpdate, ExecutionContext, MCPTool, Permission,
    ToolCategory, ToolError, ToolResult, ToolResultBuilder,
};

/// Tool for listing all blocks in the forge project
pub struct ListBlocksTool;

#[async_trait]
impl MCPTool for ListBlocksTool {
    fn name(&self) -> &str {
        "list_blocks"
    }

    fn description(&self) -> &str {
        "List all blocks in the forge project with optional filtering"
    }

    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "filter": {
                    "type": "object",
                    "properties": {
                        "status": {
                            "type": "string",
                            "enum": ["active", "completed", "failed"]
                        },
                        "has_tasks": {
                            "type": "boolean"
                        },
                        "search_term": {
                            "type": "string",
                            "description": "Search term to filter blocks by name or description"
                        }
                    }
                },
                "include_tasks": {
                    "type": "boolean",
                    "description": "Whether to include tasks in the response",
                    "default": true
                },
                "include_connections": {
                    "type": "boolean",
                    "description": "Whether to include connections in the response",
                    "default": true
                }
            }
        })
    }

    async fn execute(&self, params: Value, context: &mut ExecutionContext) -> Result<ToolResult, ToolError> {
        // Extract parameters
        match context.block_manager.load_blocks_from_file() {
            Ok(_) => info!("Blocks loaded successfully from ListBlocksTool::execute"),
            Err(e) => {
                warn!("Failed to load blocks - called from ListBlocksTool::execute - Error: {}", e);
            }
        }

        let num_blocks = context.block_manager.get_blocks().unwrap_or_default().len();
        info!(">> Num blocks (tool): {}",num_blocks);

        let include_tasks = params["include_tasks"].as_bool().unwrap_or(true);
        let include_connections = params["include_connections"].as_bool().unwrap_or(true);

        // Get filter parameters if provided
        let filter = params.get("filter");
        let status_filter = filter.and_then(|f| f["status"].as_str());
        let has_tasks_filter = filter.and_then(|f| f["has_tasks"].as_bool());
        let search_term = filter.and_then(|f| f["search_term"].as_str());

        // Get all blocks from the block manager
        let blocks = context.block_manager.get_blocks()
            .map_err(|e| ToolError::ExecutionFailed(format!("Failed to get blocks: {}", e)))?;

        // Apply filters
        let filtered_blocks = blocks.into_iter().filter(|block| {
            // Filter by status if provided
            if let Some(status) = status_filter {
                // Check if any task has the specified status
                let has_matching_status = block.todo_list.values().any(|task| task.status == status);
                if !has_matching_status {
                    return false;
                }
            }

            // Filter by has_tasks if provided
            if let Some(has_tasks) = has_tasks_filter {
                let block_has_tasks = !block.todo_list.is_empty();
                if has_tasks != block_has_tasks {
                    return false;
                }
            }

            // Filter by search term if provided
            if let Some(term) = search_term {
                let term_lower = term.to_lowercase();
                let name_match = block.name.to_lowercase().contains(&term_lower);
                let desc_match = block.description.to_lowercase().contains(&term_lower);
                if !name_match && !desc_match {
                    return false;
                }
            }

            true
        }).collect::<Vec<_>>();

        // Transform blocks to the desired output format
        let mut result_blocks = Vec::new();
        for block in filtered_blocks {
            let mut block_data = json!({
                "block_id": block.block_id,
                "name": block.name,
                "description": block.description,
            });

            // Include tasks if requested
            if include_tasks {
                let tasks = block.todo_list.values().map(|task| {
                    json!({
                        "task_id": task.task_id,
                        "task_name": task.task_name,
                        "description": task.description,
                        "status": task.status,
                    })
                }).collect::<Vec<_>>();

                block_data["tasks"] = json!(tasks);
            }

            // Include connections if requested
            if include_connections {
                block_data["connections"] = json!({
                    "inputs": block.connections.input_connections,
                    "outputs": block.connections.output_connections,
                });
            }

            result_blocks.push(block_data);
        }

        // Create context update
        let context_update = ContextUpdate {
            files_accessed: Some(vec![context.block_manager.config_file.clone()]),
            files_modified: None,
            git_status: None,
            task_updates: None,
            performance_metrics: None,
            custom_data: Some([
                ("blocks_count".to_string(), json!(result_blocks.len())),
                ("filtered".to_string(), json!(filter.is_some())),
            ].into_iter().collect()),
        };

        debug!("Listed {} blocks", result_blocks.len());

        // Format the result as pretty-printed JSON
        let formatted_blocks = serde_json::to_string_pretty(&result_blocks)
            .map_err(|e| ToolError::Internal(format!("Failed to format blocks: {}", e)))?;

        Ok(ToolResult::success()
            .with_content(Content::Text { text: formatted_blocks })
            .with_context_update(context_update))
    }

    fn required_permissions(&self) -> Vec<Permission> {
        vec![Permission::FileRead]
    }

    fn category(&self) -> ToolCategory {
        ToolCategory::Project
    }
}