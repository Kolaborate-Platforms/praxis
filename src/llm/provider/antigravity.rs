//! Google Antigravity Provider
//!
//! Implements OAuth2 loopback flow to authenticate with Google's Antigravity service.
//! Mimics the behavior of the `opencode-antigravity-auth` plugin.

use crate::core::{Config, Message, PraxisError, Result, ToolDefinition};
use crate::llm::traits::{GenerateOptions, LLMProvider, LLMResponse, StreamCallback};
use async_trait::async_trait;
use rand::distr::{Alphanumeric, SampleString};
use std::io::{BufRead, BufReader, Write};
use std::net::TcpListener;
use std::time::{SystemTime, UNIX_EPOCH};
use url::Url;

const AUTH_URL: &str = "https://accounts.google.com/o/oauth2/v2/auth";
const TOKEN_URL: &str = "https://oauth2.googleapis.com/token";

pub struct AntigravityProvider {
    config: Config,
}

impl AntigravityProvider {
    pub fn new() -> Self {
        Self::from_config(&Config::default())
    }

    pub fn from_config(config: &Config) -> Self {
        Self {
            config: config.clone(),
        }
    }

    /// Perform OAuth2 authentication
    pub async fn authenticate(&mut self) -> Result<()> {
        let client_id = std::env::var("ANTIGRAVITY_CLIENT_ID")
            .map_err(|_| PraxisError::auth("ANTIGRAVITY_CLIENT_ID not set"))?;
        let client_secret = std::env::var("ANTIGRAVITY_CLIENT_SECRET")
            .map_err(|_| PraxisError::auth("ANTIGRAVITY_CLIENT_SECRET not set"))?;

        // 1. Setup local listener
        let listener = TcpListener::bind("127.0.0.1:0")
            .map_err(|e| PraxisError::auth(format!("Failed to bind local port: {}", e)))?;
        let port = listener
            .local_addr()
            .map_err(|e| PraxisError::auth(format!("Failed to get local port: {}", e)))?
            .port();
        let redirect_uri = format!("http://127.0.0.1:{}", port);

        // 2. Generate state
        let state = Alphanumeric.sample_string(&mut rand::rng(), 30);

        // 3. Construct URL
        let mut url = Url::parse(AUTH_URL).unwrap();
        url.query_pairs_mut()
            .append_pair("client_id", &client_id)
            .append_pair("redirect_uri", &redirect_uri)
            .append_pair("response_type", "code")
            .append_pair("scope", "https://www.googleapis.com/auth/cloud-platform") // Assuming this scope
            .append_pair("state", &state)
            .append_pair("access_type", "offline")
            .append_pair("prompt", "consent");

        println!("Opening browser to authenticate...");
        if webbrowser::open(url.as_str()).is_err() {
            println!("Please open this URL manually: {}", url);
        }

        // 4. Wait for code
        let (mut stream, _) = listener
            .accept()
            .map_err(|e| PraxisError::auth(format!("Failed to accept connection: {}", e)))?;

        let mut reader = BufReader::new(&stream);
        let mut request_line = String::new();
        reader.read_line(&mut request_line).unwrap();

        let redirect_url = request_line
            .split_whitespace()
            .nth(1)
            .ok_or_else(|| PraxisError::auth("Invalid request"))?;

        let url_parsed = Url::parse(&format!("http://localhost{}", redirect_url))
            .map_err(|_| PraxisError::auth("Failed to parse redirect URL"))?;

        let pairs: std::collections::HashMap<String, String> =
            url_parsed.query_pairs().into_owned().collect();

        if let Some(error) = pairs.get("error") {
            return Err(PraxisError::auth(format!("Auth failed: {}", error)));
        }

        if pairs.get("state") != Some(&state) {
            return Err(PraxisError::auth("Invalid state parameter"));
        }

        let code = pairs
            .get("code")
            .ok_or_else(|| PraxisError::auth("No code parameter"))?;

        // Response to browser
        let response =
            "HTTP/1.1 200 OK\r\n\r\nAuthentication successful! You can close this window.";
        stream.write_all(response.as_bytes()).unwrap();

        // 5. Exchange code for token
        let client = reqwest::Client::new();
        let resp = client
            .post(TOKEN_URL)
            .form(&[
                ("client_id", client_id.as_str()),
                ("client_secret", client_secret.as_str()),
                ("code", code),
                ("grant_type", "authorization_code"),
                ("redirect_uri", &redirect_uri),
            ])
            .send()
            .await
            .map_err(|e| PraxisError::auth(format!("Token exchange failed: {}", e)))?;

        if !resp.status().is_success() {
            let error_text = resp.text().await.unwrap_or_default();
            return Err(PraxisError::auth(format!(
                "Token exchange error: {}",
                error_text
            )));
        }

        let token_data: serde_json::Value = resp
            .json()
            .await
            .map_err(|e| PraxisError::auth(format!("Failed to parse token response: {}", e)))?;

        let access_token = token_data["access_token"]
            .as_str()
            .ok_or_else(|| PraxisError::auth("No access_token"))?
            .to_string();

        let refresh_token = token_data["refresh_token"].as_str().map(|s| s.to_string());
        let expires_in = token_data["expires_in"].as_u64().unwrap_or(3600);

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let expiry = now + expires_in;

        // 6. Update config
        self.config.providers.google_antigravity.access_token = Some(access_token);
        self.config.providers.google_antigravity.refresh_token = refresh_token.or(self
            .config
            .providers
            .google_antigravity
            .refresh_token
            .clone());
        self.config.providers.google_antigravity.token_expiry = Some(expiry);

        self.config.save()?;

        println!("Authentication successful.");
        Ok(())
    }

