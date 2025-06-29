use crate::mcp::MCPTool;
use crate::mcp::tools::{
    Content, ContextUpdate, ExecutionContext, Permission, ToolCategory, ToolError, ToolResult,
    ToolResultBuilder, Notification,
};
use crate::models::Task;
use async_trait::async_trait;
use serde_json::{Value, json};
use std::collections::{HashMap, HashSet, VecDeque};
use tracing::{error, info, warn};

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

    async fn execute(
        &self,
        params: Value,
        context: &mut ExecutionContext,
    ) -> Result<ToolResult, ToolError> {
        // Extract required parameters
        let block_id = params["block_id"]
            .as_str()
            .ok_or_else(|| ToolError::InvalidParams("block_id is required".to_string()))?;

        let task_name = params["task_name"]
            .as_str()
            .ok_or_else(|| ToolError::InvalidParams("task_name is required".to_string()))?;

        let description = params["description"]
            .as_str()
            .ok_or_else(|| ToolError::InvalidParams("description is required".to_string()))?;

        // Extract optional parameters
        let acceptance_criteria = params["acceptance_criteria"]
            .as_array()
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(|s| s.to_string()))
                    .collect()
            })
            .unwrap_or_else(Vec::new);

        let dependencies = params["dependencies"]
            .as_array()
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(|s| s.to_string()))
                    .collect()
            })
            .unwrap_or_else(Vec::new);

        let estimated_effort = params["estimated_effort"]
            .as_str()
            .unwrap_or("")
            .to_string();

        let files_affected = params["files_affected"]
            .as_array()
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(|s| s.to_string()))
                    .collect()
            })
            .unwrap_or_else(Vec::new);

        let function_signatures = params["function_signatures"]
            .as_array()
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(|s| s.to_string()))
                    .collect()
            })
            .unwrap_or_else(Vec::new);

        let testing_requirements = params["testing_requirements"]
            .as_array()
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(|s| s.to_string()))
                    .collect()
            })
            .unwrap_or_else(Vec::new);

        let status = params["status"].as_str().unwrap_or("TODO").to_string();

        // Load blocks to verify the block exists
        match context.block_manager.load_blocks_from_file() {
            Ok(_) => info!("Blocks loaded successfully for CreateTaskTool"),
            Err(e) => {
                error!("Failed to load blocks in CreateTaskTool: {}", e);
            }
        }

        // Check if the block exists
        let blocks = context
            .block_manager
            .get_blocks()
            .map_err(|e| ToolError::ExecutionFailed(format!("Failed to get blocks: {}", e)))?;

        if !blocks.iter().any(|b| b.block_id == block_id) {
            return Err(ToolError::InvalidParams(format!(
                "Block with ID '{}' not found",
                block_id
            )));
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
                info!(
                    "Successfully added task '{}' (ID: {}) to block '{}'",
                    task.task_name, task_id, block_id
                );
                task_id
            }
            Err(e) => {
                return Err(ToolError::ExecutionFailed(format!(
                    "Failed to add task: {}",
                    e
                )));
            }
        };

        // Save the updated blocks to file
        match context.block_manager.save_blocks_to_file() {
            Ok(_) => {
                info!("Successfully saved blocks to file after adding task");
            }
            Err(e) => {
                error!("Failed to save blocks to file: {}", e);
                return Err(ToolError::ExecutionFailed(format!(
                    "Failed to save blocks: {}",
                    e
                )));
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
            custom_data: Some(
                [
                    ("task_created".to_string(), json!(true)),
                    ("task_id".to_string(), json!(actual_task_id)),
                    ("task_name".to_string(), json!(task.task_name)),
                    ("block_id".to_string(), json!(block_id)),
                ]
                .into_iter()
                .collect(),
            ),
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
            .with_content(Content::Text {
                text: formatted_result,
            })
            .with_context_update(context_update))
    }

    fn required_permissions(&self) -> Vec<Permission> {
        vec![
            Permission::FileWrite,
            Permission::TaskManagement,
            Permission::ProjectConfig,
        ]
    }

    fn category(&self) -> ToolCategory {
        ToolCategory::Tasks
    }
}

/// Tool for updating one or more task attributes
pub struct UpdateTaskTool;

