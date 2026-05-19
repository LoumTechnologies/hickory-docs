# hick-agent

Universal hick agent binary (`hick-agent` executable).

Reads `.hick/agent.toml` at startup to select which extension crates to
activate at runtime from a single binary that links all of them:

| Config key | Extension |
|---|---|
| `script_first` | `hick-agent-script-first` — ReAct loop via code blocks |
| `hick_edit` | `hick-agent-hick-edit` — pipeline-aware editing |
| `mcp_wasm` | `hick-agent-mcp-wasm` — MCP tools as WASM binaries |
| `mcp_sdk` | `hick-agent-mcp-sdk` — MCP tools via generated stubs |
| `mcp_xml` | `hick-agent-mcp-xml` — MCP tools via XML elements |
| `mcp_graphql` | `hick-agent-mcp-graphql` — MCP tools via GraphQL |

Unlike the A/B evaluation binaries (`copper-ledge`, `iron-peak`), the extension
combination here is driven entirely by config rather than by compile-time
`Cargo.toml` selection.
