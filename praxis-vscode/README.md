# Praxis VS Code Extension

Spawn the **Praxis** offline-first AI coding agent directly in your VS Code terminal.

## Features

- 🚀 **One-Click Launch**: Start Praxis from the Command Palette or with a keyboard shortcut
- ⌨️ **Keyboard Shortcuts**: `Cmd+Shift+;` (Mac) / `Ctrl+Shift+;` (Windows/Linux)
- ⚙️ **Configurable**: Override models, enable debug mode, and more
- 🪟 **Split Terminal**: Open Praxis in a split view alongside your current terminal

## Requirements

- **Praxis CLI** must be installed and available in your PATH
  ```bash
  # Install from source
  cargo install --path /path/to/praxis
  
  # Or using the installer (if available)
  curl --proto '=https' --tlsv1.2 -LsSf https://github.com/Kolaborate-Platforms/praxis/releases/latest/download/praxis-installer.sh | sh
  ```

- **Ollama** must be running with the required models
  ```bash
  ollama serve
  ollama pull qwen3-vl:8b
  ollama pull qwen3:8b
  ```

## Usage

1. Open the Command Palette (`Cmd+Shift+P` / `Ctrl+Shift+P`)
2. Type "Praxis: Start" and press Enter
3. Praxis will launch in a new terminal in your workspace

### Keyboard Shortcuts

| Shortcut | Action |
|----------|--------|
| `Cmd+Shift+;` / `Ctrl+Shift+;` | Start Praxis |
| `Cmd+Escape` / `Ctrl+Escape` | Start Praxis (when in terminal) |

## Extension Settings

| Setting | Description | Default |
|---------|-------------|---------|
| `praxis.executablePath` | Path to praxis executable | `praxis` |
| `praxis.orchestratorModel` | Override orchestrator model | (uses config) |
| `praxis.executorModel` | Override executor model | (uses config) |
| `praxis.debug` | Enable debug output | `false` |
| `praxis.disableBrowser` | Disable browser tools | `false` |

## Development

```bash
# Install dependencies
npm install

# Compile
npm run compile

# Watch mode
npm run watch

# Package extension
npm run package
```

## License

MIT - See [LICENSE](../LICENSE) for details.
