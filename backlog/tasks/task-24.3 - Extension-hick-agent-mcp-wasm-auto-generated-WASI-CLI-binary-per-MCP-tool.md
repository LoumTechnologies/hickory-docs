---
id: TASK-24.3
title: 'Extension hick-agent-mcp-wasm: auto-generated WASI CLI binary per MCP tool'
status: To Do
assignee: []
created_date: '2026-05-16 12:22'
updated_date: '2026-05-16 12:52'
labels:
  - mcp
  - wasm
  - wasi
  - extensions
dependencies:
  - TASK-24.1
  - TASK-24.2
  - TASK-9
references:
  - wasm-net/crates/network/src/interceptor.rs
  - >-
    backlog/tasks/task-9 -
    WASI-fast-path-agent-oriented-HTTP-fetch-tool-transparent-delegation-from-Linux-WASM-VM.md
  - hick-agent/crates/hick-agent-sdk/src/command.rs
parent_task_id: TASK-24
priority: high
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
New extension crate `hick-agent-mcp-wasm`. At agent startup it generates one WASI Preview 2 component per configured MCP tool and registers each on the WASM VM's PATH. The components call into the host via a custom WIT import — a direct Wasmtime host function — with no network, no HTTP, no proxy server, and no interceptor needed.

## Transport: WIT host import

Define a custom WIT interface in the workspace:

```wit
// wit/mcp-tool.wit
package hick:mcp@0.1.0;

interface mcp {
    /// Invoke an MCP tool. Input and output are JSON strings.
    call-tool: func(server: string, tool: string, input: string) -> result<string, string>;
}

world mcp-tool {
    import mcp;
    export run: func(args: list<string>) -> s32;
}
```

The host registers the implementation with Wasmtime's component linker:

```rust
linker.func_wrap("hick:mcp/mcp", "call-tool",
    |mut store: Caller<'_, StoreState>, server: &str, tool: &str, input: &str|
    -> Result<String, String>
{
    let state = store.data();
    // ToolGate::check (blocking call via block_on or store-held async handle)
    // McpClient::call
    // return result JSON or error string
})?;
```

The generated component calls `mcp::call-tool(...)` as a plain function call. No network stack is involved at any layer.

## Code generation

For each `McpToolDef`, generate a minimal Rust source file:

```rust
wit_bindgen::generate!({
    world: "mcp-tool",
    path: "wit/mcp-tool.wit",
});

struct Component;
impl Guest for Component {
    fn run(args: Vec<String>) -> i32 {
        let input = args_to_json(&args);   // derive JSON from positional args + JSON Schema
        match mcp::call_tool("github", "create_issue", &input) {
            Ok(result) => { println!("{result}"); 0 }
            Err(e)     => { eprintln!("error: {e}"); 1 }
        }
    }
}
```

Compiled with `cargo-component` to a `.wasm` component. Build happens at agent init time using the same toolchain as TASK-9's `hick-fetch`. Generated source is ephemeral (temp dir); never committed to the workspace.

## Auth isolation

Credentials are never referenced in the generated source. They live in `McpCredentialStore` on the host side, accessed only inside the `call-tool` host function implementation.

## VM PATH injection

The extension registers generated `.wasm` component paths with the WASM VM via TASK-9's delegation model: when the VM would execute a tool binary by name, the host-side runner intercepts and runs the WASI component instead. Tool binary name = `mcp-<server>-<tool>` (e.g. `mcp-github-create_issue`).

## ToolGate integration

`ToolGate::check` is called inside the `call-tool` host function before forwarding to `McpClient`. A blocked call returns `Err("blocked: ...")` which the component maps to a non-zero exit code and a stderr message.

## Scope boundaries
- One WASI component per tool; no mega-binary.
- A build failure for one tool is non-fatal (logs warning, tool excluded from PATH).
- Python/TypeScript wrappers for these binaries are out of scope (that is TASK-24.4).
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 A custom WIT interface (hick:mcp/mcp) is defined in the workspace with a call-tool function; the host registers an implementation via Wasmtime's component linker.
- [ ] #2 For a mock MCP server with 2 tools, the extension generates 2 WASI components that build successfully with cargo-component.
- [ ] #3 Invoking a generated binary from inside a LLM-written shell script returns the tool result on stdout.
- [ ] #4 No network socket of any kind is opened by the WASI component or its host function — the call is a pure Wasmtime host import.
- [ ] #5 Auth credentials do not appear in the generated Rust source or in any file written to the workspace or temp dir.
- [ ] #6 ToolGate::check is called inside the call-tool host function; a blocked call causes the component to exit non-zero with an error on stderr.
- [ ] #7 A classification=secret value in WASM container scope causes the host function to block the call (integration test).
- [ ] #8 Tool binary names follow the mcp-<server>-<tool> convention and are listed in the system prompt fragment.
- [ ] #9 A build failure for one tool logs a warning but does not prevent other tools from being registered.
- [ ] #10 Extension integrates into an agent binary by wiring the WIT linker and registering McpWasmExtension — no source changes to hick-agent-sdk or wasm-net.
<!-- AC:END -->
