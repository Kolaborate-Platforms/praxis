//! Ollama client implementation
//!
//! Async HTTP client for the Ollama API with full tool calling and streaming support.

use async_trait::async_trait;
use futures::StreamExt;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::io::{self, Write};
use std::time::Duration;

use crate::core::{Config, Message, PraxisError, Result, ToolCall, ToolDefinition};
use crate::llm::traits::{GenerateOptions, LLMProvider, LLMResponse, StreamCallback, TokenUsage};

/// Ollama API client
#[derive(Clone)]
pub struct OllamaClient {
    client: Client,
    base_url: String,
    debug: bool,
}

/// Ollama chat request
#[derive(Debug, Serialize)]
struct ChatRequest<'a> {
    model: &'a str,
    messages: Vec<OllamaMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tools: Option<&'a [ToolDefinition]>,
    #[serde(skip_serializing_if = "Option::is_none")]
    options: Option<OllamaOptions>,
    stream: bool,
}

/// Ollama message format
#[derive(Debug, Serialize, Deserialize)]
struct OllamaMessage {
    role: String,
    content: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    tool_calls: Option<Vec<OllamaToolCall>>,
}

/// Ollama tool call format
#[derive(Debug, Clone, Serialize, Deserialize)]
struct OllamaToolCall {
    function: OllamaFunction,
}

/// Ollama function in tool call
#[derive(Debug, Clone, Serialize, Deserialize)]
struct OllamaFunction {
    name: String,
    arguments: serde_json::Value,
}

/// Ollama generation options
#[derive(Debug, Serialize)]
struct OllamaOptions {
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    num_predict: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    stop: Option<Vec<String>>,
}

/// Ollama chat response (non-streaming)
#[derive(Debug, Deserialize)]
struct ChatResponse {
    message: OllamaMessage,
    model: String,
    #[serde(default)]
    prompt_eval_count: Option<u32>,
    #[serde(default)]
    eval_count: Option<u32>,
}

/// Ollama streaming chunk response
#[derive(Debug, Deserialize)]
struct StreamChunkResponse {
    #[serde(default)]
    message: Option<StreamMessage>,
    model: String,
    #[serde(default)]
    done: bool,
    #[serde(default)]
    prompt_eval_count: Option<u32>,
    #[serde(default)]
    eval_count: Option<u32>,
}

/// Message in streaming response
#[derive(Debug, Deserialize)]
struct StreamMessage {
    #[serde(default)]
    content: String,
    #[serde(default)]
    tool_calls: Option<Vec<OllamaToolCall>>,
}

/// Ollama models list response
#[derive(Debug, Deserialize)]
struct ModelsResponse {
    models: Vec<ModelInfo>,
}

/// Model information
#[derive(Debug, Deserialize)]
struct ModelInfo {
    name: String,
}

impl OllamaClient {
    /// Create a new Ollama client with default configuration
    pub fn new() -> Self {
        Self::from_config(&Config::default())
    }

