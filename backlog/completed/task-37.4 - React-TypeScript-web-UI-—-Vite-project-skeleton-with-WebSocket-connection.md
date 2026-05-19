---
id: TASK-37.4
title: React/TypeScript web UI — Vite project skeleton with WebSocket connection
status: Done
assignee: []
created_date: '2026-05-19 11:55'
updated_date: '2026-05-19 14:44'
labels:
  - web
  - frontend
  - react
dependencies:
  - TASK-37.3
parent_task_id: TASK-37
priority: high
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Create the React/TypeScript frontend project that will become the hick agent web UI. The SPA is pre-built (Vite production build committed to the repo) so `cargo build` has no JS toolchain dependency. This task establishes the project skeleton and proves the WebSocket connection end-to-end.

## What to build

### Project location

`hick-agent/web-ui/` — a Vite + React + TypeScript project. The production build output goes to `hick-agent/web-ui/dist/`. The web server module (task-37.3) embeds `dist/` via `include_bytes!`.

### Tech stack

- Vite 6 + React 19 + TypeScript
- Tailwind CSS for styling
- `@microsoft/adaptivecards` (installed now, wired in task-37.6)
- No other UI framework — keep the dependency footprint minimal

### WebSocket client

A `useAgentSocket` React hook that:
- Connects to `ws://localhost:7890/ws` (or the relay WebSocket URL if a relay session ID is in the URL hash)
- Exposes `events: UiEvent[]`, `send(cmd: UiCommand) => void`, `connected: boolean`
- Automatically reconnects on disconnect with exponential backoff

### UiEvent/UiCommand TypeScript types

Generate or hand-write TypeScript types mirroring the Rust `UiEvent`/`UiCommand` enums. Place in `src/types.ts`. These are the source of truth for the browser side; keep them in sync manually (no codegen required for now).

### Initial UI layout

A three-panel layout:
- **Left sidebar** (collapsible): session metadata (model, provider, start time, session ID)
- **Center main**: conversation view (populated in task-37.5)
- **Bottom input bar**: text input + send button + cancel button

For this task, the center panel just renders a raw JSON dump of received events (real tree view comes in task-37.5).

### Build script

Add a `just build-web-ui` recipe to the root `justfile` that runs `cd hick-agent/web-ui && npm run build`. Add a note in the task that implementers must run this and commit the `dist/` output after making frontend changes.

### Development workflow

`just dev-web-ui` starts `vite dev` with a proxy for `/ws` pointing to `localhost:7890`. This lets frontend developers iterate without rebuilding the Rust binary.

## Constraints

- The `dist/` directory must be committed to the repo — the Rust build embeds it at compile time
- The project must build with `npm run build` producing a single-page app in `dist/`
- No server-side rendering — pure SPA
- The `useAgentSocket` hook must handle the case where the agent is not running (show a "not connected" state gracefully)
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 hick-agent/web-ui/ is a valid Vite+React+TypeScript project that builds with npm run build
- [x] #2 dist/ output is committed to the repo and embedded by the web server module
- [x] #3 useAgentSocket hook connects to the WebSocket and streams UiEvents to the component
- [x] #4 Three-panel layout renders correctly: sidebar, main, input bar
- [x] #5 Sending a message from the input bar dispatches a UiCommand::Message to the agent
- [x] #6 Cancel button sends UiCommand::Cancel
- [x] #7 Disconnected state is shown gracefully when the agent is not running
- [x] #8 just build-web-ui and just dev-web-ui recipes exist in the root justfile
- [x] #9 cargo test --workspace passes (Rust side unaffected)
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
## Implementation plan

1. **Scaffold** `hick-agent/web-ui/`: Vite 6 + React 19 + TypeScript, Tailwind CSS v4 (or v3 per Vite template), `@microsoft/adaptivecards`, stable Rollup output filenames (`assets/app.js`, etc.) for predictable `include_bytes!` paths.

2. **Types** `src/types.ts`: Mirror `UiCommand` (tagged `type` snake_case) and wire `UiEvent` union matching `ui_event_wire_value` JSON from `ui_event.rs`.

3. **`useAgentSocket`**: WebSocket to `ws(s)://host/ws` in production (same host as page); dev: Vite proxy `/ws` → `localhost:7890`. Parse hash for relay session (e.g. `#relay=...` or documented pattern — verify in codebase). State: `events[]`, `send`, `connected`, reconnect with exponential backoff, append parsed JSON events.

4. **UI**: Three-panel layout (Tailwind): collapsible left sidebar (session metadata from `session_started` + placeholders), center = `<pre>` JSON dump of `events`, bottom bar = input + Send (`{type:"message",text}`) + Cancel (`{type:"cancel"}`). Disconnected banner when `!connected`.

5. **`web_server.rs`**: Replace `web_placeholder` `include_bytes!` with paths to `../../../web-ui/dist/index.html` and dist assets; extend `serve_asset` to serve known files from committed `dist/`.

6. **Root `justfile`**: Add `build-web-ui` and `dev-web-ui` (vite with proxy).

7. **Build & verify**: `npm ci && npm run build` in web-ui, commit `dist/`, run `cargo test --workspace -- --test-threads=1`.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Started implementation per user request; relay hash format will match existing repo convention or use `#session=`-style placeholder if none found.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
## Summary

Delivered `hick-agent/web-ui/` as Vite 6 + React 19 + TypeScript + Tailwind v4, with `adaptivecards` (npm) installed for a later Adaptive Cards task. Added `src/types.ts` (UiCommand/UiEvent aligned with Rust wire JSON), `useAgentSocket` (exponential backoff reconnect, optional relay/hash URL resolution, `sessionStartedObservedAt`), and a three-panel shell with JSON event dump + message/cancel commands.

Production assets are written to `dist/` with stable filenames (`assets/app.js`, `assets/app.css`) and embedded from `hick-agent-sdk` `web_server.rs` (placeholder HTML/JS removed; CSS served with correct `Content-Type`).

Root `justfile`: `build-web-ui` and `dev-web-ui`. `hick-agent/web-ui/.gitignore` no longer ignores `dist/` so the SPA can be committed.

**Verify:** `npm run lint` and `npm run build` in `hick-agent/web-ui`; `cargo test --workspace -- --test-threads=1` at repo root (passed).

**Note:** After any frontend change, run `just build-web-ui` and commit `dist/`.
<!-- SECTION:FINAL_SUMMARY:END -->
