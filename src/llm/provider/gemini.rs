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
        model: &str,
        messages: &[Message],
        _options: Option<GenerateOptions>,
    ) -> Result<LLMResponse> {
        // 1. Get access token from gcloud
        let output = std::process::Command::new("gcloud")
            .args(&["auth", "print-access-token"])
            .output()
            .map_err(|e| PraxisError::ProviderError(format!("Failed to execute gcloud: {}", e)))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(PraxisError::ProviderError(format!("gcloud auth failed: {}", stderr)));
        }

        let token = String::from_utf8_lossy(&output.stdout).trim().to_string();

        // 2. Prepare request
        let client = reqwest::Client::new();
        let project_id = std::env::var("GOOGLE_PROJECT_ID")
            .map_err(|_| PraxisError::Config("GOOGLE_PROJECT_ID not set".to_string()))?;
        
        // Map model name to Vertex AI endpoint format
        // e.g. gemini-1.5-pro-preview-0409 -> gemini-1.5-pro-preview-0409
        let endpoint_model = model.replace("google/", ""); 
        let location = "us-central1"; // TODO: Make configurable
        
        let url = format!(
            "https://{}-aiplatform.googleapis.com/v1/projects/{}/locations/{}/publishers/google/models/{}:generateContent",
            location, project_id, location, endpoint_model
        );

        let contents: Vec<serde_json::Value> = messages.iter().map(|m| {
            serde_json::json!({
                "role": if m.role == "user" { "user" } else { "model" },
                "parts": [{ "text": m.content }]
            })
        }).collect();

        let body = serde_json::json!({
            "contents": contents,
            "generation_config": {
                "candidate_count": 1,
            }
        });

        // 3. Send request
        let resp = client.post(&url)
            .header("Authorization", format!("Bearer {}", token))
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await?;

        if !resp.status().is_success() {
            let error_text = resp.text().await.unwrap_or_default();
            return Err(PraxisError::ProviderError(format!("Gemini API error: {}", error_text)));
        }

        let response_json: serde_json::Value = resp.json().await?;
        
        // 4. Parse response
        let content = response_json["candidates"][0]["content"]["parts"][0]["text"]
            .as_str()
            .ok_or_else(|| PraxisError::ProviderError("Failed to parse response content".to_string()))?
            .to_string();

        Ok(LLMResponse {
            content,
            tool_calls: vec![],
            usage: None,
            model: model.to_string(),
        })
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
