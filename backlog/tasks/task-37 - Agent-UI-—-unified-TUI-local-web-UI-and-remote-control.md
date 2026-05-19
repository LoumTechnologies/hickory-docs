---
id: TASK-37
title: 'Agent UI — unified TUI, local web UI, and remote control'
status: To Do
assignee: []
created_date: '2026-05-19 11:53'
labels:
  - ui
  - sdk
  - architecture
dependencies: []
priority: high
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Agent binaries (copper-ledge, iron-peak) currently each own a ~950-line tui.rs with nearly identical ratatui code, hardwired to their extension's AgentEvent/AgentCommand types. The goal of this initiative is to make UI a zero-cost concern for binary authors: they wire extensions together and get a production-quality TUI, local web UI, and optional remote control for free.

## Outcome

- Binary crates have no UI code. They call a single entry point.
- All UIs (TUI, local web, remote web) consume a unified `UiEvent`/`UiCommand` model defined in `hick-agent-sdk`.
- The conversation is rendered as a tree (git-log style), branching when sub-agents are spawned or parallel tasks are launched from the same point.
- Script source code and output are collapsed by default in all UIs (expandable by the user).
- Extensions can contribute rich UI elements via Microsoft Adaptive Cards (JSON, rendered natively in the web UI, degraded gracefully in TUI).
- A cloud relay service hosted on a Hickory domain enables remote control from any device with no port forwarding — agent connects outbound to the relay, remote browsers connect to the same relay session.
- Session XML bundles (task-36, completed) can be opened in read-only/replay mode in both the TUI and web UI.

## Architecture layers (subtask build order)

1. task-37: Unified UiEvent/UiCommand + ConversationTree model in SDK — prerequisite for everything
2. task-38: Shared hick-agent-tui crate — replaces per-binary tui.rs
3. task-39: Local web server (axum + WebSocket) in SDK — prerequisite for web UI and relay
4. task-40: React/TypeScript frontend skeleton (Vite, WebSocket, tree view)
5. task-41: Conversation tree visualization with collapsible nodes
6. task-42: Adaptive Cards renderer + extension protocol
7. task-43: Session file viewer (read-only mode in TUI and web)
8. task-44: hick-relay binary + RelayClient in SDK
9. task-45: Remote control web UI mode

## Key decisions already made

- `UiEvent`/`UiCommand` live in `hick-agent-sdk`; extensions bridge their own event types to these.
- The TUI moves to a new `hick-agent-tui` crate (separate from SDK to avoid ratatui/crossterm as SDK deps).
- The React SPA is pre-built and committed; binary crates use `include_bytes!` — no JS toolchain needed at `cargo build` time.
- The local web server listens on localhost:7890 by default.
- The relay is content-agnostic (forwards bytes, does not parse events). Auth is a random session UUID printed at agent startup.
- Adaptive Cards are emitted via `LogEvent::Custom` with a `log:adaptive-card` XML element; the web UI renders them with `@microsoft/adaptivecards`.
- The relay service is hosted on a Hickory domain (not self-hosted by the user).

## Foundation

task-36 (Agent Log XML session logging) is complete. The `SessionLog` trait and `LogEvent` enum in `hick-agent-sdk` are the session event foundation this work builds on.
<!-- SECTION:DESCRIPTION:END -->
