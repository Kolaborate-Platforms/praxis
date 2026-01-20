//! Explain code tool
//!
//! Analyzes and explains existing code.

use crate::core::{Result, ToolCall, ToolResult};

/// Tool for explaining code
pub struct ExplainTool;

impl ExplainTool {
    /// Create a new explain tool
    pub fn new() -> Self {
        Self
    }

    /// Build a prompt for the executor model
    pub fn build_prompt(&self, tool_call: &ToolCall) -> String {
        let code = tool_call.get_string("code").unwrap_or_default();
        let focus = tool_call.get_string("focus");

        let mut prompt = format!(
            "Explain the following code in detail:\n\n```\n{}\n```\n\n",
            code
        );

        if let Some(focus_area) = focus {
            prompt.push_str(&format!("Focus specifically on: {}\n\n", focus_area));
        }

        prompt.push_str(
            "Provide a comprehensive explanation including:\n\
             - What the code does at a high level\n\
             - How each major part works\n\
             - Any patterns or techniques used\n\
             - Potential improvements or considerations",
        );

        prompt
    }

    /// Execute the tool
    pub fn execute(&self, tool_call: &ToolCall) -> Result<ToolResult> {
        let prompt = self.build_prompt(tool_call);
        Ok(ToolResult::success("explain_code", prompt))
    }
}

impl Default for ExplainTool {
    fn default() -> Self {
        Self::new()
    }
}
