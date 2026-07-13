//! mcp-hello-rust-server — a minimal MCP server for demos and scaffolding.
//!
//! Exposes two tools: `server_info` (a health/status check) and `greet` (a
//! friendly greeting in one of a handful of languages, defaulting to English).
//! The library crate holds the [`greetings`] data/resolver and the [`server`]
//! (the [`server::Hello`] MCP server); the binary (`src/main.rs`) is a thin
//! wrapper that wires up the transport.

pub mod greetings;
pub mod server;
