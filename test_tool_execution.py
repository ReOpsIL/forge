#!/usr/bin/env python3
"""
Test MCP tool execution
"""
import json
import subprocess
import sys

def test_tool_execution():
    process = subprocess.Popen(
        ['./target/debug/forge', '--mcp'],
        stdin=subprocess.PIPE,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        text=True,
        bufsize=0
    )
    
    try:
        # Initialize first
        init_request = {
            "jsonrpc": "2.0",
            "id": 1,
            "method": "initialize",
            "params": {}
        }
        
        process.stdin.write(json.dumps(init_request) + '\n')
        process.stdin.flush()
        init_response = process.stdout.readline()
        print("✓ Initialized")
        
        # Create session
        session_request = {
            "jsonrpc": "2.0",
            "id": 2,
            "method": "session/create",
            "params": {
                "client_name": "test-client",
                "client_version": "1.0.0"
            }
        }
        
        process.stdin.write(json.dumps(session_request) + '\n')
        process.stdin.flush()
        session_response = json.loads(process.stdout.readline().strip())
        session_id = session_response["result"]["session_id"]
        print(f"✓ Created session: {session_id}")
        
        # Test list_directory tool
        tool_request = {
            "jsonrpc": "2.0",
            "id": 3,
            "method": "tools/call",
            "params": {
                "name": "list_directory",
                "arguments": {
                    "path": ".",
                    "max_depth": 1
                }
            }
        }
        
        process.stdin.write(json.dumps(tool_request) + '\n')
        process.stdin.flush()
        tool_response = json.loads(process.stdout.readline().strip())
        
        if "result" in tool_response:
            print("✓ Tool execution successful!")
            print(f"Directory contents: {len(tool_response['result'].get('files', []))} items")
        else:
            print(f"✗ Tool execution failed: {tool_response}")
            
    except Exception as e:
        print(f"Error: {e}")
    finally:
        process.terminate()
        process.wait()

if __name__ == "__main__":
    test_tool_execution()