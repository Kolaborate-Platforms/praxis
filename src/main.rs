//! Praxis - Offline-First AI Coding Agent
//!
//! Main entry point for the CLI application.

use clap::Parser;
use praxis::{Config, Repl};

/// Praxis - Offline-First AI Coding Agent
#[derive(Parser, Debug)]
#[command(name = "praxis")]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Orchestrator model (for function calling)
    #[arg(long, short = 'o')]
    orchestrator: Option<String>,

    /// Executor model (for code generation)
    #[arg(long, short = 'e')]
    executor: Option<String>,

    /// Enable debug output
    #[arg(long, short = 'd')]
    debug: bool,

    /// Disable browser tools
    #[arg(long)]
    no_browser: bool,

    /// Run in headed browser mode (visible window)
    #[arg(long)]
    headed: bool,

    /// Single prompt mode (non-interactive)
    #[arg(long, short = 'p')]
    prompt: Option<String>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    // Build configuration
    let mut config = Config::load();

    // Apply CLI overrides
    if let Some(ref orchestrator) = args.orchestrator {
        config.models.orchestrator = orchestrator.clone();
    }

    if let Some(ref executor) = args.executor {
        config.models.executor = executor.clone();
    }

    if args.debug {
        config.agent.debug = true;
    }

    if args.no_browser {
        config.browser.enabled = false;
    }

    if args.headed {
        config.browser.headed = true;
    }

    // Single prompt mode
    if let Some(prompt) = args.prompt {
        let mut agent = praxis::Agent::with_config(config).await?;
        agent.initialize().await?;

        let response = agent.process(&prompt).await?;
        println!("{}", response);
        return Ok(());
    }

    // Interactive REPL mode
    let mut repl = Repl::with_config(config).await?;
    repl.run().await?;

    Ok(())
}
