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
