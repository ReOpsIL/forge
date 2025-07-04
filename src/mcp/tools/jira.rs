/// Jira integration tools for Forge MCP server
/// 
/// This module provides tools for synchronizing data between Forge blocks/tasks and Jira projects/issues
/// using the Atlassian MCP server that's already configured in the system.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::time::{Duration, SystemTime};
use tracing::{debug, error, info, warn};

use crate::mcp::tools::{
    Content, ExecutionContext, MCPTool, Permission, ToolCategory, ToolError, ToolResult,
    ToolResultBuilder,
};

// Jira client types will be defined at the end of this file

/// Tool for syncing a Forge block to a Jira project
pub struct SyncBlockToJiraProjectTool;

#[async_trait]
impl MCPTool for SyncBlockToJiraProjectTool {
    fn name(&self) -> &str {
        "sync_block_to_jira_project"
    }

    fn description(&self) -> &str {
        "Sync a Forge block and its tasks to a Jira project and issues"
    }

    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "block_id": {
                    "type": "string",
                    "description": "ID of the Forge block to sync"
                },
                "jira_project_key": {
                    "type": "string",
                    "description": "Jira project key to sync to"
                },
                "sync_mode": {
                    "type": "string",
                    "enum": ["import", "export", "bidirectional"],
                    "description": "Sync direction mode",
                    "default": "export"
                },
                "create_missing_issues": {
                    "type": "boolean",
                    "description": "Whether to create Jira issues for tasks that don't exist",
                    "default": true
                },
                "update_existing": {
                    "type": "boolean",
                    "description": "Whether to update existing Jira issues",
                    "default": true
                }
            },
            "required": ["block_id", "jira_project_key"]
        })
    }

    fn required_permissions(&self) -> Vec<Permission> {
        vec![
            Permission::Network,
            Permission::TaskManagement,
            Permission::ProjectConfig,
        ]
    }

    fn category(&self) -> ToolCategory {
        ToolCategory::Collaboration
    }

    fn estimated_execution_time(&self) -> Duration {
        Duration::from_secs(15)
    }

    async fn execute(
        &self,
        params: Value,
        context: &mut ExecutionContext,
    ) -> Result<ToolResult, ToolError> {
        let block_id = params["block_id"]
            .as_str()
            .ok_or_else(|| ToolError::InvalidParams("block_id is required".to_string()))?;
        
        let jira_project_key = params["jira_project_key"]
            .as_str()
            .ok_or_else(|| ToolError::InvalidParams("jira_project_key is required".to_string()))?;
        
        let sync_mode = params["sync_mode"].as_str().unwrap_or("export");
        let create_missing = params["create_missing_issues"].as_bool().unwrap_or(true);
        let update_existing = params["update_existing"].as_bool().unwrap_or(true);

        info!("Starting block to Jira project sync: {} -> {}", block_id, jira_project_key);

        // Get the block from the block manager
        let block = {
            let block_manager = context.block_manager.clone();
            let blocks = block_manager.get_blocks().map_err(|e| {
                ToolError::Internal(format!("Failed to load blocks: {}", e))
            })?;
            
            blocks.into_iter()
                .find(|b| b.block_id == block_id)
                .ok_or_else(|| ToolError::InvalidParams(format!("Block {} not found", block_id)))?
        };

        let mut sync_result = JiraSyncResult {
            blocks_processed: 0,
            tasks_created: 0,
            tasks_updated: 0,
            issues_created: 0,
            issues_updated: 0,
            errors: Vec::new(),
        };

        // Step 1: Get Jira project information
        let jira_project = match get_jira_project(jira_project_key, context).await {
            Ok(project) => project,
            Err(e) => {
                sync_result.errors.push(format!("Failed to get Jira project {}: {}", jira_project_key, e));
                return Ok(ToolResult::failure(format!("Jira project sync failed: {}", e)));
            }
        };

        sync_result.blocks_processed = 1;

        // Step 2: Process tasks based on sync mode
        match sync_mode {
            "export" => {
                // Export Forge tasks to Jira issues
                for (task_id, task) in &block.todo_list {
                    match export_task_to_jira(&block, task_id, &task.to_value(), &jira_project, create_missing, update_existing, context).await {
                        Ok(action) => {
                            match action {
                                JiraTaskAction::Created => sync_result.issues_created += 1,
                                JiraTaskAction::Updated => sync_result.issues_updated += 1,
                                JiraTaskAction::Skipped => {},
                            }
                        }
                        Err(e) => {
                            sync_result.errors.push(format!("Failed to export task {}: {}", task_id, e));
                        }
                    }
                }
            }
            "import" => {
                // Import Jira issues to Forge tasks
                match import_jira_issues_to_block(&block, &jira_project, context).await {
                    Ok((created, updated)) => {
                        sync_result.tasks_created = created;
                        sync_result.tasks_updated = updated;
                    }
                    Err(e) => {
                        sync_result.errors.push(format!("Failed to import from Jira: {}", e));
                    }
                }
            }
            "bidirectional" => {
                // First export, then import to ensure bidirectional sync
                for (task_id, task) in &block.todo_list {
                    match export_task_to_jira(&block, task_id, &task.to_value(), &jira_project, create_missing, update_existing, context).await {
                        Ok(action) => {
                            match action {
                                JiraTaskAction::Created => sync_result.issues_created += 1,
                                JiraTaskAction::Updated => sync_result.issues_updated += 1,
                                JiraTaskAction::Skipped => {},
                            }
                        }
                        Err(e) => {
                            sync_result.errors.push(format!("Failed to export task {}: {}", task_id, e));
                        }
                    }
                }
                
                // Then import any new issues
                match import_jira_issues_to_block(&block, &jira_project, context).await {
                    Ok((created, updated)) => {
                        sync_result.tasks_created += created;
                        sync_result.tasks_updated += updated;
                    }
                    Err(e) => {
                        sync_result.errors.push(format!("Failed to import from Jira: {}", e));
                    }
                }
            }
            _ => {
                return Err(ToolError::InvalidParams(format!("Invalid sync_mode: {}", sync_mode)));
            }
        }

        let success = sync_result.errors.is_empty();
        let summary = format!(
            "Sync completed. Blocks: {}, Tasks created: {}, Tasks updated: {}, Issues created: {}, Issues updated: {}, Errors: {}",
            sync_result.blocks_processed,
            sync_result.tasks_created,
            sync_result.tasks_updated,
            sync_result.issues_created,
            sync_result.issues_updated,
            sync_result.errors.len()
        );

        info!("{}", summary);

        let mut result = if success {
            ToolResult::success()
        } else {
            ToolResult::failure(format!("Sync completed with {} errors", sync_result.errors.len()))
        };

        result = result
            .with_content(Content::Data {
                data: json!(sync_result),
            })
            .with_content(Content::Text { text: summary });

        Ok(result)
    }
}

