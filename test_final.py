#!/usr/bin/env python3
"""
Final MCP server test with correct parameters
"""
import json
import subprocess
import time

def test_mcp():
    process = subprocess.Popen(
        ['./target/debug/forge', '--mcp'],
        stdin=subprocess.PIPE,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        text=True,
        bufsize=0
    )
    
    try:
        # 1. Initialize
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
        
        # 2. Create session with all required fields
        session_request = {
            "jsonrpc": "2.0",
            "id": 2,
            "method": "session/create",
            "params": {
                "client_name": "test-client",
                "client_version": "1.0.0",
                "user_id": "test-user",
                "capabilities": ["tools"],
                "connection_time": int(time.time())
            }
        }
        
        process.stdin.write(json.dumps(session_request) + '\n')
        process.stdin.flush()
        session_response = json.loads(process.stdout.readline().strip())
        
        if "result" in session_response:
            session_id = session_response["result"]["session_id"]
            print(f"✓ Session created: {session_id}")
            
            # 3. List tools
            tools_request = {
                "jsonrpc": "2.0",
                "id": 3,
                "method": "tools/list",
                "params": {}
            }
            
            process.stdin.write(json.dumps(tools_request) + '\n')
            process.stdin.flush()
            tools_response = json.loads(process.stdout.readline().strip())
            
            if "result" in tools_response:
                tools = tools_response["result"]["tools"]
                print(f"✓ Listed {len(tools)} tools")
                
                # 4. Test read_file tool (should work without session)
                # First, let's try without session to see if it works
                read_request = {
                    "jsonrpc": "2.0",
                    "id": 4,
                    "method": "tools/call",
                    "params": {
                        "name": "read_file",
                        "arguments": {
                            "path": "Cargo.toml",
                            "max_size": 1000
                        }
                    }
                }
                
                process.stdin.write(json.dumps(read_request) + '\n')
                process.stdin.flush()
                read_response = json.loads(process.stdout.readline().strip())
                
                if "result" in read_response:
                    print("✓ File read successful!")
                    print(f"  Read {len(read_response['result'].get('content', ''))} characters")
                else:
                    print(f"✗ File read failed: {read_response}")
            else:
                print(f"✗ Tools list failed: {tools_response}")
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
    test_mcp()