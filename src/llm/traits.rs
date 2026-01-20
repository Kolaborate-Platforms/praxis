//! LLM Provider trait for abstracting different backends
//!
//! Enables swapping between Ollama, OpenAI, Anthropic, etc.

use async_trait::async_trait;
use futures::Stream;
use std::pin::Pin;

use crate::core::{Message, Result, ToolCall, ToolDefinition};

/// Response from an LLM provider
#[derive(Debug, Clone)]
pub struct LLMResponse {
    /// Text content of the response
    pub content: String,
    /// Any tool calls the model wants to make
    pub tool_calls: Vec<ToolCall>,
    /// Token usage information
    pub usage: Option<TokenUsage>,
    /// Model that generated the response
    pub model: String,
}

/// Token usage information
#[derive(Debug, Clone, Default)]
pub struct TokenUsage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
}

/// Options for LLM generation
#[derive(Debug, Clone, Default)]
pub struct GenerateOptions {
    /// Temperature for sampling (0.0 - 2.0)
    pub temperature: Option<f32>,
    /// Maximum tokens to generate
    pub max_tokens: Option<u32>,
    /// Stop sequences
    pub stop: Option<Vec<String>>,
    /// Whether to stream the response
    pub stream: bool,
}

/// A chunk from a streaming response
#[derive(Debug, Clone)]
pub struct StreamChunk {
    /// Token text content
    pub content: String,
    /// Whether this is the final chunk
    pub done: bool,
    /// Tool calls (only in final chunk usually)
    pub tool_calls: Option<Vec<ToolCall>>,
}

impl StreamChunk {
    /// Create a new content chunk
    pub fn content(text: impl Into<String>) -> Self {
        Self {
            content: text.into(),
            done: false,
            tool_calls: None,
        }
    }

    /// Create a final/done chunk
    pub fn done() -> Self {
        Self {
            content: String::new(),
            done: true,
            tool_calls: None,
        }
    }

    /// Create a done chunk with tool calls
    pub fn done_with_tools(tool_calls: Vec<ToolCall>) -> Self {
        Self {
            content: String::new(),
            done: true,
            tool_calls: Some(tool_calls),
        }
    }
}

/// Type alias for a boxed stream of chunks
pub type StreamResponse = Pin<Box<dyn Stream<Item = Result<StreamChunk>> + Send>>;

/// Callback function for streaming tokens
pub type StreamCallback = Box<dyn Fn(&str) + Send + Sync>;

/// Trait for LLM providers
#[async_trait]
pub trait LLMProvider: Send + Sync {
    /// Generate a response from messages
    async fn chat(
        &self,
        model: &str,
        messages: &[Message],
        options: Option<GenerateOptions>,
    ) -> Result<LLMResponse>;

    /// Generate a response with tool definitions
    async fn chat_with_tools(
        &self,
        model: &str,
        messages: &[Message],
        tools: &[ToolDefinition],
        options: Option<GenerateOptions>,
    ) -> Result<LLMResponse>;

    /// Generate a streaming response with a callback for each token
    async fn chat_stream(
        &self,
        model: &str,
        messages: &[Message],
        options: Option<GenerateOptions>,
        on_token: StreamCallback,
    ) -> Result<LLMResponse>;

    /// Check if a model is available
    async fn is_model_available(&self, model: &str) -> Result<bool>;

    /// List available models
    async fn list_models(&self) -> Result<Vec<String>>;

    /// Pull/download a model
    async fn pull_model(&self, model: &str) -> Result<()>;

    /// Get the provider name
    fn name(&self) -> &str;
}
