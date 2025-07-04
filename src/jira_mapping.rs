/// Jira mapping logic for bidirectional synchronization
/// 
/// This module handles the mapping between Forge blocks/tasks and Jira projects/issues,
/// providing conflict resolution, field mapping, and synchronization strategies.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;
use tracing::{debug, error, info, warn};

use crate::models::{Block, Task, JiraSyncSettings};
use crate::mcp::tools::jira::{JiraIssue, JiraProject};

/// Represents a conflict during synchronization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncConflict {
    pub conflict_type: ConflictType,
    pub forge_value: Value,
    pub jira_value: Value,
    pub field_name: String,
    pub item_id: String, // Task ID or Block ID
    pub resolution: Option<ConflictResolution>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConflictType {
    StatusMismatch,
    DescriptionMismatch,
    AssigneeMismatch,
    PriorityMismatch,
    DueDateMismatch,
    CustomFieldMismatch,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConflictResolution {
    UseForgeValue,
    UseJiraValue,
    Merge,
    Manual,
}

/// Main mapping coordinator for Jira synchronization
pub struct JiraMapper {
    pub field_mappings: FieldMappings,
    pub conflict_resolver: ConflictResolver,
}

/// Defines how Forge fields map to Jira fields
#[derive(Debug, Clone)]
pub struct FieldMappings {
    pub task_to_issue: HashMap<String, String>,
    pub block_to_project: HashMap<String, String>,
    pub status_mappings: HashMap<String, String>,
    pub priority_mappings: HashMap<String, String>,
}

impl Default for FieldMappings {
    fn default() -> Self {
        let mut task_to_issue = HashMap::new();
        task_to_issue.insert("task_name".to_string(), "summary".to_string());
        task_to_issue.insert("description".to_string(), "description".to_string());
        task_to_issue.insert("status".to_string(), "status".to_string());
        task_to_issue.insert("estimated_effort".to_string(), "timeestimate".to_string());
        task_to_issue.insert("jira_assignee".to_string(), "assignee".to_string());
        task_to_issue.insert("jira_labels".to_string(), "labels".to_string());

        let mut block_to_project = HashMap::new();
        block_to_project.insert("name".to_string(), "name".to_string());
        block_to_project.insert("description".to_string(), "description".to_string());
        block_to_project.insert("block_id".to_string(), "key".to_string());

        let mut status_mappings = HashMap::new();
        status_mappings.insert("TODO".to_string(), "To Do".to_string());
        status_mappings.insert("IN_PROGRESS".to_string(), "In Progress".to_string());
        status_mappings.insert("COMPLETED".to_string(), "Done".to_string());
        status_mappings.insert("FAILED".to_string(), "To Do".to_string());

        let mut priority_mappings = HashMap::new();
        priority_mappings.insert("Low".to_string(), "Low".to_string());
        priority_mappings.insert("Medium".to_string(), "Medium".to_string());
        priority_mappings.insert("High".to_string(), "High".to_string());
        priority_mappings.insert("Critical".to_string(), "Highest".to_string());

        Self {
            task_to_issue,
            block_to_project,
            status_mappings,
            priority_mappings,
        }
    }
}

/// Handles conflict resolution during synchronization
pub struct ConflictResolver {
    pub default_strategy: ConflictResolution,
    pub field_strategies: HashMap<String, ConflictResolution>,
}

impl Default for ConflictResolver {
    fn default() -> Self {
        let mut field_strategies = HashMap::new();
        // Status conflicts should typically favor Jira (manual workflow)
        field_strategies.insert("status".to_string(), ConflictResolution::UseJiraValue);
        // Description conflicts should be merged when possible
        field_strategies.insert("description".to_string(), ConflictResolution::Merge);
        // Assignee should favor the most recent update
        field_strategies.insert("assignee".to_string(), ConflictResolution::UseJiraValue);

        Self {
            default_strategy: ConflictResolution::Manual,
            field_strategies,
        }
    }
}

impl JiraMapper {
    pub fn new() -> Self {
        Self {
            field_mappings: FieldMappings::default(),
            conflict_resolver: ConflictResolver::default(),
        }
    }

    /// Maps a Forge task to a Jira issue representation
    pub fn map_task_to_issue(&self, task: &Task, block: &Block) -> Result<Value, String> {
        debug!("Mapping task {} to Jira issue", task.task_id);

        let mut issue_fields = json!({});

        // Map basic fields
        if !task.task_name.is_empty() {
            issue_fields["summary"] = json!(task.task_name);
        } else {
            issue_fields["summary"] = json!(task.description.chars().take(100).collect::<String>());
        }

        issue_fields["description"] = json!(self.format_task_description(task));

        // Map status using block's sync settings
        if let Some(jira_status) = block.jira_sync_settings.status_mapping.get(&task.status) {
            issue_fields["status"] = json!(jira_status);
        }

        // Map issue type using block's sync settings
        if let Some(issue_type) = block.jira_sync_settings.issue_type_mapping.get(&task.status) {
            issue_fields["issuetype"] = json!({"name": issue_type});
        }

        // Map assignee if present
        if let Some(assignee) = &task.jira_assignee {
            issue_fields["assignee"] = json!({"accountId": assignee});
        }

        // Map labels
        if !task.jira_labels.is_empty() {
            issue_fields["labels"] = json!(task.jira_labels);
        }

        // Add custom fields for Forge-specific data
        issue_fields["customfield_forge_task_id"] = json!(task.task_id);
        issue_fields["customfield_forge_block_id"] = json!(block.block_id);

        // Add acceptance criteria as a component or comment
        if !task.acceptance_criteria.is_empty() {
            let criteria_text = task.acceptance_criteria.join("\n- ");
            issue_fields["customfield_acceptance_criteria"] = json!(format!("- {}", criteria_text));
        }

        // Add dependencies information
        if !task.dependencies.is_empty() {
            issue_fields["customfield_dependencies"] = json!(task.dependencies.join(", "));
        }

        Ok(json!({"fields": issue_fields}))
    }

    /// Maps a Jira issue to a Forge task
    pub fn map_issue_to_task(&self, issue: &JiraIssue, existing_task: Option<&Task>) -> Result<Task, String> {
        debug!("Mapping Jira issue {} to Forge task", issue.key);

        let mut task = if let Some(existing) = existing_task {
            existing.clone()
        } else {
            Task::new(issue.description.as_ref().unwrap_or(&"".to_string()).clone())
        };

        // Map basic fields
        task.task_name = issue.summary.clone();

        if let Some(description) = &issue.description {
            task.description = description.clone();
        }

        // Map status
        task.status = self.map_jira_status_to_forge(&issue.status);

        // Map Jira-specific fields
        task.jira_issue_key = Some(issue.key.clone());
        task.jira_status = Some(issue.status.clone());

        if let Some(assignee) = &issue.assignee {
            task.jira_assignee = Some(assignee.clone());
        }

        // Map labels
        task.jira_labels = issue.labels.clone();

        // Update sync information
        task.jira_synced = true;
        task.jira_last_sync = Some(Utc::now().to_rfc3339());
        task.jira_sync_direction = Some("import".to_string());

        Ok(task)
    }

    /// Maps a Forge block to a Jira project/epic structure
    pub fn map_block_to_project(&self, block: &Block) -> Result<Value, String> {
        debug!("Mapping block {} to Jira project", block.block_id);

        let mut project_data = json!({});

        project_data["name"] = json!(block.name);
        project_data["description"] = json!(block.description);
        project_data["key"] = json!(self.generate_project_key(&block.name));

        // If creating epics for blocks is enabled
        if block.jira_sync_settings.create_epics_for_blocks {
            project_data["epic"] = json!({
                "summary": format!("Epic: {}", block.name),
                "description": block.description
            });
        }

        Ok(project_data)
    }

    /// Detects conflicts between Forge and Jira data
    pub fn detect_conflicts(&self, task: &Task, issue: &JiraIssue) -> Vec<SyncConflict> {
        let mut conflicts = Vec::new();

        // Check status conflicts
        let jira_status = &issue.status;
        let forge_status = &task.status;
        let mapped_forge_status = self.field_mappings.status_mappings.get(forge_status);
        
        if mapped_forge_status.map(|s| s.as_str()) != Some(jira_status) {
            conflicts.push(SyncConflict {
                conflict_type: ConflictType::StatusMismatch,
                forge_value: json!(forge_status),
                jira_value: json!(jira_status),
                field_name: "status".to_string(),
                item_id: task.task_id.clone(),
                resolution: None,
            });
        }

        // Check description conflicts
        if let Some(jira_description) = &issue.description {
            if task.description != *jira_description {
                conflicts.push(SyncConflict {
                    conflict_type: ConflictType::DescriptionMismatch,
                    forge_value: json!(task.description),
                    jira_value: json!(jira_description),
                    field_name: "description".to_string(),
                    item_id: task.task_id.clone(),
                    resolution: None,
                });
            }
        }

        // Check assignee conflicts
        if let (Some(forge_assignee), Some(jira_assignee)) = (&task.jira_assignee, &issue.assignee) {
            if forge_assignee != jira_assignee {
                conflicts.push(SyncConflict {
                    conflict_type: ConflictType::AssigneeMismatch,
                    forge_value: json!(forge_assignee),
                    jira_value: json!(jira_assignee),
                    field_name: "assignee".to_string(),
                    item_id: task.task_id.clone(),
                    resolution: None,
                });
            }
        }

        conflicts
    }

    /// Resolves conflicts according to the configured strategy
    pub fn resolve_conflicts(&self, conflicts: &mut Vec<SyncConflict>, settings: &JiraSyncSettings) -> Result<(), String> {
        for conflict in conflicts.iter_mut() {
            let strategy = self.conflict_resolver.field_strategies
                .get(&conflict.field_name)
                .unwrap_or(&self.conflict_resolver.default_strategy);

            match &settings.conflict_resolution.as_str() {
                &"jira_wins" => conflict.resolution = Some(ConflictResolution::UseJiraValue),
                &"forge_wins" => conflict.resolution = Some(ConflictResolution::UseForgeValue),
                &"manual" => {
                    // Use field-specific strategy or mark for manual resolution
                    conflict.resolution = Some(strategy.clone());
                }
                _ => {
                    warn!("Unknown conflict resolution strategy: {}", settings.conflict_resolution);
                    conflict.resolution = Some(ConflictResolution::Manual);
                }
            }
        }

        Ok(())
    }

    /// Applies conflict resolutions to update task data
    pub fn apply_resolutions(&self, task: &mut Task, conflicts: &[SyncConflict], issue: &JiraIssue) -> Result<(), String> {
        for conflict in conflicts {
            if let Some(resolution) = &conflict.resolution {
                match resolution {
                    ConflictResolution::UseJiraValue => {
                        self.apply_jira_value_to_task(task, &conflict.field_name, &conflict.jira_value)?;
                    }
                    ConflictResolution::UseForgeValue => {
                        // Keep existing Forge value, no changes needed
                        debug!("Keeping Forge value for field: {}", conflict.field_name);
                    }
                    ConflictResolution::Merge => {
                        self.merge_field_values(task, &conflict.field_name, &conflict.forge_value, &conflict.jira_value)?;
                    }
                    ConflictResolution::Manual => {
                        warn!("Manual resolution required for field: {} in task: {}", conflict.field_name, task.task_id);
                        // Could trigger a UI notification or store for later manual resolution
                    }
                }
            }
        }

        Ok(())
    }

    // Helper methods

    fn format_task_description(&self, task: &Task) -> String {
        let mut description = task.description.clone();
        
        if !task.acceptance_criteria.is_empty() {
            description.push_str("\n\n## Acceptance Criteria\n");
            for criterion in &task.acceptance_criteria {
                description.push_str(&format!("- {}\n", criterion));
            }
        }

        if !task.testing_requirements.is_empty() {
            description.push_str("\n\n## Testing Requirements\n");
            for req in &task.testing_requirements {
                description.push_str(&format!("- {}\n", req));
            }
        }

        if !task.files_affected.is_empty() {
            description.push_str("\n\n## Files Affected\n");
            for file in &task.files_affected {
                description.push_str(&format!("- {}\n", file));
            }
        }

        description
    }

    fn map_jira_status_to_forge(&self, jira_status: &str) -> String {
        // Reverse lookup in status mappings
        for (forge_status, mapped_jira_status) in &self.field_mappings.status_mappings {
            if mapped_jira_status == jira_status {
                return forge_status.clone();
            }
        }
        
        // Fallback mapping for common Jira statuses
        match jira_status.to_lowercase().as_str() {
            "to do" | "todo" | "open" | "new" => "TODO".to_string(),
            "in progress" | "in-progress" | "progress" => "IN_PROGRESS".to_string(),
            "done" | "closed" | "resolved" | "complete" => "COMPLETED".to_string(),
            _ => "TODO".to_string(), // Default fallback
        }
    }

    fn generate_project_key(&self, name: &str) -> String {
        // Generate a Jira-compatible project key from block name
        let key: String = name
            .chars()
            .filter(|c| c.is_alphanumeric() || *c == ' ')
            .collect::<String>()
            .split_whitespace()
            .take(3)
            .map(|word| word.chars().next().unwrap_or('X').to_uppercase().collect::<String>())
            .collect::<Vec<String>>()
            .join("");
        
        if key.len() < 2 {
            format!("FG{}", key)
        } else if key.len() > 10 {
            key[..10].to_string()
        } else {
            key
        }
    }

    fn apply_jira_value_to_task(&self, task: &mut Task, field_name: &str, jira_value: &Value) -> Result<(), String> {
        match field_name {
            "status" => {
                if let Some(status_str) = jira_value.as_str() {
                    task.status = self.map_jira_status_to_forge(status_str);
                    task.jira_status = Some(status_str.to_string());
                }
            }
            "description" => {
                if let Some(desc) = jira_value.as_str() {
                    task.description = desc.to_string();
                }
            }
            "assignee" => {
                if let Some(assignee) = jira_value.as_str() {
                    task.jira_assignee = Some(assignee.to_string());
                }
            }
            _ => {
                warn!("Unknown field for Jira value application: {}", field_name);
            }
        }
        Ok(())
    }

    fn merge_field_values(&self, task: &mut Task, field_name: &str, _forge_value: &Value, jira_value: &Value) -> Result<(), String> {
        match field_name {
            "description" => {
                // For descriptions, append Jira content if significantly different
                if let Some(jira_desc) = jira_value.as_str() {
                    if !task.description.contains(jira_desc) && !jira_desc.contains(&task.description) {
                        task.description = format!("{}\n\n--- From Jira ---\n{}", task.description, jira_desc);
                    }
                }
            }
            _ => {
                // For other fields, default to using Jira value
                self.apply_jira_value_to_task(task, field_name, jira_value)?;
            }
        }
        Ok(())
    }
}

/// Sync operation result
#[derive(Debug, Serialize, Deserialize)]
pub struct SyncResult {
    pub success: bool,
    pub tasks_synced: u32,
    pub issues_created: u32,
    pub issues_updated: u32,
    pub conflicts_detected: u32,
    pub conflicts_resolved: u32,
    pub errors: Vec<String>,
    pub conflicts: Vec<SyncConflict>,
}

impl Default for SyncResult {
    fn default() -> Self {
        Self {
            success: true,
            tasks_synced: 0,
            issues_created: 0,
            issues_updated: 0,
            conflicts_detected: 0,
            conflicts_resolved: 0,
            errors: Vec::new(),
            conflicts: Vec::new(),
        }
    }
}