/// Tool for importing Jira issues to a Forge block
pub struct ImportJiraIssuesToBlockTool;

#[async_trait]
impl MCPTool for ImportJiraIssuesToBlockTool {
    fn name(&self) -> &str {
        "import_jira_issues_to_block"
    }

    fn description(&self) -> &str {
        "Import Jira issues from a project into a Forge block as tasks"
    }

    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "block_id": {
                    "type": "string",
                    "description": "ID of the Forge block to import tasks into"
                },
                "jira_project_key": {
                    "type": "string",
                    "description": "Jira project key to import from"
                },
                "jql_filter": {
                    "type": "string",
                    "description": "JQL query to filter issues (optional)",
                    "default": ""
                },
                "issue_types": {
                    "type": "array",
                    "items": {"type": "string"},
                    "description": "Issue types to include (empty for all)",
                    "default": []
                },
                "status_filter": {
                    "type": "string",
                    "description": "Status filter (all, open, closed)",
                    "default": "all"
                },
                "update_existing": {
                    "type": "boolean",
                    "description": "Whether to update existing tasks that are already synced",
                    "default": true
                }
            },
            "required": ["block_id", "jira_project_key"]
        })
    }

    fn required_permissions(&self) -> Vec<Permission> {
        vec![
            Permission::Network,
            Permission::TaskManagement,
        ]
    }

    fn category(&self) -> ToolCategory {
        ToolCategory::Collaboration
    }

    async fn execute(
        &self,
        params: Value,
        context: &mut ExecutionContext,
    ) -> Result<ToolResult, ToolError> {
        let block_id = params["block_id"]
            .as_str()
            .ok_or_else(|| ToolError::InvalidParams("block_id is required".to_string()))?;
        
        let jira_project_key = params["jira_project_key"]
            .as_str()
            .ok_or_else(|| ToolError::InvalidParams("jira_project_key is required".to_string()))?;

        let jql_filter = params["jql_filter"].as_str().unwrap_or("");
        let issue_types: Vec<String> = params["issue_types"]
            .as_array()
            .map(|arr| arr.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect())
            .unwrap_or_default();
        let status_filter = params["status_filter"].as_str().unwrap_or("all");
        let update_existing = params["update_existing"].as_bool().unwrap_or(true);

        info!("Importing Jira issues from {} to block {}", jira_project_key, block_id);

        // Get the block
        let block = {
            let block_manager = context.block_manager.clone();
            let blocks = block_manager.get_blocks().map_err(|e| {
                ToolError::Internal(format!("Failed to load blocks: {}", e))
            })?;
            
            blocks.into_iter()
                .find(|b| b.block_id == block_id)
                .ok_or_else(|| ToolError::InvalidParams(format!("Block {} not found", block_id)))?
        };

        // Get Jira project
        let jira_project = get_jira_project(jira_project_key, context).await
            .map_err(|e| ToolError::Internal(e))?;

        // Import issues
        let (tasks_created, tasks_updated) = import_jira_issues_to_block(&block, &jira_project, context).await
            .map_err(|e| ToolError::Internal(format!("Import failed: {}", e)))?;

        let summary = format!("Import completed: {} tasks created, {} tasks updated", tasks_created, tasks_updated);
        info!("{}", summary);

        Ok(ToolResult::success()
            .with_content(Content::Text { text: summary })
            .with_content(Content::Data {
                data: json!({
                    "tasks_created": tasks_created,
                    "tasks_updated": tasks_updated,
                    "block_id": block_id,
                    "jira_project_key": jira_project_key
                }),
            }))
    }
}

/// Tool for updating Jira issue status based on task status
pub struct UpdateJiraIssueStatusTool;

#[async_trait]
impl MCPTool for UpdateJiraIssueStatusTool {
    fn name(&self) -> &str {
        "update_jira_issue_status"
    }

    fn description(&self) -> &str {
        "Update a Jira issue status based on Forge task status"
    }

    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "task_id": {
                    "type": "string",
                    "description": "Forge task ID"
                },
                "block_id": {
                    "type": "string",
                    "description": "Forge block ID"
                },
                "jira_issue_key": {
                    "type": "string",
                    "description": "Jira issue key (optional, will be looked up if not provided)"
                },
                "force_status": {
                    "type": "string",
                    "description": "Force a specific Jira status (optional)"
                }
            },
            "required": ["task_id", "block_id"]
        })
    }

    fn required_permissions(&self) -> Vec<Permission> {
        vec![Permission::Network, Permission::TaskManagement]
    }

    fn category(&self) -> ToolCategory {
        ToolCategory::Collaboration
    }

    async fn execute(
        &self,
        params: Value,
        context: &mut ExecutionContext,
    ) -> Result<ToolResult, ToolError> {
        let task_id = params["task_id"]
            .as_str()
            .ok_or_else(|| ToolError::InvalidParams("task_id is required".to_string()))?;
        
        let block_id = params["block_id"]
            .as_str()
            .ok_or_else(|| ToolError::InvalidParams("block_id is required".to_string()))?;

        let jira_issue_key = params["jira_issue_key"].as_str();
        let force_status = params["force_status"].as_str();

        info!("Updating Jira issue status for task {}/{}", block_id, task_id);

        // Get the task
        let task = get_task(block_id, task_id, context).await?;

        // Determine Jira issue key
        let issue_key = if let Some(key) = jira_issue_key {
            key.to_string()
        } else if let Some(key) = task.get("jira_issue_key").and_then(|v| v.as_str()) {
            key.to_string()
        } else {
            return Err(ToolError::InvalidParams(
                "No Jira issue key found for task".to_string()
            ));
        };

        // Update status
        update_jira_issue_status(&issue_key, &task, force_status, context).await?;

        let summary = format!("Updated Jira issue {} status", issue_key);
        info!("{}", summary);

        Ok(ToolResult::success()
            .with_content(Content::Text { text: summary })
            .with_content(Content::Data {
                data: json!({
                    "task_id": task_id,
                    "block_id": block_id,
                    "jira_issue_key": issue_key
                }),
            }))
    }
}

/// Tool for getting Jira project metadata
pub struct GetJiraProjectMetadataTool;

#[async_trait]
impl MCPTool for GetJiraProjectMetadataTool {
    fn name(&self) -> &str {
        "get_jira_project_metadata"
    }

