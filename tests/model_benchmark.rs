//! Model benchmark tests
//!
//! Compares multiple models on identical tasks to measure performance.

use praxis::agent::Agent;
use praxis::core::Config;
use std::time::{Duration, Instant};
use tokio::time::timeout;

/// Result of a single benchmark run
#[derive(Debug)]
pub struct BenchmarkResult {
    pub model: String,
    pub task: String,
    pub success: bool,
    pub turns: usize,
    pub duration: Duration,
    pub tools_called: Vec<String>,
    pub error: Option<String>,
}

/// Benchmark harness for comparing models
pub struct ModelBenchmark {
    pub models: Vec<String>,
    pub timeout_secs: u64,
}

impl Default for ModelBenchmark {
    fn default() -> Self {
        Self {
            models: vec![
                "qwen3-vl:8b".to_string(),
                "qwen3:8b".to_string(),
                "gemma3:4b".to_string(),
            ],
            timeout_secs: 120,
        }
    }
}

impl ModelBenchmark {
    /// Create a new benchmark with specified models
    pub fn new(models: Vec<String>) -> Self {
        Self {
            models,
            timeout_secs: 120,
        }
    }

    /// Run a task against all models and collect results
    pub async fn run_task(&self, task: &str) -> Vec<BenchmarkResult> {
        let mut results = Vec::new();

        for model in &self.models {
            println!("\n=== Testing model: {} ===", model);
            let result = self.run_single(model, task).await;
            results.push(result);
        }

        results
    }

    /// Run a single task against a single model
    async fn run_single(&self, model: &str, task: &str) -> BenchmarkResult {
        let mut config = Config::default();
        config.models.orchestrator = model.to_string();
        config.agent.max_turns = 5; // Limit turns for benchmarking

        let mut agent = Agent::with_config(config)
            .await
            .expect("Failed to create agent");

        // Initialize with timeout
        let init_result = timeout(Duration::from_secs(30), agent.initialize()).await;

        if let Err(_) = init_result {
            return BenchmarkResult {
                model: model.to_string(),
                task: task.to_string(),
                success: false,
                turns: 0,
                duration: Duration::ZERO,
                tools_called: vec![],
                error: Some("Initialization timeout".to_string()),
            };
        }

        if let Err(e) = init_result.unwrap() {
            return BenchmarkResult {
                model: model.to_string(),
                task: task.to_string(),
                success: false,
                turns: 0,
                duration: Duration::ZERO,
                tools_called: vec![],
                error: Some(format!("Init error: {}", e)),
            };
        }

        // Run the task with timeout
        let start = Instant::now();
        let process_result =
            timeout(Duration::from_secs(self.timeout_secs), agent.process(task)).await;

        let duration = start.elapsed();

        match process_result {
            Ok(Ok(_response)) => {
                BenchmarkResult {
                    model: model.to_string(),
                    task: task.to_string(),
                    success: true,
                    turns: 0, // Would need to track this in agent
                    duration,
                    tools_called: vec![], // Would need to track this in agent
                    error: None,
                }
            }
            Ok(Err(e)) => BenchmarkResult {
                model: model.to_string(),
                task: task.to_string(),
                success: false,
                turns: 0,
                duration,
                tools_called: vec![],
                error: Some(e.to_string()),
            },
            Err(_) => BenchmarkResult {
                model: model.to_string(),
                task: task.to_string(),
                success: false,
                turns: 0,
                duration,
                tools_called: vec![],
                error: Some("Task timeout".to_string()),
            },
        }
    }

    /// Print results in a formatted table
    pub fn print_results(results: &[BenchmarkResult]) {
        println!("\n╔══════════════════════════════════════════════════════════════╗");
        println!("║                    BENCHMARK RESULTS                         ║");
        println!("╠══════════════════╦══════════╦══════════╦═════════════════════╣");
        println!("║ Model            ║ Success  ║ Duration ║ Error               ║");
        println!("╠══════════════════╬══════════╬══════════╬═════════════════════╣");

        for result in results {
            let success = if result.success { "✓" } else { "✗" };
            let error = result.error.as_deref().unwrap_or("-");
            let error_short = if error.len() > 18 {
                format!("{}...", &error[..15])
            } else {
                error.to_string()
            };

            println!(
                "║ {:16} ║    {}     ║ {:7.2}s ║ {:19} ║",
                result.model,
                success,
                result.duration.as_secs_f64(),
                error_short
            );
        }

        println!("╚══════════════════╩══════════╩══════════╩═════════════════════╝");
    }
}

/// Simple arithmetic task (no tools needed)
#[tokio::test]
#[ignore] // Run with: cargo test --test model_benchmark -- --ignored
async fn test_simple_question() {
    let benchmark = ModelBenchmark::default();
    let results = benchmark
        .run_task("What is 2+2? Answer with just the number.")
        .await;
    ModelBenchmark::print_results(&results);

    // At least one model should succeed
    assert!(results.iter().any(|r| r.success));
}

/// Browser navigation task
#[tokio::test]
#[ignore]
async fn test_browser_navigation() {
    let benchmark = ModelBenchmark::new(vec!["qwen3-vl:8b".to_string(), "qwen3:8b".to_string()]);

    let results = benchmark.run_task(
        "Use browser_url to navigate to https://example.com and then use browser_snapshot to get the page elements."
    ).await;

    ModelBenchmark::print_results(&results);
}

/// Compare all available models on a coding task
#[tokio::test]
#[ignore]
async fn test_coding_task() {
    let benchmark = ModelBenchmark::default();
    let results = benchmark
        .run_task("Write a simple Python function that calculates the factorial of a number.")
        .await;

    ModelBenchmark::print_results(&results);
    assert!(results.iter().any(|r| r.success));
}
