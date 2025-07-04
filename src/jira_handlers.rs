/// HTTP handlers for Jira integration API endpoints
/// 
/// This module provides REST API endpoints for the frontend to interact with Jira
/// through the configured Atlassian MCP server and Forge's Jira integration tools.

use actix_web::{web, HttpResponse, Result};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::sync::Arc;
use tracing::{debug, error, info, warn};
use uuid;
use chrono;

use crate::mcp::tools::jira::{JiraClient, JiraProject, JiraIssue};
use crate::mcp::tools::{ExecutionContext, ToolRegistry};
use crate::models::{ClaudeSessionManager, Block, Task};
use crate::block_config::BlockConfigManager;
use crate::project_config::ProjectConfigManager;
use crate::jira_mapping::{JiraMapper, SyncResult, SyncConflict};

/// App state for Jira handlers
#[derive(Clone)]
pub struct JiraAppState {
    pub jira_client: Arc<JiraClient>,
    pub tool_registry: Arc<ToolRegistry>,
    pub project_config: Arc<ProjectConfigManager>,
    pub block_manager: Arc<BlockConfigManager>,
    pub claude_session_manager: Arc<ClaudeSessionManager>,
}

/// Request payload for Jira sync operation
#[derive(Debug, Clone, Deserialize)]
pub struct JiraSyncRequest {
    pub jira_project: String,
    pub sync_mode: String, // "import", "export", "bidirectional"
    pub create_blocks_from_projects: Option<bool>,
    pub create_tasks_from_issues: Option<bool>,
    pub include_epics: Option<bool>,
    pub include_stories: Option<bool>,
    pub include_tasks: Option<bool>,
    pub include_bugs: Option<bool>,
    pub status_filter: Option<String>, // "all", "open", "closed"
    pub assignee_filter: Option<String>, // "all", "me", "unassigned"
}

/// Response for Jira sync operation
#[derive(Debug, Serialize)]
pub struct JiraSyncResponse {
    pub success: bool,
    pub message: String,
    pub blocks_created: u32,
    pub tasks_created: u32,
    pub tasks_updated: u32,
    pub issues_created: u32,
    pub issues_updated: u32,
    pub errors: Vec<String>,
}

/// Response for Jira projects list
#[derive(Debug, Serialize)]
pub struct JiraProjectsResponse {
    pub projects: Vec<JiraProjectInfo>,
}

#[derive(Debug, Serialize)]
pub struct JiraProjectInfo {
    pub key: String,
    pub name: String,
    pub id: String,
    pub description: Option<String>,
}

/// GET /api/jira/projects - Get available Jira projects
pub async fn get_jira_projects_handler(
    data: web::Data<JiraAppState>,
) -> Result<HttpResponse> {
    info!("Getting Jira projects");

    match data.jira_client.get_visible_projects().await {
        Ok(projects) => {
            let project_infos: Vec<JiraProjectInfo> = projects
                .into_iter()
                .map(|p| JiraProjectInfo {
                    key: p.key,
                    name: p.name,
                    id: p.id,
                    description: p.description,
                })
                .collect();

            Ok(HttpResponse::Ok().json(JiraProjectsResponse {
                projects: project_infos,
            }))
        }
        Err(e) => {
            error!("Failed to get Jira projects: {}", e);
            Ok(HttpResponse::InternalServerError().json(json!({
                "error": "Failed to get Jira projects",
                "details": e.to_string()
            })))
        }
    }
}

/// GET /api/jira/projects/{project_key}/metadata - Get project metadata
pub async fn get_jira_project_metadata_handler(
    path: web::Path<String>,
    data: web::Data<JiraAppState>,
) -> Result<HttpResponse> {
    let project_key = path.into_inner();
    info!("Getting Jira project metadata for {}", project_key);

    match data.jira_client.get_project_metadata(&project_key).await {
        Ok(metadata) => Ok(HttpResponse::Ok().json(metadata)),
        Err(e) => {
            error!("Failed to get project metadata for {}: {}", project_key, e);
            Ok(HttpResponse::InternalServerError().json(json!({
                "error": "Failed to get project metadata",
                "details": e.to_string()
            })))
        }
    }
}

