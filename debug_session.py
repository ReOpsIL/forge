#!/usr/bin/env python3
import json
import subprocess

def debug_session():
    process = subprocess.Popen(
        ['./target/debug/forge', '--mcp'],
        stdin=subprocess.PIPE,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        text=True,
        bufsize=0
    )
    
    try:
        # Initialize
        init_request = {"jsonrpc": "2.0", "id": 1, "method": "initialize", "params": {}}
        process.stdin.write(json.dumps(init_request) + '\n')
        process.stdin.flush()
        init_response = process.stdout.readline()
        print("Init response:", init_response.strip())
        
        # Create session
        session_request = {"jsonrpc": "2.0", "id": 2, "method": "session/create", "params": {}}
        process.stdin.write(json.dumps(session_request) + '\n')
        process.stdin.flush()
        session_response = process.stdout.readline()
        print("Session response:", session_response.strip())
        
    except Exception as e:
        print(f"Error: {e}")
    finally:
        process.terminate()
        process.wait()

if __name__ == "__main__":
    debug_session()