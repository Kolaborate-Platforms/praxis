//! Write code tool
//!
//! Generates code based on task description and language.

use crate::core::{Result, ToolCall, ToolResult};

/// Tool for writing code
pub struct WriteTool;

impl WriteTool {
    /// Create a new write tool
    pub fn new() -> Self {
        Self
    }

    /// Build a prompt for the executor model
    pub fn build_prompt(&self, tool_call: &ToolCall) -> String {
        let task = tool_call.get_string("task").unwrap_or_default();
        let language = tool_call
            .get_string("language")
            .unwrap_or_else(|| "rust".to_string());
        let context = tool_call.get_string("context").unwrap_or_default();

        let mut prompt = format!(
            "You are an expert {} developer. Write clean, efficient code for the following task:\n\n\
             Task: {}\n",
            language, task
        );

        if !context.is_empty() {
            prompt.push_str(&format!("\nContext: {}\n", context));
        }

        prompt.push_str(
            "\nProvide well-commented code with best practices. Include:\n\
             - Clear function/variable names\n\
             - Error handling where appropriate\n\
             - Brief inline comments for complex logic\n",
        );

        prompt
    }

    /// Execute the tool (returns prompt for now, actual execution happens via LLM)
    pub fn execute(&self, tool_call: &ToolCall) -> Result<ToolResult> {
        // For coding tools, we don't execute directly - we build prompts
        // The orchestrator will send this to the executor model
        let prompt = self.build_prompt(tool_call);
        Ok(ToolResult::success("write_code", prompt))
    }
}

impl Default for WriteTool {
    fn default() -> Self {
        Self::new()
    }
}