    /// Create a new Ollama client from configuration
    pub fn from_config(config: &Config) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(config.ollama.timeout_secs))
            .build()
            .expect("Failed to create HTTP client");

        Self {
            client,
            base_url: config.ollama_url(),
            debug: config.agent.debug,
        }
    }

    /// Create a client with custom base URL
    pub fn with_base_url(base_url: impl Into<String>) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(120))
            .build()
            .expect("Failed to create HTTP client");

        Self {
            client,
            base_url: base_url.into(),
            debug: false,
        }
    }

    /// Enable or disable debug output
    pub fn set_debug(&mut self, debug: bool) {
        self.debug = debug;
    }

    /// Convert internal Message to Ollama format
    fn to_ollama_message(msg: &Message) -> OllamaMessage {
        OllamaMessage {
            role: msg.role.clone(),
            content: msg.content.clone(),
            tool_calls: msg.tool_calls.as_ref().map(|calls| {
                calls
                    .iter()
                    .map(|tc| OllamaToolCall {
                        function: OllamaFunction {
                            name: tc.name.clone(),
                            arguments: tc.arguments.clone(),
                        },
                    })
                    .collect()
            }),
        }
    }

    /// Convert Ollama response to LLMResponse
    fn to_llm_response(response: ChatResponse) -> LLMResponse {
        let tool_calls = response
            .message
            .tool_calls
            .unwrap_or_default()
            .into_iter()
            .map(|tc| ToolCall {
                name: tc.function.name,
                arguments: tc.function.arguments,
            })
            .collect();

        let usage = match (response.prompt_eval_count, response.eval_count) {
            (Some(prompt), Some(completion)) => Some(TokenUsage {
                prompt_tokens: prompt,
                completion_tokens: completion,
                total_tokens: prompt + completion,
            }),
            _ => None,
        };

        LLMResponse {
            content: response.message.content,
            tool_calls,
            usage,
            model: response.model,
        }
    }

    /// Debug print if enabled
    fn debug_print(&self, label: &str, content: &str) {
        if self.debug {
            if content.len() > 500 {
                eprintln!("DEBUG {}: {}...", label, &content[..500]);
            } else {
                eprintln!("DEBUG {}: {}", label, content);
            }
        }
    }

    /// Internal streaming implementation
    async fn chat_stream_internal(
        &self,
        model: &str,
        messages: &[Message],
        options: Option<GenerateOptions>,
        on_token: Option<&StreamCallback>,
    ) -> Result<LLMResponse> {
        let ollama_messages: Vec<OllamaMessage> =
            messages.iter().map(Self::to_ollama_message).collect();

        let ollama_options = options.as_ref().map(|opts| OllamaOptions {
            temperature: opts.temperature,
            num_predict: opts.max_tokens,
            stop: opts.stop.clone(),
        });

        let request = ChatRequest {
            model,
            messages: ollama_messages,
            tools: None,
            options: ollama_options,
            stream: true,
        };

        let request_json = serde_json::to_string(&request)?;
        self.debug_print("Stream Request", &request_json);

        let response = self
            .client
            .post(format!("{}/api/chat", self.base_url))
            .json(&request)
            .send()
            .await
            .map_err(|e| {
                if e.is_connect() {
                    PraxisError::ollama(format!(
                        "Cannot connect to Ollama at {}. Is it running?",
                        self.base_url
                    ))
                } else {
                    PraxisError::from(e)
                }
            })?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();

            if status.as_u16() == 404 && error_text.contains("not found") {
                return Err(PraxisError::ModelNotFound(model.to_string()));
            }

            return Err(PraxisError::ollama(format!(
                "Ollama API error ({}): {}",
                status, error_text
            )));
        }

        // Process the streaming response
        let mut full_content = String::new();
        let mut final_model = model.to_string();
        let mut prompt_tokens: Option<u32> = None;
        let mut completion_tokens: Option<u32> = None;
        let mut tool_calls: Vec<ToolCall> = Vec::new();

        let mut stream = response.bytes_stream();
        let mut buffer = String::new();

        while let Some(chunk_result) = stream.next().await {
            let chunk =
                chunk_result.map_err(|e| PraxisError::ollama(format!("Stream error: {}", e)))?;
            let chunk_str = String::from_utf8_lossy(&chunk);
            buffer.push_str(&chunk_str);

            // Process complete JSON lines from buffer
            while let Some(newline_pos) = buffer.find('\n') {
                let line = buffer[..newline_pos].trim().to_string();
                buffer = buffer[newline_pos + 1..].to_string();

                if line.is_empty() {
                    continue;
                }

                // Parse the JSON chunk
                match serde_json::from_str::<StreamChunkResponse>(&line) {
                    Ok(chunk_response) => {
                        final_model = chunk_response.model;

                        if let Some(ref msg) = chunk_response.message {
                            if !msg.content.is_empty() {
                                full_content.push_str(&msg.content);

                                // Call the callback if provided
                                if let Some(callback) = on_token {
                                    callback(&msg.content);
                                }

                                // Flush stdout for real-time display
                                let _ = io::stdout().flush();
                            }

                            // Collect tool calls from final message
                            if let Some(ref calls) = msg.tool_calls {
                                for tc in calls {
                                    tool_calls.push(ToolCall {
                                        name: tc.function.name.clone(),
                                        arguments: tc.function.arguments.clone(),
                                    });
                                }
                            }
                        }

                        // Capture token counts from final chunk
                        if chunk_response.done {
                            prompt_tokens = chunk_response.prompt_eval_count;
                            completion_tokens = chunk_response.eval_count;
                        }
                    }
                    Err(e) => {
                        self.debug_print("Parse Error", &format!("{}: {}", e, line));
                    }
                }
            }
        }

        // Process any remaining buffer content
        if !buffer.trim().is_empty() {
            if let Ok(chunk_response) = serde_json::from_str::<StreamChunkResponse>(buffer.trim()) {
                if let Some(ref msg) = chunk_response.message {
                    if !msg.content.is_empty() {
                        full_content.push_str(&msg.content);
                        if let Some(callback) = on_token {
                            callback(&msg.content);
                        }
                    }
                }
            }
        }

        let usage = match (prompt_tokens, completion_tokens) {
            (Some(prompt), Some(completion)) => Some(TokenUsage {
                prompt_tokens: prompt,
                completion_tokens: completion,
                total_tokens: prompt + completion,
            }),
            _ => None,
        };

        Ok(LLMResponse {
            content: full_content,
            tool_calls,
            usage,
            model: final_model,
        })
    }
}