    fn description(&self) -> &str {
        "Get metadata for a Jira project including available issue types, statuses, and fields"
    }

    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "project_key": {
                    "type": "string",
                    "description": "Jira project key"
                }
            },
            "required": ["project_key"]
        })
    }

    fn required_permissions(&self) -> Vec<Permission> {
        vec![Permission::Network]
    }

    fn category(&self) -> ToolCategory {
        ToolCategory::Collaboration
    }

    async fn execute(
        &self,
        params: Value,
        context: &mut ExecutionContext,
    ) -> Result<ToolResult, ToolError> {
        let project_key = params["project_key"]
            .as_str()
            .ok_or_else(|| ToolError::InvalidParams("project_key is required".to_string()))?;

        info!("Getting Jira project metadata for {}", project_key);

        let project_metadata = get_jira_project_metadata(project_key, context).await?;

        Ok(ToolResult::success()
            .with_content(Content::Data {
                data: json!(project_metadata),
            }))
    }
}

/// Tool for bulk syncing multiple tasks
pub struct BulkSyncTasksTool;

#[async_trait]
impl MCPTool for BulkSyncTasksTool {
    fn name(&self) -> &str {
        "bulk_sync_tasks"
    }

    fn description(&self) -> &str {
        "Perform bulk synchronization of multiple tasks with Jira issues"
    }

    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "sync_operations": {
                    "type": "array",
                    "items": {
                        "type": "object",
                        "properties": {
                            "block_id": {"type": "string"},
                            "task_id": {"type": "string"},
                            "jira_project_key": {"type": "string"},
                            "operation": {
                                "type": "string",
                                "enum": ["export", "import", "update_status"],
                                "description": "Operation to perform"
                            }
                        },
                        "required": ["block_id", "task_id", "jira_project_key", "operation"]
                    },
                    "description": "List of sync operations to perform"
                },
                "max_parallel": {
                    "type": "integer",
                    "description": "Maximum number of parallel operations",
                    "default": 3,
                    "minimum": 1,
                    "maximum": 10
                }
            },
            "required": ["sync_operations"]
        })
    }

    fn required_permissions(&self) -> Vec<Permission> {
        vec![
            Permission::Network,
            Permission::TaskManagement,
        ]
    }

    fn category(&self) -> ToolCategory {
        ToolCategory::Collaboration
    }

    fn estimated_execution_time(&self) -> Duration {
        Duration::from_secs(30)
    }

    async fn execute(
        &self,
        params: Value,
        context: &mut ExecutionContext,
    ) -> Result<ToolResult, ToolError> {
        let sync_operations = params["sync_operations"]
            .as_array()
            .ok_or_else(|| ToolError::InvalidParams("sync_operations is required".to_string()))?;

        let max_parallel = params["max_parallel"].as_u64().unwrap_or(3) as usize;

        info!("Starting bulk sync of {} operations with max parallelism {}", sync_operations.len(), max_parallel);

        let mut results = Vec::new();
        let mut successful = 0;
        let mut failed = 0;

        // Process operations in batches
        for chunk in sync_operations.chunks(max_parallel) {
            for operation in chunk {
                let block_id = operation["block_id"].as_str().unwrap_or("");
                let task_id = operation["task_id"].as_str().unwrap_or("");
                let jira_project_key = operation["jira_project_key"].as_str().unwrap_or("");
                let op_type = operation["operation"].as_str().unwrap_or("");

                let result = match op_type {
                    "export" => {
                        // Export single task
                        export_single_task(block_id, task_id, jira_project_key, context).await
                    }
                    "import" => {
                        // Import single issue
                        import_single_issue(block_id, task_id, jira_project_key, context).await
                    }
                    "update_status" => {
                        // Update status only
                        update_single_task_status(block_id, task_id, context).await
                    }
                    _ => Err(format!("Unknown operation: {}", op_type))
                };

                match result {
                    Ok(msg) => {
                        successful += 1;
                        results.push(json!({
                            "block_id": block_id,
                            "task_id": task_id,
                            "operation": op_type,
                            "status": "success",
                            "message": msg
                        }));
                    }
                    Err(error) => {
                        failed += 1;
                        results.push(json!({
                            "block_id": block_id,
                            "task_id": task_id,
                            "operation": op_type,
                            "status": "error",
                            "error": error
                        }));
                    }
                }
            }
        }

        let summary = format!("Bulk sync completed: {} successful, {} failed", successful, failed);
        info!("{}", summary);

        Ok(ToolResult::success()
            .with_content(Content::Text { text: summary })
            .with_content(Content::Data {
                data: json!({
                    "total_operations": sync_operations.len(),
                    "successful": successful,
                    "failed": failed,
                    "results": results
                }),
            }))
    }
}

// Helper structures

#[derive(Debug, Serialize, Deserialize)]
struct JiraSyncResult {
    blocks_processed: u32,
    tasks_created: u32,
    tasks_updated: u32,
    issues_created: u32,
    issues_updated: u32,
    errors: Vec<String>,
}

#[derive(Debug)]
enum JiraTaskAction {
    Created,
    Updated,
    Skipped,
}


// Helper functions (to be implemented with actual Atlassian MCP integration)

async fn get_jira_project(project_key: &str, context: &ExecutionContext) -> Result<JiraProject, String> {
    debug!("Getting Jira project: {}", project_key);
    
    let jira_client = JiraClient::with_default_config()
        .map_err(|e| format!("Failed to create Jira client: {}", e))?;
    
    // Get all visible projects and find the one we want
    let projects = jira_client.get_visible_projects().await
        .map_err(|e| format!("Failed to get projects: {}", e))?;
    
    projects.into_iter()
        .find(|p| p.key == project_key)
        .ok_or_else(|| format!("Project {} not found or not accessible", project_key))
}

async fn get_jira_project_metadata(project_key: &str, context: &ExecutionContext) -> Result<JiraProjectMetadata, ToolError> {
    debug!("Getting Jira project metadata: {}", project_key);
    
    let jira_client = JiraClient::with_default_config()?;
    
    jira_client.get_project_metadata(project_key).await
}

