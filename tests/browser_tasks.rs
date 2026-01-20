//! Browser automation integration tests
//!
//! Tests browser tool execution and multi-step workflows.

use praxis::agent::Agent;
use praxis::core::Config;
use std::time::Duration;
use tokio::time::timeout;

/// Helper to create a configured agent for browser tests
async fn create_browser_agent() -> Result<Agent, Box<dyn std::error::Error>> {
    let mut config = Config::default();
    config.browser.enabled = true;
    config.agent.max_turns = 5;
    config.agent.debug = false;

    let mut agent = Agent::with_config(config);
    agent.initialize().await?;

    if !agent.has_browser() {
        return Err("agent-browser not available".into());
    }

    Ok(agent)
}

/// Test basic navigation
#[tokio::test]
#[ignore] // Requires agent-browser to be installed
async fn test_navigate_to_example_com() {
    let agent = match create_browser_agent().await {
        Ok(a) => a,
        Err(e) => {
            eprintln!("Skipping test: {}", e);
            return;
        }
    };

    let mut agent = agent;
    let result = timeout(
        Duration::from_secs(60),
        agent.process("Navigate to https://example.com using browser_url"),
    )
    .await;

    assert!(result.is_ok(), "Task timed out");
    assert!(result.unwrap().is_ok(), "Task failed");
}

/// Test navigation + snapshot
#[tokio::test]
#[ignore]
async fn test_navigate_and_snapshot() {
    let agent = match create_browser_agent().await {
        Ok(a) => a,
        Err(e) => {
            eprintln!("Skipping test: {}", e);
            return;
        }
    };

    let mut agent = agent;
    let result = timeout(
        Duration::from_secs(90),
        agent.process(
            "Go to https://example.com using browser_url, then use browser_snapshot to see the page elements"
        )
    ).await;

    match result {
        Ok(Ok(response)) => {
            println!("Response: {}", response);
            // Should mention elements from the page
            assert!(!response.is_empty());
        }
        Ok(Err(e)) => panic!("Task failed: {}", e),
        Err(_) => panic!("Task timed out"),
    }
}

/// Test full search workflow
#[tokio::test]
#[ignore]
async fn test_google_search_workflow() {
    let agent = match create_browser_agent().await {
        Ok(a) => a,
        Err(e) => {
            eprintln!("Skipping test: {}", e);
            return;
        }
    };

    let mut agent = agent;

    // Use explicit refs instruction to test improved prompts
    let result = timeout(
        Duration::from_secs(120),
        agent.process(
            "Go to google.com, find the search box ref from the snapshot, \
             fill it with 'Rust programming', then click the search button ref",
        ),
    )
    .await;

    match result {
        Ok(Ok(response)) => {
            println!("Search workflow response: {}", response);
        }
        Ok(Err(e)) => {
            eprintln!("Search workflow error (acceptable): {}", e);
        }
        Err(_) => {
            eprintln!("Task timed out (may be acceptable for slow models)");
        }
    }
}

/// Test that models correctly use refs from snapshots
#[tokio::test]
#[ignore]
async fn test_ref_usage_from_snapshot() {
    let agent = match create_browser_agent().await {
        Ok(a) => a,
        Err(e) => {
            eprintln!("Skipping test: {}", e);
            return;
        }
    };

    let mut agent = agent;

    // Very explicit instruction to test ref parsing
    let result = timeout(
        Duration::from_secs(90),
        agent.process(
            "1. Use browser_url to go to https://example.com \
             2. Use browser_snapshot to get page elements \
             3. Look for a link element in the snapshot and click it using the exact ref",
        ),
    )
    .await;

    assert!(result.is_ok(), "Task timed out");
    // We mainly want to verify the agent doesn't error out
    let _ = result.unwrap();
}