#[async_trait]
impl MCPTool for UpdateTaskTool {
    fn name(&self) -> &str {
        "update_task"
    }

    fn description(&self) -> &str {
        "Update one or more task attributes like description, commit_id, status, etc."
    }

    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "block_id": {
                    "type": "string",
                    "description": "The ID of the block containing the task"
                },
                "task_id": {
                    "type": "string",
                    "description": "The ID of the task to update"
                },
                "task_name": {
                    "type": "string",
                    "description": "The name/title of the task"
                },
                "description": {
                    "type": "string",
                    "description": "A detailed description of what the task should accomplish"
                },
                "status": {
                    "type": "string",
                    "description": "Status of the task (e.g., 'TODO', 'IN_PROGRESS', 'COMPLETED', 'FAILED')"
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
                "log": {
                    "type": "string",
                    "description": "Log information for the task"
                },
                "commit_id": {
                    "type": "string",
                    "description": "Git commit ID associated with the task"
                }
            },
            "required": ["block_id", "task_id"]
        })
    }

    async fn execute(
        &self,
        params: Value,
        context: &mut ExecutionContext,
    ) -> Result<ToolResult, ToolError> {
        // Extract required parameters
        let block_id = params["block_id"]
            .as_str()
            .ok_or_else(|| ToolError::InvalidParams("block_id is required".to_string()))?;

        let task_id = params["task_id"]
            .as_str()
            .ok_or_else(|| ToolError::InvalidParams("task_id is required".to_string()))?;

        // Load blocks to get the latest state
        match context.block_manager.load_blocks_from_file() {
            Ok(_) => info!("Blocks loaded successfully for UpdateTaskTool"),
            Err(e) => {
                error!("Failed to load blocks in UpdateTaskTool: {}", e);
                return Err(ToolError::ExecutionFailed(format!("Failed to load blocks: {}", e)));
            }
        }

        // Get the blocks and find the target block
        let mut blocks = context
            .block_manager
            .get_blocks()
            .map_err(|e| ToolError::ExecutionFailed(format!("Failed to get blocks: {}", e)))?;

        let block = blocks
            .iter_mut()
            .find(|b| b.block_id == block_id)
            .ok_or_else(|| ToolError::InvalidParams(format!("Block with ID '{}' not found", block_id)))?;

        // Find and update the task
        let mut updated_fields = Vec::new();

        {
            let task = block
                .todo_list
                .get_mut(task_id)
                .ok_or_else(|| ToolError::InvalidParams(format!("Task with ID '{}' not found in block '{}'", task_id, block_id)))?;

            // Update optional fields if provided
            if let Some(name) = params["task_name"].as_str() {
                task.task_name = name.to_string();
                updated_fields.push("task_name");
            }

            if let Some(description) = params["description"].as_str() {
                task.description = description.to_string();
                updated_fields.push("description");
            }

            if let Some(status) = params["status"].as_str() {
                task.status = status.to_string();
                updated_fields.push("status");
            }

            if let Some(acceptance_criteria) = params["acceptance_criteria"].as_array() {
                task.acceptance_criteria = acceptance_criteria
                    .iter()
                    .filter_map(|v| v.as_str().map(|s| s.to_string()))
                    .collect();
                updated_fields.push("acceptance_criteria");
            }

            if let Some(dependencies) = params["dependencies"].as_array() {
                task.dependencies = dependencies
                    .iter()
                    .filter_map(|v| v.as_str().map(|s| s.to_string()))
                    .collect();
                updated_fields.push("dependencies");
            }

            if let Some(estimated_effort) = params["estimated_effort"].as_str() {
                task.estimated_effort = estimated_effort.to_string();
                updated_fields.push("estimated_effort");
            }

            if let Some(files_affected) = params["files_affected"].as_array() {
                task.files_affected = files_affected
                    .iter()
                    .filter_map(|v| v.as_str().map(|s| s.to_string()))
                    .collect();
                updated_fields.push("files_affected");
            }

            if let Some(function_signatures) = params["function_signatures"].as_array() {
                task.function_signatures = function_signatures
                    .iter()
                    .filter_map(|v| v.as_str().map(|s| s.to_string()))
                    .collect();
                updated_fields.push("function_signatures");
            }

            if let Some(testing_requirements) = params["testing_requirements"].as_array() {
                task.testing_requirements = testing_requirements
                    .iter()
                    .filter_map(|v| v.as_str().map(|s| s.to_string()))
                    .collect();
                updated_fields.push("testing_requirements");
            }

            if let Some(log) = params["log"].as_str() {
                task.log = log.to_string();
                updated_fields.push("log");
            }

            if let Some(commit_id) = params["commit_id"].as_str() {
                task.commit_id = commit_id.to_string();
                updated_fields.push("commit_id");
            }
        }

        if updated_fields.is_empty() {
            return Err(ToolError::InvalidParams("No fields provided for update".to_string()));
        }

        // Get the updated task for response
        let updated_task = block.todo_list.get(task_id).unwrap().clone();

        // Update the block in the database
        match context.block_manager.update_block(block.clone()) {
            Ok(_) => {
                info!(
                    "Successfully updated task '{}' (ID: {}) in block '{}', fields: {:?}",
                    updated_task.task_name, task_id, block_id, updated_fields
                );
            }
            Err(e) => {
                return Err(ToolError::ExecutionFailed(format!(
                    "Failed to update task: {}",
                    e
                )));
            }
        }

        // Save the updated blocks to file
        match context.block_manager.save_blocks_to_file() {
            Ok(_) => {
                info!("Successfully saved blocks to file after updating task");
            }
            Err(e) => {
                error!("Failed to save blocks to file: {}", e);
                return Err(ToolError::ExecutionFailed(format!(
                    "Failed to save blocks: {}",
                    e
                )));
            }
        }

        // Create context update
        let context_update = ContextUpdate {
            files_accessed: Some(vec![context.block_manager.config_file.clone()]),
            files_modified: Some(vec![context.block_manager.config_file.clone()]),
            git_status: None,
            task_updates: Some(vec![crate::mcp::tools::TaskUpdate {
                task_id: task_id.to_string(),
                block_id: block_id.to_string(),
                status: updated_task.status.clone(),
                progress: if updated_task.status == "COMPLETED" { 100.0 } else { 0.0 },
                message: format!("Updated task fields: {}", updated_fields.join(", ")),
            }]),
            performance_metrics: None,
            custom_data: Some(
                [
                    ("task_updated".to_string(), json!(true)),
                    ("task_id".to_string(), json!(task_id)),
                    ("block_id".to_string(), json!(block_id)),
                    ("updated_fields".to_string(), json!(updated_fields)),
                ]
                .into_iter()
                .collect(),
            ),
        };

        // Format the result
        let result_data = json!({
            "success": true,
            "message": format!("Successfully updated task '{}' in block '{}'", updated_task.task_name, block_id),
            "updated_fields": updated_fields,
            "task": {
                "task_id": updated_task.task_id,
                "task_name": updated_task.task_name,
                "description": updated_task.description,
                "status": updated_task.status,
                "acceptance_criteria": updated_task.acceptance_criteria,
                "dependencies": updated_task.dependencies,
                "estimated_effort": updated_task.estimated_effort,
                "files_affected": updated_task.files_affected,
                "function_signatures": updated_task.function_signatures,
                "testing_requirements": updated_task.testing_requirements,
                "log": updated_task.log,
                "commit_id": updated_task.commit_id,
                "block_id": block_id
            }
        });

        let formatted_result = serde_json::to_string_pretty(&result_data)
            .map_err(|e| ToolError::Internal(format!("Failed to format result: {}", e)))?;

        Ok(ToolResult::success()
            .with_content(Content::Text {
                text: formatted_result,
            })
            .with_context_update(context_update))
    }

    fn required_permissions(&self) -> Vec<Permission> {
        vec![
            Permission::FileRead,
            Permission::FileWrite,
            Permission::TaskManagement,
            Permission::ProjectConfig,
        ]
    }

    fn category(&self) -> ToolCategory {
        ToolCategory::Tasks
    }
}