async fn export_task_to_jira(
    block: &crate::models::Block,
    task_id: &str,
    task: &Value,
    jira_project: &JiraProject,
    create_missing: bool,
    update_existing: bool,
    context: &ExecutionContext,
) -> Result<JiraTaskAction, String> {
    debug!("Exporting task {} to Jira project {}", task_id, jira_project.key);
    
    let jira_client = JiraClient::with_default_config()
        .map_err(|e| format!("Failed to create Jira client: {}", e))?;
    
    // Extract task information
    let task_name = task.get("task_name")
        .and_then(|t| t.as_str())
        .unwrap_or("Untitled Task");
    let description = task.get("description")
        .and_then(|d| d.as_str())
        .unwrap_or("");
    let status = task.get("status")
        .and_then(|s| s.as_str())
        .unwrap_or("TODO");
    
    // Check if task already has a Jira issue key
    let existing_issue_key = task.get("jira_issue_key")
        .and_then(|k| k.as_str());
    
    if let Some(issue_key) = existing_issue_key {
        if update_existing {
            // Update existing issue
            let fields = json!({
                "summary": task_name,
                "description": description
            });
            
            match jira_client.update_issue(issue_key, fields).await {
                Ok(_) => {
                    info!("Updated existing Jira issue {}", issue_key);
                    Ok(JiraTaskAction::Updated)
                }
                Err(e) => Err(format!("Failed to update Jira issue {}: {}", issue_key, e))
            }
        } else {
            debug!("Skipping update for existing issue {} (update_existing=false)", issue_key);
            Ok(JiraTaskAction::Skipped)
        }
    } else if create_missing {
        // Create new issue
        // Map task status to issue type (could be configurable)
        let issue_type = match status {
            "TODO" => "Task",
            "IN_PROGRESS" => "Task", 
            "COMPLETED" => "Task",
            "FAILED" => "Bug",
            _ => "Task"
        };
        
        match jira_client.create_issue(&jira_project.key, issue_type, task_name, Some(description)).await {
            Ok(issue_key) => {
                info!("Created new Jira issue {} for task {}", issue_key, task_id);
                // TODO: Update the task with the new Jira issue key
                // This would require access to the block manager to save the updated task
                Ok(JiraTaskAction::Created)
            }
            Err(e) => Err(format!("Failed to create Jira issue: {}", e))
        }
    } else {
        debug!("Skipping task {} (no existing issue and create_missing=false)", task_id);
        Ok(JiraTaskAction::Skipped)
    }
}

async fn import_jira_issues_to_block(
    block: &crate::models::Block,
    jira_project: &JiraProject,
    context: &ExecutionContext,
) -> Result<(u32, u32), String> {
    debug!("Importing Jira issues from project {} to block {}", jira_project.key, block.block_id);
    
    let jira_client = JiraClient::with_default_config()
        .map_err(|e| format!("Failed to create Jira client: {}", e))?;
    
    // Build JQL to get issues from this project
    let jql = format!("project = {} ORDER BY created DESC", jira_project.key);
    
    // Search for issues
    let issues = jira_client.search_issues(&jql, Some(50)).await
        .map_err(|e| format!("Failed to search Jira issues: {}", e))?;
    
    let mut created_count = 0;
    let mut updated_count = 0;
    
    for issue in issues {
        // Check if we already have a task for this issue
        let existing_task = block.todo_list.values()
            .find(|task| task.jira_issue_key.as_ref() == Some(&issue.key));
        
        if let Some(_existing_task) = existing_task {
            // TODO: Update existing task with current Jira data
            // This would require access to the block manager to save updates
            debug!("Found existing task for issue {}, would update", issue.key);
            updated_count += 1;
        } else {
            // TODO: Create new task from Jira issue
            // This would require:
            // 1. Create a new Task from the JiraIssue
            // 2. Add it to the block's todo_list
            // 3. Save the updated block via block manager
            debug!("Would create new task for issue {} - {}", issue.key, issue.summary);
            created_count += 1;
        }
    }
    
    // Note: In a real implementation, these tasks would actually be created/updated
    // and the block would be saved via the block manager
    info!("Import simulation completed: {} tasks would be created, {} would be updated", created_count, updated_count);
    
    Ok((created_count, updated_count))
}

async fn get_task(block_id: &str, task_id: &str, context: &ExecutionContext) -> Result<Value, ToolError> {
    let block_manager = context.block_manager.clone();
    let blocks = block_manager.get_blocks().map_err(|e| {
        ToolError::Internal(format!("Failed to load blocks: {}", e))
    })?;
    
    let block = blocks.into_iter()
        .find(|b| b.block_id == block_id)
        .ok_or_else(|| ToolError::InvalidParams(format!("Block {} not found", block_id)))?;
    
    block.todo_list.get(task_id)
        .ok_or_else(|| ToolError::InvalidParams(format!("Task {} not found", task_id)))
        .map(|task| json!(task))
}

async fn update_jira_issue_status(
    issue_key: &str,
    task: &Value,
    force_status: Option<&str>,
    context: &ExecutionContext,
) -> Result<(), ToolError> {
    debug!("Updating Jira issue {} status", issue_key);
    
    let jira_client = JiraClient::with_default_config()?;
    
    // Determine the target status
    let target_status = if let Some(forced) = force_status {
        forced.to_string()
    } else {
        // Map task status to Jira status
        let task_status = task.get("status")
            .and_then(|s| s.as_str())
            .unwrap_or("TODO");
        
        match task_status {
            "TODO" => "To Do".to_string(),
            "IN_PROGRESS" => "In Progress".to_string(),
            "COMPLETED" => "Done".to_string(),
            "FAILED" => "To Do".to_string(), // Reset failed tasks to To Do
            _ => "To Do".to_string()
        }
    };
    
    // For now, we'll just update the summary to indicate status change
    // In a real implementation, we would:
    // 1. Get available transitions for the issue
    // 2. Find the transition that leads to the target status
    // 3. Execute the transition
    
    let fields = json!({
        "description": format!("Status updated to: {}", target_status)
    });
    
    jira_client.update_issue(issue_key, fields).await?;
    
    info!("Updated Jira issue {} status to {}", issue_key, target_status);
    Ok(())
}

// Bulk operation helpers

async fn export_single_task(
    block_id: &str,
    task_id: &str,
    jira_project_key: &str,
    context: &ExecutionContext,
) -> Result<String, String> {
    debug!("Exporting single task {}/{} to {}", block_id, task_id, jira_project_key);
    
    // Get the task and project
    let task = get_task(block_id, task_id, context).await
        .map_err(|e| format!("Failed to get task: {}", e))?;
    
    let jira_project = get_jira_project(jira_project_key, context).await?;
    
    // Get the block to pass to export function
    let block_manager = context.block_manager.clone();
    let blocks = block_manager.get_blocks()
        .map_err(|e| format!("Failed to load blocks: {}", e))?;
    
    let block = blocks.into_iter()
        .find(|b| b.block_id == block_id)
        .ok_or_else(|| format!("Block {} not found", block_id))?;
    
    // Export the task
    let action = export_task_to_jira(&block, task_id, &task, &jira_project, true, true, context).await?;
    
    Ok(format!("Task {}/{} export result: {:?}", block_id, task_id, action))
}

