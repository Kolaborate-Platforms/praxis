//! LLM module - Language Model integrations
//!
//! Provides abstractions for different LLM backends with Ollama as the primary.

pub mod models;
pub mod ollama;
pub mod traits;

pub use models::*;
pub use ollama::OllamaClient;
pub use traits::{
    GenerateOptions, LLMProvider, LLMResponse, StreamCallback, StreamChunk, TokenUsage,
};
