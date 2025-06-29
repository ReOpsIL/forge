pub mod context;
pub mod errors;
pub mod protocol;
pub mod server;
pub mod session;
pub mod state;
pub mod tools;
/// Model Context Protocol (MCP) implementation for Forge IDE
///
/// This module provides a comprehensive MCP server implementation that enables
/// bidirectional communication between Forge and Claude Code, replacing the
/// brittle CLI subprocess approach with structured tool-based interactions.
pub mod transport;

// Re-export core types for easier access
pub use self::{server::MCPServer, tools::MCPTool, transport::MCPTransport};

/// MCP Protocol version implemented by this server
pub const MCP_PROTOCOL_VERSION: &str = "2024-11-05";

/// Server information
pub const SERVER_NAME: &str = "Forge MCP Server";
pub const SERVER_VERSION: &str = env!("CARGO_PKG_VERSION");
