//! CLI commands
//!
//! Special commands that can be executed in the REPL.

use crate::agent::Agent;
use crate::core::Result;
use crate::llm::models::{recommended_executors, recommended_orchestrators};

/// Result of parsing a command
pub enum CommandResult {
    /// Continue processing as normal input
    Continue(String),
    /// Command was handled, show output
    Handled(String),
    /// Exit the REPL
    Exit,
    /// Clear history
    Clear,
    /// No output needed
    None,
}

/// Parse and handle special commands
pub async fn handle_command(input: &str, agent: &mut Agent) -> Result<CommandResult> {
    let input = input.trim();
    let parts: Vec<&str> = input.splitn(2, ' ').collect();
    let cmd = parts[0].to_lowercase();
    let args = parts.get(1).map(|s| s.trim()).unwrap_or("");

    match cmd.as_str() {
        "exit" | "quit" | "q" => Ok(CommandResult::Exit),

        "clear" | "reset" => {
            agent.clear_history();
            Ok(CommandResult::Clear)
        }

        "help" | "?" => Ok(CommandResult::Handled(help_text())),

        "models" => {
            let models = agent.list_models().await?;
            let output = format!(
                "Available models:\n{}\n\nCurrent:\n  Orchestrator: {}\n  Executor: {}",
                models
                    .iter()
                    .map(|m| format!("  - {}", m))
                    .collect::<Vec<_>>()
                    .join("\n"),
                agent.config().models.orchestrator,
                agent.config().models.executor
            );
            Ok(CommandResult::Handled(output))
        }

        "set" => handle_set_command(args, agent).await,

        "status" => {
            let status = format!(
                "Praxis Status:\n\
                 ─────────────────────────────\n\
                 Orchestrator: {}\n\
                 Executor:     {}\n\
                 Browser:      {}\n\
                 History:      {} messages\n\
                 Debug:        {}",
                agent.config().models.orchestrator,
                agent.config().models.executor,
                if agent.has_browser() {
                    "enabled"
                } else {
                    "disabled"
                },
                agent.conversation_length(),
                if agent.config().agent.debug {
                    "on"
                } else {
                    "off"
                }
            );
            Ok(CommandResult::Handled(status))
        }

        "debug" => {
            let new_state = !agent.config().agent.debug;
            agent.set_debug(new_state);
            Ok(CommandResult::Handled(format!(
                "Debug mode: {}",
                if new_state { "ON" } else { "OFF" }
            )))
        }

        "recommend" => Ok(CommandResult::Handled(recommend_models())),

        _ => {
            // Not a command, treat as normal input
            if input.starts_with('/') {
                Ok(CommandResult::Handled(format!(
                    "Unknown command: {}. Type 'help' for available commands.",
                    cmd
                )))
            } else {
                Ok(CommandResult::Continue(input.to_string()))
            }
        }
    }
}

/// Handle 'set' subcommands
async fn handle_set_command(args: &str, agent: &mut Agent) -> Result<CommandResult> {
    let parts: Vec<&str> = args.splitn(2, ' ').collect();

    if parts.is_empty() || parts[0].is_empty() {
        return Ok(CommandResult::Handled(
            "Usage: set <orchestrator|executor|debug> <value>\n\
             Examples:\n\
               set orchestrator functiongemma\n\
               set executor gemma3:4b\n\
               set debug on"
                .to_string(),
        ));
    }

    let key = parts[0].to_lowercase();
    let value = parts.get(1).map(|s| s.trim()).unwrap_or("");

    match key.as_str() {
        "orchestrator" | "orch" => {
            if value.is_empty() {
                return Ok(CommandResult::Handled(format!(
                    "Current orchestrator: {}",
                    agent.config().models.orchestrator
                )));
            }
            agent.set_orchestrator_model(value);
            Ok(CommandResult::Handled(format!(
                "Orchestrator model set to: {}",
                value
            )))
        }

        "executor" | "exec" => {
            if value.is_empty() {
                return Ok(CommandResult::Handled(format!(
                    "Current executor: {}",
                    agent.config().models.executor
                )));
            }
            agent.set_executor_model(value);
            Ok(CommandResult::Handled(format!(
                "Executor model set to: {}",
                value
            )))
        }

        "debug" => {
            let enabled = matches!(value.to_lowercase().as_str(), "on" | "true" | "1" | "yes");
            agent.set_debug(enabled);
            Ok(CommandResult::Handled(format!(
                "Debug mode: {}",
                if enabled { "ON" } else { "OFF" }
            )))
        }

        _ => Ok(CommandResult::Handled(format!(
            "Unknown setting: {}. Available: orchestrator, executor, debug",
            key
        ))),
    }
}

/// Generate help text
fn help_text() -> String {
    r#"Praxis Commands:
─────────────────────────────────────────────
  help, ?          Show this help message
  exit, quit, q    Exit Praxis
  clear, reset     Clear conversation history
  status           Show current configuration
  models           List available Ollama models
  debug            Toggle debug mode
  recommend        Show recommended models

  set orchestrator <model>   Set the orchestrator model
  set executor <model>       Set the executor model
  set debug <on|off>         Enable/disable debug output

Keyboard Shortcuts:
  Ctrl+C           Cancel current operation
  Ctrl+D           Exit Praxis

Tips:
  - The orchestrator decides which tools to use
  - The executor generates code and responses
  - Use 'set' to switch between models on the fly
─────────────────────────────────────────────"#
        .to_string()
}

/// Generate model recommendations
fn recommend_models() -> String {
    let mut output = String::from("Recommended Models:\n\n");

    output.push_str("Orchestrators (for function calling):\n");
    for model in recommended_orchestrators() {
        output.push_str(&format!(
            "  {} ({})\n    {}\n",
            model.name, model.parameters, model.description
        ));
    }

    output.push_str("\nExecutors (for code generation):\n");
    for model in recommended_executors() {
        output.push_str(&format!(
            "  {} ({})\n    {}\n",
            model.name, model.parameters, model.description
        ));
    }

    output
}
