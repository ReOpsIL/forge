# MCP Communication Flow: Forge ↔ Claude Code

## Phase 1: Connection Setup

### 1.1 Claude Code Connects to Forge MCP Server
```json
// Claude Code sends MCP initialize request
{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "initialize",
  "params": {
    "protocolVersion": "2024-11-05",
    "capabilities": {
      "tools": {},
      "notifications": {},
      "logging": {}
    },
    "clientInfo": {
      "name": "Claude Code",
      "version": "1.0.0"
    }
  }
}

// Forge MCP Server responds with available tools
{
  "jsonrpc": "2.0",
  "id": 1,
  "result": {
    "protocolVersion": "2024-11-05",
    "capabilities": {
      "tools": {
        "listChanged": true
      },
      "notifications": {
        "toolsChanged": true
      }
    },
    "serverInfo": {
      "name": "Forge MCP Server",
      "version": "1.0.0"
    }
  }
}
```

### 1.2 Tool Discovery
```json
// Claude Code requests available tools
{
  "jsonrpc": "2.0",
  "id": 2,
  "method": "tools/list",
  "params": {}
}

// Forge responds with tool catalog
{
  "jsonrpc": "2.0",
  "id": 2,
  "result": {
    "tools": [
      {
        "name": "forge_get_project_info",
        "description": "Get comprehensive project information",
        "inputSchema": {
          "type": "object",
          "properties": {
            "include_git_info": {"type": "boolean", "default": true}
          }
        }
      },
      {
        "name": "forge_get_tasks",
        "description": "Get tasks with filtering and context",
        "inputSchema": {
          "type": "object",
          "properties": {
            "block_id": {"type": "string"},
            "status": {"type": "array", "items": {"type": "string"}},
            "include_dependencies": {"type": "boolean", "default": true}
          }
        }
      },
      {
        "name": "forge_execute_task",
        "description": "Execute a task with real-time progress",
        "inputSchema": {
          "type": "object",
          "properties": {
            "block_id": {"type": "string"},
            "task_id": {"type": "string"},
            "options": {"type": "object"}
          },
          "required": ["block_id", "task_id"]
        }
      }
      // ... many more tools
    ]
  }
}
```

## Phase 2: Interactive Task Execution

### 2.1 Claude Gathers Context
```json
// Step 1: Get project information
{
  "jsonrpc": "2.0",
  "id": 3,
  "method": "tools/call",
  "params": {
    "name": "forge_get_project_info",
    "arguments": {
      "include_git_info": true,
      "include_stats": true
    }
  }
}

// Forge responds with rich project context
{
  "jsonrpc": "2.0",
  "id": 3,
  "result": {
    "content": [
      {
        "type": "text",
        "text": {
          "project": {
            "name": "forge",
            "type": "rust",
            "root_directory": "/Users/dovcaspi/develop/forge",
            "architecture": "actix-web backend + react frontend",
            "current_branch": "main",
            "git_status": "clean",
            "dependencies": ["actix-web", "serde", "tokio"],
            "test_framework": "cargo test",
            "build_command": "cargo build"
          },
          "statistics": {
            "total_files": 45,
            "rust_files": 23,
            "js_files": 15,
            "total_blocks": 8,
            "total_tasks": 24,
            "completed_tasks": 18
          }
        }
      }
    ]
  }
}

// Step 2: Get specific task details
{
  "jsonrpc": "2.0",
  "id": 4,
  "method": "tools/call",
  "params": {
    "name": "forge_get_task_details",
    "arguments": {
      "block_id": "abc123",
      "task_id": "xyz789",
      "include_context": true
    }
  }
}

// Forge responds with full task context
{
  "jsonrpc": "2.0",
  "id": 4,
  "result": {
    "content": [
      {
        "type": "text",
        "text": {
          "task": {
            "id": "xyz789",
            "name": "Add user authentication",
            "description": "Implement JWT-based authentication system",
            "acceptance_criteria": [
              "User can register with email/password",
              "User can login and receive JWT token",
              "Protected routes validate JWT tokens"
            ],
            "dependencies": ["task_abc", "task_def"],
            "files_affected": ["src/auth.rs", "src/models/user.rs"],
            "related_blocks": ["user_management", "api_gateway"]
          },
          "context": {
            "existing_files": {
              "src/main.rs": "Main application entry point with Actix setup",
              "src/models.rs": "Existing data models for blocks and tasks"
            },
            "dependencies_status": {
              "task_abc": "completed",
              "task_def": "completed"
            },
            "related_code_patterns": [
              "Uses actix-web for HTTP handling",
              "Serde for JSON serialization",
              "Arc<Mutex<>> for thread-safe state"
            ]
          }
        }
      }
    ]
  }
}
```

