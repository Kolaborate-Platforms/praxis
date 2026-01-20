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
    pub async fn new() -> Result<Self> {
        Ok(Self {
            agent: Agent::new().await?,
        })
    }

    /// Create a REPL with custom configuration
    pub async fn with_config(config: Config) -> Result<Self> {
        Ok(Self {
            agent: Agent::with_config(config).await?,
        })
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
                println!("\n\n❌ Initialization Error: {}\n", e);
                return Ok(());
            }
        }

        // Enable session persistence
        let cwd = std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));
        let session_path = cwd.join(".praxis").join("session.json");

        // Notify user about session persistence
        if session_path.exists() {
            println!("📂 Loaded previous session from .praxis/session.json");
        } else {
            println!("💾 Session will be saved to .praxis/session.json");
        }

        if let Err(e) = self.agent.enable_persistence(session_path) {
            eprintln!("⚠️  Warning: Failed to enable session persistence: {}", e);
        }

        // Check for agent-browser if enabled but not found
        if self.agent.config().browser.enabled && !self.agent.has_browser() {
            println!("⚠️  agent-browser not found. Browser automation disabled.");
            println!("   To enable: npm install -g agent-browser && agent-browser install");
            print!("\nContinue without browser tools? [Y/n]: ");
            io::stdout().flush()?;

            let mut choice = String::new();
            io::stdin().read_line(&mut choice)?;
            let choice = choice.trim().to_lowercase();
            if !choice.is_empty() && choice != "y" && choice != "yes" {
                println!("Goodbye!");
                return Ok(());
            }
            println!();
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
        println!("Ollama:     {}", config.ollama_url());
        println!("Models:");
        println!("  Orchestrator: {}", config.models.orchestrator);
        println!("  Executor:     {}", config.models.executor);
        println!();
        println!("Commands: help, clear, models, status, exit");
        println!("─────────────────────────────────────────────────────────────");
    }
}
