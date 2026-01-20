//! Agent orchestrator
//!
//! Main agent that coordinates between models, tools, and conversation.
//! Implements a ReAct-style reasoning loop (Thought → Action → Observation).

use std::io::{self, Write};
use std::sync::Arc;

use crate::agent::conversation::Conversation;
use crate::agent::loop_state::{AgentLoopState, Observation};
use crate::core::{Config, Message, PraxisError, Result, ToolCall, ToolDefinition};
use crate::llm::{GenerateOptions, LLMProvider, OllamaClient};
use crate::tools::browser::BrowserExecutor;
use crate::tools::ToolRegistry;

/// Main agent that orchestrates LLM and tools
pub struct Agent {
    /// Configuration
    config: Config,
    /// LLM client
    llm: OllamaClient,
    /// Tool registry (wrapped in Arc for parallel execution)
    tools: Arc<ToolRegistry>,
    /// Conversation history
    conversation: Conversation,
    /// Whether browser is available
    browser_available: bool,
}

impl Agent {
    /// Create a new agent with default configuration
    pub fn new() -> Self {
        Self::with_config(Config::load())
    }

    /// Create an agent with custom configuration
    pub fn with_config(config: Config) -> Self {
        let llm = OllamaClient::from_config(&config);

        let tools = if config.browser.enabled {
            ToolRegistry::with_browser(&config.browser.session_name)
        } else {
            ToolRegistry::new()
        };

        let mut conversation = Conversation::new(config.agent.max_history);

        // Set system prompt if configured
        if let Some(ref prompt) = config.agent.system_prompt {
            conversation.set_system_prompt(prompt.clone());
        }

        Self {
            config,
            llm,
            tools: Arc::new(tools),
            conversation,
            browser_available: false, // Will be checked on first use
        }
    }

    /// Initialize the agent (check dependencies, models, etc.)
    pub async fn initialize(&mut self) -> Result<()> {
        // Check if Ollama is reachable
        let models = match self.llm.list_models().await {
            Ok(m) => m,
            Err(_) => {
                return Err(PraxisError::OllamaNotReachable(
                    self.config.ollama_url(),
                    self.config.models.orchestrator.clone(),
                    self.config.models.executor.clone(),
                ));
            }
        };

        if self.config.agent.debug {
            eprintln!("DEBUG: Available models: {:?}", models);
        }

        // Check orchestrator model
        if !self
            .llm
            .is_model_available(&self.config.models.orchestrator)
            .await?
        {
            return Err(PraxisError::ModelNotFound(
                self.config.models.orchestrator.clone(),
            ));
        }

        // Check executor model
        if !self
            .llm
            .is_model_available(&self.config.models.executor)
            .await?
        {
            return Err(PraxisError::ModelNotFound(
                self.config.models.executor.clone(),
            ));
        }

        // Check if agent-browser is available
        if self.config.browser.enabled {
            self.browser_available = BrowserExecutor::is_available().await;
        }

        Ok(())
    }

    /// Process a user message using ReAct reasoning loop
    ///
    /// The loop continues until:
    /// 1. The model produces a response without tool calls (final answer)
    /// 2. Maximum turns are reached
    pub async fn process(&mut self, user_input: &str) -> Result<String> {
        // Add user message to history
        self.conversation.add_user(user_input);

        // Initialize loop state
        let mut state = AgentLoopState::new(self.config.agent.max_turns);

        println!(
            "\n[Agent] Starting reasoning loop (max {} turns)",
            state.max_turns
        );

        // ReAct Loop: Thought → Action → Observation
        while state.should_continue() {
            let turn = state.turn + 1;
            println!("\n[Turn {}/{}] Analyzing...", turn, state.max_turns);

            // Build context with observations from previous turns
            let response = self
                .call_orchestrator_with_context(user_input, &state)
                .await?;

            // Check if the model wants to use tools
            if response.tool_calls.is_empty() {
                // No tool calls = final answer
                if !response.content.is_empty() {
                    state.final_answer = Some(response.content.clone());
                    if self.config.agent.debug {
                        eprintln!("DEBUG: Final answer received on turn {}", turn);
                    }
                } else {
                    // Empty response with no tools - shouldn't happen but handle gracefully
                    state.final_answer =
                        Some("I apologize, but I couldn't generate a response.".to_string());
                }
                break;
            }

            // Execute tools
            println!(
                "[Turn {}] Executing {} tool(s)...",
                turn,
                response.tool_calls.len()
            );

            let observations = self.execute_tools(&response.tool_calls).await?;

            // Print tool results
            for obs in &observations {
                let status = if obs.success { "✓" } else { "✗" };
                println!("  {} {} ", status, obs.tool_name);
            }

            // Add observations to state
            state.add_observations(observations);
            state.next_turn();
        }

        // Handle max turns reached without final answer
        let answer = if let Some(answer) = state.final_answer {
            answer
        } else {
            // Max turns reached - synthesize from observations
            println!("\n[Agent] Max turns reached. Synthesizing response...");
            self.synthesize_from_observations(&state).await?
        };

        // Add to conversation history
        self.conversation.add_assistant(&answer);

        println!(
            "\n[Agent] Complete ({} turns, {} observations)",
            state.turn,
            state.observations.len()
        );

        Ok(answer)
    }

