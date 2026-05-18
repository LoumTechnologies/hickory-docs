---
id: m-5
title: "m-5: MCP Server Sandboxing"
---

## Description

Run each untrusted MCP server in its own WASM sandbox with per-server network filtering and credential isolation. The LLM never sees MCP secrets; MCP servers cannot access each other's credentials or host resources beyond their declared permissions.
