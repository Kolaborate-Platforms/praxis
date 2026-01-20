//! Context tools - allow the agent to analyze conversation history
//!
//! This implements the "Recursive Language Model" pattern where the agent
//! can query its own history as an external resource.

use crate::core::ToolCall;

/// Tool for recursively analyzing conversation history
#[derive(Debug, Clone, Default)]
pub struct RecursiveContextTool;

impl RecursiveContextTool {
    /// Create a new instance
    pub fn new() -> Self {
        Self
    }

    /// Execute the tool (logic handled in Agent::process due to state requirements)
    /// This is just a placeholder for the registry
    pub fn execute(&self, _tool_call: &ToolCall) -> String {
        "Context analysis is handled by the agent orchestrator directly.".to_string()
    }

    /// Build a prompt for the recursive call
    pub fn build_prompt(&self, query: &str, context_messages: &[crate::core::Message]) -> String {
        let mut prompt = String::new();

        prompt.push_str("Analyze the following conversation segment to answer the query.\n\n");
        prompt.push_str("QUERY: ");
        prompt.push_str(query);
        prompt.push_str("\n\n=== CONVERSATION SEGMENT ===\n");

        for (i, msg) in context_messages.iter().enumerate() {
            prompt.push_str(&format!("\n[Message {} - {}]\n", i, msg.role));
            prompt.push_str(&msg.content);
            prompt.push_str("\n-------------------");
        }

        prompt.push_str("\n\n=== END SEGMENT ===\n\n");
        prompt.push_str("Provide a concise answer to the query based ONLY on the segment above.");

        prompt
    }
}
