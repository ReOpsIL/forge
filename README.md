# Forge IDE

A Visual Software Architecture & Modules-Driven Development Platform with LLM-Powered Task Programming

## Overview

Forge IDE is a revolutionary development platform that visualizes software as an interconnected graph of intelligent modules. Developers can architect systems through high-level specifications and task definitions rather than writing code directly. The integrated LLM backend translates architectural intent into functioning code while maintaining perfect awareness of inter-module relationships and data flows.

## Features

### Block-Based Architecture
- Software modules represented as blocks with inputs, outputs, and connections
- JSON-based configuration for storing block definitions
- Dynamic loading and management of block configurations

### Visual Interface
- Card-based view for detailed block information
- Interactive flow diagram for visualizing block connections using ReactFlow
- Ability to edit block descriptions and manage todo items
- Built with Vite, React, and styled with PrimeReact components

### Backend API
- RESTful API for CRUD operations on blocks
- Support for adding and removing todo items
- Sample configuration generation

### LLM Integration
- Multiple LLM providers supported (OpenRouter, Gemini, Anthropic)
- Auto-completion and enhancement of block descriptions
- Automatic task generation from block descriptions
- Processing markdown specifications to generate blocks

### Git Integration
- Branch creation and management
- Commit, merge, push, and pull operations
- Task execution with Git integration
- Diff viewing for tasks
- Build handling

### Task Management
- Task execution functionality
- Task status tracking
- Task logs and commit tracking

## Quick Start

To run the application:

## Getting Started

### Prerequisites

- Node.js (v14 or later)
- npm (v6 or later)

### Installation

1. Clone the repository
2. Install the frontend dependencies:

```bash
npm run install-frontend
```

### Development

To start the development server:

```bash
npm run dev
```

This will start the Vite development server at http://localhost:5173.

Note: The development server will keep running in your terminal. To stop it, press `Ctrl+C` in the terminal.

### Building for Production

To build the application for production:

```bash
npm run build
```

The build output will be in the `frontend/dist` directory.

### Preview Production Build

To preview the production build:

```bash
npm run preview
```

## Project Structure

- `frontend/`: Contains the React application
  - `src/`: Source code
    - `App.jsx`: Main application component with the top menu
    - `main.jsx`: Entry point for the React application
    - `index.css`: Global styles
    - `App.css`: Styles for the App component
  - `public/`: Static assets
  - `index.html`: HTML template
  - `vite.config.js`: Vite configuration
  - `package.json`: Frontend dependencies and scripts
- `src/`: Contains the Rust backend
  - `main.rs`: Entry point for the Rust server
  - `models.rs`: Data models for blocks
  - `block_config.rs`: Block configuration management
  - `block_handlers.rs`: API handlers for block operations
  - `lib.rs`: Library exports for examples
- `examples/`: Contains example Rust code
  - `generate_config.rs`: Example to generate a new blocks_config.json file
- `blocks_config.json`: Configuration file for blocks

## Backend Development

### Starting the Backend Server

To start the backend server:

```bash
cargo run
```

This will start the server at http://localhost:8080.

### Generating Blocks Configuration

There are two ways to generate a new blocks_config.json file:

1. Using the shell script (requires the server to be running):

```bash
chmod +x generate_config.sh
./generate_config.sh
```

2. Using the Rust example (does not require the server to be running):

```bash
cargo run --example generate_config
```

Both methods will generate a new blocks_config.json file with 10 random blocks.

### API Endpoints

The backend provides the following RESTful API endpoints:

| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | /api/blocks | Get all blocks |
| POST | /api/blocks | Add a new block |
| PUT | /api/blocks | Update an existing block |
| DELETE | /api/blocks/{name} | Delete a block by name |
| POST | /api/blocks/{name}/todo | Add a todo item to a block |
| DELETE | /api/blocks/{name}/todo/{index} | Remove a todo item from a block |
| POST | /api/generate-sample | Generate a new sample configuration |

#### Example API Usage

##### Block Management

Get all blocks:
```bash
curl -X GET http://localhost:8080/api/blocks
```

Add a new block:
```bash
curl -X POST http://localhost:8080/api/blocks \
  -H "Content-Type: application/json" \
  -d '{
    "name": "NewBlock",
    "description": "A new block",
    "inputs": ["Input1"],
    "outputs": ["Output1"],
    "connections": {
      "input_connections": [],
      "output_connections": []
    },
    "todo_list": ["Implement this block"]
  }'
```

Update a block:
```bash
curl -X PUT http://localhost:8080/api/blocks \
  -H "Content-Type: application/json" \
  -d '{
    "name": "ExistingBlock",
    "description": "Updated description",
    "inputs": ["Input1", "Input2"],
    "outputs": ["Output1"],
    "connections": {
      "input_connections": [],
      "output_connections": []
    },
    "todo_list": ["Updated todo item"]
  }'
```

Delete a block:
```bash
curl -X DELETE http://localhost:8080/api/blocks/BlockName
```

Add a todo item to a block:
```bash
curl -X POST http://localhost:8080/api/blocks/BlockName/todo \
  -H "Content-Type: application/json" \
  -d '"New todo item"'
```

