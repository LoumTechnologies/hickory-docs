# hick-agent

AI command-line coding agent using Hick XML as the edit format and Hick WASM containers for sandboxed command execution. Implements a full ReAct/tool-calling loop like Claude Code.

## Architecture

The implementation has **clean separation** between:

1. **Coding agent** (`hick-agent`) — The AI loop, LLM integration, and orchestration
2. **Sandboxed command execution** (`hick-agent-core::CommandExecutor`) — Trait and `WasmSandboxExecutor` implementation
3. **Hick edit format** (`hick-agent-core::HickEditFormat`) — Pipeline, structure query, provenance, edit computation

Other agents (MCP servers, IDE plugins, etc.) can use `hick-agent-core` directly without the LLM layer.

## ReAct Loop

The agent uses a strongly-typed state machine (`AgentState`) and tool-calling loop:

- **Tools**: `run_pipeline`, `exec`, `query`, `structure`, `provenance`, `compute_edits`, `apply_edits`, `read_file`, `write_file`, `list_dir`, `create_backing`
- **File I/O**: Hick files (.hick) and generated output files are both written to disk and intended for check-in
- **Backing escape hatch**: When a generated file is edited outside the pipeline (provenance lost), use `create_backing` to generate a .hick wrapper so the edit format can still work

## Usage

```bash
# Run interactively with OpenAI (requires OPENAI_API_KEY)
hick-agent run

# Run with xAI (requires XAI_API_KEY)
hick-agent run --llm xai

# With custom paths
hick-agent run --project-dir ./my-project --images-dir ~/wasm-images
```

## Environment Variables

- **OPENAI_API_KEY** — OpenAI completions (default)
- **XAI_API_KEY** — xAI completions (`--llm xai`)
- **HICK_IMAGES_DIR** — Directory with .wasm container images
- **HICK_OUTPUT_DIR** — Output directory for generated files (default: project root)

## Dependencies

- **WASM images** — Use `c2w` to convert Docker images (e.g. `alpine`, `ubuntu`) to `.wasm` and place in `--images-dir`

## In-process design

All operations run in-process except for:
- Third-party MCP servers (if configured)
- LLM API calls (OpenAI, xAI, etc.)

The Hick pipeline, edit format, and WASM container execution are all in-process.
