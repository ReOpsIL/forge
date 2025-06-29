#!/usr/bin/env python3
"""
Complete MCP server test
"""
import json
import subprocess

def test_complete():
    process = subprocess.Popen(
        ['./target/debug/forge', '--mcp'],
        stdin=subprocess.PIPE,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        text=True,
        bufsize=0
    )
    
    try:
        # 1. Initialize with correct parameters
        init_request = {
            "jsonrpc": "2.0",
            "id": 1,
            "method": "initialize",
            "params": {
                "protocolVersion": "2024-11-05",
                "capabilities": {},
                "clientInfo": {
                    "name": "test-client",
                    "version": "1.0.0"
                }
            }
        }
        
        process.stdin.write(json.dumps(init_request) + '\n')
        process.stdin.flush()
        init_response = json.loads(process.stdout.readline().strip())
        print("✓ Initialize:", "OK" if "result" in init_response else f"ERROR: {init_response}")
        
        # 2. Create session with correct parameters
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
        
        if "result" in session_response:
            session_id = session_response["result"]["session_id"]
            print(f"✓ Session created: {session_id}")
            
            # 3. Test tool execution
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
                result = tool_response["result"]
                if "files" in result:
                    print(f"  Found {len(result['files'])} files/directories")
            else:
                print(f"✗ Tool execution failed: {tool_response}")
        else:
            print(f"✗ Session creation failed: {session_response}")
            
    except Exception as e:
        print(f"Error: {e}")
        import traceback
        traceback.print_exc()
    finally:
        process.terminate()
        process.wait()

if __name__ == "__main__":
    test_complete()