/// Tool for executing block tasks with dependency resolution and tracking
pub struct ExecTaskTool;

#[async_trait]
impl MCPTool for ExecTaskTool {
    fn name(&self) -> &str {
        "exec_task"
    }

    fn description(&self) -> &str {
        "Execute block tasks with dependency resolution, topological sorting, and execution tracking. Supports resuming from failed tasks and skipping completed ones."
    }

    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "block_id": {
                    "type": "string",
                    "description": "The ID of the block whose tasks to execute"
                },
                "task_ids": {
                    "type": "array",
                    "items": { "type": "string" },
                    "description": "Optional array of specific task IDs to execute. If not provided, all pending tasks will be executed."
                },
                "resume_from_last": {
                    "type": "boolean",
                    "description": "Whether to resume execution from the last incomplete task",
                    "default": false
                },
                "skip_completed": {
                    "type": "boolean", 
                    "description": "Whether to skip tasks already marked as completed",
                    "default": true
                },
                "parallel_execution": {
                    "type": "boolean",
                    "description": "Whether to allow parallel execution when dependencies permit",
                    "default": false
                },
                "max_parallel_tasks": {
                    "type": "integer",
                    "description": "Maximum number of tasks to execute in parallel",
                    "default": 3
                },
                "dry_run": {
                    "type": "boolean",
                    "description": "If true, only show the execution plan without actually executing tasks",
                    "default": false
                }
            },
            "required": ["block_id"]
        })
    }

    async fn execute(
        &self,
        params: Value,
        context: &mut ExecutionContext,
    ) -> Result<ToolResult, ToolError> {
        let block_id = params["block_id"]
            .as_str()
            .ok_or_else(|| ToolError::InvalidParams("block_id is required".to_string()))?;

        let specific_task_ids = params["task_ids"]
            .as_array()
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(|s| s.to_string()))
                    .collect::<Vec<String>>()
            });

        let resume_from_last = params["resume_from_last"].as_bool().unwrap_or(false);
        let skip_completed = params["skip_completed"].as_bool().unwrap_or(true);
        let parallel_execution = params["parallel_execution"].as_bool().unwrap_or(false);
        let max_parallel_tasks = params["max_parallel_tasks"].as_u64().unwrap_or(3) as usize;
        let dry_run = params["dry_run"].as_bool().unwrap_or(false);

        // Load blocks to get the latest state
        match context.block_manager.load_blocks_from_file() {
            Ok(_) => info!("Blocks loaded successfully for ExecuteBlockTaskTool"),
            Err(e) => {
                error!("Failed to load blocks in ExecuteBlockTaskTool: {}", e);
                return Err(ToolError::ExecutionFailed(format!("Failed to load blocks: {}", e)));
            }
        }

        // Get the block
        let blocks = context.block_manager.get_blocks()
            .map_err(|e| ToolError::ExecutionFailed(format!("Failed to get blocks: {}", e)))?;
        
        let block = blocks.iter()
            .find(|b| b.block_id == block_id)
            .ok_or_else(|| ToolError::InvalidParams(format!("Block with ID '{}' not found", block_id)))?;

        // Get tasks to execute
        let tasks_to_execute: Vec<&Task> = if let Some(task_ids) = specific_task_ids {
            // Execute specific tasks
            task_ids.iter()
                .filter_map(|task_id| block.todo_list.get(task_id))
                .collect()
        } else {
            // Execute all tasks, potentially filtered by status
            block.todo_list.values()
                .filter(|task| {
                    if skip_completed && (task.status == "COMPLETED" || task.status == "DONE") {
                        false
                    } else {
                        true
                    }
                })
                .collect()
        };

        if tasks_to_execute.is_empty() {
            return Ok(ToolResult::success()
                .with_content(Content::Text {
                    text: format!("No tasks to execute for block '{}'", block_id),
                })
            );
        }

        // Build dependency graph and perform topological sort
        let execution_plan = self.build_execution_plan(&tasks_to_execute, resume_from_last)?;
        
        if dry_run {
            return self.create_dry_run_result(&execution_plan, block_id);
        }

        // Execute tasks according to plan
        let execution_results = if parallel_execution {
            self.execute_tasks_parallel(&execution_plan, context, block_id, max_parallel_tasks).await?
        } else {
            self.execute_tasks_sequential(&execution_plan, context, block_id).await?
        };

        // Save updated block state
        match context.block_manager.save_blocks_to_file() {
            Ok(_) => info!("Successfully saved blocks after task execution"),
            Err(e) => warn!("Failed to save blocks after task execution: {}", e),
        }

        self.create_execution_result(&execution_results, block_id, context)
    }

    fn required_permissions(&self) -> Vec<Permission> {
        vec![
            Permission::FileRead,
            Permission::FileWrite,
            Permission::TaskManagement,
            Permission::ProjectConfig,
            Permission::Execute,
        ]
    }

    fn category(&self) -> ToolCategory {
        ToolCategory::Tasks
    }
}

