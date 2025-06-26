/// Block management tools for MCP
///
/// This module provides tools for managing blocks in the forge project,
/// including listing, creating, updating, and deleting blocks.

use async_trait::async_trait;
use serde_json::{json, Value};
use tracing::{debug, error, info, warn};


use crate::llm_handler::BlockConnection;
use crate::mcp::tools::{
    Content, ContextUpdate, ExecutionContext, MCPTool, Permission,
    ToolCategory, ToolError, ToolResult, ToolResultBuilder,
};
use crate::models::{Block, Connections, Task};

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
                error!("Failed to load blocks - called from ListBlocksTool::execute - Error: {}", e);
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

        info!("Listed {} blocks", result_blocks.len());

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

/// Tool for creating a new block in the forge project
pub struct CreateBlockTool;

#[async_trait]
impl MCPTool for CreateBlockTool {
    fn name(&self) -> &str {
        "create_block"
    }

    fn description(&self) -> &str {
        "Create a new block in the forge project"
    }

    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "name": {
                    "type": "string",
                    "description": "The name of the block"
                },
                "description": {
                    "type": "string",
                    "description": "A description of what the block does"
                },
                "block_id": {
                    "type": "string",
                    "description": "Optional custom block ID (will be auto-generated if not provided)"
                }
            },
            "required": ["name", "description"]
        })
    }

    async fn execute(&self, params: Value, context: &mut ExecutionContext) -> Result<ToolResult, ToolError> {
        // Extract parameters
        let name = params["name"].as_str()
            .ok_or_else(|| ToolError::InvalidParams("name is required".to_string()))?
            .to_string();

        let description = params["description"].as_str()
            .ok_or_else(|| ToolError::InvalidParams("description is required".to_string()))?
            .to_string();

        let block_id = params["block_id"].as_str().unwrap_or("").to_string();

        // Create default inputs and outputs
        let inputs = vec![BlockConnection::new()];
        let outputs = vec![BlockConnection::new()];

        // Create the new block
        let mut new_block = Block::new(name.clone(), description, inputs, outputs);

        // Set custom block_id if provided
        if !block_id.is_empty() {
            new_block.block_id = block_id;
        }

        // Add the block to the block manager
        match context.block_manager.add_block(new_block.clone()) {
            Ok(_) => {
                info!("Successfully added new block: {} (ID: {})", new_block.name, new_block.block_id);
            }
            Err(e) => {
                return Err(ToolError::ExecutionFailed(format!("Failed to add block: {}", e)));
            }
        }

        // Save the updated blocks to file
        match context.block_manager.save_blocks_to_file() {
            Ok(_) => {
                info!("Successfully saved blocks to file");
            }
            Err(e) => {
                error!("Failed to save blocks to file: {}", e);
                return Err(ToolError::ExecutionFailed(format!("Failed to save blocks: {}", e)));
            }
        }

        // Create context update
        let context_update = ContextUpdate {
            files_accessed: Some(vec![context.block_manager.config_file.clone()]),
            files_modified: Some(vec![context.block_manager.config_file.clone()]),
            git_status: None,
            task_updates: None,
            performance_metrics: None,
            custom_data: Some([
                ("block_created".to_string(), json!(true)),
                ("block_id".to_string(), json!(new_block.block_id)),
                ("block_name".to_string(), json!(new_block.name)),
            ].into_iter().collect()),
        };

        // Format the result
        let result_data = json!({
            "success": true,
            "message": format!("Successfully created block '{}'", new_block.name),
            "block": {
                "block_id": new_block.block_id,
                "name": new_block.name,
                "description": new_block.description,
                "connections": {
                    "inputs": new_block.connections.input_connections,
                    "outputs": new_block.connections.output_connections,
                },
                "tasks": new_block.todo_list
            }
        });

        let formatted_result = serde_json::to_string_pretty(&result_data)
            .map_err(|e| ToolError::Internal(format!("Failed to format result: {}", e)))?;

        Ok(ToolResult::success()
            .with_content(Content::Text { text: formatted_result })
            .with_context_update(context_update))
    }

    fn required_permissions(&self) -> Vec<Permission> {
        vec![Permission::FileWrite, Permission::ProjectConfig]
    }

    fn category(&self) -> ToolCategory {
        ToolCategory::Project
    }
}

/// Tool for creating a new task for a block in the forge project
pub struct CreateTaskTool;

#[async_trait]
impl MCPTool for CreateTaskTool {
    fn name(&self) -> &str {
        "create_task"
    }

