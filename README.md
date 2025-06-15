# Forge

A React application with PrimeReact UI components.

## Quick Start

To run the application:

1. Open a terminal
2. Navigate to the project directory
3. Install frontend dependencies (if not already installed):
   ```bash
   npm run install-frontend
   ```
4. Start the development server:
   ```bash
   npm run dev
   ```
5. Open your browser and go to http://localhost:5173

## Features

- Top menu with Blocks, Flow, and Help items
- Built with Vite and React
- Styled with PrimeReact, PrimeIcons, and PrimeFlex
- Interactive block visualization with ReactFlow
- Dynamic block configuration management
- RESTful API for CRUD operations on blocks
- JSON-based configuration storage

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