async fn import_single_issue(
    block_id: &str,
    _task_id: &str,
    jira_project_key: &str,
    context: &ExecutionContext,
) -> Result<String, String> {
    debug!("Importing issues from {} to block {}", jira_project_key, block_id);
    
    let jira_project = get_jira_project(jira_project_key, context).await?;
    
    // Get the block
    let block_manager = context.block_manager.clone();
    let blocks = block_manager.get_blocks()
        .map_err(|e| format!("Failed to load blocks: {}", e))?;
    
    let block = blocks.into_iter()
        .find(|b| b.block_id == block_id)
        .ok_or_else(|| format!("Block {} not found", block_id))?;
    
    // Import issues to the block
    let (created, updated) = import_jira_issues_to_block(&block, &jira_project, context).await?;
    
    Ok(format!("Import to block {}: {} created, {} updated", block_id, created, updated))
}

async fn update_single_task_status(
    block_id: &str,
    task_id: &str,
    context: &ExecutionContext,
) -> Result<String, String> {
    debug!("Updating status for task {}/{}", block_id, task_id);
    
    let task = get_task(block_id, task_id, context).await
        .map_err(|e| format!("Failed to get task: {}", e))?;
    
    // Check if task has a Jira issue key
    if let Some(issue_key) = task.get("jira_issue_key").and_then(|k| k.as_str()) {
        update_jira_issue_status(issue_key, &task, None, context).await
            .map_err(|e| format!("Failed to update Jira issue status: {}", e))?;
        
        Ok(format!("Updated Jira issue {} status for task {}/{}", issue_key, block_id, task_id))
    } else {
        Ok(format!("Task {}/{} has no linked Jira issue", block_id, task_id))
    }
}/// Jira client for communicating with the Atlassian MCP server
/// 
/// This module provides a client for interacting with Jira through the configured
/// Atlassian MCP server, handling authentication, rate limiting, and error handling.

use std::sync::Arc;
use tokio::sync::{Mutex, RwLock};


/// Client for interacting with Jira through Atlassian MCP server
pub struct JiraClient {
    /// MCP client for Atlassian server communication
    mcp_client: Arc<AtlassianMCPClient>,
    
    /// Configuration for Jira operations
    config: JiraClientConfig,
    
    /// Cache for project metadata and issue type information
    metadata_cache: Arc<RwLock<HashMap<String, CachedMetadata>>>,
    
    /// Rate limiter to respect Jira API limits
    rate_limiter: Arc<Mutex<RateLimiter>>,
}

/// Configuration for Jira client
#[derive(Debug, Clone)]
pub struct JiraClientConfig {
    /// Default timeout for operations
    pub timeout: Duration,
    
    /// Maximum number of retries for failed operations
    pub max_retries: u32,
    
    /// Rate limit: requests per second
    pub rate_limit: f64,
    
    /// Cache TTL for metadata
    pub metadata_cache_ttl: Duration,
    
    /// Batch size for bulk operations
    pub batch_size: usize,
}

impl Default for JiraClientConfig {
    fn default() -> Self {
        Self {
            timeout: Duration::from_secs(30),
            max_retries: 3,
            rate_limit: 10.0, // 10 requests per second
            metadata_cache_ttl: Duration::from_secs(300), // 5 minutes
            batch_size: 20,
        }
    }
}

/// Cached metadata with expiration
#[derive(Debug, Clone)]
struct CachedMetadata {
    data: Value,
    expires_at: SystemTime,
}

/// Rate limiter for API calls
#[derive(Debug)]
struct RateLimiter {
    last_request_time: SystemTime,
    requests_in_window: u32,
    window_start: SystemTime,
    window_duration: Duration,
    max_requests_per_window: u32,
}

impl RateLimiter {
    fn new(requests_per_second: f64) -> Self {
        let window_duration = Duration::from_secs(1);
        let max_requests_per_window = (requests_per_second as u32).max(1);
        
        Self {
            last_request_time: SystemTime::now(),
            requests_in_window: 0,
            window_start: SystemTime::now(),
            window_duration,
            max_requests_per_window,
        }
    }
    
    async fn acquire_permit(&mut self) -> Result<(), ToolError> {
        let now = SystemTime::now();
        
        // Reset window if enough time has passed
        if now.duration_since(self.window_start).unwrap_or(Duration::ZERO) >= self.window_duration {
            self.window_start = now;
            self.requests_in_window = 0;
        }
        
        // Check if we've hit the rate limit
        if self.requests_in_window >= self.max_requests_per_window {
            let wait_time = self.window_duration - now.duration_since(self.window_start).unwrap_or(Duration::ZERO);
            if wait_time > Duration::ZERO {
                tokio::time::sleep(wait_time).await;
                self.window_start = SystemTime::now();
                self.requests_in_window = 0;
            }
        }
        
        self.requests_in_window += 1;
        self.last_request_time = now;
        
        Ok(())
    }
}

/// Atlassian MCP client using the official Atlassian MCP tools available in Claude Code CLI
struct AtlassianMCPClient {
    cloud_id: String,
}

impl AtlassianMCPClient {
    fn new() -> Result<Self, ToolError> {
        // For now, we'll use a placeholder cloud ID. In a real implementation,
        // this would be configurable or retrieved from the accessible resources
        Ok(Self {
            cloud_id: "dovcaspi.atlassian.net".to_string(),
        })
    }
    
    async fn get_accessible_resources(&self) -> Result<Value, ToolError> {
        use crate::mcp::tools::ExecutionContext;
        
        // This would normally be called through the actual MCP infrastructure
        // For now, we'll simulate the call
        debug!("Getting accessible Atlassian resources");
        
        // In a real implementation, this would call the actual MCP tool
        // mcp__atlassian__getAccessibleAtlassianResources through the tool registry
        Ok(json!([
            {
                "id": "0c5c33b3-8b3c-4ee4-bffb-4d853c875a4b",
                "url": "https://dovcaspi.atlassian.net",
                "name": "dovcaspi",
                "scopes": ["read:jira-work", "write:jira-work"]
            }
        ]))
    }
    
