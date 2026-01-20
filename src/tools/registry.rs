//! Tool registry - manages and dispatches tool calls
//!
//! Central hub for registering tools and routing tool calls to handlers.

use std::collections::HashMap;

use crate::core::{Result, ToolCall, ToolCategory, ToolDefinition, ToolResult};
use crate::tools::browser::BrowserExecutor;
use crate::tools::coding::{DebugTool, ExplainTool, WriteTool};
use crate::tools::context::RecursiveContextTool;

/// Registry of available tools
pub struct ToolRegistry {
    /// Tool definitions indexed by name
    definitions: HashMap<String, ToolDefinition>,
    /// Tool categories
    categories: HashMap<String, ToolCategory>,
    /// Browser executor instance
    browser: Option<BrowserExecutor>,
    /// Coding tools
    write_tool: WriteTool,
    explain_tool: ExplainTool,
    debug_tool: DebugTool,
    /// Context tools
    context_tool: RecursiveContextTool,
}

impl ToolRegistry {
    /// Create a new tool registry with default tools
    pub fn new() -> Self {
        let mut registry = Self {
            definitions: HashMap::new(),
            categories: HashMap::new(),
            browser: None,
            write_tool: WriteTool::new(),
            explain_tool: ExplainTool::new(),
            debug_tool: DebugTool::new(),
            context_tool: RecursiveContextTool::new(),
        };

        // Register coding tools
        registry.register_coding_tools();
        // Register context tools
        registry.register_context_tools();

        registry
    }

    /// Create a registry with browser tools enabled
    pub fn with_browser(session_name: impl Into<String>) -> Self {
        let mut registry = Self::new();
        registry.browser = Some(BrowserExecutor::new(session_name));
        registry.register_browser_tools();
        registry
    }

    /// Register the core coding tools
    fn register_coding_tools(&mut self) {
        // Write code tool
        self.register(
            ToolDefinition::function(
                "write_code",
                "Write or modify code for a specific file or task",
                serde_json::json!({
                    "type": "object",
                    "properties": {
                        "task": {
                            "type": "string",
                            "description": "The coding task to perform"
                        },
                        "language": {
                            "type": "string",
                            "description": "Programming language (rust, python, javascript, etc.)"
                        },
                        "context": {
                            "type": "string",
                            "description": "Additional context or requirements"
                        }
                    },
                    "required": ["task", "language"]
                }),
            ),
            ToolCategory::Coding,
        );

        // Explain code tool
        self.register(
            ToolDefinition::function(
                "explain_code",
                "Explain how code works or analyze existing code",
                serde_json::json!({
                    "type": "object",
                    "properties": {
                        "code": {
                            "type": "string",
                            "description": "The code to explain"
                        },
                        "focus": {
                            "type": "string",
                            "description": "Specific aspect to focus on"
                        }
                    },
                    "required": ["code"]
                }),
            ),
            ToolCategory::Coding,
        );

        // Debug code tool
        self.register(
            ToolDefinition::function(
                "debug_code",
                "Debug code and identify issues",
                serde_json::json!({
                    "type": "object",
                    "properties": {
                        "code": {
                            "type": "string",
                            "description": "The code to debug"
                        },
                        "error": {
                            "type": "string",
                            "description": "Error message if available"
                        }
                    },
                    "required": ["code"]
                }),
            ),
            ToolCategory::Coding,
        );
    }

    /// Register context tools
    fn register_context_tools(&mut self) {
        self.register(
            ToolDefinition::function(
                "analyze_conversation",
                "Recursively analyze past conversation history to answer a query. Use this instead of relying on memory.",
                serde_json::json!({
                    "type": "object",
                    "properties": {
                        "query": {
                            "type": "string",
                            "description": "The question to answer about the conversation history"
                        },
                        "start_index": {
                            "type": "integer",
                            "description": "Start index of messages to analyze (optional, defaults to beginning)"
                        },
                        "end_index": {
                            "type": "integer",
                            "description": "End index of messages to analyze (optional, defaults to current)"
                        }
                    },
                    "required": ["query"]
                }),
            ),
            ToolCategory::Context,
        );
    }

