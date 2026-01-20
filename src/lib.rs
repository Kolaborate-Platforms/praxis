//! Praxis - Offline-First AI Coding Agent
//!
//! A Rust-based coding assistant that uses Ollama for local LLM inference
//! and optionally integrates with agent-browser for web automation.
//!
//! # Architecture
//!
//! - **Core**: Shared types, configuration, and error handling
//! - **LLM**: LLM provider abstraction with Ollama implementation
//! - **Tools**: Tool registry with coding and browser tools
//! - **Agent**: Orchestration logic and conversation management
//! - **CLI**: Command-line interface and REPL
//!
//! # Usage
//!
//! ```rust,no_run
//! use praxis::agent::Agent;
//!
//! #[tokio::main]
//! async fn main() {
//!     let mut agent = Agent::new();
//!     agent.initialize().await.unwrap();
//!     
//!     let response = agent.process("Write a hello world in Rust").await.unwrap();
//!     println!("{}", response);
//! }
//! ```

pub mod agent;
pub mod cli;
pub mod core;
pub mod llm;
pub mod tools;

// Re-export commonly used items
pub use agent::Agent;
pub use cli::Repl;
pub use core::{Config, PraxisError, Result};
