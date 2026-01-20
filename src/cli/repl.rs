//! Interactive REPL for Praxis
//!
//! Provides the main user interaction loop.

use std::io::{self, BufRead, Write};

use crate::agent::Agent;
use crate::cli::commands::{handle_command, CommandResult};
use crate::core::{Config, Result};

/// Interactive REPL (Read-Eval-Print Loop)
pub struct Repl {
    agent: Agent,
}

impl Repl {
    /// Create a new REPL with default configuration
    pub fn new() -> Self {
        Self {
            agent: Agent::new(),
        }
    }

    /// Create a REPL with custom configuration
    pub fn with_config(config: Config) -> Self {
        Self {
            agent: Agent::with_config(config),
        }
    }

    /// Run the REPL
    pub async fn run(&mut self) -> Result<()> {
        self.print_banner();

        // Initialize agent
        print!("Initializing...");
        io::stdout().flush()?;

        match self.agent.initialize().await {
            Ok(()) => println!(" Ready!\n"),
            Err(e) => {
                println!("\nInitialization warning: {}\n", e);
            }
        }

        let stdin = io::stdin();
        let mut stdout = io::stdout();

        loop {
            // Print prompt
            print!("You: ");
            stdout.flush()?;

            // Read input
            let mut input = String::new();
            match stdin.lock().read_line(&mut input) {
                Ok(0) => {
                    // EOF (Ctrl+D)
                    println!("\nGoodbye!");
                    break;
                }
                Ok(_) => {}
                Err(e) => {
                    eprintln!("Error reading input: {}", e);
                    continue;
                }
            }

            let input = input.trim();

            if input.is_empty() {
                continue;
            }

            // Handle commands
            match handle_command(input, &mut self.agent).await {
                Ok(CommandResult::Exit) => {
                    println!("\nGoodbye!");
                    break;
                }
                Ok(CommandResult::Clear) => {
                    println!("Conversation cleared.\n");
                    continue;
                }
                Ok(CommandResult::Handled(output)) => {
                    println!("{}\n", output);
                    continue;
                }
                Ok(CommandResult::None) => continue,
                Ok(CommandResult::Continue(input)) => {
                    // Process as normal input
                    match self.agent.process(&input).await {
                        Ok(response) => {
                            println!("\nAssistant:\n{}\n", response);
                        }
                        Err(e) => {
                            eprintln!("\nError: {}\n", e);
                        }
                    }
                }
                Err(e) => {
                    eprintln!("Command error: {}\n", e);
                }
            }
        }

        Ok(())
    }

    /// Print the startup banner
    fn print_banner(&self) {
        let config = self.agent.config();

        println!(
            r#"
╔═══════════════════════════════════════════════════════════╗
║                                                           ║
║   ██████╗ ██████╗  █████╗ ██╗  ██╗██╗███████╗             ║
║   ██╔══██╗██╔══██╗██╔══██╗╚██╗██╔╝██║██╔════╝             ║
║   ██████╔╝██████╔╝███████║ ╚███╔╝ ██║███████╗             ║
║   ██╔═══╝ ██╔══██╗██╔══██║ ██╔██╗ ██║╚════██║             ║
║   ██║     ██║  ██║██║  ██║██╔╝ ██╗██║███████║             ║
║   ╚═╝     ╚═╝  ╚═╝╚═╝  ╚═╝╚═╝  ╚═╝╚═╝╚══════╝             ║
║                                                           ║
║   Offline-First AI Coding Agent                           ║
║                                                           ║
╚═══════════════════════════════════════════════════════════╝
"#
        );
        println!("Models:");
        println!("  Orchestrator: {}", config.models.orchestrator);
        println!("  Executor:     {}", config.models.executor);
        println!();
        println!("Commands: help, clear, models, status, exit");
        println!("─────────────────────────────────────────────────────────────");
    }
}

impl Default for Repl {
    fn default() -> Self {
        Self::new()
    }
}