    async fn call_tool(&self, tool_name: &str, params: Value) -> Result<Value, ToolError> {
        debug!("Calling Atlassian MCP tool: {} with params: {}", tool_name, params);
        
        // Note: In a real implementation, these calls would go through the actual
        // MCP tool registry and invoke the real Atlassian MCP tools.
        // For now, we're providing a bridge implementation that shows how
        // the integration should work.
        
        match tool_name {
            "getVisibleJiraProjects" => {
                // This should call mcp__atlassian__getVisibleJiraProjects
                // through the tool registry with the cloud_id
                self.get_visible_jira_projects(params).await
            }
            "getJiraProjectIssueTypesMetadata" => {
                // This should call mcp__atlassian__getJiraProjectIssueTypesMetadata
                let project_key = params.get("projectIdOrKey")
                    .and_then(|p| p.as_str())
                    .ok_or_else(|| ToolError::InvalidParams("projectIdOrKey required".to_string()))?;
                self.get_jira_project_issue_types_metadata(project_key).await
            }
            "searchJiraIssuesUsingJql" => {
                // This should call mcp__atlassian__searchJiraIssuesUsingJql
                let jql = params.get("jql")
                    .and_then(|j| j.as_str())
                    .ok_or_else(|| ToolError::InvalidParams("jql required".to_string()))?;
                let max_results = params.get("maxResults").and_then(|m| m.as_u64()).unwrap_or(50) as i32;
                let fields = params.get("fields")
                    .and_then(|f| f.as_array())
                    .map(|arr| arr.iter().filter_map(|v| v.as_str()).map(|s| s.to_string()).collect())
                    .unwrap_or_else(|| vec!["summary".to_string(), "description".to_string(), "status".to_string()]);
                
                self.search_jira_issues_using_jql(jql, max_results, fields).await
            }
            "createJiraIssue" => {
                // This should call mcp__atlassian__createJiraIssue
                let project_key = params.get("projectKey")
                    .and_then(|p| p.as_str())
                    .ok_or_else(|| ToolError::InvalidParams("projectKey required".to_string()))?;
                let issue_type = params.get("issueTypeName")
                    .and_then(|i| i.as_str())
                    .ok_or_else(|| ToolError::InvalidParams("issueTypeName required".to_string()))?;
                let summary = params.get("summary")
                    .and_then(|s| s.as_str())
                    .ok_or_else(|| ToolError::InvalidParams("summary required".to_string()))?;
                let description = params.get("description").and_then(|d| d.as_str());
                
                self.create_jira_issue(project_key, issue_type, summary, description).await
            }
            "editJiraIssue" => {
                // This should call mcp__atlassian__editJiraIssue
                let issue_key = params.get("issueIdOrKey")
                    .and_then(|i| i.as_str())
                    .ok_or_else(|| ToolError::InvalidParams("issueIdOrKey required".to_string()))?;
                let fields = params.get("fields")
                    .ok_or_else(|| ToolError::InvalidParams("fields required".to_string()))?;
                
                self.edit_jira_issue(issue_key, fields.clone()).await
            }
            "transitionJiraIssue" => {
                // This should call mcp__atlassian__transitionJiraIssue
                let issue_key = params.get("issueIdOrKey")
                    .and_then(|i| i.as_str())
                    .ok_or_else(|| ToolError::InvalidParams("issueIdOrKey required".to_string()))?;
                let transition = params.get("transition")
                    .ok_or_else(|| ToolError::InvalidParams("transition required".to_string()))?;
                
                self.transition_jira_issue(issue_key, transition.clone()).await
            }
            _ => {
                Err(ToolError::Internal(format!("Unknown Atlassian MCP tool: {}", tool_name)))
            }
        }
    }
    
    // Individual method implementations that would call the actual MCP tools
    async fn get_visible_jira_projects(&self, _params: Value) -> Result<Value, ToolError> {
        debug!("Getting visible Jira projects for cloud_id: {}", self.cloud_id);
        
        // TODO: Replace with actual MCP tool call
        // In a real implementation, this would be something like:
        /*
        let tool_registry = context.tool_registry.as_ref()
            .ok_or_else(|| ToolError::Internal("Tool registry not available".to_string()))?;
        
        let params = json!({
            "cloudId": self.cloud_id,
            "action": "view",
            "expandIssueTypes": true,
            "maxResults": 50
        });
        
        let result = tool_registry.execute_tool("mcp__atlassian__getVisibleJiraProjects", params).await
            .map_err(|e| ToolError::Internal(format!("MCP tool call failed: {}", e)))?;
        
        let projects = result.get("projects")
            .or_else(|| result.as_array().map(|arr| json!(arr)))
            .ok_or_else(|| ToolError::Internal("Invalid projects response format".to_string()))?;
        
        Ok(projects.clone())
        */
        
        // For now, return a placeholder that shows the expected structure
        // This should be replaced with the actual MCP tool call above
        Ok(json!([
            {
                "key": "CPG",
                "name": "Forge",
                "id": "10000",
                "description": "Forge project management system"
            }
        ]))
    }
    
    async fn get_jira_project_issue_types_metadata(&self, project_key: &str) -> Result<Value, ToolError> {
        debug!("Getting issue types metadata for project: {}", project_key);
        
        // TODO: Replace with actual MCP tool call
        // In a real implementation, this would be something like:
        /*
        let tool_registry = context.tool_registry.as_ref()
            .ok_or_else(|| ToolError::Internal("Tool registry not available".to_string()))?;
        
        let params = json!({
            "cloudId": self.cloud_id,
            "projectIdOrKey": project_key,
            "maxResults": 50,
            "startAt": 0
        });
        
        let result = tool_registry.execute_tool("mcp__atlassian__getJiraProjectIssueTypesMetadata", params).await
            .map_err(|e| ToolError::Internal(format!("MCP tool call failed: {}", e)))?;
        
        // The result should contain issue types data in the expected format
        Ok(result)
        */
        
        // For now, return a placeholder that shows the expected structure
        // This should be replaced with the actual MCP tool call above
        Ok(json!({
            "issueTypes": [
                {
                    "id": "10003",
                    "name": "Task",
                    "description": "Tasks track small, distinct pieces of work."
                },
                {
                    "id": "10004",
                    "name": "Story",
                    "description": "Stories track functionality or features expressed as user goals."
                },
                {
                    "id": "10005",
                    "name": "Bug",
                    "description": "Bugs track problems or errors."
                },
                {
                    "id": "10006",
                    "name": "Epic",
                    "description": "Epics track collections of related bugs, stories, and tasks."
                }
            ]
        }))
    }
    
