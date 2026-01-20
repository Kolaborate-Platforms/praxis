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

1. **Ollama**: Install from [ollama.ai](https://ollama.ai/).
   - Required models (defaults): `qwen3-vl:8b` (orchestrator), `gemma3:4b` (executor).
2. **agent-browser (Optional)**: For web automation features.
   ```bash
   npm install -g agent-browser
   agent-browser install
   ```

## ⚙️ Configuration

Praxis looks for configuration in `~/.config/praxis/config.toml`. You can also use environment variables.

### Example `config.toml`

```toml
[models]
orchestrator = "qwen3-vl:8b"
executor = "gemma3:4b"

[agent]
max_history = 1000
max_turns = 10
debug = false

[browser]
enabled = true
session_name = "default"

[streaming]
enabled = true
```

## 🚀 Getting Started

### Installation

```bash
cargo build --release
```

### Usage

**Interactive REPL:**
```bash
./target/release/praxis
```

**Single Prompt:**
```bash
./target/release/praxis -p "Research the latest Rust 1.84 features and summarize them."
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