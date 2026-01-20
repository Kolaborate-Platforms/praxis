//! Configuration management for Praxis
//!
//! Supports environment variables, config files, and runtime overrides.
//! Models are interchangeable via settings.
//!
//! Config file location: ~/.config/praxis/config.toml

use serde::{Deserialize, Serialize};
use std::env;
use std::fs;
use std::path::PathBuf;

use crate::core::error::{PraxisError, Result};

/// Main configuration for Praxis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Ollama configuration
    pub ollama: OllamaConfig,
    /// Model configuration
    pub models: ModelConfig,
    /// Browser configuration
    pub browser: BrowserConfig,
    /// Agent configuration
    pub agent: AgentConfig,
    /// Streaming configuration
    #[serde(default)]
    pub streaming: StreamingConfig,
}

/// Ollama server configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OllamaConfig {
    /// Host address (default: localhost)
    pub host: String,
    /// Port number (default: 11434)
    pub port: u16,
    /// Request timeout in seconds
    pub timeout_secs: u64,
}

/// Model configuration - interchangeable models
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelConfig {
    /// Model used for function calling / orchestration
    /// Default: functiongemma
    pub orchestrator: String,
    /// Model used for code generation and responses
    /// Default: gemma3:4b
    pub executor: String,
    /// Alternative models that can be switched to
    #[serde(default)]
    pub alternatives: ModelAlternatives,
}

/// Alternative model configurations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelAlternatives {
    /// Alternative orchestrator models
    pub orchestrators: Vec<String>,
    /// Alternative executor models
    pub executors: Vec<String>,
}

/// Browser automation configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrowserConfig {
    /// Whether browser tools are enabled
    pub enabled: bool,
    /// Session name for agent-browser
    pub session_name: String,
    /// Whether to run in headed mode (visible browser)
    pub headed: bool,
    /// Default timeout for browser operations in ms
    pub timeout_ms: u64,
}

/// Agent behavior configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentConfig {
    /// Maximum conversation history length (storage limit)
    /// Default: 1000
    pub max_history: usize,
    /// Number of recent messages to include in context window
    /// Default: 20
    pub context_window: usize,
    /// Maximum reasoning loop turns before stopping
    /// Default: 10
    pub max_turns: usize,
    /// Whether to show debug output
    pub debug: bool,
    /// System prompt prefix
    pub system_prompt: Option<String>,
}

impl Default for AgentConfig {
    fn default() -> Self {
        Self {
            max_history: 1000,
            context_window: 20,
            max_turns: 10,
            debug: env::var("PRAXIS_DEBUG")
                .map(|v| v == "true" || v == "1")
                .unwrap_or(false),
            system_prompt: None,
        }
    }
}

/// Streaming configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamingConfig {
    /// Whether to stream responses in real-time
    pub enabled: bool,
    /// Print tokens as they arrive (vs buffering)
    pub print_tokens: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            ollama: OllamaConfig::default(),
            models: ModelConfig::default(),
            browser: BrowserConfig::default(),
            agent: AgentConfig::default(),
            streaming: StreamingConfig::default(),
        }
    }
}

impl Default for OllamaConfig {
    fn default() -> Self {
        Self {
            host: env::var("OLLAMA_HOST").unwrap_or_else(|_| "localhost".to_string()),
            port: env::var("OLLAMA_PORT")
                .ok()
                .and_then(|p| p.parse().ok())
                .unwrap_or(11434),
            timeout_secs: 120,
        }
    }
}

impl Default for ModelConfig {
    fn default() -> Self {
        Self {
            orchestrator: env::var("PRAXIS_ORCHESTRATOR_MODEL")
                .unwrap_or_else(|_| "qwen3-vl:8b".to_string()),
            executor: env::var("PRAXIS_EXECUTOR_MODEL").unwrap_or_else(|_| "qwen3:8b".to_string()),
            alternatives: ModelAlternatives::default(),
        }
    }
}

impl Default for ModelAlternatives {
    fn default() -> Self {
        Self {
            orchestrators: vec![
                "functiongemma".to_string(),
                "qwen2.5-coder:7b".to_string(),
                "mistral:7b".to_string(),
            ],
            executors: vec![
                "gemma3:4b".to_string(),
                "gemma3:12b".to_string(),
                "qwen2.5-coder:7b".to_string(),
                "codellama:7b".to_string(),
                "deepseek-coder:6.7b".to_string(),
            ],
        }
    }
}

impl Default for BrowserConfig {
    fn default() -> Self {
        Self {
            enabled: env::var("PRAXIS_BROWSER_ENABLED")
                .map(|v| v == "true" || v == "1")
                .unwrap_or(true),
            session_name: env::var("PRAXIS_BROWSER_SESSION")
                .unwrap_or_else(|_| "praxis".to_string()),
            headed: env::var("PRAXIS_BROWSER_HEADED")
                .map(|v| v == "true" || v == "1")
                .unwrap_or(false),
            timeout_ms: 30000,
        }
    }
}