/// POST /api/jira/sync - Main Jira sync endpoint
pub async fn jira_sync_handler(
    req: web::Json<JiraSyncRequest>,
    data: web::Data<JiraAppState>,
) -> Result<HttpResponse> {
    info!("Starting Jira sync: {:?}", req);

    let sync_request = req.into_inner();

    // Create execution context
    let context = ExecutionContext {
        session_id: format!("jira-sync-{}", uuid::Uuid::new_v4()),
        project_config: data.project_config.clone(),
        block_manager: data.block_manager.clone(),
        working_directory: std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from(".")),
        context_store: Arc::new(tokio::sync::RwLock::new(crate::mcp::context::ContextStore::new())),
        execution_history: Vec::new(),
        user_preferences: crate::mcp::tools::UserPreferences::default(),
        permissions: crate::mcp::tools::SessionPermissions::default(),
        performance_tracker: Arc::new(tokio::sync::Mutex::new(crate::mcp::tools::PerformanceTracker::default())),
        tool_registry: Some(data.tool_registry.clone()),
        claude_session_manager: Some(data.claude_session_manager.clone()),
    };

    // Perform sync operation based on mode
    match sync_request.sync_mode.as_str() {
        "import" => handle_import_sync(sync_request, &context, &data).await,
        "export" => handle_export_sync(sync_request, &context, &data).await,
        "bidirectional" => handle_bidirectional_sync(sync_request, &context, &data).await,
        _ => Ok(HttpResponse::BadRequest().json(json!({
            "error": "Invalid sync_mode",
            "details": "sync_mode must be one of: import, export, bidirectional"
        })))
    }
}

async fn handle_import_sync(
    request: JiraSyncRequest,
    context: &ExecutionContext,
    data: &JiraAppState,
) -> Result<HttpResponse> {
    info!("Handling import sync from Jira project {}", request.jira_project);

    // Build JQL query based on filters
    let mut jql_parts = vec![format!("project = {}", request.jira_project)];

    if let Some(status_filter) = &request.status_filter {
        match status_filter.as_str() {
            "open" => jql_parts.push("status NOT IN (Done, Closed, Resolved)".to_string()),
            "closed" => jql_parts.push("status IN (Done, Closed, Resolved)".to_string()),
            _ => {} // "all" - no filter
        }
    }

    // Add issue type filters
    let mut issue_types = Vec::new();
    if request.include_epics.unwrap_or(true) {
        issue_types.push("Epic");
    }
    if request.include_stories.unwrap_or(true) {
        issue_types.push("Story");
    }
    if request.include_tasks.unwrap_or(true) {
        issue_types.push("Task");
    }
    if request.include_bugs.unwrap_or(true) {
        issue_types.push("Bug");
    }

    if !issue_types.is_empty() {
        jql_parts.push(format!("issuetype IN ({})", issue_types.join(", ")));
    }

    let jql = jql_parts.join(" AND ");
    debug!("Using JQL query: {}", jql);

    // Search for issues
    match data.jira_client.search_issues(&jql, Some(100)).await {
        Ok(issues) => {
            let mut tasks_created = 0;
            let mut tasks_updated = 0;
            let mut errors = Vec::new();

            // For each issue, create or update a corresponding task
            // This is a simplified implementation - in reality, you'd want to:
            // 1. Group issues by project/epic to create blocks
            // 2. Handle dependencies between issues
            // 3. Map custom fields properly

            for issue in issues {
                match create_task_from_jira_issue(&issue, &request.jira_project, context, data).await {
                    Ok(true) => tasks_created += 1,
                    Ok(false) => tasks_updated += 1,
                    Err(e) => errors.push(format!("Failed to sync issue {}: {}", issue.key, e)),
                }
            }

            let success = errors.is_empty();
            let message = if success {
                format!("Import completed successfully: {} tasks created, {} tasks updated", tasks_created, tasks_updated)
            } else {
                format!("Import completed with {} errors", errors.len())
            };

            Ok(HttpResponse::Ok().json(JiraSyncResponse {
                success,
                message,
                blocks_created: if tasks_created > 0 { 1 } else { 0 }, // Simplified
                tasks_created,
                tasks_updated,
                issues_created: 0,
                issues_updated: 0,
                errors,
            }))
        }
        Err(e) => {
            error!("Failed to search Jira issues: {}", e);
            Ok(HttpResponse::InternalServerError().json(json!({
                "error": "Failed to search Jira issues",
                "details": e.to_string()
            })))
        }
    }
}

