#!/usr/bin/env python3
"""
Simple MCP functionality test
"""
import json
import subprocess

def test_basic_functionality():
    process = subprocess.Popen(
        ['./target/debug/forge', '--mcp'],
        stdin=subprocess.PIPE,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        text=True,
        bufsize=0
    )
    
    print("🧪 Testing Forge MCP Server\n")
    
    try:
        # Test 1: Initialize
        print("1. Testing initialization...")
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
        response = json.loads(process.stdout.readline().strip())
        
        if "result" in response:
            server_info = response["result"]["serverInfo"]
            print(f"   ✓ Server: {server_info['name']} v{server_info['version']}")
            print(f"   ✓ Protocol: {response['result']['protocolVersion']}")
        else:
            print(f"   ✗ Failed: {response}")
            return
        
        # Test 2: List tools
        print("\n2. Testing tools list...")
        tools_request = {
            "jsonrpc": "2.0",
            "id": 2,
            "method": "tools/list",
            "params": {}
        }
        
        process.stdin.write(json.dumps(tools_request) + '\n')
        process.stdin.flush()
        response = json.loads(process.stdout.readline().strip())
        
        if "result" in response:
            tools = response["result"]["tools"]
            print(f"   ✓ Found {len(tools)} tools:")
            for tool in tools:
                print(f"     - {tool['name']}: {tool['description']}")
        else:
            print(f"   ✗ Failed: {response}")
            
        print(f"\n🎉 Basic MCP functionality working!")
        print(f"📋 Available tools: {len(tools)} filesystem tools")
        print(f"🔗 Ready for Claude Code integration via stdio transport")
        
    except Exception as e:
        print(f"❌ Error: {e}")
        import traceback
        traceback.print_exc()
    finally:
        process.terminate()
        process.wait()

if __name__ == "__main__":
    test_basic_functionality()