---
id: TASK-37.3
title: Local web server — axum HTTP + WebSocket bridge in hick-agent-sdk
status: To Do
assignee: []
created_date: '2026-05-19 11:54'
labels:
  - web
  - sdk
dependencies:
  - TASK-37.1
parent_task_id: TASK-37
priority: high
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Add a local web server to `hick-agent-sdk` that serves the React SPA and exposes a WebSocket endpoint for real-time bidirectional communication between the browser and the agent. This is the prerequisite for the web UI (task-37.4) and the relay client (task-37.8).

## What to build

### `hick-agent-sdk::web_server` module

```rust
pub struct WebServer { ... }

impl WebServer {
    /// Bind on localhost:port (default 7890, configurable via HICK_WEB_PORT env var).
    pub async fn bind(config: WebServerConfig) -> anyhow::Result<Self>;

    /// Attach the UiEvent broadcast sender and UiCommand sender.
    /// Returns a receiver for UiCommands arriving from browser clients.
    pub fn attach(
        &self,
        event_tx: broadcast::Sender<UiEvent>,
        cmd_tx: mpsc::UnboundedSender<UiCommand>,
    );

    /// Start serving. Does not block — runs in the background via tokio::spawn.
    pub async fn start(&self) -> anyhow::Result<String>; // returns "http://localhost:7890"
}
```

### HTTP routes

- `GET /` — serve the React SPA (embedded via `include_bytes!` from the pre-built `dist/index.html`)
- `GET /assets/*` — serve bundled JS/CSS assets
- `GET /ws` — WebSocket upgrade; streams `UiEvent`s to the browser and receives `UiCommand`s from the browser

### WebSocket wire format

Events are serialized as compact JSON (not XML — XML is for the session log file, JSON is for the browser wire format). Each message is one JSON object with a `type` field matching the `UiEvent` variant name in snake_case. Example:

```json
{"type":"token","node_id":5,"text":"Hello"}
{"type":"action_start","node_id":7,"parent_id":5,"lang":"python","code":"print(1)","collapsed_by_default":true}
```

Commands from browser to server:
```json
{"type":"message","text":"do the thing"}
{"type":"cancel"}
{"type":"expand_node","node_id":7}
```

### Multiple browser clients

Use `tokio::sync::broadcast` for the event fan-out so multiple browser tabs can connect simultaneously (read-only tabs, remote control tab, etc.).

### SPA placeholder

For this task, the embedded SPA is a minimal `index.html` that connects to the WebSocket, pretty-prints incoming JSON events, and provides a text input to send commands. The real React UI comes in task-37.4. The placeholder just proves the plumbing works.

### Binary crate integration

Update copper-ledge and iron-peak `main.rs` to optionally start the web server (enabled by default; disable with `--no-web` flag or `HICK_NO_WEB=1`). Print the URL to stdout at startup: `Web UI: http://localhost:7890`.

## Constraints

- Web server feature should be behind a Cargo feature flag `web` on `hick-agent-sdk` so the SDK can be used in environments without axum
- `UiEvent` must implement `serde::Serialize` for JSON wire format (add `serde` derive in task-37.1 or here)
- Default port 7890; configurable via `HICK_WEB_PORT` env var
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 WebServer starts on localhost:7890 by default; URL is printed at agent startup
- [ ] #2 GET / serves a working HTML page that connects to the WebSocket
- [ ] #3 WebSocket streams UiEvents as compact JSON to all connected browser clients
- [ ] #4 Browser clients can send UiCommand JSON messages and they are forwarded to the agent
- [ ] #5 Multiple simultaneous browser connections all receive the same event stream
- [ ] #6 The web server is disabled with --no-web flag or HICK_NO_WEB=1
- [ ] #7 The web feature is a Cargo feature flag; SDK compiles without it
- [ ] #8 cargo test --workspace passes
<!-- AC:END -->
