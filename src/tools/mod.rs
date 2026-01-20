//! Tools module - Tool implementations for the agent
//!
//! Contains coding tools, browser automation, and the tool registry.

pub mod browser;
pub mod coding;
pub mod context;
pub mod registry;

pub use registry::ToolRegistry;
