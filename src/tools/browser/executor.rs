//! Browser executor - wraps agent-browser CLI
//!
//! Provides async interface to agent-browser commands.

use std::process::Stdio;
use tokio::process::Command;

use crate::core::{PraxisError, Result, ToolResult};
use crate::tools::browser::snapshot::Snapshot;

/// Executor for browser automation via agent-browser CLI
pub struct BrowserExecutor {
    /// Session name for isolation
    session_name: String,
    /// Whether to run in headed mode
    headed: bool,
}

impl BrowserExecutor {
    /// Create a new browser executor
    pub fn new(session_name: impl Into<String>) -> Self {
        Self {
            session_name: session_name.into(),
            headed: false,
        }
    }

    /// Set headed mode
    pub fn set_headed(&mut self, headed: bool) {
        self.headed = headed;
    }

    /// Check if agent-browser is installed
    pub async fn is_available() -> bool {
        Command::new("agent-browser")
            .arg("--version")
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .await
            .map(|s| s.success())
            .unwrap_or(false)
    }

    /// Run an agent-browser command
    async fn run_command(&self, args: &[&str]) -> Result<String> {
        let mut cmd = Command::new("agent-browser");
        cmd.args(["--session", &self.session_name]);

        if self.headed {
            cmd.arg("--headed");
        }

        cmd.args(args);
        cmd.stdout(Stdio::piped());
        cmd.stderr(Stdio::piped());

        let output = cmd.output().await.map_err(|e| {
            if e.kind() == std::io::ErrorKind::NotFound {
                PraxisError::AgentBrowserNotFound
            } else {
                PraxisError::browser(format!("Failed to run agent-browser: {}", e))
            }
        })?;

        if output.status.success() {
            Ok(String::from_utf8_lossy(&output.stdout).into_owned())
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            Err(PraxisError::browser(format!(
                "agent-browser command failed: {}",
                stderr
            )))
        }
    }

    /// Run a command and return JSON output
    async fn run_json_command(&self, args: &[&str]) -> Result<String> {
        let mut full_args: Vec<&str> = args.to_vec();
        full_args.push("--json");
        self.run_command(&full_args).await
    }

    /// Navigate to a URL
    pub async fn open(&self, url: &str, wait_for_load: bool) -> Result<ToolResult> {
        // Open the URL
        self.run_command(&["open", url]).await?;

        // Optionally wait for network idle
        if wait_for_load {
            let _ = self.run_command(&["wait", "--load", "networkidle"]).await;
        }

        // Get a snapshot
        let snapshot_output = self.run_json_command(&["snapshot", "-i"]).await?;

        Ok(ToolResult::success_with_data(
            "browse_url",
            format!("Navigated to {}. Page snapshot:\n{}", url, &snapshot_output),
            serde_json::from_str(&snapshot_output).unwrap_or(serde_json::Value::Null),
        ))
    }

    /// Click an element by ref
    pub async fn click(&self, ref_id: &str) -> Result<ToolResult> {
        self.run_command(&["click", ref_id]).await?;

        // Get updated snapshot after click
        let snapshot_output = self.run_json_command(&["snapshot", "-i"]).await?;

        Ok(ToolResult::success_with_data(
            "browser_click",
            format!("Clicked {}. Updated page:\n{}", ref_id, &snapshot_output),
            serde_json::from_str(&snapshot_output).unwrap_or(serde_json::Value::Null),
        ))
    }

    /// Fill an input field
    pub async fn fill(&self, ref_id: &str, text: &str) -> Result<ToolResult> {
        self.run_command(&["fill", ref_id, text]).await?;

        Ok(ToolResult::success(
            "browser_fill",
            format!("Filled {} with '{}'", ref_id, text),
        ))
    }

    /// Get text from an element
    pub async fn get_text(&self, ref_id: &str) -> Result<ToolResult> {
        let output = self.run_command(&["get", "text", ref_id]).await?;

        Ok(ToolResult::success("browser_get_text", output.trim()))
    }

    /// Take a screenshot
    pub async fn screenshot(&self, path: Option<&str>, full_page: bool) -> Result<ToolResult> {
        let mut args = vec!["screenshot"];

        if let Some(p) = path {
            args.push(p);
        }

        if full_page {
            args.push("--full");
        }

        let output = self.run_command(&args).await?;

        let message = if let Some(p) = path {
            format!("Screenshot saved to {}", p)
        } else {
            format!(
                "Screenshot captured (base64): {}...",
                &output[..output.len().min(100)]
            )
        };

        Ok(ToolResult::success("browser_screenshot", message))
    }

    /// Get page snapshot
    pub async fn snapshot(&self, interactive_only: bool) -> Result<ToolResult> {
        let args = if interactive_only {
            vec!["snapshot", "-i"]
        } else {
            vec!["snapshot"]
        };

        let output = self.run_json_command(&args).await?;

        // Try to parse and store the snapshot
        if let Ok(snapshot) = serde_json::from_str::<Snapshot>(&output) {
            let element_count = snapshot.count_elements();
            return Ok(ToolResult::success_with_data(
                "browser_snapshot",
                format!("Page snapshot ({} elements):\n{}", element_count, output),
                serde_json::to_value(&snapshot).unwrap_or(serde_json::Value::Null),
            ));
        }

        Ok(ToolResult::success("browser_snapshot", output))
    }

    /// Close the browser
    pub async fn close(&self) -> Result<ToolResult> {
        self.run_command(&["close"]).await?;
        Ok(ToolResult::success("browser_close", "Browser closed"))
    }

    /// Press a key
    pub async fn press(&self, key: &str) -> Result<ToolResult> {
        self.run_command(&["press", key]).await?;
        Ok(ToolResult::success(
            "browser_press",
            format!("Pressed {}", key),
        ))
    }

    /// Scroll the page
    pub async fn scroll(&self, direction: &str, pixels: Option<u32>) -> Result<ToolResult> {
        let mut args = vec!["scroll", direction];
        let px_str;

        if let Some(px) = pixels {
            px_str = px.to_string();
            args.push(&px_str);
        }

        self.run_command(&args).await?;
        Ok(ToolResult::success(
            "browser_scroll",
            format!("Scrolled {}", direction),
        ))
    }

    /// Get current URL
    pub async fn get_url(&self) -> Result<String> {
        self.run_command(&["get", "url"])
            .await
            .map(|s| s.trim().to_string())
    }

    /// Get page title
    pub async fn get_title(&self) -> Result<String> {
        self.run_command(&["get", "title"])
            .await
            .map(|s| s.trim().to_string())
    }

    /// Wait for an element
    pub async fn wait_for(&self, selector: &str) -> Result<ToolResult> {
        self.run_command(&["wait", selector]).await?;
        Ok(ToolResult::success(
            "browser_wait",
            format!("Element {} is now visible", selector),
        ))
    }

    /// Wait for text to appear
    pub async fn wait_for_text(&self, text: &str) -> Result<ToolResult> {
        self.run_command(&["wait", "--text", text]).await?;
        Ok(ToolResult::success(
            "browser_wait",
            format!("Text '{}' is now visible", text),
        ))
    }

    /// Evaluate JavaScript
    pub async fn eval(&self, script: &str) -> Result<ToolResult> {
        let output = self.run_command(&["eval", script]).await?;
        Ok(ToolResult::success("browser_eval", output))
    }
}

impl Default for BrowserExecutor {
    fn default() -> Self {
        Self::new("praxis")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_executor_creation() {
        let executor = BrowserExecutor::new("test-session");
        assert_eq!(executor.session_name, "test-session");
        assert!(!executor.headed);
    }
}