    async fn get_valid_token(&self) -> Result<String> {
        // TODO: Implement refresh logic
        self.config
            .providers
            .google_antigravity
            .access_token
            .clone()
            .ok_or_else(|| {
                PraxisError::auth(
                    "Not authenticated. Please run with --auth or check configuration.",
                )
            })
    }
}

#[async_trait]
impl LLMProvider for AntigravityProvider {
    async fn chat(
        &self,
        model: &str,
        messages: &[Message],
        _options: Option<GenerateOptions>,
    ) -> Result<LLMResponse> {
        let token = self.get_valid_token().await?;

        let client = reqwest::Client::new();

        // Convert messages to Gemini format
        let contents: Vec<serde_json::Value> = messages
            .iter()
            .map(|m| {
                serde_json::json!({
                    "role": if m.role == "user" { "user" } else { "model" },
                    "parts": [{ "text": m.content }]
                })
            })
            .collect();

        // Used discovered endpoint
        let url = "https://cloudcode-pa.googleapis.com/v1internal:generateContent";

        let body = serde_json::json!({
            "model": model,
            "contents": contents,
            "generation_config": {
                "candidate_count": 1,
            }
        });

        let resp = client
            .post(url)
            .header("Authorization", format!("Bearer {}", token))
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await?;

        if !resp.status().is_success() {
            let error_text = resp.text().await.unwrap_or_default();
            return Err(PraxisError::ProviderError(format!(
                "Antigravity API error: {}",
                error_text
            )));
        }

        let response_json: serde_json::Value = resp.json().await?;

        // Extract content from response
        let content = response_json["candidates"][0]["content"]["parts"][0]["text"]
            .as_str()
            .ok_or_else(|| {
                PraxisError::ProviderError("Failed to parse response content".to_string())
            })?
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
        todo!("Antigravity chat_with_tools not implemented")
    }

    async fn chat_stream(
        &self,
        _model: &str,
        _messages: &[Message],
        _options: Option<GenerateOptions>,
        _on_token: StreamCallback,
    ) -> Result<LLMResponse> {
        todo!("Antigravity chat_stream not implemented")
    }

    async fn is_model_available(&self, _model: &str) -> Result<bool> {
        Ok(true) // Stub
    }

    async fn list_models(&self) -> Result<Vec<String>> {
        Ok(vec![
            "google/antigravity-claude-sonnet-4-5-thinking".to_string(),
            "google/gemini-3-pro".to_string(),
        ])
    }

    async fn pull_model(&self, _model: &str) -> Result<()> {
        Ok(())
    }

    fn name(&self) -> &str {
        "google_antigravity"
    }
}
