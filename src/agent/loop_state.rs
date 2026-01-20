//! Agent loop state management
//!
//! Tracks the state of the ReAct reasoning loop including observations from tool executions.

use serde::{Deserialize, Serialize};

/// State of the agent reasoning loop
#[derive(Debug, Clone)]
pub struct AgentLoopState {
    /// Current turn number (0-indexed)
    pub turn: usize,
    /// Maximum allowed turns
    pub max_turns: usize,
    /// Observations collected from tool executions
    pub observations: Vec<Observation>,
    /// Final answer if the agent has completed reasoning
    pub final_answer: Option<String>,
}

impl AgentLoopState {
    /// Create a new loop state with the given max turns
    pub fn new(max_turns: usize) -> Self {
        Self {
            turn: 0,
            max_turns,
            observations: Vec::new(),
            final_answer: None,
        }
    }

    /// Check if the loop should continue
    pub fn should_continue(&self) -> bool {
        self.turn < self.max_turns && self.final_answer.is_none()
    }

    /// Format observations for inclusion in the next prompt
    pub fn format_observations(&self) -> String {
        if self.observations.is_empty() {
            return String::new();
        }

        let mut output = String::from("\n\n## Tool Observations:\n");
        for (i, obs) in self.observations.iter().enumerate() {
            output.push_str(&format!(
                "\n### Observation {} ({})\n{}\n",
                i + 1,
                obs.tool_name,
                obs.output
            ));
        }
        output
    }

    /// Add observations from a batch of tool executions
    pub fn add_observations(&mut self, observations: Vec<Observation>) {
        self.observations.extend(observations);
    }

    /// Increment the turn counter
    pub fn next_turn(&mut self) {
        self.turn += 1;
    }
}

/// An observation from a tool execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Observation {
    /// Name of the tool that produced this observation
    pub tool_name: String,
    /// Whether the tool execution was successful
    pub success: bool,
    /// Human-readable output from the tool
    pub output: String,
    /// Optional structured data from the tool
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<serde_json::Value>,
}

impl Observation {
    /// Create a successful observation
    pub fn success(tool_name: impl Into<String>, output: impl Into<String>) -> Self {
        Self {
            tool_name: tool_name.into(),
            success: true,
            output: output.into(),
            data: None,
        }
    }

    /// Create an error observation
    pub fn error(tool_name: impl Into<String>, error: impl Into<String>) -> Self {
        Self {
            tool_name: tool_name.into(),
            success: false,
            output: error.into(),
            data: None,
        }
    }

    /// Create an observation with structured data
    pub fn with_data(
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
}

impl From<crate::core::ToolResult> for Observation {
    fn from(result: crate::core::ToolResult) -> Self {
        Self {
            tool_name: result.tool_name,
            success: result.success,
            output: result.output,
            data: result.data,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_loop_state_new() {
        let state = AgentLoopState::new(10);
        assert_eq!(state.turn, 0);
        assert_eq!(state.max_turns, 10);
        assert!(state.observations.is_empty());
        assert!(state.final_answer.is_none());
    }

    #[test]
    fn test_should_continue() {
        let mut state = AgentLoopState::new(2);
        assert!(state.should_continue());

        state.next_turn();
        assert!(state.should_continue());

        state.next_turn();
        assert!(!state.should_continue()); // Reached max turns
    }

    #[test]
    fn test_format_observations() {
        let mut state = AgentLoopState::new(10);
        state.add_observations(vec![
            Observation::success("browser_url", "Navigated to google.com"),
            Observation::success("browser_snapshot", "Found 22 elements"),
        ]);

        let formatted = state.format_observations();
        assert!(formatted.contains("browser_url"));
        assert!(formatted.contains("browser_snapshot"));
    }
}