impl Default for OllamaClient {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl LLMProvider for OllamaClient {
    async fn chat(
        &self,
        model: &str,
        messages: &[Message],
        options: Option<GenerateOptions>,
    ) -> Result<LLMResponse> {
        let ollama_messages: Vec<OllamaMessage> =
            messages.iter().map(Self::to_ollama_message).collect();

        let ollama_options = options.map(|opts| OllamaOptions {
            temperature: opts.temperature,
            num_predict: opts.max_tokens,
            stop: opts.stop,
        });

        let request = ChatRequest {
            model,
            messages: ollama_messages,
            tools: None,
            options: ollama_options,
            stream: false,
        };

        let request_json = serde_json::to_string(&request)?;
        self.debug_print("Request", &request_json);

        let response = self
            .client
            .post(format!("{}/api/chat", self.base_url))
            .json(&request)
            .send()
            .await
            .map_err(|e| {
                if e.is_connect() {
                    PraxisError::ollama(format!(
                        "Cannot connect to Ollama at {}. Is it running?",
                        self.base_url
                    ))
                } else {
                    PraxisError::from(e)
                }
            })?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();

            if status.as_u16() == 404 && error_text.contains("not found") {
                return Err(PraxisError::ModelNotFound(model.to_string()));
            }

            return Err(PraxisError::ollama(format!(
                "Ollama API error ({}): {}",
                status, error_text
            )));
        }

        let response_text = response.text().await?;
        self.debug_print("Response", &response_text);

        let chat_response: ChatResponse = serde_json::from_str(&response_text)
            .map_err(|e| PraxisError::ollama(format!("Failed to parse response: {}", e)))?;

        Ok(Self::to_llm_response(chat_response))
    }

    async fn chat_with_tools(
        &self,
        model: &str,
        messages: &[Message],
        tools: &[ToolDefinition],
        options: Option<GenerateOptions>,
    ) -> Result<LLMResponse> {
        let ollama_messages: Vec<OllamaMessage> =
            messages.iter().map(Self::to_ollama_message).collect();

        let ollama_options = options.map(|opts| OllamaOptions {
            temperature: opts.temperature,
            num_predict: opts.max_tokens,
            stop: opts.stop,
        });

        let request = ChatRequest {
            model,
            messages: ollama_messages,
            tools: Some(tools),
            options: ollama_options,
            stream: false, // Tool calling doesn't support streaming well
        };

        let request_json = serde_json::to_string(&request)?;
        self.debug_print("Request (with tools)", &request_json);

        let response = self
            .client
            .post(format!("{}/api/chat", self.base_url))
            .json(&request)
            .send()
            .await
            .map_err(|e| {
                if e.is_connect() {
                    PraxisError::ollama(format!(
                        "Cannot connect to Ollama at {}. Is it running?",
                        self.base_url
                    ))
                } else {
                    PraxisError::from(e)
                }
            })?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();

            if status.as_u16() == 404 && error_text.contains("not found") {
                return Err(PraxisError::ModelNotFound(model.to_string()));
            }

            return Err(PraxisError::ollama(format!(
                "Ollama API error ({}): {}",
                status, error_text
            )));
        }

        let response_text = response.text().await?;
        self.debug_print("Response", &response_text);

        let chat_response: ChatResponse = serde_json::from_str(&response_text)
            .map_err(|e| PraxisError::ollama(format!("Failed to parse response: {}", e)))?;

        Ok(Self::to_llm_response(chat_response))
    }

    async fn chat_stream(
        &self,
        model: &str,
        messages: &[Message],
        options: Option<GenerateOptions>,
        on_token: StreamCallback,
    ) -> Result<LLMResponse> {
        self.chat_stream_internal(model, messages, options, Some(&on_token))
            .await
    }

    async fn is_model_available(&self, model: &str) -> Result<bool> {
        let models = self.list_models().await?;
        Ok(models
            .iter()
            .any(|m| m == model || m.split(':').next() == model.split(':').next()))
    }

    async fn list_models(&self) -> Result<Vec<String>> {
        let response = self
            .client
            .get(format!("{}/api/tags", self.base_url))
            .send()
            .await
            .map_err(|e| {
                if e.is_connect() {
                    PraxisError::ollama(format!(
                        "Cannot connect to Ollama at {}. Is it running?",
                        self.base_url
                    ))
                } else {
                    PraxisError::from(e)
                }
            })?;

        if !response.status().is_success() {
            return Err(PraxisError::ollama("Failed to list models"));
        }

        let models_response: ModelsResponse = response.json().await?;
        Ok(models_response.models.into_iter().map(|m| m.name).collect())
    }

    async fn pull_model(&self, model: &str) -> Result<()> {
        #[derive(Serialize)]
        struct PullRequest<'a> {
            name: &'a str,
        }

        let response = self
            .client
            .post(format!("{}/api/pull", self.base_url))
            .json(&PullRequest { name: model })
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(PraxisError::ollama(format!(
                "Failed to pull model: {}",
                model
            )));
        }

        Ok(())
    }

    fn name(&self) -> &str {
        "ollama"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_creation() {
        let client = OllamaClient::new();
        assert_eq!(client.base_url, "http://localhost:11434");
    }

    #[test]
    fn test_message_conversion() {
        let msg = Message::user("Hello");
        let ollama_msg = OllamaClient::to_ollama_message(&msg);
        assert_eq!(ollama_msg.role, "user");
        assert_eq!(ollama_msg.content, "Hello");
    }
}
