use serde::{Serialize, Deserialize};
use rand::{Rng, distributions::Alphanumeric};

// Define the structure for module connections
#[derive(Serialize, Deserialize, Clone)]
pub struct InputConnection {
    pub from_module: String,
    pub output_type: String,
    pub unique_id: String,
}

impl InputConnection {
    pub fn new(from_module: String, output_type: String) -> Self {
        // Generate a random 4-character alphanumeric ID
        let unique_id: String = rand::thread_rng()
            .sample_iter(&Alphanumeric)
            .take(4)
            .map(char::from)
            .collect();

        Self {
            from_module,
            output_type,
            unique_id,
        }
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct OutputConnection {
    pub to_module: String,
    pub input_type: String,
    pub unique_id: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Connections {
    pub input_connections: Vec<InputConnection>,
    pub output_connections: Vec<OutputConnection>,
}

// Define the structure for a software module
#[derive(Serialize, Deserialize, Clone)]
pub struct Block {
    pub name: String,
    pub description: String,
    pub inputs: Vec<String>,
    pub outputs: Vec<String>,
    pub connections: Connections,
    pub todo_list: Vec<String>,
}

// Function to get a list of blocks (in a real application, this would likely fetch from a database)
pub fn get_blocks() -> Vec<Block> {
    vec![
        Block {
            name: "DataIngestion".to_string(),
            description: "Handles the ingestion of raw data from various sources".to_string(),
            inputs: vec!["RawData".to_string()],
            outputs: vec!["ParsedData".to_string()],
            connections: Connections {
                input_connections: vec![],
                output_connections: vec![
                    OutputConnection {
                        to_module: "DataProcessing".to_string(),
                        input_type: "ParsedData".to_string(),
                        unique_id: "conn-1".to_string(),
                    }
                ],
            },
            todo_list: vec![
                "Add support for CSV files".to_string(),
                "Improve error handling".to_string(),
            ],
        },
        Block {
            name: "DataProcessing".to_string(),
            description: "Processes and transforms the parsed data".to_string(),
            inputs: vec!["ParsedData".to_string()],
            outputs: vec!["ProcessedData".to_string()],
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
                        unique_id: "conn-2".to_string(),
                    }
                ],
            },
            todo_list: vec![
                "Implement data normalization".to_string(),
                "Add support for filtering".to_string(),
            ],
        },
        Block {
            name: "DataVisualization".to_string(),
            description: "Visualizes the processed data".to_string(),
            inputs: vec!["ProcessedData".to_string()],
            outputs: vec!["Visualization".to_string()],
            connections: Connections {
                input_connections: vec![
                    InputConnection::new(
                        "DataProcessing".to_string(),
                        "ProcessedData".to_string(),
                    )
                ],
                output_connections: vec![],
            },
            todo_list: vec![
                "Add more chart types".to_string(),
                "Implement interactive visualizations".to_string(),
            ],
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
        assert_eq!(conn.unique_id.len(), 4);

        // Check that multiple calls generate different IDs
        let conn2 = InputConnection::new("TestModule".to_string(), "TestOutput".to_string());
        assert_ne!(conn.unique_id, conn2.unique_id);
    }
}
