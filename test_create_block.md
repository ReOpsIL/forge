# CreateBlockTool - Sample Usage

The new MCP CreateBlockTool has been successfully implemented and integrated into the forge project.

## Tool Details

- **Name**: `create_block`
- **Description**: Create a new block in the forge project
- **Category**: Project
- **Permissions**: FileWrite, ProjectConfig

## Input Schema

```json
{
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
}
```

## Sample Usage

### Example 1: Basic Block Creation

```json
{
  "name": "DataValidator",
  "description": "Validates incoming data before processing"
}
```

### Example 2: Block with Custom ID

```json
{
  "name": "UserAuthentication", 
  "description": "Handles user login and authentication",
  "block_id": "auth01"
}
```

## Output

The tool returns a JSON response with:

- Success status
- Created block details (ID, name, description)
- Empty connections and tasks (newly created)
- Context update information

## Integration

The tool is registered in the MCP server alongside other forge tools:

- `list_blocks` - List all blocks
- `create_block` - Create new blocks (NEW)
- File system tools (read, write, list, create, delete)

## Implementation Details

- Creates blocks using the existing Block model
- Auto-generates 6-character alphanumeric IDs if not provided
- Validates input parameters
- Saves to the blocks configuration file
- Updates execution context with file access information
- Follows the established MCP tool pattern