# hick-agent

AI command-line coding agent using Hick XML as the edit format and Hick WASM containers for sandboxed command execution.

## Architecture

The implementation has **clean separation** between:

1. **Coding agent** (`hick-agent`) — The AI loop, LLM integration, and orchestration
2. **Sandboxed command execution** (`hick-agent-core::CommandExecutor`) — Trait and `WasmSandboxExecutor` implementation
3. **Hick edit format** (`hick-agent-core::HickEditFormat`) — Pipeline, structure query, provenance, edit computation

Other agents (MCP servers, IDE plugins, etc.) can use `hick-agent-core` directly without the LLM layer.

## Usage

```bash
# Run interactively with OpenAI (requires OPENAI_API_KEY)
hick-agent run

# Run with xAI (requires XAI_API_KEY)
hick-agent run --llm xai

# With custom paths
hick-agent run --project-dir ./my-project --images-dir ~/wasm-images
```

## Dependencies

- **OPENAI_API_KEY** — Set for OpenAI-backed LLM completions (default)
- **XAI_API_KEY** — Set for xAI-backed LLM completions (`--llm xai`)
- **WASM images** — Use `c2w` to convert Docker images (e.g. `alpine`, `ubuntu`) to `.wasm` and place in `--images-dir`

## In-process design

All operations run in-process except for:
- Third-party MCP servers (if configured)
- LLM API calls (OpenAI, etc.)

The Hick pipeline, edit format, and WASM container execution are all in-process.