async fn handle_export_sync(
    request: JiraSyncRequest,
    context: &ExecutionContext,
    data: &JiraAppState,
) -> Result<HttpResponse> {
    info!("Handling export sync to Jira project {}", request.jira_project);

    // Get all blocks and their tasks
    let blocks = match data.block_manager.get_blocks() {
        Ok(blocks) => blocks,
        Err(e) => {
            error!("Failed to load blocks: {}", e);
            return Ok(HttpResponse::InternalServerError().json(json!({
                "error": "Failed to load blocks",
                "details": e.to_string()
            })));
        }
    };

    let mut issues_created = 0;
    let mut issues_updated = 0;
    let mut errors = Vec::new();

    // For each block, create or update corresponding Jira issues
    for block in blocks {
        for (task_id, task) in &block.todo_list {
            // Check if task already has a Jira issue key
            let jira_issue_key = None; // TODO: Add jira_issue_key field to Task struct
            
            match export_task_to_jira_issue(&task.to_value(), &request.jira_project, jira_issue_key, context, data).await {
                Ok(true) => issues_created += 1,
                Ok(false) => issues_updated += 1,
                Err(e) => errors.push(format!("Failed to export task {}: {}", task_id, e)),
            }
        }
    }

    let success = errors.is_empty();
    let message = if success {
        format!("Export completed successfully: {} issues created, {} issues updated", issues_created, issues_updated)
    } else {
        format!("Export completed with {} errors", errors.len())
    };

    Ok(HttpResponse::Ok().json(JiraSyncResponse {
        success,
        message,
        blocks_created: 0,
        tasks_created: 0,
        tasks_updated: 0,
        issues_created,
        issues_updated,
        errors,
    }))
}

