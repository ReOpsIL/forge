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
            description,
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

    /// Converts the Task attributes into a markdown-formatted prompt for LLM execution
    pub fn to_prompt(&self) -> String {
        let mut p = String::new();

        // Add the title (task name)
        p.push_str(&format!("# {} Task\n\n", self.task_name));

        // Add the objective (description)
        p.push_str("## Objective\n");
        p.push_str(&format!("{}\n\n", self.description));

        // Add task details section
        p.push_str("## Task Details\n\n");

        // Add primary requirements (acceptance criteria)
        if !self.acceptance_criteria.is_empty() {
            p.push_str("### Primary Requirements\n");
            for (i, criterion) in self.acceptance_criteria.iter().enumerate() {
                p.push_str(&format!("{}. {}\n", i + 1, criterion));
            }
            p.push_str("\n");
        }

        // Add function signatures if available
        if !self.function_signatures.is_empty() {
            p.push_str("### Function Signatures\n");
            p.push_str("```rust\n");
            for signature in &self.function_signatures {
                p.push_str(&format!("{}\n", signature));
            }
            p.push_str("```\n\n");
        }

        // Add acceptance criteria
        if !self.acceptance_criteria.is_empty() {
            p.push_str("### Acceptance Criteria\n");
            for criterion in &self.acceptance_criteria {
                p.push_str(&format!("- {}\n", criterion));
            }
            p.push_str("\n");
        }

        // Add files to modify
        if !self.files_affected.is_empty() {
            p.push_str("### Files to Modify\n");
            for file in &self.files_affected {
                p.push_str(&format!("- {}\n", file));
            }
            p.push_str("\n");
        }

        // Add dependencies
        if !self.dependencies.is_empty() {
            p.push_str("### Dependencies\n");
            for dependency in &self.dependencies {
                p.push_str(&format!("- {}\n", dependency));
            }
            p.push_str("\n");
        }

        // Add testing requirements
        if !self.testing_requirements.is_empty() {
            p.push_str("### Testing Requirements\n");
            for (i, requirement) in self.testing_requirements.iter().enumerate() {
                p.push_str(&format!("{}. {}\n", i + 1, requirement));
            }
            p.push_str("\n");
        }

        // Add implementation notes (if any)
        if !self.log.is_empty() {
            p.push_str("### Implementation Notes\n");
            p.push_str(&format!("{}\n\n", self.log));
        }

        // Add deliverables section
        p.push_str("## Deliverables\n");
        p.push_str("- Complete implementation of the task according to the requirements\n");
        if !self.testing_requirements.is_empty() {
            p.push_str("- Unit tests covering all acceptance criteria\n");
        }
        if !self.files_affected.is_empty() {
            p.push_str("- Updated files with the necessary changes\n");
        }

        p
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
    pub name: String,
    pub block_id: String,
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

    #[test]
    fn test_task_to_markdown_prompt() {
        // Create a sample task with all fields populated
        let mut task = Task::new("Implement a Deck struct in Rust with standard playing card functionality".to_string());
        task.task_name = "Rust Card Deck Implementation".to_string();
        task.acceptance_criteria = vec![
            "Deck::new() must create a deck with exactly 52 unique cards".to_string(),
            "shuffle() must randomize the card order in place".to_string(),
            "deal(n) must return a Vec<Card> of size n and reduce the deck size by n".to_string(),
            "deal() should return None if insufficient cards remain".to_string(),
        ];
        task.dependencies = vec![
            "rand = \"0.8\"".to_string(),
        ];
        task.files_affected = vec![
            "src/game/deck.rs".to_string(),
            "src/game/mod.rs".to_string(),
            "Cargo.toml".to_string(),
        ];
        task.function_signatures = vec![
            "struct Deck {".to_string(),
            "    cards: Vec<Card>".to_string(),
            "}".to_string(),
            "".to_string(),
            "impl Deck {".to_string(),
            "    pub fn new() -> Self;".to_string(),
            "    pub fn shuffle(&mut self);".to_string(),
            "    pub fn deal(&mut self, count: usize) -> Option<Vec<Card>>;".to_string(),
            "}".to_string(),
        ];
        task.testing_requirements = vec![
            "Deck Creation Test: Verify new() creates exactly 52 unique cards".to_string(),
            "Deal Functionality Test: Verify deal() returns correct number of cards and updates deck state".to_string(),
            "Shuffle Test: Verify shuffled deck order differs from newly created ordered deck".to_string(),
            "Edge Case Test: Test deal() behavior when requesting more cards than available".to_string(),
        ];
        task.log = "Ensure the Card type is properly imported/defined. Use rand::thread_rng() and shuffle() method from rand::seq::SliceRandom trait. Handle edge cases gracefully.".to_string();

        // Generate the markdown prompt
        let markdown = task.to_prompt();

        // Verify the markdown contains expected sections
        assert!(markdown.contains("# Rust Card Deck Implementation Task"));
        assert!(markdown.contains("## Objective"));
        assert!(markdown.contains("## Task Details"));
        assert!(markdown.contains("### Primary Requirements"));
        assert!(markdown.contains("### Function Signatures"));
        assert!(markdown.contains("### Acceptance Criteria"));
        assert!(markdown.contains("### Files to Modify"));
        assert!(markdown.contains("### Dependencies"));
        assert!(markdown.contains("### Testing Requirements"));
        assert!(markdown.contains("### Implementation Notes"));
        assert!(markdown.contains("## Deliverables"));

        // Verify specific content is included
        assert!(markdown.contains("Implement a Deck struct in Rust with standard playing card functionality"));
        assert!(markdown.contains("```rust"));
        assert!(markdown.contains("pub fn new() -> Self;"));
        assert!(markdown.contains("- src/game/deck.rs"));
        assert!(markdown.contains("- rand = \"0.8\""));
        assert!(markdown.contains("1. Deck Creation Test: Verify new() creates exactly 52 unique cards"));
    }
}