Remove a todo item from a block:
```bash
curl -X DELETE http://localhost:8080/api/blocks/BlockName/todo/0
```

Generate a new sample configuration:
```bash
curl -X POST http://localhost:8080/api/generate-sample
```

##### LLM Integration

Enhance a block's description:
```bash
curl -X PUT http://localhost:8080/api/blocks/BlockName/enhance \
  -H "Content-Type: application/json" \
  -d '{}'
```

Generate tasks for a block:
```bash
curl -X PUT http://localhost:8080/api/blocks/BlockName/generate-tasks \
  -H "Content-Type: application/json" \
  -d '{}'
```

Auto-complete a partial description:
```bash
curl -X POST http://localhost:8080/api/blocks/auto-complete \
  -H "Content-Type: application/json" \
  -d '{
    "partial_description": "This block handles user authentication and"
  }'
```

Process markdown specifications:
```bash
curl -X POST http://localhost:8080/api/blocks/process-markdown \
  -H "Content-Type: application/json" \
  -d '{
    "markdown_content": "# User Authentication Module\n\nThis module handles user login, registration, and session management."
  }'
```

Execute a task:
```bash
curl -X POST http://localhost:8080/api/blocks/execute-task \
  -H "Content-Type: application/json" \
  -d '{
    "block_name": "BlockName",
    "task_index": 0
  }'
```

##### Project Configuration

Get project configuration:
```bash
curl -X GET http://localhost:8080/api/project
```

Update project configuration:
```bash
curl -X PUT http://localhost:8080/api/project \
  -H "Content-Type: application/json" \
  -d '{
    "project_name": "My Project",
    "project_home_directory": "/path/to/project",
    "git_repository_url": "https://github.com/username/repo.git",
    "llm_provider": "openrouter"
  }'
```

Test Git connection:
```bash
curl -X POST http://localhost:8080/api/project/test-git-connection \
  -H "Content-Type: application/json" \
  -d '{}'
```

##### Git Integration

Create a new branch:
```bash
curl -X POST http://localhost:8080/api/git/branch \
  -H "Content-Type: application/json" \
  -d '{
    "branch_name": "feature/new-feature"
  }'
```

Commit changes:
```bash
curl -X POST http://localhost:8080/api/git/commit \
  -H "Content-Type: application/json" \
  -d '{
    "commit_message": "Implement new feature"
  }'
```

Merge a branch:
```bash
curl -X POST http://localhost:8080/api/git/merge \
  -H "Content-Type: application/json" \
  -d '{
    "source_branch": "feature/new-feature",
    "target_branch": "main"
  }'
```

Push changes:
```bash
curl -X POST http://localhost:8080/api/git/push \
  -H "Content-Type: application/json" \
  -d '{}'
```

Pull changes:
```bash
curl -X POST http://localhost:8080/api/git/pull \
  -H "Content-Type: application/json" \
  -d '{}'
```

Execute a task with Git integration:
```bash
curl -X POST http://localhost:8080/api/git/execute-task \
  -H "Content-Type: application/json" \
  -d '{
    "block_name": "BlockName",
    "task_index": 0,
    "branch_name": "feature/task-implementation"
  }'
```

### Data Models

The application uses the following data models:

#### Block

The main data structure representing a software module:

```rust
pub struct Block {
    pub name: String,              // Unique identifier for the block
    pub description: String,       // Description of the block's functionality
    pub inputs: Vec<String>,       // List of input port types
    pub outputs: Vec<String>,      // List of output port types
    pub connections: Connections,  // Input and output connections to other blocks
    pub todo_list: Vec<String>,    // List of todo items for this block
}
```

#### Connections

Contains lists of input and output connections:

```rust
pub struct Connections {
    pub input_connections: Vec<InputConnection>,    // Connections from other blocks to this block
    pub output_connections: Vec<OutputConnection>,  // Connections from this block to other blocks
}
```

#### InputConnection

Represents a connection from another block to this block:

```rust
pub struct InputConnection {
    pub from_module: String,   // Name of the source block
    pub output_type: String,   // Type of the output port on the source block
    pub unique_id: String,     // Unique identifier for this connection
}
```

#### OutputConnection

Represents a connection from this block to another block:

```rust
pub struct OutputConnection {
    pub to_module: String,     // Name of the target block
    pub input_type: String,    // Type of the input port on the target block
    pub unique_id: String,     // Unique identifier for this connection
}
```

### Block Configuration Management

The `BlockConfigManager` is responsible for loading, saving, and manipulating blocks:

```rust
pub struct BlockConfigManager {
    blocks: Arc<Mutex<Vec<Block>>>,  // Thread-safe storage for blocks
    config_file: String,             // Path to the configuration file
}
```

Key methods:

