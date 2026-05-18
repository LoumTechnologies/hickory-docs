---
id: TASK-34.3
title: >-
  Implement: smoltcp interceptor for sentinel credential substitution and
  per-server network filtering
status: Done
assignee: []
created_date: '2026-05-18 13:02'
updated_date: '2026-05-18 17:30'
labels: []
milestone: m-5
dependencies:
  - TASK-34.2
parent_task_id: TASK-34
priority: high
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
## Goal

Add a smoltcp network interceptor that: (a) recognises sentinel tokens in outbound HTTP `Authorization` headers, substitutes the real credential from `McpCredentialStore`, and (b) enforces the per-server allowed-hosts list from the sandbox config.

## Scope: stdio servers only

This interceptor only applies to stdio MCP servers running in the WASM sandbox. Remote HTTP servers (`HttpMcpClient`) already inject credentials on the host at call time — no change needed there.

## How it fits the existing architecture

`hick-agent-mcp-sdk` already uses this exact pattern: a magic-IP interceptor (`169.254.1.2:80`) inside the smoltcp network loop intercepts HTTP calls from WASM scripts. This task adds a sibling interceptor (or extends the existing one) that handles MCP server outbound traffic.

The interceptor:
1. Inspects the `Authorization` header of every outbound HTTP request from the sandbox
2. If the value matches `hick-proxy:<server-name>:<nonce>`, looks up the real credential for `<server-name>` in `McpCredentialStore`
3. Replaces the header value with the real credential (e.g. `Bearer ghp_realtoken...`)
4. Checks the destination host against the per-server `allowed_hosts` config — blocks if not permitted
5. Forwards the modified request

## Security note

The sentinel token format (`hick-proxy:<server-name>:<nonce>`) is meaningless outside the sandbox. Even if an LLM script extracts it via the MCP server's env, the sentinel cannot be used to call real APIs — only the interceptor can resolve it to a real credential. The nonce prevents a script from constructing a valid sentinel for a server it didn't start.

## References

- `hick-agent-mcp-sdk/src/lib.rs` — `McpSdkInterceptor`, magic-IP interceptor pattern to extend
- `hick-agent-sdk/src/mcp_config.rs` — `McpCredentialStore` for real credential lookup
- TASK-34.1 — `allowed_hosts` config consumed here
- TASK-34.2 — sentinel token generated and injected here; interceptor validates it
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 Interceptor recognises hick-proxy:<server>:<nonce> sentinel in Authorization header and substitutes the real credential
- [x] #2 Outbound calls to hosts not in allowed_hosts are blocked and return an error to the MCP server
- [x] #3 Real credential is never logged or exposed in interceptor error messages
- [x] #4 An LLM script cannot use a captured sentinel token to make authenticated calls — the nonce ties the sentinel to a specific server startup
- [x] #5 Works alongside the existing McpSdkInterceptor at 169.254.1.2 without conflict
- [x] #6 Test: server configured with allowed_hosts=[api.github.com] cannot reach api.openai.com
- [x] #7 Test: real credential appears in the substituted request header; sentinel does not
<!-- AC:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Added `McpSentinelInterceptor` to `hick-agent-mcp-sdk/src/lib.rs` at magic IP `169.254.1.4:80` (distinct from SDK interceptor at `169.254.1.2`). `register()` associates a server name, sentinel token, and sandbox config. `resolve_sentinel()` returns `Ok(None)` for non-sentinels (pass-through), `Ok(Some(header))` for valid sentinels (real credential from McpCredentialStore), and `Err(reason)` for nonce mismatch/unknown server. Real credentials are never logged. `handle_proxy_request()` implements HTTP/1.x proxy: reads request, substitutes credential, enforces allowed_hosts, forwards to real server. Nonce mismatch returns 403. Added `credential_to_header()` and `base64_encode()` helpers. Tests cover: IP matching, valid sentinel resolution, nonce mismatch rejection, unknown server, non-sentinel pass-through, host filtering, and end-to-end credential substitution.
<!-- SECTION:FINAL_SUMMARY:END -->
