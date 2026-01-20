# Praxis 🛰️

Praxis is a high-performance, offline-first AI coding agent built in Rust. It utilizes a dual-model orchestration approach powered by [Ollama](https://ollama.ai/) to perform complex tasks, manage browser automation, and execute multi-step reasoning loops.

## ✨ Key Features

- **🧠 ReAct Reasoning Loop**: Iterative Thought → Action → Observation pattern for autonomous problem solving.
- **🎭 Dual-Model Orchestration**: Specialized "Orchestrator" for tool selection and "Executor" for code generation.
- **🌐 Browser Automation**: Seamless integration with `agent-browser` for web navigation, filling forms, and scraping.
- **⚡ Parallel Execution**: Concurrent tool execution using Tokio `JoinSet` for maximum efficiency.
- **🤝 Sub-Agent Architecture**: Capability to spawn specialized sub-agents for delegation and parallel task processing.
- **🛠️ Extensible Toolset**: Built-in support for coding (write, explain, debug), context analysis, and web tools.
- **🚀 High Performance**: Built with Rust for safety, speed, and minimal resource footprint.

## 📋 Prerequisites

1.  **Ollama**: Install from [ollama.ai](https://ollama.ai/).
    *   **Required models**: `qwen3-vl:8b` (orchestrator), `qwen3:8b` (executor).
    *   **Setup**:
        ```bash
        ollama serve
        ollama pull qwen3-vl:8b
        ollama pull qwen3:8b
        ```
2.  **agent-browser (Optional)**: For web automation features.
    ```bash
    npm install -g agent-browser
    agent-browser install
    ```

## 🚀 Installation

### npm (Recommended)
```bash
npm install -g @kolaborate/praxis
```

### Homebrew (macOS)
```bash
brew tap kolaborate-platforms/tap
brew install praxis
```

### Direct Download (Shell Script)

**macOS / Linux:**
```bash
curl --proto '=https' --tlsv1.2 -LsSf https://github.com/Kolaborate-Platforms/praxis/releases/latest/download/praxis-installer.sh | sh
```

**Windows (PowerShell):**
```bash
irm https://github.com/Kolaborate-Platforms/praxis/releases/latest/download/praxis-installer.ps1 | iex
### From Source

```bash
cargo install --path .
```

## 🔄 Updating Praxis

**npm:**
```bash
npm install -g @kolaborate/praxis@latest
```

**Homebrew:**
```bash
brew update && brew upgrade praxis
```

## 🗑️ Uninstalling

**npm:**
```bash
npm uninstall -g @kolaborate/praxis
```

**Homebrew:**
```bash
brew uninstall praxis && brew untap kolaborate-platforms/tap
```

## ⚙️ Configuration

Praxis looks for configuration in `~/.config/praxis/config.toml`. It also respects environment variables like `OLLAMA_HOST` and `OLLAMA_PORT`.

### Example `config.toml`

```toml
[ollama]
host = "localhost"
port = 11434

[models]
orchestrator = "qwen3-vl:8b"
executor = "qwen3:8b"

[agent]
max_history = 1000
max_turns = 10
debug = false

[browser]
enabled = true
session_name = "praxis"

[streaming]
enabled = true
```

## 🚀 Usage

**Interactive REPL:**
```bash
praxis
```

**Single Prompt:**
```bash
praxis -p "Research the latest Rust 1.84 features and summarize them."
```

**Debug Mode:**
```bash
PRAXIS_DEBUG=true ./target/release/praxis
```

## 🧪 Testing & Benchmarking

Praxis includes a benchmark harness to compare different models.

**Run All Tests:**
```bash
cargo test
```

**Run Model Benchmarks:**
```bash
cargo test --test model_benchmark -- --ignored
```

## 🗺️ Project Structure

- `src/agent`: Core agent logic, conversation management, and ReAct loop.
- `src/llm`: Client implementations for Ollama.
- `src/tools`: Tool registry and individual tool implementations (Coding, Browser, etc.).
- `src/core`: Shared types, error handling, and configuration logic.
- `src/cli`: CLI interface and REPL implementation.

## 📄 License

MIT License - See [LICENSE](LICENSE) for details. (Place holder)

## Possible Future Integrations

- https://aspectron.org/en/projects/workflow-rs.html
- https://github.com/workflow-rs/workflow-rs   