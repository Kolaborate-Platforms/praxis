//! Custom error types for Praxis
//!
//! Provides a unified error handling system across all modules.

use thiserror::Error;

/// Main error type for Praxis operations
#[derive(Error, Debug)]
pub enum PraxisError {
    /// Ollama connection or API errors
    #[error("Ollama error: {0}")]
    Ollama(String),

    /// Browser automation errors
    #[error("Browser error: {0}")]
    Browser(String),

    /// Tool execution errors
    #[error("Tool execution error: {0}")]
    ToolExecution(String),

    /// Configuration errors
    #[error("Configuration error: {0}")]
    Config(String),

    /// JSON parsing errors
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    /// HTTP request errors
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),

    /// IO errors
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// Agent-browser not installed
    #[error("agent-browser not found. Install with: npm install -g agent-browser && agent-browser install")]
    AgentBrowserNotFound,

    /// Model not available
    #[error("Model '{0}' not available in Ollama. Run: ollama pull {0}")]
    ModelNotFound(String),

    /// Generic error with context
    #[error("{context}: {source}")]
    WithContext {
        context: String,
        #[source]
        source: Box<dyn std::error::Error + Send + Sync>,
    },

    /// Generic error for other cases
    #[error("{0}")]
    Other(String),
}

/// Convenience Result type for Praxis operations
pub type Result<T> = std::result::Result<T, PraxisError>;

impl PraxisError {
    /// Create an Ollama error
    pub fn ollama(msg: impl Into<String>) -> Self {
        Self::Ollama(msg.into())
    }

    /// Create a browser error
    pub fn browser(msg: impl Into<String>) -> Self {
        Self::Browser(msg.into())
    }

    /// Create a tool execution error
    pub fn tool(msg: impl Into<String>) -> Self {
        Self::ToolExecution(msg.into())
    }

    /// Create a config error
    pub fn config(msg: impl Into<String>) -> Self {
        Self::Config(msg.into())
    }

    /// Wrap an error with additional context
    pub fn with_context<E>(context: impl Into<String>, error: E) -> Self
    where
        E: std::error::Error + Send + Sync + 'static,
    {
        Self::WithContext {
            context: context.into(),
            source: Box::new(error),
        }
    }
}
