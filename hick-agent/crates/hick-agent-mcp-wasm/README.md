# hick-agent-mcp-wasm

MCP extension that compiles one WASI Preview 2 component per MCP tool at agent
startup and registers each as an `mcp-<server>-<tool>` binary on the WASM VM's
PATH.

`McpWasmExtension` generates Rust source + `Cargo.toml` for each tool and
invokes `cargo component build`; build failures are non-fatal. Scripts running
in the sandbox can then call MCP tools as ordinary command-line binaries.

`McpWasmContainerBackend` wraps another `ContainerBackend` and intercepts
commands whose name matches the `mcp-` prefix, calling `ToolGate::check` then
dispatching to the MCP client on the host. Credentials never appear in the
generated source.