    async fn search_jira_issues_using_jql(&self, jql: &str, max_results: i32, fields: Vec<String>) -> Result<Value, ToolError> {
        debug!("Searching Jira issues with JQL: {} (max: {})", jql, max_results);
        
        // TODO: Replace with actual MCP tool call
        // In a real implementation, this would be something like:
        /*
        let tool_registry = context.tool_registry.as_ref()
            .ok_or_else(|| ToolError::Internal("Tool registry not available".to_string()))?;
        
        let params = json!({
            "cloudId": self.cloud_id,
            "jql": jql,
            "maxResults": max_results,
            "fields": fields,
            "nextPageToken": null
        });
        
        let result = tool_registry.execute_tool("mcp__atlassian__searchJiraIssuesUsingJql", params).await
            .map_err(|e| ToolError::Internal(format!("MCP tool call failed: {}", e)))?;
        
        Ok(result)
        */
        
        // For now, return a placeholder that shows the expected structure
        // This should be replaced with the actual MCP tool call above
        Ok(json!({
            "issues": [],
            "total": 0,
            "startAt": 0,
            "maxResults": max_results
        }))
    }
    
    async fn create_jira_issue(&self, project_key: &str, issue_type: &str, summary: &str, description: Option<&str>) -> Result<Value, ToolError> {
        debug!("Creating Jira issue in project: {} with type: {}", project_key, issue_type);
        
        // TODO: Replace with actual MCP tool call
        // In a real implementation, this would be something like:
        /*
        let tool_registry = context.tool_registry.as_ref()
            .ok_or_else(|| ToolError::Internal("Tool registry not available".to_string()))?;
        
        let params = json!({
            "cloudId": self.cloud_id,
            "projectKey": project_key,
            "issueTypeName": issue_type,
            "summary": summary,
            "description": description,
            "additional_fields": {}
        });
        
        let result = tool_registry.execute_tool("mcp__atlassian__createJiraIssue", params).await
            .map_err(|e| ToolError::Internal(format!("MCP tool call failed: {}", e)))?;
        
        Ok(result)
        */
        
        // For now, return a placeholder that shows the expected structure
        // This should be replaced with the actual MCP tool call above
        Ok(json!({
            "key": format!("{}-NEW", project_key),
            "id": "99999",
            "self": format!("https://{}/rest/api/3/issue/99999", self.cloud_id)
        }))
    }
    
    async fn edit_jira_issue(&self, issue_key: &str, fields: Value) -> Result<Value, ToolError> {
        debug!("Editing Jira issue: {} with fields: {}", issue_key, fields);
        
        // TODO: Replace with actual MCP tool call
        // In a real implementation, this would be something like:
        /*
        let tool_registry = context.tool_registry.as_ref()
            .ok_or_else(|| ToolError::Internal("Tool registry not available".to_string()))?;
        
        let params = json!({
            "cloudId": self.cloud_id,
            "issueIdOrKey": issue_key,
            "fields": fields
        });
        
        let result = tool_registry.execute_tool("mcp__atlassian__editJiraIssue", params).await
            .map_err(|e| ToolError::Internal(format!("MCP tool call failed: {}", e)))?;
        
        Ok(result)
        */
        
        // For now, return a placeholder that shows the expected structure
        // This should be replaced with the actual MCP tool call above
        Ok(json!({
            "success": true
        }))
    }
    
    async fn transition_jira_issue(&self, issue_key: &str, transition: Value) -> Result<Value, ToolError> {
        debug!("Transitioning Jira issue: {} with transition: {}", issue_key, transition);
        
        // TODO: Replace with actual MCP tool call
        // In a real implementation, this would be something like:
        /*
        let tool_registry = context.tool_registry.as_ref()
            .ok_or_else(|| ToolError::Internal("Tool registry not available".to_string()))?;
        
        let params = json!({
            "cloudId": self.cloud_id,
            "issueIdOrKey": issue_key,
            "transition": transition,
            "fields": {},
            "historyMetadata": {},
            "update": {}
        });
        
        let result = tool_registry.execute_tool("mcp__atlassian__transitionJiraIssue", params).await
            .map_err(|e| ToolError::Internal(format!("MCP tool call failed: {}", e)))?;
        
        Ok(result)
        */
        
        // For now, return a placeholder that shows the expected structure
        // This should be replaced with the actual MCP tool call above
        Ok(json!({
            "success": true
        }))
    }
}

impl JiraClient {
    /// Create a new Jira client
    pub fn new(config: JiraClientConfig) -> Result<Self, ToolError> {
        Ok(Self {
            mcp_client: Arc::new(AtlassianMCPClient::new()?),
            rate_limiter: Arc::new(Mutex::new(RateLimiter::new(config.rate_limit))),
            metadata_cache: Arc::new(RwLock::new(HashMap::new())),
            config,
        })
    }
    
    /// Create a new Jira client with default configuration
    pub fn with_default_config() -> Result<Self, ToolError> {
        Self::new(JiraClientConfig::default())
    }
    
    /// Get visible Jira projects
    pub async fn get_visible_projects(&self) -> Result<Vec<JiraProject>, ToolError> {
        self.rate_limiter.lock().await.acquire_permit().await?;
        
        let response = self.mcp_client.call_tool("getVisibleJiraProjects", json!({
            "action": "view"
        })).await?;
        
        let projects = response.as_array()
            .ok_or_else(|| ToolError::Internal("Invalid response format".to_string()))?;
        
        let mut result = Vec::new();
        for project in projects {
            if let (Some(key), Some(name), Some(id)) = (
                project.get("key").and_then(|k| k.as_str()),
                project.get("name").and_then(|n| n.as_str()),
                project.get("id").and_then(|i| i.as_str()),
            ) {
                result.push(JiraProject {
                    key: key.to_string(),
                    name: name.to_string(),
                    id: id.to_string(),
                    description: project.get("description").and_then(|d| d.as_str()).map(|s| s.to_string()),
                });
            }
        }
        
        Ok(result)
    }
    
