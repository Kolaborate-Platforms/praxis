//! LLM Provider implementations and factory
//!
//! Submodules implement specific providers (Ollama, Antigravity, Gemini, etc.)

pub mod antigravity;
pub mod gemini;
pub mod kolaborate;
pub mod openrouter;

use std::sync::Arc;

use crate::core::config::{Config, ProviderType};
use crate::core::Result;
use crate::llm::traits::LLMProvider;
use crate::llm::OllamaClient;

use self::antigravity::AntigravityProvider;
use self::gemini::GeminiProvider;
use self::kolaborate::KolaborateProvider;
use self::openrouter::OpenRouterProvider;

/// Create a new LLM provider based on configuration
pub async fn create_provider(config: &Config) -> Result<Arc<dyn LLMProvider>> {
    let provider: Arc<dyn LLMProvider> = match config.provider {
        ProviderType::Ollama => Arc::new(OllamaClient::from_config(config)),
        ProviderType::GoogleAntigravity => {
            // Antigravity might need some async init if we were to do it properly,
            // but for now we'll just construct it.
            Arc::new(AntigravityProvider::from_config(config))
        }
        ProviderType::GoogleGeminiCli => Arc::new(GeminiProvider::from_config(config)),
        ProviderType::OpenRouter => Arc::new(OpenRouterProvider::from_config(config)),
        ProviderType::Kolaborate => Arc::new(KolaborateProvider::from_config(config)),
    };
    Ok(provider)
}
