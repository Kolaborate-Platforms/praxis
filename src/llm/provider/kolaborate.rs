//! Kolaborate Provider
//!
//! Stub for the future Kolaborate provider.

use crate::core::{Config, Message, Result, ToolDefinition};
use crate::llm::traits::{GenerateOptions, LLMProvider, LLMResponse, StreamCallback};
use async_trait::async_trait;

pub struct KolaborateProvider {
    #[allow(dead_code)]
    config: Config,
}

impl KolaborateProvider {
    pub fn from_config(config: &Config) -> Self {
        Self {
            config: config.clone(),
        }
    }
}

#[async_trait]
impl LLMProvider for KolaborateProvider {
    async fn chat(
        &self,
        _model: &str,
        _messages: &[Message],
        _options: Option<GenerateOptions>,
    ) -> Result<LLMResponse> {
        todo!("Kolaborate provider not implemented")
    }

    async fn chat_with_tools(
        &self,
        _model: &str,
        _messages: &[Message],
        _tools: &[ToolDefinition],
        _options: Option<GenerateOptions>,
    ) -> Result<LLMResponse> {
        todo!("Kolaborate provider not implemented")
    }

    async fn chat_stream(
        &self,
        _model: &str,
        _messages: &[Message],
        _options: Option<GenerateOptions>,
        _on_token: StreamCallback,
    ) -> Result<LLMResponse> {
        todo!("Kolaborate provider not implemented")
    }

    async fn is_model_available(&self, _model: &str) -> Result<bool> {
        Ok(false)
    }

    async fn list_models(&self) -> Result<Vec<String>> {
        Ok(vec![])
    }

    async fn pull_model(&self, _model: &str) -> Result<()> {
        Ok(())
    }

    fn name(&self) -> &str {
        "kolaborate"
    }
}
