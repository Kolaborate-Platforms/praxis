//! Google Gemini CLI Provider
//!
//! Wraps the official `@google/gemini-cli` tool.

use crate::core::{Config, Message, Result, ToolDefinition};
use crate::llm::traits::{GenerateOptions, LLMProvider, LLMResponse, StreamCallback};
use async_trait::async_trait;

pub struct GeminiProvider {
    #[allow(dead_code)]
    config: Config,
}

impl GeminiProvider {
    pub fn from_config(config: &Config) -> Self {
        Self {
            config: config.clone(),
        }
    }
}

#[async_trait]
impl LLMProvider for GeminiProvider {
    async fn chat(
        &self,
        _model: &str,
        _messages: &[Message],
        _options: Option<GenerateOptions>,
    ) -> Result<LLMResponse> {
        todo!("Gemini CLI chat not implemented")
    }

    async fn chat_with_tools(
        &self,
        _model: &str,
        _messages: &[Message],
        _tools: &[ToolDefinition],
        _options: Option<GenerateOptions>,
    ) -> Result<LLMResponse> {
        todo!("Gemini CLI tools not implemented")
    }

    async fn chat_stream(
        &self,
        _model: &str,
        _messages: &[Message],
        _options: Option<GenerateOptions>,
        _on_token: StreamCallback,
    ) -> Result<LLMResponse> {
        todo!("Gemini CLI stream not implemented")
    }

    async fn is_model_available(&self, _model: &str) -> Result<bool> {
        Ok(true)
    }

    async fn list_models(&self) -> Result<Vec<String>> {
        Ok(vec!["gemini-2.0-flash".to_string()])
    }

    async fn pull_model(&self, _model: &str) -> Result<()> {
        Ok(())
    }

    fn name(&self) -> &str {
        "google_gemini_cli"
    }
}