    fn description(&self) -> &str {
        "Create a new task for a block in the forge project"
    }

    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "block_id": {
                    "type": "string",
                    "description": "The ID of the block to add the task to"
                },
                "task_name": {
                    "type": "string",
                    "description": "The name/title of the task"
                },
                "description": {
                    "type": "string",
                    "description": "A detailed description of what the task should accomplish"
                },
                "acceptance_criteria": {
                    "type": "array",
                    "items": { "type": "string" },
                    "description": "List of acceptance criteria for the task"
                },
                "dependencies": {
                    "type": "array",
                    "items": { "type": "string" },
                    "description": "List of dependencies for this task"
                },
                "estimated_effort": {
                    "type": "string",
                    "description": "Estimated effort required (e.g., '2 hours', 'small', 'large')"
                },
                "files_affected": {
                    "type": "array",
                    "items": { "type": "string" },
                    "description": "List of files that will be affected by this task"
                },
                "function_signatures": {
                    "type": "array",
                    "items": { "type": "string" },
                    "description": "Function signatures that need to be implemented"
                },
                "testing_requirements": {
                    "type": "array",
                    "items": { "type": "string" },
                    "description": "Testing requirements for the task"
                },
                "status": {
                    "type": "string",
                    "description": "Initial status of the task (default: 'TODO')"
                }
            },
            "required": ["block_id", "task_name", "description"]
        })
    }

    async fn execute(&self, params: Value, context: &mut ExecutionContext) -> Result<ToolResult, ToolError> {
        // Extract required parameters
        let block_id = params["block_id"].as_str()
            .ok_or_else(|| ToolError::InvalidParams("block_id is required".to_string()))?;
        
        let task_name = params["task_name"].as_str()
            .ok_or_else(|| ToolError::InvalidParams("task_name is required".to_string()))?;
        
        let description = params["description"].as_str()
            .ok_or_else(|| ToolError::InvalidParams("description is required".to_string()))?;

        // Extract optional parameters
        let acceptance_criteria = params["acceptance_criteria"].as_array()
            .map(|arr| arr.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect())
            .unwrap_or_else(Vec::new);

        let dependencies = params["dependencies"].as_array()
            .map(|arr| arr.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect())
            .unwrap_or_else(Vec::new);

        let estimated_effort = params["estimated_effort"].as_str()
            .unwrap_or("")
            .to_string();

        let files_affected = params["files_affected"].as_array()
            .map(|arr| arr.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect())
            .unwrap_or_else(Vec::new);

        let function_signatures = params["function_signatures"].as_array()
            .map(|arr| arr.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect())
            .unwrap_or_else(Vec::new);

        let testing_requirements = params["testing_requirements"].as_array()
            .map(|arr| arr.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect())
            .unwrap_or_else(Vec::new);

        let status = params["status"].as_str()
            .unwrap_or("TODO")
            .to_string();

        // Load blocks to verify the block exists
        match context.block_manager.load_blocks_from_file() {
            Ok(_) => info!("Blocks loaded successfully for CreateTaskTool"),
            Err(e) => {
                error!("Failed to load blocks in CreateTaskTool: {}", e);
            }
        }

        // Check if the block exists
        let blocks = context.block_manager.get_blocks()
            .map_err(|e| ToolError::ExecutionFailed(format!("Failed to get blocks: {}", e)))?;
        
        if !blocks.iter().any(|b| b.block_id == block_id) {
            return Err(ToolError::InvalidParams(format!("Block with ID '{}' not found", block_id)));
        }

        // Create a detailed task
        let mut task = Task::new(description.to_string());
        task.task_name = task_name.to_string();
        task.acceptance_criteria = acceptance_criteria;
        task.dependencies = dependencies;
        task.estimated_effort = estimated_effort;
        task.files_affected = files_affected;
        task.function_signatures = function_signatures;
        task.testing_requirements = testing_requirements;
        task.status = status;

        let task_id = task.task_id.clone();

        // Add the full task using the new dedicated method
        let actual_task_id = match context.block_manager.add_task(block_id, task.clone()) {
            Ok(task_id) => {
                info!("Successfully added task '{}' (ID: {}) to block '{}'", task.task_name, task_id, block_id);
                task_id
            },
            Err(e) => {
                return Err(ToolError::ExecutionFailed(format!("Failed to add task: {}", e)));
            }
        };

        // Save the updated blocks to file
        match context.block_manager.save_blocks_to_file() {
            Ok(_) => {
                info!("Successfully saved blocks to file after adding task");
            },
            Err(e) => {
                error!("Failed to save blocks to file: {}", e);
                return Err(ToolError::ExecutionFailed(format!("Failed to save blocks: {}", e)));
            }
        }

        // Create context update
        let context_update = ContextUpdate {
            files_accessed: Some(vec![context.block_manager.config_file.clone()]),
            files_modified: Some(vec![context.block_manager.config_file.clone()]),
            git_status: None,
            task_updates: Some(vec![crate::mcp::tools::TaskUpdate {
                task_id: actual_task_id.clone(),
                block_id: block_id.to_string(),
                status: task.status.clone(),
                progress: 0.0,
                message: format!("Created new task: {}", task.task_name),
            }]),
            performance_metrics: None,
            custom_data: Some([
                ("task_created".to_string(), json!(true)),
                ("task_id".to_string(), json!(actual_task_id)),
                ("task_name".to_string(), json!(task.task_name)),
                ("block_id".to_string(), json!(block_id)),
            ].into_iter().collect()),
        };

        // Format the result
        let result_data = json!({
            "success": true,
            "message": format!("Successfully created task '{}' for block '{}'", task.task_name, block_id),
            "task": {
                "task_id": task.task_id,
                "task_name": task.task_name,
                "description": task.description,
                "acceptance_criteria": task.acceptance_criteria,
                "dependencies": task.dependencies,
                "estimated_effort": task.estimated_effort,
                "files_affected": task.files_affected,
                "function_signatures": task.function_signatures,
                "testing_requirements": task.testing_requirements,
                "status": task.status,
                "block_id": block_id
            }
        });

        let formatted_result = serde_json::to_string_pretty(&result_data)
            .map_err(|e| ToolError::Internal(format!("Failed to format result: {}", e)))?;

        Ok(ToolResult::success()
            .with_content(Content::Text { text: formatted_result })
            .with_context_update(context_update))
    }

    fn required_permissions(&self) -> Vec<Permission> {
        vec![Permission::FileWrite, Permission::TaskManagement, Permission::ProjectConfig]
    }

    fn category(&self) -> ToolCategory {
        ToolCategory::Tasks
    }
}