    /// Register browser automation tools
    fn register_browser_tools(&mut self) {
        // Browse URL
        self.register(
            ToolDefinition::function(
                "browser_url",
                "Navigate to a URL and get the page structure for analysis",
                serde_json::json!({
                    "type": "object",
                    "properties": {
                        "url": {
                            "type": "string",
                            "description": "The URL to navigate to"
                        },
                        "wait_for_load": {
                            "type": "boolean",
                            "description": "Wait for network idle before snapshot"
                        }
                    },
                    "required": ["url"]
                }),
            ),
            ToolCategory::Browser,
        );

        // Click element
        self.register(
            ToolDefinition::function(
                "browser_click",
                "Click an element on the page by its ref from snapshot",
                serde_json::json!({
                    "type": "object",
                    "properties": {
                        "ref": {
                            "type": "string",
                            "description": "Element ref from snapshot (e.g., @e1, @e2)"
                        }
                    },
                    "required": ["ref"]
                }),
            ),
            ToolCategory::Browser,
        );

        // Fill input
        self.register(
            ToolDefinition::function(
                "browser_fill",
                "Fill text into an input field by its ref",
                serde_json::json!({
                    "type": "object",
                    "properties": {
                        "ref": {
                            "type": "string",
                            "description": "Element ref from snapshot"
                        },
                        "text": {
                            "type": "string",
                            "description": "Text to enter"
                        }
                    },
                    "required": ["ref", "text"]
                }),
            ),
            ToolCategory::Browser,
        );

        // Get text
        self.register(
            ToolDefinition::function(
                "browser_get_text",
                "Get text content from an element",
                serde_json::json!({
                    "type": "object",
                    "properties": {
                        "ref": {
                            "type": "string",
                            "description": "Element ref from snapshot"
                        }
                    },
                    "required": ["ref"]
                }),
            ),
            ToolCategory::Browser,
        );

        // Take screenshot
        self.register(
            ToolDefinition::function(
                "browser_screenshot",
                "Take a screenshot of the current page",
                serde_json::json!({
                    "type": "object",
                    "properties": {
                        "path": {
                            "type": "string",
                            "description": "File path to save screenshot (optional)"
                        },
                        "full_page": {
                            "type": "boolean",
                            "description": "Capture full page instead of viewport"
                        }
                    }
                }),
            ),
            ToolCategory::Browser,
        );

        // Close browser
        self.register(
            ToolDefinition::function(
                "browser_close",
                "Close the browser session",
                serde_json::json!({
                    "type": "object",
                    "properties": {}
                }),
            ),
            ToolCategory::Browser,
        );

        // Get page snapshot
        self.register(
            ToolDefinition::function(
                "browser_snapshot",
                "Get current page accessibility tree with interactive element refs",
                serde_json::json!({
                    "type": "object",
                    "properties": {
                        "interactive_only": {
                            "type": "boolean",
                            "description": "Only return interactive elements (buttons, links, inputs)"
                        }
                    }
                }),
            ),
            ToolCategory::Browser,
        );
    }

    /// Register a tool definition
    pub fn register(&mut self, definition: ToolDefinition, category: ToolCategory) {
        let name = definition.function.name.clone();
        self.definitions.insert(name.clone(), definition);
        self.categories.insert(name, category);
    }

    /// Get all tool definitions
    pub fn all_definitions(&self) -> Vec<&ToolDefinition> {
        self.definitions.values().collect()
    }

    /// Get tool definitions by category
    pub fn definitions_by_category(&self, category: ToolCategory) -> Vec<&ToolDefinition> {
        self.definitions
            .iter()
            .filter(|(name, _)| self.categories.get(*name) == Some(&category))
            .map(|(_, def)| def)
            .collect()
    }

    /// Get coding tool definitions
    pub fn coding_tools(&self) -> Vec<&ToolDefinition> {
        self.definitions_by_category(ToolCategory::Coding)
    }