impl ExecTaskTool {
    /// Build execution plan with dependency resolution using topological sorting
    fn build_execution_plan(&self, tasks: &[&Task], resume_from_last: bool) -> Result<ExecutionPlan, ToolError> {
        let mut task_map: HashMap<String, &Task> = HashMap::new();
        let mut in_degree: HashMap<String, usize> = HashMap::new();
        let mut adj_list: HashMap<String, Vec<String>> = HashMap::new();

        // Build task mapping and initialize in-degree
        for task in tasks {
            task_map.insert(task.task_id.clone(), task);
            in_degree.insert(task.task_id.clone(), 0);
            adj_list.insert(task.task_id.clone(), Vec::new());
        }

        // Build dependency graph
        for task in tasks {
            for dependency in &task.dependencies {
                if let Some(_dep_task) = task_map.get(dependency) {
                    // Add edge from dependency to current task
                    adj_list.get_mut(dependency)
                        .unwrap()
                        .push(task.task_id.clone());
                    
                    // Increment in-degree for current task
                    *in_degree.get_mut(&task.task_id).unwrap() += 1;
                } else {
                    warn!("Dependency '{}' not found for task '{}'", dependency, task.task_id);
                }
            }
        }

        // Detect cycles using DFS
        if self.has_cycle(&adj_list, &task_map.keys().cloned().collect()) {
            return Err(ToolError::Validation("Circular dependency detected in tasks".to_string()));
        }

        // Perform topological sort using Kahn's algorithm
        let mut queue: VecDeque<String> = VecDeque::new();
        let mut execution_order: Vec<String> = Vec::new();

        // Find all nodes with in-degree 0
        for (task_id, degree) in &in_degree {
            if *degree == 0 {
                queue.push_back(task_id.clone());
            }
        }

        // Process queue
        while let Some(current_task_id) = queue.pop_front() {
            execution_order.push(current_task_id.clone());

            // Reduce in-degree of adjacent nodes
            if let Some(adjacent) = adj_list.get(&current_task_id) {
                for adjacent_task_id in adjacent {
                    if let Some(degree) = in_degree.get_mut(adjacent_task_id) {
                        *degree -= 1;
                        if *degree == 0 {
                            queue.push_back(adjacent_task_id.clone());
                        }
                    }
                }
            }
        }

        // Check if all tasks are included (no cycles)
        if execution_order.len() != tasks.len() {
            return Err(ToolError::Validation("Unable to resolve all task dependencies - possible circular dependency".to_string()));
        }

        // Handle resume from last incomplete task
        let start_index = if resume_from_last {
            self.find_resume_point(&execution_order, &task_map)?
        } else {
            0
        };

        Ok(ExecutionPlan {
            tasks: execution_order.into_iter().skip(start_index).collect(),
            task_map: task_map.into_iter().map(|(k, v)| (k, v.clone())).collect(),
            start_index,
        })
    }

