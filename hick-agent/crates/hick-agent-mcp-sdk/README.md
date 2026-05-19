# hick-agent-mcp-sdk

MCP extension that exposes MCP tools to LLM-written scripts via auto-generated
stub files.

## How it works

At agent startup, stubs (`_mcp.py`, `_mcp.ts`, etc.) are generated and written
into the sandbox. When a script calls a stub, the stub makes an HTTP request
to a magic gateway IP (`169.254.1.2:80`) that is intercepted **inside** the
smoltcp network loop by `McpSdkInterceptor` — no OS TCP port is ever bound.

A second interceptor, `McpSentinelInterceptor` (at `169.254.1.4:80`), acts as
an HTTP proxy for sandboxed stdio MCP servers. It validates nonce-based sentinel
tokens, substitutes real credentials from `McpCredentialStore`, and enforces
per-server host allow-lists before forwarding requests.

Credentials never appear in the generated stub files.