    /// Call the orchestrator model with context from previous observations
    async fn call_orchestrator_with_context(
        &self,
        user_input: &str,
        state: &AgentLoopState,
    ) -> Result<crate::llm::LLMResponse> {
        // Build system prompt with ReAct instructions and ref usage guidance
        let browser_instructions = if self.browser_available {
            r#"
## Browser Tools
- `browser_url`: Navigate to a URL. Returns a COMPACT snapshot.
- `browser_snapshot`: Get interactive elements. Returns elements with [ref=eN] tags.
- `browser_fill`: Type text into an element. Args: {"ref": "e5", "text": "search query"}
- `browser_click`: Click an element. Args: {"ref": "e8"}

## Optimal Browser Workflow:
1. `browser_url`: Navigate to the site.
2. **OBSERVE**: Identify the target element's ref (e.g., `e5`) from the snapshot provided in the observation.
3. **ACT**: Use the EXACT ref (e.g., `e5`) with `browser_fill` or `browser_click`.
4. **REPEAT**: Each action returns an updated snapshot. Always check the LATEST observation before selecting the next ref.

## CRITICAL: Element References
When a snapshot returns: `link "Sign in" [ref=e12]`, use `{"ref": "e12"}`.
The system automatically handles the `@` prefix for you. DO NOT use descriptions or URLs as refs."#
        } else {
            ""
        };

        let system_prompt = format!(
            r#"You are an AI agent that uses tools to accomplish tasks. Follow the ReAct pattern:
1. THINK about what you need to do.
2. ACT by calling appropriate tools.
3. OBSERVE the results and continue or provide final answer.

## Coding Tools
- `write_code`, `explain_code`, `debug_code`
{}

## Rules
- Respond with your final answer ONLY when the task is complete.
- ALWAYS read the latest tool observation carefully before choosing your next action.
- Use EXACT element refs from snapshots for all browser interactions."#,
            browser_instructions
        );

        // Build message with user input and any observations
        let user_content = if state.observations.is_empty() {
            user_input.to_string()
        } else {
            format!("{}\n{}", user_input, state.format_observations())
        };

        let messages = vec![Message::system(system_prompt), Message::user(user_content)];

        // Get appropriate tool definitions
        let mut tool_defs: Vec<ToolDefinition> =
            self.tools.coding_tools().into_iter().cloned().collect();

        if self.browser_available {
            tool_defs.extend(self.tools.browser_tools().into_iter().cloned());
        }

        if self.config.agent.debug {
            eprintln!("DEBUG: Calling orchestrator with {} tools", tool_defs.len());
        }

        self.llm
            .chat_with_tools(
                &self.config.models.orchestrator,
                &messages,
                &tool_defs,
                Some(GenerateOptions {
                    temperature: Some(0.1), // Low temperature for tool selection
                    ..Default::default()
                }),
            )
            .await
    }

    /// Execute tools and collect observations
    ///
    /// Coding/context tools run in parallel for efficiency.
    /// Browser tools run sequentially (required for proper page state).
    async fn execute_tools(&self, tool_calls: &[ToolCall]) -> Result<Vec<Observation>> {
        use tokio::task::JoinSet;

        // Separate browser tools from parallelizable tools
        let (browser_calls, parallel_calls): (Vec<_>, Vec<_>) = tool_calls
            .iter()
            .partition(|call| self.is_browser_tool(&call.name));

        let mut observations = Vec::with_capacity(tool_calls.len());

        // Execute parallelizable tools concurrently
        if !parallel_calls.is_empty() {
            let mut set: JoinSet<(String, std::result::Result<String, String>)> = JoinSet::new();

            for tool_call in parallel_calls {
                let name = tool_call.name.clone();
                let prompt = self.tools.build_coding_prompt(tool_call);

                // Clone what we need for the spawned task
                let llm = self.llm.clone();
                let model = self.config.models.executor.clone();

                set.spawn(async move {
                    let messages = vec![crate::core::Message::user(&prompt)];
                    match llm.chat(&model, &messages, None).await {
                        Ok(resp) => (name, Ok(resp.content)),
                        Err(e) => (name, Err(e.to_string())),
                    }
                });
            }

            // Collect parallel results
            while let Some(result) = set.join_next().await {
                match result {
                    Ok((name, Ok(content))) => {
                        observations.push(Observation::success(&name, content));
                    }
                    Ok((name, Err(e))) => {
                        observations.push(Observation::error(&name, &e));
                    }
                    Err(e) => {
                        observations.push(Observation::error(
                            "parallel_task",
                            format!("Task panic: {}", e),
                        ));
                    }
                }
            }
        }

        // Execute browser tools sequentially (page state dependent)
        for tool_call in browser_calls {
            if self.config.agent.debug {
                eprintln!("DEBUG: Executing browser tool: {}", tool_call.name);
            }

            match self.tools.execute(tool_call).await {
                Ok(result) => {
                    observations.push(Observation::from(result));
                }
                Err(e) => {
                    observations.push(Observation::error(&tool_call.name, e.to_string()));
                }
            }
        }

        Ok(observations)
    }