async fn handle_bidirectional_sync(
    request: JiraSyncRequest,
    context: &ExecutionContext,
    data: &JiraAppState,
) -> Result<HttpResponse> {
    info!("Handling bidirectional sync with Jira project {}", request.jira_project);

    let mapper = JiraMapper::new();
    let mut sync_result = SyncResult::default();

    // Get all blocks from the block manager
    let blocks = match data.block_manager.get_blocks() {
        Ok(blocks) => blocks,
        Err(e) => {
            error!("Failed to load blocks: {}", e);
            return Ok(HttpResponse::InternalServerError().json(json!({
                "error": "Failed to load blocks",
                "details": e.to_string()
            })));
        }
    };

    // Get Jira issues for the project
    let jql = format!("project = {}", request.jira_project);
    let jira_issues = match data.jira_client.search_issues(&jql, Some(100)).await {
        Ok(issues) => issues,
        Err(e) => {
            error!("Failed to search Jira issues: {}", e);
            sync_result.errors.push(format!("Failed to search Jira issues: {}", e));
            return Ok(HttpResponse::Ok().json(sync_result));
        }
    };

    // Create a map of existing Jira issues by custom field (Forge task ID)
    // Note: In a real implementation, we would use custom fields or issue descriptions
    // to store the Forge task ID mapping. For now, we'll use issue key mapping.
    let mut jira_issues_by_task_id: HashMap<String, &JiraIssue> = HashMap::new();
    for issue in &jira_issues {
        // In a real implementation, we would extract the Forge task ID from a custom field
        // For now, we'll just use the issue key as a placeholder
        jira_issues_by_task_id.insert(issue.key.clone(), issue);
    }

    // Process each block and its tasks
    for mut block in blocks {
        // Only sync blocks that are configured for Jira sync
        if !block.jira_sync_settings.auto_sync && block.jira_project_key.is_none() {
            continue;
        }

        // Set the block's Jira project if not already set
        if block.jira_project_key.is_none() {
            block.jira_project_key = Some(request.jira_project.clone());
        }

        let mut block_updated = false;

        // Process each task in the block
        for (task_id, mut task) in block.todo_list.clone() {
            match sync_task_bidirectionally(&mut task, &block, &jira_issues_by_task_id, &mapper, data).await {
                Ok(task_sync_result) => {
                    // Update the task in the block
                    block.todo_list.insert(task_id, task);
                    block_updated = true;
                    
                    // Aggregate results
                    sync_result.tasks_synced += 1;
                    if task_sync_result.issue_created {
                        sync_result.issues_created += 1;
                    }
                    if task_sync_result.issue_updated {
                        sync_result.issues_updated += 1;
                    }
                    sync_result.conflicts_detected += task_sync_result.conflicts.len() as u32;
                    sync_result.conflicts.extend(task_sync_result.conflicts);
                }
                Err(e) => {
                    sync_result.errors.push(format!("Failed to sync task {}: {}", task_id, e));
                }
            }
        }

        // Save the updated block if changes were made
        if block_updated {
            // Update the block's last sync time
            block.jira_last_sync = Some(chrono::Utc::now().to_rfc3339());
            block.jira_synced = true;

            // Save the updated block back to the manager
            // Note: This would need to be implemented in BlockConfigManager
            // For now, we'll just log that it should be saved
            info!("Block {} has been updated and should be saved", block.block_id);
        }
    }

    // Handle any Jira issues that don't have corresponding Forge tasks
    for issue in jira_issues.iter() {
        // In a real implementation, we would check if this issue has a corresponding Forge task
        // For now, we'll just log that we found the issue
        info!("Found Jira issue {} - {}", issue.key, issue.summary);
        
        // For demonstration, we could import some issues as new tasks
        // but we'll skip this to avoid creating duplicate mappings
    }

    // Resolve conflicts based on block settings
    for conflict in &mut sync_result.conflicts {
        // Apply default resolution strategy - in a real implementation,
        // this would use the block's conflict resolution settings
        if conflict.resolution.is_none() {
            conflict.resolution = Some(crate::jira_mapping::ConflictResolution::Manual);
        }
    }

    sync_result.conflicts_resolved = sync_result.conflicts.iter()
        .filter(|c| c.resolution.is_some())
        .count() as u32;

    sync_result.success = sync_result.errors.is_empty();

    let message = format!(
        "Bidirectional sync completed: {} tasks synced, {} issues created, {} issues updated, {} conflicts detected",
        sync_result.tasks_synced, sync_result.issues_created, sync_result.issues_updated, sync_result.conflicts_detected
    );

    Ok(HttpResponse::Ok().json(JiraSyncResponse {
        success: sync_result.success,
        message,
        blocks_created: 0, // Blocks aren't created in bidirectional sync
        tasks_created: 0,  // Tasks are synced, not created
        tasks_updated: sync_result.tasks_synced,
        issues_created: sync_result.issues_created,
        issues_updated: sync_result.issues_updated,
        errors: sync_result.errors,
    }))
}

/// Result of syncing a single task
#[derive(Debug)]
struct TaskSyncResult {
    issue_created: bool,
    issue_updated: bool,
    conflicts: Vec<SyncConflict>,
}

