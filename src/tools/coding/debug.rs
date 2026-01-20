//! Debug code tool
//!
//! Analyzes code for bugs and provides fixes.

use crate::core::{Result, ToolCall, ToolResult};

/// Tool for debugging code
pub struct DebugTool;

impl DebugTool {
    /// Create a new debug tool
    pub fn new() -> Self {
        Self
    }

    /// Build a prompt for the executor model
    pub fn build_prompt(&self, tool_call: &ToolCall) -> String {
        let code = tool_call.get_string("code").unwrap_or_default();
        let error = tool_call.get_string("error");

        let mut prompt = format!(
            "Debug the following code and identify any issues:\n\n```\n{}\n```\n\n",
            code
        );

        if let Some(error_msg) = error {
            prompt.push_str(&format!("Error message: {}\n\n", error_msg));
        }

        prompt.push_str(
            "Please:\n\
             1. Identify the bug(s) or issue(s)\n\
             2. Explain why the problem occurs\n\
             3. Provide a corrected version of the code\n\
             4. Suggest any additional improvements",
        );

        prompt
    }

    /// Execute the tool
    pub fn execute(&self, tool_call: &ToolCall) -> Result<ToolResult> {
        let prompt = self.build_prompt(tool_call);
        Ok(ToolResult::success("debug_code", prompt))
    }
}

impl Default for DebugTool {
    fn default() -> Self {
        Self::new()
    }
}