impl Default for StreamingConfig {
    fn default() -> Self {
        Self {
            enabled: env::var("PRAXIS_STREAMING")
                .map(|v| v == "true" || v == "1")
                .unwrap_or(true), // Streaming enabled by default
            print_tokens: true,
        }
    }
}

impl Config {
    /// Get the config directory path
    pub fn config_dir() -> PathBuf {
        dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("praxis")
    }

    /// Get the config file path
    pub fn config_file() -> PathBuf {
        Self::config_dir().join("config.toml")
    }

    /// Load configuration from file, environment, and defaults
    /// Priority: CLI args > env vars > config file > defaults
    pub fn load() -> Self {
        // Try to load .env file if it exists
        let _ = dotenvy::dotenv();

        // Try to load from config file
        if let Ok(config) = Self::load_from_file() {
            return config;
        }

        // Fall back to defaults (which respect env vars)
        Self::default()
    }

    /// Load configuration from file only
    pub fn load_from_file() -> Result<Self> {
        let config_path = Self::config_file();

        if !config_path.exists() {
            return Err(PraxisError::config("Config file not found"));
        }

        let content = fs::read_to_string(&config_path)
            .map_err(|e| PraxisError::config(format!("Failed to read config: {}", e)))?;

        let config: Config = toml::from_str(&content)
            .map_err(|e| PraxisError::config(format!("Failed to parse config: {}", e)))?;

        Ok(config)
    }

    /// Save configuration to file
    pub fn save(&self) -> Result<()> {
        let config_dir = Self::config_dir();
        let config_path = Self::config_file();

        // Create config directory if it doesn't exist
        if !config_dir.exists() {
            fs::create_dir_all(&config_dir)
                .map_err(|e| PraxisError::config(format!("Failed to create config dir: {}", e)))?;
        }

        // Serialize to TOML
        let content = toml::to_string_pretty(self)
            .map_err(|e| PraxisError::config(format!("Failed to serialize config: {}", e)))?;

        // Write to file
        fs::write(&config_path, content)
            .map_err(|e| PraxisError::config(format!("Failed to write config: {}", e)))?;

        Ok(())
    }

    /// Save configuration and return the path
    pub fn save_and_get_path(&self) -> Result<PathBuf> {
        self.save()?;
        Ok(Self::config_file())
    }

    /// Check if a config file exists
    pub fn config_exists() -> bool {
        Self::config_file().exists()
    }

    /// Delete the config file
    pub fn delete_config() -> Result<()> {
        let config_path = Self::config_file();
        if config_path.exists() {
            fs::remove_file(&config_path)
                .map_err(|e| PraxisError::config(format!("Failed to delete config: {}", e)))?;
        }
        Ok(())
    }

    /// Get the full Ollama API URL
    pub fn ollama_url(&self) -> String {
        format!("http://{}:{}", self.ollama.host, self.ollama.port)
    }

    /// Update the orchestrator model
    pub fn set_orchestrator(&mut self, model: impl Into<String>) {
        self.models.orchestrator = model.into();
    }

    /// Update the executor model
    pub fn set_executor(&mut self, model: impl Into<String>) {
        self.models.executor = model.into();
    }

    /// Check if a model is in the known alternatives
    pub fn is_known_orchestrator(&self, model: &str) -> bool {
        self.models
            .alternatives
            .orchestrators
            .iter()
            .any(|m| m == model)
            || model == self.models.orchestrator
    }

    /// Check if a model is in the known alternatives
    pub fn is_known_executor(&self, model: &str) -> bool {
        self.models
            .alternatives
            .executors
            .iter()
            .any(|m| m == model)
            || model == self.models.executor
    }

    /// Set streaming enabled/disabled
    pub fn set_streaming(&mut self, enabled: bool) {
        self.streaming.enabled = enabled;
    }

    /// Generate a default config file content for display
    pub fn default_config_toml() -> String {
        let config = Config::default();
        toml::to_string_pretty(&config)
            .unwrap_or_else(|_| String::from("# Error generating config"))
    }
}

impl OllamaConfig {
    /// Get the socket address
    pub fn addr(&self) -> String {
        format!("{}:{}", self.host, self.port)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert_eq!(config.models.orchestrator, "qwen3-vl:8b");
        assert_eq!(config.models.executor, "qwen3:8b");
        assert_eq!(config.ollama.port, 11434);
        assert!(config.streaming.enabled);
        assert_eq!(config.agent.max_turns, 10);
    }

    #[test]
    fn test_ollama_url() {
        let config = Config::default();
        assert_eq!(config.ollama_url(), "http://localhost:11434");
    }

    #[test]
    fn test_config_serialization() {
        let config = Config::default();
        let toml_str = toml::to_string_pretty(&config).unwrap();
        assert!(toml_str.contains("orchestrator"));
        assert!(toml_str.contains("executor"));
    }

    #[test]
    fn test_config_dir() {
        let dir = Config::config_dir();
        assert!(dir.to_string_lossy().contains("praxis"));
    }
}
