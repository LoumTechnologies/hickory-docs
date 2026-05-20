---
id: TASK-53
title: >-
  Design: Tauri desktop app — independent agent, conversation federation, and
  MCP-driven IDE
status: To Do
assignee: []
created_date: '2026-05-20 12:37'
updated_date: '2026-05-20 12:43'
labels:
  - design
  - tauri
  - ide
  - desktop
  - ux
dependencies:
  - TASK-43
  - TASK-52
references:
  - hick-agent/web-ui/resolveWebSocketUrl.ts
  - hick-agent/crates/hick-relay/src/main.rs
priority: medium
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Design and prototype a Tauri desktop application that is an **independent agent host**, not a frontend for the terminal agent. The Tauri app owns its own conversation and runs its own agent process. Shared infrastructure (relay, session registry) enables optional cross-process/machine conversation discovery, presence awareness, and draft-prompt sync.\n\n## Ownership model\n\nEach frontend (terminal TUI, browser tab, Tauri desktop app) **owns its own conversation**. There is no default pairing between a terminal agent and a Tauri app. A user who opens the Tauri app gets a fresh conversation backed by a Tauri-embedded agent process.\n\nOptional cross-instance features (conversation list, presence, draft sync — see below) are additive; they do not change the ownership model.\n\n## Why Tauri (motivations separate from screen-lock, which is solved in TASK-52.1)\n\n1. **Launcher independence**: open the app from the OS launcher, no terminal window required. Tauri Rust backend spawns the `hick` binary as a child process.\n2. **Native window management**: configurable panes at OS level, persistent layout, multi-window.\n3. **System tray**: background operation; tray icon surfaces AFK state, token cost, agent activity.\n4. **Protocol handlers / deep links**: `hick://session/xyz` to jump to a specific conversation.\n5. **Native OS notifications**: toast when a long-running task completes.\n\n## Architecture\n\n```\n[Tauri webview]  — same React app (hick-agent/web-ui/)\n       ↕  tauri::invoke IPC\n[Tauri Rust backend]\n       ↕  local Unix socket  (embedded mode)\n[hick child process]  — full agent with its own session\n```\n\nFor the **remote/observe mode** (reading another session without owning it — see federation below), the Tauri backend opens a second WebSocket connection to the target session's relay URL, read-only.\n\n`resolveWebSocketUrl.ts` is the only web-UI file that needs a Tauri-aware code path (returns local socket URL via `tauri::invoke` instead of a relay URL).\n\n## Conversation federation\n\nA **conversations list** view shows all known sessions across the user's processes and machines:\n- Local sessions: discovered via the hick session registry (a small local SQLite or append-only file that each hick process writes its active session ID + relay URL to on startup).\n- Remote sessions (same user, different machine): discovered via the eval store git-sync (TASK-44) which carries session metadata.\n- Active sessions are flagged with their relay URL so the Tauri app can open an observe connection.\n\nIn **observe mode** the Tauri app connects to a foreign session's relay as a read-only client. It renders the conversation but the input bar is hidden (or shows a \"this is a live view\" banner). No `UiCommand` messages are sent. The existing relay already supports multiple browser connections; observe mode is just a browser connection that sends no commands.\n\n## Presence awareness\n\nEach frontend announces its type when it connects to a session relay:\n```json\n{ \"type\": \"presence_join\", \"client_kind\": \"terminal\" | \"browser\" | \"tauri\", \"client_id\": \"<uuid>\" }\n```\nOn disconnect:\n```json\n{ \"type\": \"presence_leave\", \"client_id\": \"<uuid>\" }\n```\nThe relay broadcasts these to all session members. Each UI maintains a presence list and shows it as an ambient indicator:\n- TUI: a compact status-line section `[also open: browser, tauri]`\n- Web-UI / Tauri: a small presence bar `● terminal  ● browser` with coloured dots.\n\nPresence indicators make it clear when multiple people or processes are watching the same session.\n\n## Draft prompt sync\n\nNon-submitted prompt text (and file attachments) is synced across all frontends connected to the same session, so a draft started in the TUI appears in the Tauri app and vice versa.\n\nProtocol (relay message, broadcast to all session members except sender):\n```json\n{ \"type\": \"draft_update\", \"client_id\": \"<uuid>\", \"text\": \"...\", \"attachments\": [ { \"name\": \"foo.rs\", \"size\": 1234 } ] }\n```\nRules:\n- Sent on every keystroke (debounced 150 ms).\n- Last-write-wins: whichever client most recently updated the draft owns it; other clients update their input field.\n- File attachment metadata is synced (name, size, MIME); the binary content is NOT transferred over the relay (too large). A receiving client shows the attachment as a badge with a `[file not available locally]` warning if the path does not exist on that machine.\n- Draft sync only applies within a single session. Opening the conversations list and switching sessions clears the local draft.\n\n## IDE replacement vision (unchanged from original)\n\nConfigurable panes, MCP-driven syntax highlighting (CodeMirror 6 / Monaco backed by semble + code-review-graph LSP shim), and graph-data gutter icons (test coverage, churn, impact radius, knowledge gaps). See original TASK-53 description for full detail.\n\n## Deliverables\n1. ADR in `docs/` covering ownership model, embedded vs remote/observe modes, federation design, and LSP-shim tradeoff.\n2. Prototype: Tauri shell + embedded hick child + web-UI loaded, conversation works end-to-end.\n3. Presence protocol implemented in relay bridge + shown in TUI status line and web-UI.\n4. Draft sync protocol implemented; draft typed in TUI appears in Tauri webview within 300 ms.\n5. Conversations list view in web-UI and Tauri webview showing local + remote sessions.\n6. MCP-driven highlighting spike (Rust, hover type via semble + code-review-graph, CodeMirror 6 or Monaco).\n7. Gutter icon prototype: coverage dots + change-risk heat map from graph data.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 ADR written covering ownership model, embedded vs observe mode, federation, presence, draft sync, and MCP-IDE tradeoff
- [ ] #2 Tauri prototype builds on Linux and macOS: embedded hick child starts, conversation works, no terminal required
- [ ] #3 System tray icon reflects agent state (idle / active / AFK) and current session token cost
- [ ] #4 Conversations list shows local sessions and, after git-sync, sessions from a second machine
- [ ] #5 Observe mode: connecting to a foreign session renders its conversation read-only; input bar is hidden
- [ ] #6 Presence protocol: terminal and browser frontends show each other's presence within 1 s of connect/disconnect
- [ ] #7 Draft sync: text typed in TUI input bar appears in Tauri webview within 300 ms (debounced)
- [ ] #8 File attachment metadata syncs; non-local files show a [not available locally] badge
- [ ] #9 Pane configuration schema documented; editor + conversation pane types render in prototype
- [ ] #10 Rust hover type works via semble + code-review-graph in at least one file (no traditional LSP)
- [ ] #11 Gutter icons: coverage dots and change-risk heat map shown in prototype
<!-- AC:END -->
