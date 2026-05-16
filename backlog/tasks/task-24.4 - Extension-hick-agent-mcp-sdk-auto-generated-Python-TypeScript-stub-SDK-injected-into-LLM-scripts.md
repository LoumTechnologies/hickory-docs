---
id: TASK-24.4
title: >-
  Extension hick-agent-mcp-sdk: auto-generated Python/TypeScript stub SDK
  injected into LLM scripts
status: To Do
assignee: []
created_date: '2026-05-16 12:23'
updated_date: '2026-05-16 12:51'
labels:
  - mcp
  - extensions
  - sdk
  - python
  - typescript
dependencies:
  - TASK-24.1
  - TASK-24.2
references:
  - wasm-net/crates/network/src/interceptor.rs
  - wasm-net/crates/network/src/network_loop.rs
  - hick-agent/crates/hick-agent-script-first/src/tools.rs
  - hick-agent/crates/hick-agent-script-first/src/react_loop.rs
parent_task_id: TASK-24
priority: high
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
New extension crate `hick-agent-mcp-sdk`. At agent startup, for each MCP tool it generates Python and TypeScript stub files. The stubs call a magic gateway IP intercepted by the smoltcp network loop (the same `Interceptor` approach as TASK-24.6) — no OS port is bound, no Unix socket is mounted, no loopback reachability concern.

## Transport: Interceptor, not a real server

The extension registers an `McpSdkInterceptor` (implementing `wasm-net`'s `Interceptor` trait) at a magic IP (e.g. `169.254.1.2:80`, distinct from the GraphQL interceptor's address so both can coexist). Generated stubs call `http://169.254.1.2/call` with a simple JSON body:

```json
{"server": "github", "tool": "create_issue", "input": {"owner": "...", ...}}
```

The interceptor reads this request inside the smoltcp event loop, calls `ToolGate::check`, then `McpClient::call`, and writes the JSON response back. Auth credentials stay in `McpCredentialStore` in the host Rust process — they never cross into the container side.

## Stub generation

For each `McpToolDef`, generate:

**Python (`_mcp.py`)**
```python
import urllib.request, json

_MCP_URL = "http://169.254.1.2/call"

def github_create_issue(owner: str, repo: str, title: str, body: str = "") -> dict:
    req = {"server": "github", "tool": "create_issue",
           "input": {"owner": owner, "repo": repo, "title": title, "body": body}}
    resp = urllib.request.urlopen(
        urllib.request.Request(_MCP_URL, json.dumps(req).encode(), method="POST"))
    return json.loads(resp.read())
```

**TypeScript (`_mcp.ts`)**
```typescript
const MCP_URL = "http://169.254.1.2/call";

export async function githubCreateIssue(
    owner: string, repo: string, title: string, body = ""
): Promise<Record<string, unknown>> {
    const res = await fetch(MCP_URL, {
        method: "POST",
        body: JSON.stringify({server: "github", tool: "create_issue",
                              input: {owner, repo, title, body}})
    });
    return res.json();
}
```

No credentials, no real server URLs, no `MCP_PROXY_ADDR` env var — the magic IP is the only transport detail in the generated file.

## Preamble injection

The extension hooks into the script-first ReAct loop's script dispatch path. Before writing a Python or TypeScript script file to the WASM container, it prepends:

```python
# auto-generated — do not edit
from _mcp import github_create_issue, filesystem_read_file, ...
```

```typescript
import { githubCreateIssue, filesystemReadFile } from './_mcp';
```

`_mcp.py` / `_mcp.ts` are written once per session into the container's working directory. The preamble is prepended to a copy of the script written to the container — the original LLM response stored in conversation history is not modified.

## System prompt

The extension emits a system prompt section listing all available functions with their Python signatures and a brief usage note so the LLM knows what to call.

## Integration point

This extension wraps or composes with `hick-agent-script-first`. It intercepts the script dispatch step and prepends the preamble before handing off to the executor. The `McpSdkInterceptor` is registered in the `InterceptorRegistry` at agent init, ahead of `PassthroughInterceptor`.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 McpSdkInterceptor implements the Interceptor trait from wasm-net and is registered in InterceptorRegistry ahead of PassthroughInterceptor, at a distinct magic IP from the GraphQL interceptor.
- [ ] #2 No OS TCP port is bound for the SDK proxy; test confirms no socket is opened on the host.
- [ ] #3 For a mock MCP server with 2 tools, the extension writes a valid _mcp.py and _mcp.ts into the container working directory at session start.
- [ ] #4 _mcp.py and _mcp.ts contain no credentials and no real server URLs — only the magic gateway IP.
- [ ] #5 LLM-written Python script that calls a generated function receives the tool result as a return value.
- [ ] #6 LLM-written TypeScript script that calls a generated function receives the tool result.
- [ ] #7 ToolGate::check is called inside the interceptor before McpClient::call; a blocked call returns a non-2xx HTTP response causing the stub to raise an exception with a descriptive message.
- [ ] #8 A classification=secret value in container scope blocks the interceptor call (integration test using a Python script).
- [ ] #9 The system prompt fragment lists all available functions with their Python signatures.
- [ ] #10 Preamble injection prepends to a copy written to the container — the LLM output in conversation history is not modified.
- [ ] #11 Extension integrates into an agent binary without source changes to hick-agent-sdk, hick-agent-script-first, or wasm-net.
<!-- AC:END -->
