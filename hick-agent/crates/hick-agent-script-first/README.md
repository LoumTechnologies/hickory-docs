# hick-agent-script-first

Script-first ReAct strategy extension for the agent runtime.

The LLM is prompted to emit fenced code blocks (Python, shell, JS, C#,
TypeScript). `ReactLoop` extracts those blocks via `extract_code_blocks`, runs
them inside the WASM sandbox, and feeds stdout back as the next observation —
implementing a tool-calling loop without requiring a structured tool-call API.

## Key types

- `ReactLoop` — the main ReAct while-loop
- `AgentState` — state machine driving the loop
- `CodeBlock` / `Language` / `ScriptResult` — tool execution primitives
- `AgentEvent` / `AgentCommand` — channel protocol between agent core and the UI

This crate has no knowledge of hick pipelines and works on any codebase.
