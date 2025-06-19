use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::io::{self, Write};
use serde_json;
use rand::{Rng, distributions::Alphanumeric};
use crate::llm_handler::BlockConnection;
use crate::models::{Block, Connections, InputConnection, OutputConnection, Task};

// Struct to manage block configurations
pub struct BlockConfigManager {
    blocks: Arc<Mutex<Vec<Block>>>,
    config_file: String,
}

impl BlockConfigManager {
    // Create a new BlockConfigManager
    pub fn new(config_file: &str) -> Self {
        BlockConfigManager {
            blocks: Arc::new(Mutex::new(Vec::new())),
            config_file: config_file.to_string(),
        }
    }

    // Load blocks from a JSON file
    pub fn load_blocks_from_file(&self) -> Result<Vec<Block>, String> {
        let path = Path::new(&self.config_file);

        // Check if the file exists
        if !path.exists() {
            return Err(format!("Config file {} does not exist", self.config_file));
        }

        // Read the file
        let file_content = match fs::read_to_string(path) {
            Ok(content) => content,
            Err(e) => return Err(format!("Failed to read config file: {}", e)),
        };

        // Parse the JSON
        let blocks: Vec<Block> = match serde_json::from_str(&file_content) {
            Ok(blocks) => blocks,
            Err(e) => return Err(format!("Failed to parse JSON: {}", e)),
        };

        // Update the in-memory state
        let mut blocks_lock = match self.blocks.lock() {
            Ok(lock) => lock,
            Err(_) => return Err("Failed to acquire lock on blocks".to_string()),
        };
        *blocks_lock = blocks.clone();

        Ok(blocks)
    }

    // Save blocks to a JSON file
    pub fn save_blocks_to_file(&self) -> Result<(), String> {
        let blocks_lock = match self.blocks.lock() {
            Ok(lock) => lock,
            Err(_) => return Err("Failed to acquire lock on blocks".to_string()),
        };

        // Serialize the blocks to JSON
        let json = match serde_json::to_string_pretty(&*blocks_lock) {
            Ok(json) => json,
            Err(e) => return Err(format!("Failed to serialize blocks to JSON: {}", e)),
        };

        // Write to the file
        let mut file = match fs::File::create(&self.config_file) {
            Ok(file) => file,
            Err(e) => return Err(format!("Failed to create config file: {}", e)),
        };

        match file.write_all(json.as_bytes()) {
            Ok(_) => Ok(()),
            Err(e) => Err(format!("Failed to write to config file: {}", e)),
        }
    }

    // Get all blocks
    pub fn get_blocks(&self) -> Result<Vec<Block>, String> {
        let blocks_lock = match self.blocks.lock() {
            Ok(lock) => lock,
            Err(_) => return Err("Failed to acquire lock on blocks".to_string()),
        };

        Ok(blocks_lock.clone())
    }

    // Add a new block
    pub fn add_block(&self, mut block: Block) -> Result<(), String> {
        let mut blocks_lock = match self.blocks.lock() {
            Ok(lock) => lock,
            Err(_) => return Err("Failed to acquire lock on blocks".to_string()),
        };

        // Check if a block with the same name already exists
        if blocks_lock.iter().any(|b| b.name == block.name) {
            return Err(format!("Block with name {} already exists", block.name));
        }

        // Generate a block_id if not provided
        if block.block_id.is_empty() {
            // Generate a random 6-character alphanumeric ID
            let block_id: String = rand::thread_rng()
                .sample_iter(&Alphanumeric)
                .take(6)
                .map(char::from)
                .collect();
            block.block_id = block_id;
        }

        blocks_lock.push(block);
        Ok(())
    }

    // Update an existing block
    pub fn update_block(&self, block: Block) -> Result<(), String> {
        let mut blocks_lock = match self.blocks.lock() {
            Ok(lock) => lock,
            Err(_) => return Err("Failed to acquire lock on blocks".to_string()),
        };

        // Find the block to update by block_id
        let index = blocks_lock.iter().position(|b| b.block_id == block.block_id);
        match index {
            Some(i) => {
                blocks_lock[i] = block;
                Ok(())
            },
            None => Err(format!("Block with ID {} not found", block.block_id)),
        }
    }

    // Delete a block
    pub fn delete_block(&self, block_id: &str) -> Result<(), String> {
        let mut blocks_lock = match self.blocks.lock() {
            Ok(lock) => lock,
            Err(_) => return Err("Failed to acquire lock on blocks".to_string()),
        };

        // Find the block to delete
        let index = blocks_lock.iter().position(|b| b.block_id == block_id);
        match index {
            Some(i) => {
                blocks_lock.remove(i);
                Ok(())
            },
            None => Err(format!("Block with ID {} not found", block_id)),
        }
    }

    // Add a todo item to a block
    pub fn add_todo_item(&self, block_id: &str, todo_item: &str) -> Result<(), String> {
        let mut blocks_lock = match self.blocks.lock() {
            Ok(lock) => lock,
            Err(_) => return Err("Failed to acquire lock on blocks".to_string()),
        };

        // Find the block to update
        let index = blocks_lock.iter().position(|b| b.block_id == block_id);
        match index {
            Some(i) => {
                let task = Task::new(todo_item.to_string());
                let task_id = task.task_id.clone();
                blocks_lock[i].todo_list.insert(task_id, task);
                Ok(())
            },
            None => Err(format!("Block with ID {} not found", block_id)),
        }
    }