    /// Detect cycles in the dependency graph using DFS
    fn has_cycle(&self, adj_list: &HashMap<String, Vec<String>>, nodes: &Vec<String>) -> bool {
        let mut visited = HashSet::new();
        let mut rec_stack = HashSet::new();

        for node in nodes {
            if !visited.contains(node) {
                if self.dfs_cycle_check(node, adj_list, &mut visited, &mut rec_stack) {
                    return true;
                }
            }
        }
        false
    }

    fn dfs_cycle_check(
        &self,
        node: &str,
        adj_list: &HashMap<String, Vec<String>>,
        visited: &mut HashSet<String>,
        rec_stack: &mut HashSet<String>,
    ) -> bool {
        visited.insert(node.to_string());
        rec_stack.insert(node.to_string());

        if let Some(neighbors) = adj_list.get(node) {
            for neighbor in neighbors {
                if !visited.contains(neighbor) {
                    if self.dfs_cycle_check(neighbor, adj_list, visited, rec_stack) {
                        return true;
                    }
                } else if rec_stack.contains(neighbor) {
                    return true;
                }
            }
        }

        rec_stack.remove(node);
        false
    }

    /// Find the point from which to resume execution
    fn find_resume_point(&self, execution_order: &[String], task_map: &HashMap<String, &Task>) -> Result<usize, ToolError> {
        for (index, task_id) in execution_order.iter().enumerate() {
            if let Some(task) = task_map.get(task_id) {
                if task.status != "COMPLETED" && task.status != "DONE" {
                    info!("Resuming execution from task: {} ({})", task.task_name, task_id);
                    return Ok(index);
                }
            }
        }
        Ok(0) // Resume from beginning if all tasks are incomplete
    }