/// Sync a single task bidirectionally with Jira
async fn sync_task_bidirectionally(
    task: &mut Task,
    block: &Block,
    jira_issues_by_task_id: &HashMap<String, &JiraIssue>,
    mapper: &JiraMapper,
    data: &JiraAppState,
) -> Result<TaskSyncResult, String> {
    let mut result = TaskSyncResult {
        issue_created: false,
        issue_updated: false,
        conflicts: Vec::new(),
    };

    // Check if this task already has a linked Jira issue
    if let Some(jira_issue) = jira_issues_by_task_id.get(&task.task_id) {
        // Task exists in both systems - detect and resolve conflicts
        let conflicts = mapper.detect_conflicts(task, jira_issue);
        result.conflicts = conflicts.clone();

        if !conflicts.is_empty() {
            // Apply conflict resolution
            let mut resolved_conflicts = conflicts;
            mapper.resolve_conflicts(&mut resolved_conflicts, &block.jira_sync_settings)
                .map_err(|e| format!("Failed to resolve conflicts: {}", e))?;
            
            mapper.apply_resolutions(task, &resolved_conflicts, jira_issue)
                .map_err(|e| format!("Failed to apply conflict resolutions: {}", e))?;
            
            result.conflicts = resolved_conflicts;
        }

        // Update Jira issue with any changes from Forge
        if should_update_jira_issue(task, jira_issue, &block.jira_sync_settings) {
            let issue_update = mapper.map_task_to_issue(task, block)
                .map_err(|e| format!("Failed to map task to issue: {}", e))?;
            
            match data.jira_client.update_issue(&jira_issue.key, issue_update).await {
                Ok(_) => {
                    result.issue_updated = true;
                    task.jira_last_sync = Some(chrono::Utc::now().to_rfc3339());
                    info!("Updated Jira issue {} for task {}", jira_issue.key, task.task_id);
                }
                Err(e) => {
                    return Err(format!("Failed to update Jira issue {}: {}", jira_issue.key, e));
                }
            }
        }

        // Update task with any changes from Jira
        if should_update_forge_task(task, jira_issue, &block.jira_sync_settings) {
            let updated_task = mapper.map_issue_to_task(jira_issue, Some(task))
                .map_err(|e| format!("Failed to map issue to task: {}", e))?;
            
            *task = updated_task;
            info!("Updated Forge task {} from Jira issue {}", task.task_id, jira_issue.key);
        }

    } else if task.jira_issue_key.is_none() {
        // Task exists only in Forge - create corresponding Jira issue
        let issue_data = mapper.map_task_to_issue(task, block)
            .map_err(|e| format!("Failed to map task to issue: {}", e))?;
        
        let project_key = block.jira_project_key.as_ref()
            .ok_or("Block does not have a Jira project key set")?;
        
        let default_issue_type = "Task".to_string();
        let issue_type = block.jira_sync_settings.issue_type_mapping
            .get(&task.status)
            .unwrap_or(&default_issue_type);
        
        match data.jira_client.create_issue(project_key, issue_type, &task.task_name, Some(&task.description)).await {
            Ok(issue_key) => {
                result.issue_created = true;
                task.jira_issue_key = Some(issue_key.clone());
                task.jira_synced = true;
                task.jira_last_sync = Some(chrono::Utc::now().to_rfc3339());
                task.jira_sync_direction = Some("export".to_string());
                info!("Created Jira issue {} for task {}", issue_key, task.task_id);
            }
            Err(e) => {
                return Err(format!("Failed to create Jira issue for task {}: {}", task.task_id, e));
            }
        }
    }

    Ok(result)
}

/// Import a Jira issue as a new Forge task
async fn import_jira_issue_as_new_task(
    issue: &JiraIssue,
    project_key: &str,
    mapper: &JiraMapper,
    data: &JiraAppState,
) -> Result<Task, String> {
    let task = mapper.map_issue_to_task(issue, None)
        .map_err(|e| format!("Failed to map issue to task: {}", e))?;

    // Find or create a block to hold this task
    // For now, we'll try to find an existing block linked to the Jira project
    let blocks = data.block_manager.get_blocks()
        .map_err(|e| format!("Failed to get blocks: {}", e))?;
    
    let target_block = blocks.iter()
        .find(|b| b.jira_project_key.as_ref() == Some(&project_key.to_string()))
        .or_else(|| blocks.first());

    if let Some(_block) = target_block {
        // In a real implementation, we would add the task to the block
        // For now, just log that we would do this
        info!("Would add imported task {} to block {}", task.task_id, _block.block_id);
    } else {
        info!("No suitable block found for imported task {}, would create new block", task.task_id);
    }

    Ok(task)
}

