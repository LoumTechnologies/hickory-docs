# hick-agent-engine

Re-export shim sitting above `hick-agent-sdk` and `hick-agent-script-first`,
with one unique addition: local LLM inference.

## Local inference (optional features)

The `local` / `local-cuda` features add the `llm_local` module, which wraps
`llama.cpp` via the `llama-cpp-2` crate for on-device GGUF model inference.
This heavy dependency lives here specifically so no other crate in the agent
stack is forced to carry it.

## Re-exports

Everything else is forwarded from the two underlying crates — `LlmClient`,
`ReactLoop`, `AgentEvent`, `AgentCommand`, and sub-module aliases (`events`,
`network_filter`, `tools`) that preserve the import paths used by TUI code.