    /// Execute tasks sequentially according to the execution plan
    async fn execute_tasks_sequential(
        &self,
        plan: &ExecutionPlan,
        context: &mut ExecutionContext,
        block_id: &str,
    ) -> Result<Vec<TaskExecutionResult>, ToolError> {
        let mut results = Vec::new();

        for task_id in &plan.tasks {
            if let Some(task) = plan.task_map.get(task_id) {
                info!("Executing task: {} ({})", task.task_name, task_id);
                
                let result = self.execute_single_task(task, context, block_id).await?;
                results.push(result);
                
                // Break execution if task failed and we're not continuing on error
                if !results.last().unwrap().success {
                    warn!("Task execution failed, stopping sequential execution");
                    break;
                }
            }
        }

        Ok(results)
    }

    /// Execute tasks in parallel where dependencies allow
    async fn execute_tasks_parallel(
        &self,
        plan: &ExecutionPlan,
        context: &mut ExecutionContext,
        block_id: &str,
        max_parallel: usize,
    ) -> Result<Vec<TaskExecutionResult>, ToolError> {
        // For now, implement as sequential since true parallel execution
        // would require more complex dependency tracking and context management
        warn!("Parallel execution not fully implemented, falling back to sequential");
        self.execute_tasks_sequential(plan, context, block_id).await
    }

    /// Execute a single task
    async fn execute_single_task(
        &self,
        task: &Task,
        context: &mut ExecutionContext,
        block_id: &str,
    ) -> Result<TaskExecutionResult, ToolError> {
        let start_time = std::time::SystemTime::now();
        
        // Update task status to IN_PROGRESS
        if let Err(e) = context.block_manager.update_task_status(block_id, &task.task_id, "IN_PROGRESS") {
            warn!("Failed to update task status to IN_PROGRESS: {}", e);
        }

        // Execute task by injecting prompt to running Claude CLI process
        let success = self.execute_task_using_injecting_prompt(task, context).await;
        
        let end_time = std::time::SystemTime::now();
        let duration = end_time.duration_since(start_time).unwrap_or_default();

        let final_status = if success { "COMPLETED" } else { "FAILED" };
        
        // Update task status
        if let Err(e) = context.block_manager.update_task_status(block_id, &task.task_id, final_status) {
            warn!("Failed to update task status to {}: {}", final_status, e);
        }

        Ok(TaskExecutionResult {
            task_id: task.task_id.clone(),
            task_name: task.task_name.clone(),
            success,
            duration,
            error_message: if success { None } else { Some("Task execution failed".to_string()) },
            output: if success { 
                Some(format!("Successfully executed task: {}", task.task_name)) 
            } else { 
                None 
            },
        })
    }

