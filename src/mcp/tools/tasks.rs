use async_trait::async_trait;
use serde_json::{json, Value};
use tracing::{error, info};
use crate::mcp::MCPTool;
use crate::mcp::tools::{Content, ContextUpdate, ExecutionContext, Permission, ToolCategory, ToolError, ToolResult, ToolResultBuilder};
use crate::models::Task;

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