/// Determine if a Jira issue should be updated based on Forge task changes
fn should_update_jira_issue(task: &Task, issue: &JiraIssue, settings: &crate::models::JiraSyncSettings) -> bool {
    if !settings.auto_sync {
        return false;
    }
    
    match settings.sync_direction.as_str() {
        "export" | "bidirectional" => {
            // Check if task has been modified more recently than last sync
            if let Some(last_sync) = &task.jira_last_sync {
                // In a real implementation, we would compare timestamps
                // For now, assume updates are needed if sync direction allows it
                true
            } else {
                true // Never synced before
            }
        }
        _ => false,
    }
}

/// Determine if a Forge task should be updated based on Jira issue changes  
fn should_update_forge_task(task: &Task, issue: &JiraIssue, settings: &crate::models::JiraSyncSettings) -> bool {
    if !settings.auto_sync {
        return false;
    }
    
    match settings.sync_direction.as_str() {
        "import" | "bidirectional" => {
            // Check if issue has been modified more recently than last sync
            if let Some(last_sync) = &task.jira_last_sync {
                // In a real implementation, we would compare Jira's updated timestamp
                // with our last sync timestamp
                true
            } else {
                true // Never synced before
            }
        }
        _ => false,
    }
}

async fn perform_export_operations(
    request: &JiraSyncRequest,
    context: &ExecutionContext,
    data: &JiraAppState,
) -> Result<(u32, u32), Vec<String>> {
    // Simplified export logic - replaced by bidirectional sync
    Ok((0, 0)) // (created, updated)
}

async fn perform_import_operations(
    request: &JiraSyncRequest,
    context: &ExecutionContext,
    data: &JiraAppState,
) -> Result<(u32, u32), Vec<String>> {
    // Simplified import logic - replaced by bidirectional sync
    Ok((0, 0)) // (created, updated)
}

// Helper functions

async fn create_task_from_jira_issue(
    issue: &crate::mcp::tools::jira::JiraIssue,
    project_key: &str,
    context: &ExecutionContext,
    data: &JiraAppState,
) -> Result<bool, String> {
    debug!("Creating task from Jira issue: {}", issue.key);

    // First, find or create a block for this Jira project
    let block_id = find_or_create_jira_block(project_key, data).await?;
    
    // Check if this Jira issue is already imported as a task
    let existing_task = find_task_by_jira_key(&block_id, &issue.key, data).await?;
    if existing_task.is_some() {
        debug!("Task for Jira issue {} already exists, skipping", issue.key);
        return Ok(false); // false = already exists, not created
    }

    // Create a new task from the Jira issue
    let mut task = Task::new(issue.description.clone().unwrap_or_else(|| issue.summary.clone()));
    task.task_name = issue.summary.clone();
    task.status = map_jira_status_to_forge_status(&issue.status);
    
    // Set Jira integration fields
    task.jira_issue_key = Some(issue.key.clone());
    task.jira_status = Some(issue.status.clone());
    task.jira_assignee = issue.assignee.clone();
    task.jira_labels = issue.labels.clone();
    task.jira_synced = true;
    task.jira_last_sync = Some(chrono::Utc::now().to_rfc3339());
    task.jira_sync_direction = Some("import".to_string());
    
    // Map Jira priority to estimated effort
    if let Some(priority) = &issue.priority {
        task.estimated_effort = match priority.to_lowercase().as_str() {
            "highest" | "high" => "large".to_string(),
            "medium" => "medium".to_string(),
            "low" | "lowest" => "small".to_string(),
            _ => "medium".to_string(),
        };
    }

    // Add labels as acceptance criteria if available
    if !issue.labels.is_empty() {
        task.acceptance_criteria.push(format!("Labels: {}", issue.labels.join(", ")));
    }

    // Add the task to the block
    match data.block_manager.add_task(&block_id, task) {
        Ok(_task_id) => {
            info!("Successfully created task from Jira issue: {}", issue.key);
            Ok(true) // true = created
        }
        Err(e) => {
            error!("Failed to add task to block {}: {}", block_id, e);
            Err(format!("Failed to add task to block: {}", e))
        }
    }
}

