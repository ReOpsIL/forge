# MCP-Based Markdown Processing Prompts

## Overview

Two new constant prompts have been created that implement the same functionality as the existing `DEFAULT_PROCESS_MARKDOWN_SPEC_SYSTEM_PROMPT` and `DEFAULT_PROCESS_MARKDOWN_SPEC_USER_PROMPT`, but use MCP tools (`create_block` and `create_task`) as the mechanism for creating forge Blocks and Tasks.

## New Constants

### DEFAULT_PROCESS_MARKDOWN_SPEC_SYSTEM_PROMPT_MCP

**Location**: `/Users/dovcaspi/develop/forge/src/project_config.rs:149-160`

**Purpose**: System prompt that defines the AI's role as a software architecture analyst using MCP tools.

**Key Features**:
- Identifies available MCP tools (`create_block` and `create_task`)
- Defines the AI's role in parsing specifications and creating components
- Emphasizes direct tool usage for component creation
- Ensures proper relationships between blocks and tasks

### DEFAULT_PROCESS_MARKDOWN_SPEC_USER_PROMPT_MCP

**Location**: `/Users/dovcaspi/develop/forge/src/project_config.rs:162-225`

**Purpose**: User prompt that provides detailed instructions for processing markdown specifications using MCP tools.

**Key Features**:
- **Two-phase process**: First create blocks, then create tasks
- **Comprehensive guidelines** for both block and task creation
- **Detailed MCP tool usage examples** with JSON parameters
- **Implementation order** specifications for proper dependency management

## Functionality Comparison

| Aspect | Original Prompts | New MCP Prompts |
|--------|-----------------|-----------------|
| **Output Format** | JSON array of block specifications | Direct MCP tool calls |
| **Block Creation** | JSON structure definition | `create_block` tool usage |
| **Task Creation** | Not included | `create_task` tool with full metadata |
| **Persistence** | Manual processing required | Automatic via MCP tools |
| **Validation** | External validation needed | Built-in tool validation |
| **Integration** | Requires parsing and conversion | Direct forge integration |

## MCP Tools Integration

### create_block Tool Usage
```json
{
  "name": "UserAuthenticationService",
  "description": "Handles user authentication with JWT tokens, password hashing, and session management"
}
```

### create_task Tool Usage
```json
{
  "block_id": "[block_id_from_create_block_response]",
  "task_name": "Implement JWT Token Generation",
  "description": "Create JWT token generation and validation functionality",
  "acceptance_criteria": ["Tokens expire after 24 hours", "Include user ID and role in payload"],
  "dependencies": ["User Model", "Security Configuration"],
  "estimated_effort": "4 hours",
  "files_affected": ["src/auth/jwt.rs", "src/models/user.rs"],
  "function_signatures": ["pub fn generate_token(user_id: u64) -> Result<String, AuthError>"],
  "testing_requirements": ["Unit tests for token generation", "Integration tests for auth flow"]
}
```

## Configuration Integration

Both prompts are integrated into the `ProjectConfig` struct:

```rust
pub struct ProjectConfig {
    // ... existing fields ...
    pub process_specification_system_prompt_mcp: Option<String>,
    pub process_specification_user_prompt_mcp: Option<String>,
}
```

Default values are automatically populated in the `Default` implementation using the new constants.

## Benefits of MCP-Based Approach

1. **Direct Integration**: Creates blocks and tasks directly in the forge system
2. **Rich Metadata**: Supports comprehensive task details (acceptance criteria, dependencies, etc.)
3. **Validation**: Built-in validation through MCP tool schemas
4. **Persistence**: Automatic saving to forge configuration files
5. **Context Tracking**: Full execution context updates for monitoring
6. **Error Handling**: Proper error reporting through MCP tool framework

## Usage Context

These prompts are designed for scenarios where:
- Markdown specifications need to be converted to forge blocks and tasks
- Direct integration with the forge project management system is required
- Comprehensive task tracking with metadata is needed
- Automated project setup from technical specifications is desired

The MCP-based approach provides a more integrated and automated solution compared to the original JSON-based approach, enabling seamless creation of project structures directly within the forge ecosystem.