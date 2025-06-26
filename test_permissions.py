#!/usr/bin/env python3
"""
Test script to check MCP server permissions for block creation tools
"""
import json
import subprocess
import sys
import time

def test_mcp_permissions():
    """Test MCP server permissions for block and task creation"""
    # Start the MCP server process
    process = subprocess.Popen(
        ['./target/debug/forge', '--mcp'],
        stdin=subprocess.PIPE,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        text=True,
        bufsize=0
    )
    
    try:
        # Test 1: Initialize request
        init_request = {
            "jsonrpc": "2.0",
            "id": 1,
            "method": "initialize",
            "params": {
                "protocolVersion": "2024-11-05",
                "capabilities": {},
                "clientInfo": {
                    "name": "permission-test-client",
                    "version": "1.0.0"
                }
            }
        }
        
        print("1. Sending initialize request...")
        process.stdin.write(json.dumps(init_request) + '\n')
        process.stdin.flush()
        
        response_line = process.stdout.readline()
        if response_line:
            response = json.loads(response_line.strip())
            print(f"   Initialize response: {response.get('result', {}).get('server_info', {})}")
        else:
            print("   No response received")
            return False
            
        # Test 2: List tools request to see available tools
        tools_request = {
            "jsonrpc": "2.0",
            "id": 2,
            "method": "tools/list",
            "params": {}
        }
        
        print("\n2. Checking available tools...")
        process.stdin.write(json.dumps(tools_request) + '\n')
        process.stdin.flush()
        
        response_line = process.stdout.readline()
        if response_line:
            response = json.loads(response_line.strip())
            tools = response.get('result', {}).get('tools', [])
            print(f"   Available tools: {len(tools)}")
            for tool in tools:
                print(f"   - {tool['name']}: {tool['description']}")
        else:
            print("   No response received")
            return False
            
        # Test 3: Try to create a block (should fail due to permissions)
        create_block_request = {
            "jsonrpc": "2.0",
            "id": 3,
            "method": "tools/call",
            "params": {
                "name": "create_block",
                "arguments": {
                    "name": "Test Permission Block",
                    "description": "Testing if block creation works with current permissions"
                }
            }
        }
        
        print("\n3. Testing create_block tool (should fail with permission error)...")
        process.stdin.write(json.dumps(create_block_request) + '\n')
        process.stdin.flush()
        
        response_line = process.stdout.readline()
        if response_line:
            response = json.loads(response_line.strip())
            if 'error' in response:
                print(f"   Expected permission error: {response['error']['message']}")
                if 'permission' in response['error']['message'].lower():
                    print("   ✅ Permission system is working - correctly blocking unauthorized access")
                else:
                    print(f"   ❌ Unexpected error (not permission-related): {response['error']['message']}")
            elif 'result' in response:
                print("   ❌ Block creation succeeded - permissions may be too permissive!")
                print(f"   Result: {response['result']}")
        else:
            print("   No response received")
            
        # Test 4: Try to list blocks (should work with read permission)
        list_blocks_request = {
            "jsonrpc": "2.0",
            "id": 4,
            "method": "tools/call",
            "params": {
                "name": "list_blocks",
                "arguments": {}
            }
        }
        
        print("\n4. Testing list_blocks tool (should work with read permission)...")
        process.stdin.write(json.dumps(list_blocks_request) + '\n')
        process.stdin.flush()
        
        response_line = process.stdout.readline()
        if response_line:
            response = json.loads(response_line.strip())
            if 'error' in response:
                print(f"   Error: {response['error']['message']}")
            elif 'result' in response:
                print("   ✅ List blocks succeeded - read permission is working")
                # Try to parse the result to see block count
                try:
                    result_data = json.loads(response['result']['content'][0]['text'])
                    print(f"   Found {len(result_data)} blocks")
                except:
                    print("   Block list retrieved successfully")
        else:
            print("   No response received")
            
    except Exception as e:
        print(f"Error during testing: {e}")
        return False
    finally:
        process.terminate()
        process.wait()
        
    return True

if __name__ == "__main__":
    print("MCP Server Permission Test")
    print("=" * 50)
    success = test_mcp_permissions()
    if success:
        print("\n✅ Permission test completed")
    else:
        print("\n❌ Permission test failed")
        sys.exit(1)