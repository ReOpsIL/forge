# CreateTaskTool - MCP Tool Documentation

The CreateTaskTool is a new MCP tool that creates detailed tasks for forge Blocks.

## Tool Details

- **Name**: `create_task`
- **Description**: Create a new task for a block in the forge project
- **Category**: Tasks
- **Permissions**: FileWrite, TaskManagement, ProjectConfig

## Features

### Comprehensive Task Creation

- Creates detailed Task objects with full metadata
- Supports all Task fields: acceptance criteria, dependencies, effort estimation, etc.
- Validates block existence before creating tasks
- Auto-generates unique task IDs

### Input Schema

```json
{
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
}
```

## Usage Examples

### Basic Task Creation

```json
{
  "block_id": "oceUDc",
  "task_name": "Implement User Authentication",
  "description": "Create a secure user authentication system with JWT tokens"
}
```

### Detailed Task with All Fields

```json
{
  "block_id": "oceUDc",
  "task_name": "Implement Data Validation Layer",
  "description": "Create comprehensive data validation for user inputs",
  "acceptance_criteria": [
    "All user inputs must be validated before processing",
    "Validation errors must be clearly communicated to users",
    "Performance impact should be minimal"
  ],
  "dependencies": [
    "User Authentication System",
    "Database Schema Setup"
  ],
  "estimated_effort": "3 days",
  "files_affected": [
    "src/validation/mod.rs",
    "src/models/user.rs",
    "tests/validation_tests.rs"
  ],
  "function_signatures": [
    "pub fn validate_user_input(input: &UserInput) -> Result<(), ValidationError>",
    "pub fn sanitize_string(input: &str) -> String"
  ],
  "testing_requirements": [
    "Unit tests for all validation functions",
    "Integration tests with actual user data",
    "Performance benchmarks"
  ],
  "status": "TODO"
}
```

## Implementation Details

### New BlockConfigManager Method

Added `add_task()` method to BlockConfigManager:

- **Location**: `/Users/dovcaspi/develop/forge/src/block_config.rs:190-207`
- **Function**: `pub fn add_task(&self, block_id: &str, task: Task) -> Result<String, String>`
- **Returns**: Task ID on success, error message on failure

### Workflow

1. **Parameter Validation**: Validates required fields (block_id, task_name, description)
2. **Block Existence Check**: Verifies the target block exists
3. **Task Creation**: Creates comprehensive Task object with all provided metadata
4. **Persistence**: Adds task to block and saves to configuration file
5. **Context Updates**: Updates execution context with task creation details

### Output Format

Returns JSON with:

- Success status and message
- Complete task details including generated task_id
- Context update information including file modifications

## Integration

The tool integrates seamlessly with the existing forge MCP ecosystem:

- **Registered alongside**: `list_blocks`, `create_block`, and filesystem tools
- **Uses existing**: Block and Task models from `src/models.rs`
- **Leverages**: BlockConfigManager for persistence
- **Follows**: Established MCP tool patterns for error handling and context updates

## Error Handling

- **Invalid Parameters**: Returns detailed error messages for missing/invalid inputs
- **Block Not Found**: Validates block existence before task creation
- **Persistence Failures**: Handles file save errors gracefully
- **Execution Context**: Tracks all file modifications and task updates

This tool enables programmatic creation of detailed, structured tasks within the forge project management system, supporting comprehensive project planning and task tracking workflows.