    /// Execute task by injecting prompt to running Claude CLI process
    async fn execute_task_using_injecting_prompt(&self, task: &Task, context: &mut ExecutionContext) -> bool {
        info!("Executing task by injecting prompt to Claude CLI: {}", task.task_name);
        info!("Task description: {}", task.description);
        info!("Files affected: {:?}", task.files_affected);
        info!("Function signatures: {:?}", task.function_signatures);

        // Get Claude session manager from context
        let claude_session_manager = match &context.claude_session_manager {
            Some(manager) => manager.clone(),
            None => {
                error!("Claude session manager not available in execution context");
                return false;
            }
        };

        // Generate the prompt for Claude CLI
        let prompt = self.generate_task_execution_prompt(task);
        
        info!("Generated prompt for Claude CLI execution:\n{}", prompt);

        // Use the default session ID for Claude CLI (matching claude_handlers.rs)
        let claude_session_id = "default-claude-session";

        // Ensure Claude session exists or create it
        match claude_session_manager.create_session(claude_session_id.to_string()) {
            Ok(_) => {
                info!("Claude session {} ready for task execution", claude_session_id);
            }
            Err(e) => {
                error!("Failed to create/get Claude session {}: {}", claude_session_id, e);
                return false;
            }
        }

        // Send the prompt to Claude CLI via stdin
        if let Some(session) = claude_session_manager.get_session(claude_session_id) {
            session.update_activity();

            // Send prompt to Claude CLI stdin
            if let Ok(stdin_opt) = session.stdin_tx.lock() {
                if let Some(ref tx) = stdin_opt.as_ref() {
                    match tx.send(format!("{}\r", prompt)) {
                        Ok(_) => {
                            info!("Successfully sent task execution prompt to Claude CLI session {}", claude_session_id);
                            // The user will see the output streaming through the WebSocket
                            // The actual execution will be handled by Claude CLI using its MCP tools
                            return true;
                        }
                        Err(e) => {
                            error!("Failed to send prompt to Claude CLI session {}: {}", claude_session_id, e);
                            return false;
                        }
                    }
                } else {
                    error!("No stdin channel available for Claude session {}", claude_session_id);
                    return false;
                }
            } else {
                error!("Failed to acquire stdin lock for Claude session {}", claude_session_id);
                return false;
            }
        } else {
            error!("Claude session {} not found", claude_session_id);
            return false;
        }
    }

    /// Generate a task execution prompt for Claude CLI
    fn generate_task_execution_prompt(&self, task: &Task) -> String {
        let mut prompt = String::new();
        
        prompt.push_str(&format!("# Task Execution Request\n\n"));
        prompt.push_str(&format!("**Task Name:** {}\n\n", task.task_name));
        prompt.push_str(&format!("**Description:** {}\n\n", task.description));
        
        if !task.files_affected.is_empty() {
            prompt.push_str("**Files to work with:**\n");
            for file in &task.files_affected {
                prompt.push_str(&format!("- {}\n", file));
            }
            prompt.push_str("\n");
        }
        
        if !task.function_signatures.is_empty() {
            prompt.push_str("**Function signatures to implement:**\n");
            for signature in &task.function_signatures {
                prompt.push_str(&format!("```\n{}\n```\n", signature));
            }
            prompt.push_str("\n");
        }
        
        if !task.acceptance_criteria.is_empty() {
            prompt.push_str("**Acceptance criteria:**\n");
            for criteria in &task.acceptance_criteria {
                prompt.push_str(&format!("- {}\n", criteria));
            }
            prompt.push_str("\n");
        }
        
        if !task.testing_requirements.is_empty() {
            prompt.push_str("**Testing requirements:**\n");
            for requirement in &task.testing_requirements {
                prompt.push_str(&format!("- {}\n", requirement));
            }
            prompt.push_str("\n");
        }
        
        prompt.push_str("Please execute this task using your available MCP tools. ");
        prompt.push_str("Create, modify, or analyze files as needed to complete the task requirements. ");
        prompt.push_str("Provide clear feedback on what you're doing and the results.\n\n");
        
        prompt
    }