    // Remove a todo item from a block
    pub fn remove_todo_item(&self, block_id: &str, task_id: String) -> Result<(), String> {
        let mut blocks_lock = match self.blocks.lock() {
            Ok(lock) => lock,
            Err(_) => return Err("Failed to acquire lock on blocks".to_string()),
        };

        // Find the block to update
        let block_index = blocks_lock.iter().position(|b| b.block_id == block_id);
        match block_index {
            Some(i) => {
                if blocks_lock[i].todo_list.contains_key(&task_id) {
                    blocks_lock[i].todo_list.remove(&task_id);
                    Ok(())
                } else {
                    Err(format!("Todo item index {} out of bounds", task_id))
                }
            },
            None => Err(format!("Block with ID {} not found", block_id)),
        }
    }
}

// Function to generate a sample JSON file with 10 random blocks
pub fn generate_sample_config(filename: &str) -> Result<(), io::Error> {
    let mut blocks = Vec::new();
    let block_names = vec![
        "DataIngestion", "DataProcessing", "DataVisualization",
        "DataStorage", "DataAnalysis", "DataExport",
        "DataValidation", "DataTransformation", "DataAggregation", "DataReporting"
    ];

    // Create 10 random blocks
    for i in 0..10 {
        let name = block_names[i].to_string();
        let description = format!("This is the {} module", name);

        // Generate random inputs and outputs
        let num_inputs = rand::thread_rng().gen_range(1..=3);
        let num_outputs = rand::thread_rng().gen_range(1..=3);

        // Generate random connections
        let mut input_connections = Vec::new();
        let mut output_connections = Vec::new();

        // Only create connections if not the first block
        if i > 0 {
            // Add an input connection from a previous block
            let from_block_index = rand::thread_rng().gen_range(0..i);
            let from_block = block_names[from_block_index].to_string();
            input_connections.push(InputConnection::new(
                from_block.clone(),
                format!("Output{}", rand::thread_rng().gen_range(1..=3)),
            ));
        }

        // Only create output connections if not the last block
        if i < 9 {
            // Add an output connection to a next block
            let to_block_index = rand::thread_rng().gen_range(i+1..10);
            let to_block = block_names[to_block_index].to_string();

            // Generate a random 4-character alphanumeric ID
            let unique_id: String = rand::thread_rng()
                .sample_iter(&Alphanumeric)
                .take(4)
                .map(char::from)
                .collect();

            output_connections.push(OutputConnection {
                to_module: to_block,
                input_type: format!("Input{}", rand::thread_rng().gen_range(1..=3)),
                output_id: unique_id,
            });
        }

        // Generate random todo items
        let num_todos = rand::thread_rng().gen_range(1..=4);
        let todo_list = (0..num_todos)
            .map(|j| format!("Todo item {} for {}", j + 1, name));

        let tasks = {
            let mut map = HashMap::new();
            todo_list.for_each(|t| {
                let task = Task::new(t);
                let task_id = task.task_id.clone();
                map.insert(task_id, task);
            });
            map
        };

        // Generate a random 6-character alphanumeric ID for the block
        let block_id: String = rand::thread_rng()
            .sample_iter(&Alphanumeric)
            .take(6)
            .map(char::from)
            .collect();

        // Create the block
        let block = Block {
            block_id,
            name,
            description,
            inputs: vec![BlockConnection::new()],
            outputs: vec![BlockConnection::new()],
            connections: Connections {
                input_connections,
                output_connections,
            },
            todo_list: tasks,
        };

        blocks.push(block);
    }

    // Serialize the blocks to JSON
    let json = serde_json::to_string_pretty(&blocks)?;

    // Create the directory if it doesn't exist
    if let Some(parent) = Path::new(filename).parent() {
        fs::create_dir_all(parent)?;
    }

    // Write to the file
    let mut file = fs::File::create(filename)?;
    file.write_all(json.as_bytes())?;

    Ok(())
}

// Function to load blocks from a file (replacement for the hard-coded get_blocks function)
pub fn load_blocks_from_file(filename: &str) -> Vec<Block> {
    let path = Path::new(filename);

    // Check if the file exists
    if !path.exists() {
        // If the file doesn't exist, generate a sample config
        match generate_sample_config(filename) {
            Ok(_) => println!("Generated sample config file: {}", filename),
            Err(e) => {
                eprintln!("Failed to generate sample config: {}", e);
                return Vec::new();
            }
        }
    }

    // Read the file
    let file_content = match fs::read_to_string(path) {
        Ok(content) => content,
        Err(e) => {
            eprintln!("Failed to read config file: {}", e);
            return Vec::new();
        }
    };

    // Parse the JSON
    match serde_json::from_str(&file_content) {
        Ok(blocks) => blocks,
        Err(e) => {
            eprintln!("Failed to parse JSON: {}", e);
            Vec::new()
        }
    }
}