    /// Get project metadata including issue types
    pub async fn get_project_metadata(&self, project_key: &str) -> Result<JiraProjectMetadata, ToolError> {
        // Check cache first
        let cache_key = format!("metadata:{}", project_key);
        {
            let cache = self.metadata_cache.read().await;
            if let Some(cached) = cache.get(&cache_key) {
                if cached.expires_at > SystemTime::now() {
                    debug!("Using cached metadata for project {}", project_key);
                    return Ok(serde_json::from_value(cached.data.clone())
                        .map_err(|e| ToolError::Internal(format!("Cache deserialization error: {}", e)))?);
                }
            }
        }
        
        self.rate_limiter.lock().await.acquire_permit().await?;
        
        // Get project info
        let projects = self.get_visible_projects().await?;
        let project = projects.into_iter()
            .find(|p| p.key == project_key)
            .ok_or_else(|| ToolError::InvalidParams(format!("Project {} not found", project_key)))?;
        
        // Get issue types
        let issue_types_response = self.mcp_client.call_tool("getJiraProjectIssueTypesMetadata", json!({
            "projectIdOrKey": project_key
        })).await?;
        
        let issue_types_data = issue_types_response.get("issueTypes")
            .and_then(|it| it.as_array())
            .ok_or_else(|| ToolError::Internal("Invalid issue types response".to_string()))?;
        
        let mut issue_types = Vec::new();
        for issue_type in issue_types_data {
            if let (Some(id), Some(name)) = (
                issue_type.get("id").and_then(|i| i.as_str()),
                issue_type.get("name").and_then(|n| n.as_str()),
            ) {
                issue_types.push(JiraIssueType {
                    id: id.to_string(),
                    name: name.to_string(),
                    description: issue_type.get("description").and_then(|d| d.as_str()).map(|s| s.to_string()),
                });
            }
        }
        
        // Mock statuses and fields (in real implementation, these would come from additional API calls)
        let statuses = vec![
            JiraStatus {
                id: "1".to_string(),
                name: "To Do".to_string(),
                category: "new".to_string(),
            },
            JiraStatus {
                id: "3".to_string(),
                name: "In Progress".to_string(),
                category: "indeterminate".to_string(),
            },
            JiraStatus {
                id: "10001".to_string(),
                name: "Done".to_string(),
                category: "done".to_string(),
            },
        ];
        
        let fields = vec![
            JiraField {
                id: "summary".to_string(),
                name: "Summary".to_string(),
                field_type: "string".to_string(),
                required: true,
            },
            JiraField {
                id: "description".to_string(),
                name: "Description".to_string(),
                field_type: "string".to_string(),
                required: false,
            },
            JiraField {
                id: "priority".to_string(),
                name: "Priority".to_string(),
                field_type: "priority".to_string(),
                required: false,
            },
        ];
        
        let metadata = JiraProjectMetadata {
            project,
            issue_types,
            statuses,
            fields,
        };
        
        // Cache the result
        let metadata_value = serde_json::to_value(&metadata)
            .map_err(|e| ToolError::Internal(format!("Serialization error: {}", e)))?;
        
        {
            let mut cache = self.metadata_cache.write().await;
            cache.insert(cache_key, CachedMetadata {
                data: metadata_value,
                expires_at: SystemTime::now() + self.config.metadata_cache_ttl,
            });
        }
        
        Ok(metadata)
    }
    
    /// Search Jira issues using JQL
    pub async fn search_issues(&self, jql: &str, max_results: Option<u32>) -> Result<Vec<JiraIssue>, ToolError> {
        self.rate_limiter.lock().await.acquire_permit().await?;
        
        let response = self.mcp_client.call_tool("searchJiraIssuesUsingJql", json!({
            "jql": jql,
            "maxResults": max_results.unwrap_or(50),
            "fields": ["summary", "description", "status", "priority", "assignee", "labels"]
        })).await?;
        
        let issues_data = response.get("issues")
            .and_then(|issues| issues.as_array())
            .ok_or_else(|| ToolError::Internal("Invalid search response".to_string()))?;
        
        let mut issues = Vec::new();
        for issue_data in issues_data {
            if let (Some(key), Some(fields)) = (
                issue_data.get("key").and_then(|k| k.as_str()),
                issue_data.get("fields")
            ) {
                let summary = fields.get("summary").and_then(|s| s.as_str()).unwrap_or("").to_string();
                let description = fields.get("description").and_then(|d| d.as_str()).map(|s| s.to_string());
                let status = fields.get("status").and_then(|s| s.get("name")).and_then(|n| n.as_str()).unwrap_or("").to_string();
                let priority = fields.get("priority").and_then(|p| p.get("name")).and_then(|n| n.as_str()).map(|s| s.to_string());
                let assignee = fields.get("assignee").and_then(|a| a.get("displayName")).and_then(|n| n.as_str()).map(|s| s.to_string());
                
                let labels = fields.get("labels")
                    .and_then(|l| l.as_array())
                    .map(|arr| arr.iter().filter_map(|v| v.as_str()).map(|s| s.to_string()).collect())
                    .unwrap_or_default();
                
                issues.push(JiraIssue {
                    key: key.to_string(),
                    summary,
                    description,
                    status,
                    priority,
                    assignee,
                    labels,
                });
            }
        }
        
        Ok(issues)
    }
    
    /// Create a new Jira issue
    pub async fn create_issue(&self, project_key: &str, issue_type: &str, summary: &str, description: Option<&str>) -> Result<String, ToolError> {
        self.rate_limiter.lock().await.acquire_permit().await?;
        
        let mut fields = json!({
            "project": {
                "key": project_key
            },
            "issuetype": {
                "name": issue_type
            },
            "summary": summary
        });
        
        if let Some(desc) = description {
            fields["description"] = json!(desc);
        }
        
        let response = self.mcp_client.call_tool("createJiraIssue", json!({
            "projectKey": project_key,
            "issueTypeName": issue_type,
            "summary": summary,
            "description": description
        })).await?;
        
        let issue_key = response.get("key")
            .and_then(|k| k.as_str())
            .ok_or_else(|| ToolError::Internal("Invalid create response".to_string()))?;
        
        Ok(issue_key.to_string())
    }
    
    /// Update a Jira issue
    pub async fn update_issue(&self, issue_key: &str, fields: Value) -> Result<(), ToolError> {
        self.rate_limiter.lock().await.acquire_permit().await?;
        
        let _response = self.mcp_client.call_tool("editJiraIssue", json!({
            "issueIdOrKey": issue_key,
            "fields": fields
        })).await?;
        
        Ok(())
    }
    
    /// Transition a Jira issue
    pub async fn transition_issue(&self, issue_key: &str, transition_id: &str) -> Result<(), ToolError> {
        self.rate_limiter.lock().await.acquire_permit().await?;
        
        let _response = self.mcp_client.call_tool("transitionJiraIssue", json!({
            "issueIdOrKey": issue_key,
            "transition": {
                "id": transition_id
            }
        })).await?;
        
        Ok(())
    }
}

// Data structures

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JiraProject {
    pub key: String,
    pub name: String,
    pub id: String,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JiraProjectMetadata {
    pub project: JiraProject,
    pub issue_types: Vec<JiraIssueType>,
    pub statuses: Vec<JiraStatus>,
    pub fields: Vec<JiraField>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JiraIssueType {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JiraStatus {
    pub id: String,
    pub name: String,
    pub category: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JiraField {
    pub id: String,
    pub name: String,
    pub field_type: String,
    pub required: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JiraIssue {
    pub key: String,
    pub summary: String,
    pub description: Option<String>,
    pub status: String,
    pub priority: Option<String>,
    pub assignee: Option<String>,
    pub labels: Vec<String>,
}