    /// Create dry run result showing execution plan
    fn create_dry_run_result(&self, plan: &ExecutionPlan, block_id: &str) -> Result<ToolResult, ToolError> {
        let plan_data = json!({
            "dry_run": true,
            "block_id": block_id,
            "execution_plan": {
                "total_tasks": plan.tasks.len(),
                "execution_order": plan.tasks.iter().enumerate().map(|(index, task_id)| {
                    let task = plan.task_map.get(task_id).unwrap();
                    json!({
                        "order": index + 1,
                        "task_id": task_id,
                        "task_name": task.task_name,
                        "status": task.status,
                        "dependencies": task.dependencies,
                        "estimated_effort": task.estimated_effort
                    })
                }).collect::<Vec<_>>(),
                "resuming_from_index": plan.start_index
            }
        });

        let formatted_plan = serde_json::to_string_pretty(&plan_data)
            .map_err(|e| ToolError::Internal(format!("Failed to format execution plan: {}", e)))?;

        Ok(ToolResult::success()
            .with_content(Content::Text { text: formatted_plan })
            .with_notification(Notification::Info {
                message: format!("Execution plan ready for {} tasks", plan.tasks.len()),
            }))
    }

    /// Create execution result with detailed information
    fn create_execution_result(
        &self,
        results: &[TaskExecutionResult],
        block_id: &str,
        context: &ExecutionContext,
    ) -> Result<ToolResult, ToolError> {
        let successful_tasks = results.iter().filter(|r| r.success).count();
        let failed_tasks = results.len() - successful_tasks;
        let total_duration: std::time::Duration = results.iter().map(|r| r.duration).sum();

        let result_data = json!({
            "success": failed_tasks == 0,
            "block_id": block_id,
            "execution_summary": {
                "total_tasks": results.len(),
                "successful_tasks": successful_tasks,
                "failed_tasks": failed_tasks,
                "total_duration_ms": total_duration.as_millis(),
                "average_task_duration_ms": if !results.is_empty() { 
                    total_duration.as_millis() / results.len() as u128 
                } else { 0 }
            },
            "task_results": results.iter().map(|result| {
                json!({
                    "task_id": result.task_id,
                    "task_name": result.task_name,
                    "success": result.success,
                    "duration_ms": result.duration.as_millis(),
                    "output": result.output,
                    "error": result.error_message
                })
            }).collect::<Vec<_>>()
        });

        let formatted_result = serde_json::to_string_pretty(&result_data)
            .map_err(|e| ToolError::Internal(format!("Failed to format execution result: {}", e)))?;

        let mut tool_result = ToolResult::success()
            .with_content(Content::Text { text: formatted_result });

        // Add notifications
        if successful_tasks > 0 {
            tool_result = tool_result.with_notification(Notification::Info {
                message: format!("Successfully executed {} tasks", successful_tasks),
            });
        }
        
        if failed_tasks > 0 {
            tool_result = tool_result.with_notification(Notification::Warning {
                message: format!("{} tasks failed execution", failed_tasks),
            });
        }

        // Add context update
        let context_update = ContextUpdate {
            files_accessed: Some(vec![context.block_manager.config_file.clone()]),
            files_modified: Some(vec![context.block_manager.config_file.clone()]),
            git_status: None,
            task_updates: Some(results.iter().map(|result| {
                crate::mcp::tools::TaskUpdate {
                    task_id: result.task_id.clone(),
                    block_id: block_id.to_string(),
                    status: if result.success { "COMPLETED".to_string() } else { "FAILED".to_string() },
                    progress: if result.success { 100.0 } else { 0.0 },
                    message: result.output.clone().unwrap_or_else(|| 
                        result.error_message.clone().unwrap_or_else(|| "Task executed".to_string())
                    ),
                }
            }).collect()),
            performance_metrics: None,
            custom_data: Some([
                ("tasks_executed".to_string(), json!(results.len())),
                ("successful_tasks".to_string(), json!(successful_tasks)),
                ("failed_tasks".to_string(), json!(failed_tasks)),
                ("total_duration_ms".to_string(), json!(total_duration.as_millis())),
            ].into_iter().collect()),
        };

        Ok(tool_result.with_context_update(context_update))
    }
}

/// Execution plan containing ordered tasks and metadata
#[derive(Debug)]
struct ExecutionPlan {
    tasks: Vec<String>,
    task_map: HashMap<String, Task>,
    start_index: usize,
}

/// Result of executing a single task
#[derive(Debug)]
struct TaskExecutionResult {
    task_id: String,
    task_name: String,
    success: bool,
    duration: std::time::Duration,
    error_message: Option<String>,
    output: Option<String>,
}