### 2.2 Claude Executes Task Interactively
```json
// Step 3: Claude starts task execution
{
  "jsonrpc": "2.0",
  "id": 5,
  "method": "tools/call",
  "params": {
    "name": "forge_execute_task_stream",
    "arguments": {
      "block_id": "abc123",
      "task_id": "xyz789",
      "options": {
        "create_branch": true,
        "run_tests": true,
        "notify_progress": true
      }
    }
  }
}

// Forge creates git branch and responds
{
  "jsonrpc": "2.0",
  "id": 5,
  "result": {
    "content": [
      {
        "type": "text",
        "text": {
          "status": "started",
          "branch_created": "task-xyz789-auth-implementation",
          "execution_id": "exec_12345",
          "message": "Task execution started. You can now make changes and I'll track them."
        }
      }
    ]
  }
}

// Step 4: Claude reads existing files to understand structure
{
  "jsonrpc": "2.0",
  "id": 6,
  "method": "tools/call",
  "params": {
    "name": "forge_read_file",
    "arguments": {
      "file_path": "src/main.rs",
      "include_context": true
    }
  }
}

// Step 5: Claude creates/modifies files
{
  "jsonrpc": "2.0",
  "id": 7,
  "method": "tools/call",
  "params": {
    "name": "forge_write_file",
    "arguments": {
      "file_path": "src/auth.rs",
      "content": "// JWT authentication implementation\nuse jsonwebtoken::{encode, decode, Header, Algorithm, Validation, EncodingKey, DecodingKey};\n...",
      "create_if_not_exists": true
    }
  }
}

// Forge responds with file modification status
{
  "jsonrpc": "2.0",
  "id": 7,
  "result": {
    "content": [
      {
        "type": "text",
        "text": {
          "status": "success",
          "file_path": "src/auth.rs",
          "action": "created",
          "lines_added": 45,
          "git_status": "new file"
        }
      }
    ]
  }
}

// Step 6: Claude runs tests to verify implementation
{
  "jsonrpc": "2.0",
  "id": 8,
  "method": "tools/call",
  "params": {
    "name": "forge_run_tests",
    "arguments": {
      "scope": "related_to_files",
      "files": ["src/auth.rs"],
      "test_types": ["unit", "integration"]
    }
  }
}

// Forge runs tests and reports results
{
  "jsonrpc": "2.0",
  "id": 8,
  "result": {
    "content": [
      {
        "type": "text",
        "text": {
          "test_results": {
            "status": "passed",
            "total_tests": 8,
            "passed": 8,
            "failed": 0,
            "duration": "2.3s"
          },
          "coverage": {
            "lines_covered": 42,
            "total_lines": 45,
            "percentage": 93.3
          }
        }
      }
    ]
  }
}

// Step 7: Claude commits changes
{
  "jsonrpc": "2.0",
  "id": 9,
  "method": "tools/call",
  "params": {
    "name": "forge_commit_changes",
    "arguments": {
      "message": "Implement JWT authentication system\n\n- Add auth.rs with JWT token generation and validation\n- Add user registration and login endpoints\n- Include comprehensive tests with 93% coverage\n- Meets all acceptance criteria",
      "files": ["src/auth.rs", "src/models/user.rs", "tests/auth_test.rs"]
    }
  }
}

// Forge commits and merges back to main
{
  "jsonrpc": "2.0",
  "id": 9,
  "result": {
    "content": [
      {
        "type": "text",
        "text": {
          "commit_hash": "abc123def456",
          "branch_merged": true,
          "branch_deleted": true,
          "task_status": "completed",
          "files_modified": ["src/auth.rs", "src/models/user.rs", "tests/auth_test.rs"],
          "summary": "Successfully implemented JWT authentication with full test coverage"
        }
      }
    ]
  }
}
```