/// Find an existing block for the Jira project or create a new one
async fn find_or_create_jira_block(
    project_key: &str,
    data: &JiraAppState,
) -> Result<String, String> {
    // First, try to find an existing block linked to this Jira project
    let blocks = data.block_manager.get_blocks()
        .map_err(|e| format!("Failed to get blocks: {}", e))?;
    
    // Look for a block that's already linked to this Jira project
    for block in &blocks {
        if let Some(ref linked_project) = block.jira_project_key {
            if linked_project == project_key {
                debug!("Found existing block {} for Jira project {}", block.block_id, project_key);
                return Ok(block.block_id.clone());
            }
        }
    }
    
    // If no existing block found, create a new one
    info!("Creating new block for Jira project: {}", project_key);
    
    let mut new_block = crate::models::Block::new(
        format!("Jira Project: {}", project_key),
        format!("Imported from Jira project {}", project_key),
        Vec::new(), // empty inputs
        Vec::new(), // empty outputs
    );
    
    // Set Jira integration fields
    new_block.jira_project_key = Some(project_key.to_string());
    new_block.jira_synced = true;
    new_block.jira_last_sync = Some(chrono::Utc::now().to_rfc3339());
    new_block.jira_sync_settings.auto_sync = true;
    new_block.jira_sync_settings.sync_direction = "import".to_string();
    
    let block_id = new_block.block_id.clone();
    
    // Add the block to the manager
    data.block_manager.add_block(new_block)
        .map_err(|e| format!("Failed to create block: {}", e))?;
    
    info!("Created new block {} for Jira project {}", block_id, project_key);
    Ok(block_id)
}

/// Find a task by its Jira issue key within a specific block
async fn find_task_by_jira_key(
    block_id: &str,
    jira_key: &str,
    data: &JiraAppState,
) -> Result<Option<String>, String> {
    let blocks = data.block_manager.get_blocks()
        .map_err(|e| format!("Failed to get blocks: {}", e))?;
    
    // Find the block by ID
    let block = blocks.iter()
        .find(|b| b.block_id == block_id)
        .ok_or_else(|| format!("Block {} not found", block_id))?;
    
    // Search through tasks in the block for one with matching Jira key
    for (task_id, task) in &block.todo_list {
        if let Some(ref task_jira_key) = task.jira_issue_key {
            if task_jira_key == jira_key {
                return Ok(Some(task_id.clone()));
            }
        }
    }
    
    Ok(None)
}

/// Map Jira status to Forge task status
fn map_jira_status_to_forge_status(jira_status: &str) -> String {
    match jira_status.to_lowercase().as_str() {
        "to do" | "todo" | "open" | "new" | "created" => "TODO".to_string(),
        "in progress" | "in-progress" | "progress" | "doing" => "IN_PROGRESS".to_string(),
        "done" | "closed" | "resolved" | "complete" | "completed" => "COMPLETED".to_string(),
        _ => "TODO".to_string(), // Default to TODO for unknown statuses
    }
}

async fn export_task_to_jira_issue(
    task: &serde_json::Value,
    project_key: &str,
    existing_issue_key: Option<&str>,
    context: &ExecutionContext,
    data: &JiraAppState,
) -> Result<bool, String> {
    debug!("Exporting task to Jira: {:?}", task);

    let summary = task.get("task_name")
        .or_else(|| task.get("description"))
        .and_then(|v| v.as_str())
        .unwrap_or("Untitled Task");

    let description = task.get("description")
        .and_then(|v| v.as_str());

    if let Some(issue_key) = existing_issue_key {
        // Update existing issue
        match data.jira_client.update_issue(issue_key, json!({
            "summary": summary,
            "description": description
        })).await {
            Ok(_) => Ok(false), // false = updated
            Err(e) => Err(format!("Failed to update issue {}: {}", issue_key, e)),
        }
    } else {
        // Create new issue
        match data.jira_client.create_issue(project_key, "Task", summary, description).await {
            Ok(issue_key) => {
                debug!("Created Jira issue: {}", issue_key);
                // TODO: Update the task with the new Jira issue key
                Ok(true) // true = created
            }
            Err(e) => Err(format!("Failed to create issue: {}", e)),
        }
    }
}