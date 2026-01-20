//! Sub-agent support
//!
//! Lightweight agents that can be spawned for delegated tasks.

use std::sync::Arc;

use crate::core::{Config, Message, Result, ToolDefinition};
use crate::llm::{GenerateOptions, LLMProvider, OllamaClient};
use crate::tools::ToolRegistry;

/// A lightweight sub-agent for delegated tasks
#[derive(Clone)]
pub struct SubAgent {
    /// Name of this sub-agent
    name: String,
    /// System prompt defining the sub-agent's role
    system_prompt: String,
    /// Which tool names this sub-agent can use (empty = all)
    allowed_tools: Vec<String>,
    /// LLM client
    llm: OllamaClient,
    /// Model to use
    model: String,
    /// Tool registry
    tools: Arc<ToolRegistry>,
    /// Maximum turns for this sub-agent
    max_turns: usize,
}

/// Builder for creating SubAgents
pub struct SubAgentBuilder {
    name: String,
    system_prompt: Option<String>,
    allowed_tools: Vec<String>,
    llm: Option<OllamaClient>,
    model: Option<String>,
    tools: Option<Arc<ToolRegistry>>,
    max_turns: usize,
}

impl SubAgentBuilder {
    /// Create a new builder with the given name
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            system_prompt: None,
            allowed_tools: Vec::new(),
            llm: None,
            model: None,
            tools: None,
            max_turns: 5,
        }
    }

    /// Set the system prompt
    pub fn system_prompt(mut self, prompt: impl Into<String>) -> Self {
        self.system_prompt = Some(prompt.into());
        self
    }

    /// Set allowed tools (empty = all tools allowed)
    pub fn allowed_tools(mut self, tools: Vec<String>) -> Self {
        self.allowed_tools = tools;
        self
    }

    /// Set the LLM client
    pub fn llm(mut self, llm: OllamaClient) -> Self {
        self.llm = Some(llm);
        self
    }

    /// Set the model to use
    pub fn model(mut self, model: impl Into<String>) -> Self {
        self.model = Some(model.into());
        self
    }

    /// Set the tool registry
    pub fn tools(mut self, tools: Arc<ToolRegistry>) -> Self {
        self.tools = Some(tools);
        self
    }

    /// Set maximum turns
    pub fn max_turns(mut self, max: usize) -> Self {
        self.max_turns = max;
        self
    }

    /// Build the SubAgent
    pub fn build(self) -> Result<SubAgent> {
        let config = Config::default();

        Ok(SubAgent {
            name: self.name.clone(),
            system_prompt: self.system_prompt.unwrap_or_else(|| {
                format!(
                    "You are a helpful sub-agent named '{}'. Complete the task you are given.",
                    self.name
                )
            }),
            allowed_tools: self.allowed_tools,
            llm: self
                .llm
                .unwrap_or_else(|| OllamaClient::from_config(&config)),
            model: self.model.unwrap_or_else(|| config.models.executor.clone()),
            tools: self.tools.unwrap_or_else(|| Arc::new(ToolRegistry::new())),
            max_turns: self.max_turns,
        })
    }
}

impl SubAgent {
    /// Create a new sub-agent with default settings
    pub fn new(name: impl Into<String>) -> Self {
        SubAgentBuilder::new(name)
            .build()
            .expect("Failed to build sub-agent")
    }

    /// Create a builder for more control
    pub fn builder(name: impl Into<String>) -> SubAgentBuilder {
        SubAgentBuilder::new(name)
    }

    /// Get the name of this sub-agent
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Run the sub-agent on a task
    pub async fn run(&self, task: &str) -> Result<String> {
        let messages = vec![Message::system(&self.system_prompt), Message::user(task)];

        // Get tool definitions if we have any
        let tool_defs: Vec<ToolDefinition> = if self.allowed_tools.is_empty() {
            // Use all coding tools by default
            self.tools.coding_tools().into_iter().cloned().collect()
        } else {
            // Filter to allowed tools
            self.tools
                .coding_tools()
                .into_iter()
                .filter(|t| self.allowed_tools.contains(&t.function.name))
                .cloned()
                .collect()
        };

        if tool_defs.is_empty() {
            // No tools - just get a response
            let response = self
                .llm
                .chat(
                    &self.model,
                    &messages,
                    Some(GenerateOptions {
                        temperature: Some(0.7),
                        ..Default::default()
                    }),
                )
                .await?;

            Ok(response.content)
        } else {
            // With tools - do a tool-calling loop (simplified)
            let response = self
                .llm
                .chat_with_tools(
                    &self.model,
                    &messages,
                    &tool_defs,
                    Some(GenerateOptions {
                        temperature: Some(0.3),
                        ..Default::default()
                    }),
                )
                .await?;

            // For now, just return the content (full loop would execute tools)
            Ok(response.content)
        }
    }

    /// Spawn this sub-agent as a background task
    pub fn spawn(self, task: String) -> tokio::task::JoinHandle<Result<String>> {
        tokio::spawn(async move { self.run(&task).await })
    }
}

/// Manager for coordinating multiple sub-agents
pub struct SubAgentManager {
    agents: Vec<SubAgent>,
}

impl SubAgentManager {
    /// Create a new manager
    pub fn new() -> Self {
        Self { agents: Vec::new() }
    }

    /// Add a sub-agent
    pub fn add_agent(&mut self, agent: SubAgent) {
        self.agents.push(agent);
    }

    /// Run all agents in parallel on the same task
    pub async fn run_all(&self, task: &str) -> Vec<Result<String>> {
        use tokio::task::JoinSet;

        let mut set = JoinSet::new();

        for agent in &self.agents {
            let agent = agent.clone();
            let task = task.to_string();
            set.spawn(async move { agent.run(&task).await });
        }

        let mut results = Vec::new();
        while let Some(result) = set.join_next().await {
            match result {
                Ok(r) => results.push(r),
                Err(e) => results.push(Err(crate::core::PraxisError::Other(format!(
                    "Task panicked: {}",
                    e
                )))),
            }
        }

        results
    }

    /// Delegate task to first available agent matching a capability
    pub fn get_agent(&self, name: &str) -> Option<&SubAgent> {
        self.agents.iter().find(|a| a.name() == name)
    }
}

impl Default for SubAgentManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_subagent_builder() {
        let agent = SubAgent::builder("test_agent")
            .system_prompt("You are a test agent")
            .max_turns(3)
            .build()
            .unwrap();

        assert_eq!(agent.name(), "test_agent");
        assert_eq!(agent.max_turns, 3);
    }

    #[test]
    fn test_subagent_manager() {
        let mut manager = SubAgentManager::new();
        manager.add_agent(SubAgent::new("agent1"));
        manager.add_agent(SubAgent::new("agent2"));

        assert!(manager.get_agent("agent1").is_some());
        assert!(manager.get_agent("agent3").is_none());
    }
}