- `new(config_file: &str) -> Self`: Creates a new BlockConfigManager
- `load_blocks_from_file(&self) -> Result<Vec<Block>, String>`: Loads blocks from the config file
- `save_blocks_to_file(&self) -> Result<(), String>`: Saves blocks to the config file
- `get_blocks(&self) -> Result<Vec<Block>, String>`: Returns all blocks
- `add_block(&self, block: Block) -> Result<(), String>`: Adds a new block
- `update_block(&self, block: Block) -> Result<(), String>`: Updates an existing block
- `delete_block(&self, block_name: &str) -> Result<(), String>`: Deletes a block
- `add_todo_item(&self, block_name: &str, todo_item: &str) -> Result<(), String>`: Adds a todo item to a block
- `remove_todo_item(&self, block_name: &str, todo_index: usize) -> Result<(), String>`: Removes a todo item from a block

### Configuration

#### Project Configuration

The project configuration is stored in `project_config.json` and can be managed through the API or by directly editing the file. The configuration includes:

- `project_name`: Name of the project
- `project_home_directory`: Path to the project's home directory
- `git_repository_url`: URL of the Git repository
- `llm_provider`: Default LLM provider to use (openrouter, gemini, or anthropic)
- `openrouter_model`: Model to use with OpenRouter
- `gemini_model`: Model to use with Gemini
- `anthropic_model`: Model to use with Anthropic
- `auto_complete_system_prompt`: System prompt for auto-completion
- `auto_complete_user_prompt`: User prompt template for auto-completion
- `enhance_description_system_prompt`: System prompt for enhancing descriptions
- `enhance_description_user_prompt`: User prompt template for enhancing descriptions
- `generate_tasks_system_prompt`: System prompt for generating tasks
- `generate_tasks_user_prompt`: User prompt template for generating tasks
- `process_specification_system_prompt`: System prompt for processing markdown specifications
- `process_specification_user_prompt`: User prompt template for processing markdown specifications

#### Block Configuration

Block configurations are stored in `blocks_config.json` (or a custom path specified in the project configuration). Each block includes:

- `name`: Unique identifier for the block
- `description`: Description of the block's functionality
- `inputs`: List of input port types
- `outputs`: List of output port types
- `connections`: Input and output connections to other blocks
- `todo_list`: List of todo items for this block

### Frontend Components

The frontend is built with React and includes the following main components:

#### App Component

The main application component that renders the top menu and manages the active view:

```jsx
function App() {
  const [activeView, setActiveView] = useState('home')

  // Menu items: Blocks, Flow, Help, Home
  // ...

  return (
    <div className="app-container">
      <Menubar model={items} />
      {renderContent()}
    </div>
  )
}
```

#### BlocksView Component

Displays blocks in a card-based layout with the ability to view and edit block details:

```jsx
const BlocksView = () => {
  const [blocks, setBlocks] = useState([])
  const [loading, setLoading] = useState(true)
  const [editingDescription, setEditingDescription] = useState({})

  // Fetch blocks from API
  // Handle description editing
  // ...

  return (
    <div className="blocks-container">
      <h2>Blocks</h2>
      <div className="grid">
        {blocks.map((block) => (
          <Card key={block.name} title={block.name} className="block-card">
            {/* Block details, inputs, outputs, connections, todo list */}
          </Card>
        ))}
      </div>
    </div>
  )
}
```

#### FlowView Component

Visualizes blocks and their connections as an interactive flow diagram using ReactFlow:

```jsx
const FlowView = () => {
  const [blocks, setBlocks] = useState([])
  const [loading, setLoading] = useState(true)
  const [nodes, setNodes, onNodesChange] = useNodesState([])
  const [edges, setEdges, onEdgesChange] = useEdgesState([])

  // Fetch blocks from API
  // Process blocks data to create nodes and edges
  // ...

  return (
    <div className="flow-container">
      <div className="flow-header">
        <button onClick={() => onLayout('TB')} className="layout-button">
          Auto Layout
        </button>
      </div>
      <div className="flow-canvas">
        <ReactFlow
          nodes={nodes}
          edges={edges}
          onNodesChange={onNodesChange}
          onEdgesChange={onEdgesChange}
          fitView
          className="dark-flow"
        >
          <Controls />
          <Background variant="dots" gap={12} size={1} color="#444" />
        </ReactFlow>
      </div>
    </div>
  )
}
```

## Contributing

Contributions to Forge IDE are welcome! Here's how you can contribute:

1. **Report Bugs**: If you find a bug, please create an issue with a detailed description of the problem, steps to reproduce, and your environment.

2. **Suggest Features**: Have an idea for a new feature? Open an issue to discuss it.

3. **Submit Pull Requests**: Want to contribute code? Fork the repository, create a branch, make your changes, and submit a pull request.

4. **Improve Documentation**: Help improve the documentation by fixing errors, adding examples, or clarifying explanations.

### Development Workflow

1. Fork the repository
2. Create a feature branch: `git checkout -b feature/your-feature-name`
3. Make your changes
4. Run tests to ensure your changes don't break existing functionality
5. Commit your changes: `git commit -m "Add some feature"`
6. Push to the branch: `git push origin feature/your-feature-name`
7. Submit a pull request

### Code Style

- Follow the existing code style
- Write clear, concise comments
- Include tests for new functionality

## License

This project is licensed under the MIT License - see the LICENSE file for details.
