#!/bin/bash

# This script calls the API endpoint to generate a new blocks_config.json file

echo "Generating new blocks_config.json file..."

# Check if the server is running by trying to access the API endpoint
if curl -s -o /dev/null -w "%{http_code}" http://localhost:8080/api/blocks > /dev/null 2>&1; then
    # Server is running, call the API endpoint
    response=$(curl -s -X POST http://localhost:8080/api/generate-sample)
    echo "Response from server: $response"
else
    echo "Server is not running. Please start the server first with 'cargo run' and then run this script again."
    exit 1
fi

echo "Done. The new blocks_config.json file has been generated."