    /// Get context tool definitions
    pub fn context_tools(&self) -> Vec<&ToolDefinition> {
        self.definitions_by_category(ToolCategory::Context)
    }

    /// Get browser tool definitions
    pub fn browser_tools(&self) -> Vec<&ToolDefinition> {
        self.definitions_by_category(ToolCategory::Browser)
    }

    /// Check if browser is enabled
    pub fn has_browser(&self) -> bool {
        self.browser.is_some()
    }

    /// Get the browser executor
    pub fn browser_executor(&self) -> Option<&BrowserExecutor> {
        self.browser.as_ref()
    }

    /// Get mutable browser executor
    pub fn browser_executor_mut(&mut self) -> Option<&mut BrowserExecutor> {
        self.browser.as_mut()
    }

    /// Execute a tool call
    pub async fn execute(&self, tool_call: &ToolCall) -> Result<ToolResult> {
        let category = self.categories.get(&tool_call.name);

        match category {
            Some(ToolCategory::Coding) => self.execute_coding_tool(tool_call).await,
            Some(ToolCategory::Browser) => self.execute_browser_tool(tool_call).await,
            _ => Ok(ToolResult::failure(
                &tool_call.name,
                format!("Unknown tool: {}", tool_call.name),
            )),
        }
    }

    /// Execute a coding tool
    async fn execute_coding_tool(&self, tool_call: &ToolCall) -> Result<ToolResult> {
        match tool_call.name.as_str() {
            "write_code" => self.write_tool.execute(tool_call),
            "explain_code" => self.explain_tool.execute(tool_call),
            "debug_code" => self.debug_tool.execute(tool_call),
            _ => Ok(ToolResult::failure(
                &tool_call.name,
                format!("Unknown coding tool: {}", tool_call.name),
            )),
        }
    }

    /// Execute a browser tool
    async fn execute_browser_tool(&self, tool_call: &ToolCall) -> Result<ToolResult> {
        let browser = match &self.browser {
            Some(b) => b,
            None => {
                return Ok(ToolResult::failure(
                    &tool_call.name,
                    "Browser tools are not enabled",
                ))
            }
        };

        match tool_call.name.as_str() {
            "browser_url" => {
                let url = tool_call.get_string("url").unwrap_or_default();
                let wait = tool_call.get_bool("wait_for_load").unwrap_or(true);
                browser.open(&url, wait).await
            }
            "browser_click" => {
                let ref_id = tool_call.get_string("ref").unwrap_or_default();
                browser.click(&ref_id).await
            }
            "browser_fill" => {
                let ref_id = tool_call.get_string("ref").unwrap_or_default();
                let text = tool_call.get_string("text").unwrap_or_default();
                browser.fill(&ref_id, &text).await
            }
            "browser_get_text" => {
                let ref_id = tool_call.get_string("ref").unwrap_or_default();
                browser.get_text(&ref_id).await
            }
            "browser_screenshot" => {
                let path = tool_call.get_string("path");
                let full = tool_call.get_bool("full_page").unwrap_or(false);
                browser.screenshot(path.as_deref(), full).await
            }
            "browser_snapshot" => {
                let interactive = tool_call.get_bool("interactive_only").unwrap_or(true);
                browser.snapshot(interactive).await
            }
            "browser_close" => browser.close().await,
            _ => Ok(ToolResult::failure(
                &tool_call.name,
                format!("Unknown browser tool: {}", tool_call.name),
            )),
        }
    }

    /// Get a prompt for a coding tool (for the executor model)
    pub fn build_coding_prompt(&self, tool_call: &ToolCall) -> String {
        match tool_call.name.as_str() {
            "write_code" => self.write_tool.build_prompt(tool_call),
            "explain_code" => self.explain_tool.build_prompt(tool_call),
            "debug_code" => self.debug_tool.build_prompt(tool_call),
            _ => format!("Execute tool: {}", tool_call.name),
        }
    }

    /// Get the context tool helper
    pub fn context_tool(&self) -> &RecursiveContextTool {
        &self.context_tool
    }
}

impl Default for ToolRegistry {
    fn default() -> Self {
        Self::new()
    }
}