    /// Check if a tool is a browser tool (requires sequential execution)
    fn is_browser_tool(&self, name: &str) -> bool {
        matches!(
            name,
            "browser_url"
                | "browser_click"
                | "browser_fill"
                | "browser_snapshot"
                | "browser_screenshot"
                | "browser_close"
                | "browser_get_text"
        )
    }

    /// Synthesize a response from observations when max turns is reached
    async fn synthesize_from_observations(&self, state: &AgentLoopState) -> Result<String> {
        let synthesis_prompt = format!(
            "Based on the following tool observations, provide a comprehensive answer:\n\n{}",
            state.format_observations()
        );

        let messages = vec![Message::user(synthesis_prompt)];

        let response = self
            .llm
            .chat(
                &self.config.models.executor,
                &messages,
                Some(GenerateOptions {
                    temperature: Some(0.7),
                    ..Default::default()
                }),
            )
            .await?;

        Ok(response.content)
    }

    /// Call the executor model for code generation (non-streaming)
    async fn call_executor(&self, prompt: &str) -> Result<String> {
        if self.config.streaming.enabled {
            // Use streaming for executor too
            let messages = vec![Message::user(prompt)];

            print!("\n"); // New line before streaming output

            let response = self
                .llm
                .chat_stream(
                    &self.config.models.executor,
                    &messages,
                    Some(GenerateOptions {
                        temperature: Some(0.7),
                        ..Default::default()
                    }),
                    Box::new(|token| {
                        print!("{}", token);
                        let _ = io::stdout().flush();
                    }),
                )
                .await?;

            println!("\n"); // New line after streaming
            Ok(response.content)
        } else {
            let messages = vec![Message::user(prompt)];

            let response = self
                .llm
                .chat(
                    &self.config.models.executor,
                    &messages,
                    Some(GenerateOptions {
                        temperature: Some(0.7),
                        ..Default::default()
                    }),
                )
                .await?;

            Ok(response.content)
        }
    }

    /// Check if a tool is a coding tool (needs executor)
    fn is_coding_tool(&self, name: &str) -> bool {
        matches!(name, "write_code" | "explain_code" | "debug_code")
    }

    /// Clear conversation history
    pub fn clear_history(&mut self) {
        self.conversation.clear();
    }

    /// Get current configuration
    pub fn config(&self) -> &Config {
        &self.config
    }

    /// Get mutable configuration
    pub fn config_mut(&mut self) -> &mut Config {
        &mut self.config
    }

    /// Set the orchestrator model
    pub fn set_orchestrator_model(&mut self, model: impl Into<String>) {
        self.config.models.orchestrator = model.into();
    }

    /// Set the executor model
    pub fn set_executor_model(&mut self, model: impl Into<String>) {
        self.config.models.executor = model.into();
    }

    /// Get conversation length
    pub fn conversation_length(&self) -> usize {
        self.conversation.len()
    }

    /// Check if browser is available
    pub fn has_browser(&self) -> bool {
        self.browser_available
    }

    /// Check if streaming is enabled
    pub fn is_streaming(&self) -> bool {
        self.config.streaming.enabled
    }

    /// Enable or disable streaming
    pub fn set_streaming(&mut self, enabled: bool) {
        self.config.streaming.enabled = enabled;
    }

    /// Enable debug mode
    pub fn set_debug(&mut self, debug: bool) {
        self.config.agent.debug = debug;
    }

    /// List available models
    pub async fn list_models(&self) -> Result<Vec<String>> {
        self.llm.list_models().await
    }

    /// Save current configuration to file
    pub fn save_config(&self) -> Result<std::path::PathBuf> {
        self.config.save_and_get_path()
    }
}

impl Default for Agent {
    fn default() -> Self {
        Self::new()
    }
}
