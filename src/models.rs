use std::collections::HashMap;
use serde::{Serialize, Deserialize};
use rand::{Rng, distributions::Alphanumeric};
use crate::llm_handler::BlockConnection;

// Define the structure for a task

// Define the structure for task response from LLM
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Task {
    pub task_id: String,
    pub task_name: String,
    pub description: String,
    pub acceptance_criteria: Vec<String>,
    pub dependencies: Vec<String>,
    pub estimated_effort: String,
    pub files_affected: Vec<String>,
    pub function_signatures: Vec<String>,
    pub testing_requirements: Vec<String>,
    pub log: String,
    pub commit_id: String,
    pub status: String,
}

impl Task {
    pub fn new(description: String) -> Self {
        let unique_id: String = rand::thread_rng()
            .sample_iter(&Alphanumeric)
            .take(6)
            .map(char::from)
            .collect();
        Self {
            task_id: unique_id,
            task_name: "".to_string(),
            description: description,
            acceptance_criteria: Vec::new(),
            dependencies: Vec::new(),
            estimated_effort: "".to_string(),
            files_affected: Vec::new(),
            function_signatures: Vec::new(),
            testing_requirements: Vec::new(),
            log: String::new(),
            commit_id: "".to_string(),
            status: "".to_string()

        }
    }
}

// Define the structure for module connections
#[derive(Serialize, Deserialize, Clone)]
pub struct InputConnection {
    pub from_module: String,
    pub output_type: String,
    pub input_id: String,
}

impl InputConnection {
    pub fn new(from_module: String, output_type: String) -> Self {
        // Generate a random 4-character alphanumeric ID
        let unique_id: String = rand::thread_rng()
            .sample_iter(&Alphanumeric)
            .take(6)
            .map(char::from)
            .collect();

        Self {
            from_module,
            output_type,
            input_id: unique_id,
        }
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct OutputConnection {
    pub to_module: String,
    pub input_type: String,
    pub output_id: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Connections {
    pub input_connections: Vec<InputConnection>,
    pub output_connections: Vec<OutputConnection>,
}

// Define the structure for a software module
#[derive(Serialize, Deserialize, Clone)]
pub struct Block {
    pub block_id: String,
    pub name: String,
    pub description: String,
    pub inputs: Vec<BlockConnection>,
    pub outputs: Vec<BlockConnection>,
    pub connections: Connections,
    pub todo_list: HashMap<String,Task>,
}

impl Block {
    pub fn new(name: String, description: String, inputs: Vec<BlockConnection>, outputs: Vec<BlockConnection>) -> Self {
        // Generate a random 6-character alphanumeric ID
        let block_id: String = rand::thread_rng()
            .sample_iter(&Alphanumeric)
            .take(6)
            .map(char::from)
            .collect();

        Self {
            block_id,
            name,
            description,
            inputs,
            outputs,
            connections: Connections {
                input_connections: Vec::new(),
                output_connections: Vec::new(),
            },
            todo_list: HashMap::new(),
        }
    }
    pub fn update_task(mut self, task: Task) {
        println!("Updating task status {} {}", task.task_id, task.status);
        let task_id = task.clone().task_id;
        self.todo_list.insert(task_id, task);

    }
}

// Function to get a list of blocks (in a real application, this would likely fetch from a database)
pub fn get_blocks() -> Vec<Block> {
    vec![
        Block {
            block_id: "abc123".to_string(), // Sample block_id
            name: "DataIngestion".to_string(),
            description: "Handles the ingestion of raw data from various sources".to_string(),
            inputs: vec![BlockConnection::new()],
            outputs: vec![BlockConnection::new()],
            connections: Connections {
                input_connections: vec![],
                output_connections: vec![
                    OutputConnection {
                        to_module: "DataProcessing".to_string(),
                        input_type: "ParsedData".to_string(),
                        output_id: "conn-1".to_string(),
                    }
                ],
            },
            todo_list: {
                let mut map = HashMap::new();
                let task1 = Task::new( "Add support for CSV files".to_string());
                let task2 = Task::new( "Improve error handling".to_string());
                map.insert(task1.task_id.clone(), task1);
                map.insert(task2.task_id.clone(), task2);
                map
            },
        },
        Block {
            block_id: "def456".to_string(), // Sample block_id
            name: "DataProcessing".to_string(),
            description: "Processes and transforms the parsed data".to_string(),
            inputs: vec![BlockConnection::new()],
            outputs: vec![BlockConnection::new()],
            connections: Connections {
                input_connections: vec![
                    InputConnection::new(
                        "DataIngestion".to_string(),
                        "ParsedData".to_string(),
                    )
                ],
                output_connections: vec![
                    OutputConnection {
                        to_module: "DataVisualization".to_string(),
                        input_type: "ProcessedData".to_string(),
                        output_id: "conn-2".to_string(),
                    }
                ],
            },
            todo_list: {
                let mut map = HashMap::new();
                let task1 = Task::new( "Implement data normalization".to_string());
                let task2 = Task::new( "Add support for filtering".to_string());
                map.insert(task1.task_id.clone(), task1);
                map.insert(task2.task_id.clone(), task2);
                map
            },
        },
        Block {
            block_id: "ghi789".to_string(), // Sample block_id
            name: "DataVisualization".to_string(),
            description: "Visualizes the processed data".to_string(),
            inputs: vec![BlockConnection::new()],
            outputs: vec![BlockConnection::new()],
            connections: Connections {
                input_connections: vec![
                    InputConnection::new(
                        "DataProcessing".to_string(),
                        "ProcessedData".to_string(),
                    )
                ],
                output_connections: vec![],
            },
            todo_list: {
                let mut map = HashMap::new();
                let task1 = Task::new( "Add more chart types".to_string());
                let task2 = Task::new( "Implement interactive visualizations".to_string());
                map.insert(task1.task_id.clone(), task1);
                map.insert(task2.task_id.clone(), task2);
                map
            },
        },
    ]
}

// Test function to verify that the random ID generation works correctly
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_input_connection_id_generation() {
        let conn = InputConnection::new("TestModule".to_string(), "TestOutput".to_string());

        // Check that the unique_id is 4 characters long
        assert_eq!(conn.input_id.len(), 4);

        // Check that multiple calls generate different IDs
        let conn2 = InputConnection::new("TestModule".to_string(), "TestOutput".to_string());
        assert_ne!(conn.input_id, conn2.input_id);
    }
}