## Phase 3: Real-time Progress Notifications

### 3.1 Forge Sends Progress Updates
```json
// Forge can send notifications during task execution
{
  "jsonrpc": "2.0",
  "method": "notifications/progress",
  "params": {
    "execution_id": "exec_12345",
    "progress": {
      "percentage": 60,
      "stage": "implementing",
      "description": "Creating authentication endpoints",
      "files_in_progress": ["src/auth.rs"]
    }
  }
}

// Forge can notify about issues that need attention
{
  "jsonrpc": "2.0",
  "method": "notifications/issue",
  "params": {
    "execution_id": "exec_12345",
    "issue": {
      "type": "compilation_error",
      "file": "src/auth.rs",
      "line": 23,
      "message": "cannot find type `User` in this scope",
      "suggestion": "Import User model from models module"
    }
  }
}
```

## How Forge "Understands" Claude Output

### 1. Structured Responses (Not Text Parsing!)
```json
// Instead of parsing this text output:
"Created file src/auth.rs with JWT implementation. Tests passing. ✅"

// MCP gives structured data:
{
  "actions_performed": [
    {
      "type": "file_created",
      "path": "src/auth.rs",
      "lines": 45,
      "purpose": "JWT authentication implementation"
    },
    {
      "type": "tests_run",
      "passed": 8,
      "failed": 0,
      "coverage": 93.3
    }
  ],
  "task_status": "completed",
  "success": true
}
```

### 2. Real-time State Updates
```json
// Forge knows exactly what happened because Claude called specific tools:
{
  "tool_calls": [
    {"name": "forge_write_file", "result": "success"},
    {"name": "forge_run_tests", "result": "8 passed, 0 failed"},
    {"name": "forge_commit_changes", "result": "committed abc123def"}
  ],
  "final_state": {
    "task_completed": true,
    "files_modified": ["src/auth.rs", "tests/auth_test.rs"],
    "git_commit": "abc123def456",
    "test_results": "all_passed"
  }
}
```

### 3. Error Handling & Recovery
```json
// If something goes wrong, Claude can ask for help:
{
  "jsonrpc": "2.0",
  "id": 10,
  "method": "tools/call",
  "params": {
    "name": "forge_analyze_error",
    "arguments": {
      "error_type": "compilation_error",
      "file": "src/auth.rs",
      "error_message": "cannot find type `User` in this scope"
    }
  }
}

// Forge can provide context and suggestions:
{
  "jsonrpc": "2.0",
  "id": 10,
  "result": {
    "content": [
      {
        "type": "text",
        "text": {
          "analysis": {
            "issue": "Missing import for User type",
            "available_user_types": [
              "models::User",
              "auth::UserCredentials"
            ],
            "suggested_fix": "Add 'use crate::models::User;' at top of file",
            "related_files": ["src/models.rs"]
          }
        }
      }
    ]
  }
}
```

## Key Benefits vs Current CLI Approach

| Aspect | Current CLI | MCP Server |
|--------|-------------|------------|
| **Communication** | One-way text | Bidirectional structured data |
| **Context** | Single prompt | Rich project context on demand |
| **Progress** | Guess from text | Real-time structured updates |
| **Error Handling** | Parse stderr | Structured error analysis |
| **File Operations** | Hope Claude does it right | Explicit tool calls with confirmations |
| **Testing** | Assume tests ran | Explicit test execution with results |
| **Git Operations** | Automated but opaque | Interactive with full visibility |
| **Recovery** | Start over if failed | Analyze issues and retry specific steps |

## Summary

With MCP, Forge doesn't need to "understand" Claude's output because:

1. **Claude calls explicit tools** instead of producing text
2. **Each tool call returns structured data** with success/failure status
3. **Forge tracks state changes in real-time** through tool calls
4. **Progress is reported through structured notifications**
5. **Errors are handled through specific error analysis tools**

This transforms the interaction from "guess what Claude did by parsing text" to "know exactly what Claude did because it used our APIs."