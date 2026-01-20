//! OpenRouter Provider
//!
//! Implementation for OpenRouter API.

use crate::core::{Config, Message, Result, ToolDefinition};
use crate::llm::traits::{GenerateOptions, LLMProvider, LLMResponse, StreamCallback};
use async_trait::async_trait;

pub struct OpenRouterProvider {
    #[allow(dead_code)]
    config: Config,
}

impl OpenRouterProvider {
    pub fn from_config(config: &Config) -> Self {
        Self {
            config: config.clone(),
        }
    }
}

#[async_trait]
impl LLMProvider for OpenRouterProvider {
    async fn chat(
        &self,
        _model: &str,
        _messages: &[Message],
        _options: Option<GenerateOptions>,
    ) -> Result<LLMResponse> {
        todo!("OpenRouter chat not implemented")
    }

    async fn chat_with_tools(
        &self,
        _model: &str,
        _messages: &[Message],
        _tools: &[ToolDefinition],
        _options: Option<GenerateOptions>,
    ) -> Result<LLMResponse> {
        todo!("OpenRouter tools not implemented")
    }

    async fn chat_stream(
        &self,
        _model: &str,
        _messages: &[Message],
        _options: Option<GenerateOptions>,
        _on_token: StreamCallback,
    ) -> Result<LLMResponse> {
        todo!("OpenRouter stream not implemented")
    }

    async fn is_model_available(&self, _model: &str) -> Result<bool> {
        Ok(true)
    }

    async fn list_models(&self) -> Result<Vec<String>> {
        Ok(vec![
            "anthropic/claude-3-opus".to_string(),
            "openai/gpt-4o".to_string(),
        ])
    }

    async fn pull_model(&self, _model: &str) -> Result<()> {
        Ok(())
    }

    fn name(&self) -> &str {
        "openrouter"
    }
}
