//! Shared types used across Praxis modules
//!
//! Contains message structures, tool definitions, and common data types.

use serde::{Deserialize, Serialize};

/// A message in a conversation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    /// Role of the message sender (user, assistant, system)
    pub role: String,
    /// Content of the message
    pub content: String,
    /// Optional tool calls made by the assistant
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_calls: Option<Vec<ToolCall>>,
}

impl Message {
    /// Create a new user message
    pub fn user(content: impl Into<String>) -> Self {
        Self {
            role: "user".to_string(),
            content: content.into(),
            tool_calls: None,
        }
    }

    /// Create a new assistant message
    pub fn assistant(content: impl Into<String>) -> Self {
        Self {
            role: "assistant".to_string(),
            content: content.into(),
            tool_calls: None,
        }
    }

    /// Create a new system message
    pub fn system(content: impl Into<String>) -> Self {
        Self {
            role: "system".to_string(),
            content: content.into(),
            tool_calls: None,
        }
    }
}

/// A tool call made by the LLM
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCall {
    /// Name of the tool to invoke
    pub name: String,
    /// JSON arguments for the tool
    pub arguments: serde_json::Value,
}

impl ToolCall {
    /// Create a new tool call
    pub fn new(name: impl Into<String>, arguments: serde_json::Value) -> Self {
        Self {
            name: name.into(),
            arguments,
        }
    }

    /// Get a string argument by key
    pub fn get_string(&self, key: &str) -> Option<String> {
        self.arguments
            .get(key)
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
    }

    /// Get a boolean argument by key
    pub fn get_bool(&self, key: &str) -> Option<bool> {
        self.arguments.get(key).and_then(|v| v.as_bool())
    }
}

/// Definition of a tool that can be called by the LLM
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolDefinition {
    /// Type of tool (always "function" for now)
    #[serde(rename = "type")]
    pub tool_type: String,
    /// Function details
    pub function: FunctionDefinition,
}

/// Function definition within a tool
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionDefinition {
    /// Name of the function
    pub name: String,
    /// Description of what the function does
    pub description: String,
    /// JSON Schema for the parameters
    pub parameters: serde_json::Value,
}

impl ToolDefinition {
    /// Create a new function tool definition
    pub fn function(
        name: impl Into<String>,
        description: impl Into<String>,
        parameters: serde_json::Value,
    ) -> Self {
        Self {
            tool_type: "function".to_string(),
            function: FunctionDefinition {
                name: name.into(),
                description: description.into(),
                parameters,
            },
        }
    }
}

/// Result of executing a tool
#[derive(Debug, Clone)]
pub struct ToolResult {
    /// Name of the tool that was executed
    pub tool_name: String,
    /// Whether the execution was successful
    pub success: bool,
    /// Output from the tool
    pub output: String,
    /// Optional structured data
    pub data: Option<serde_json::Value>,
}

impl ToolResult {
    /// Create a successful result
    pub fn success(tool_name: impl Into<String>, output: impl Into<String>) -> Self {
        Self {
            tool_name: tool_name.into(),
            success: true,
            output: output.into(),
            data: None,
        }
    }

    /// Create a successful result with structured data
    pub fn success_with_data(
        tool_name: impl Into<String>,
        output: impl Into<String>,
        data: serde_json::Value,
    ) -> Self {
        Self {
            tool_name: tool_name.into(),
            success: true,
            output: output.into(),
            data: Some(data),
        }
    }

    /// Create a failed result
    pub fn failure(tool_name: impl Into<String>, error: impl Into<String>) -> Self {
        Self {
            tool_name: tool_name.into(),
            success: false,
            output: error.into(),
            data: None,
        }
    }
}

/// Category of tools
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ToolCategory {
    /// Code writing, explanation, debugging
    Coding,
    /// Web browsing and automation
    Browser,
    /// File system operations
    FileSystem,
    /// System commands
    System,
    /// Context management and recursive analysis
    Context,
}

impl std::fmt::Display for ToolCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ToolCategory::Coding => write!(f, "coding"),
            ToolCategory::Browser => write!(f, "browser"),
            ToolCategory::FileSystem => write!(f, "filesystem"),
            ToolCategory::System => write!(f, "system"),
            ToolCategory::Context => write!(f, "context"),
        }
    }
}
