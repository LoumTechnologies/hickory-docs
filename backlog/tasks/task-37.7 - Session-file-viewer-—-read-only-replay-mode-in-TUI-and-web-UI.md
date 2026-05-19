---
id: TASK-37.7
title: Session file viewer — read-only replay mode in TUI and web UI
status: Done
assignee: []
created_date: '2026-05-19 11:56'
updated_date: '2026-05-19 17:33'
labels:
  - web
  - tui
  - sdk
dependencies:
  - TASK-37.2
  - TASK-37.5
parent_task_id: TASK-37
priority: medium
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Both the TUI and web UI should be able to open a completed session XML bundle (produced by task-36's `XmlSessionLog`) and display it in read-only mode. This is useful for reviewing past agent runs, debugging, and sharing session logs with teammates.

## What to build

### Session bundle reader (Rust, in hick-agent-sdk)

A `SessionBundleReader` that reads a `session.xml` file from a bundle directory and emits the recorded events as a `Vec<UiEvent>` (or streams them via a channel). It parses the Agent Log XML format (namespace `http://www.hickorydocs.com/agent-log/1.0`) and maps each element to the corresponding `UiEvent` variant.

```rust
pub struct SessionBundleReader;

impl SessionBundleReader {
    pub fn read(bundle_dir: &Path) -> anyhow::Result<Vec<UiEvent>>;
    pub async fn stream(bundle_dir: &Path, tx: mpsc::Sender<UiEvent>) -> anyhow::Result<()>;
}
```

### TUI read-only mode

Add a `--replay <session-dir>` flag to copper-ledge and iron-peak. When present:
- Load the session bundle and feed all events into the TUI as if they arrived live (with a configurable replay speed: `--replay-speed 1.0` for real-time, `0` for instant)
- The input bar is hidden / disabled; no commands can be sent
- A header banner reads: `[REPLAY] session-20260519-143000`

### Web UI read-only mode

Two entry points:
1. **URL parameter**: `http://localhost:7890/?replay=<path-to-session-dir>` — the running agent web server opens the session file and streams it over the existing WebSocket
2. **Drag-and-drop**: User drags a `session.xml` file (or the bundle directory as a zip) onto the web UI; the browser parses it client-side using a TypeScript `SessionBundleParser` and replays it into the conversation tree

The web UI shows a read-only banner and disables the input bar when in replay mode. A playback scrubber appears in the toolbar (skip to timestamp, pause, resume).

### TypeScript session parser

A `SessionBundleParser` class in `src/session-parser.ts` that parses the `session.xml` DOM and emits `UiEvent[]`. Uses the browser's built-in `DOMParser` — no external XML library needed.

## Constraints

- The reader must handle sessions produced by older versions gracefully (unknown elements are skipped, not errors)
- Drag-and-drop replay must work entirely client-side (no server round-trip) so users can share session XML files and open them offline
- The `--replay` flag is mutually exclusive with normal agent operation (cannot replay and run simultaneously)
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 SessionBundleReader in hick-agent-sdk parses session.xml and returns Vec<UiEvent>
- [x] #2 --replay <dir> flag on copper-ledge and iron-peak launches TUI in read-only mode with the session events
- [x] #3 TUI read-only mode hides the input bar and shows the REPLAY banner
- [x] #4 Web UI accepts a ?replay=<path> URL parameter and streams the session over WebSocket
- [x] #5 Web UI accepts drag-and-drop of a session.xml file and replays it client-side
- [x] #6 Playback scrubber in web UI allows jumping to any timestamp
- [x] #7 Unknown XML elements in the session file are skipped without error
- [x] #8 cargo test --workspace passes
<!-- AC:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Implemented full read-only replay mode for both TUI and web UI.

**Rust (hick-agent-sdk)**
- Added `session_bundle_reader.rs` with `SessionBundleReader::read()` and `::stream()`. Line-by-line parser maps `log:*` elements → `UiEvent` variants; unknown elements are silently skipped. `ActionStart`/`ActionEnd` share the same `node_id` so `ConversationTree::patch_action_end` works correctly. 9 new unit tests cover all element types, entity unescaping, node-id pairing, and unknown-element skipping.
- Exported `SessionBundleReader` from `lib.rs`.
- Added `/ws/replay?path=<dir>` WebSocket endpoint to `web_server.rs` that reads a session bundle and streams all events over the socket.

**TUI (hick-agent-tui)**
- Added `replay_label: Option<String>` to `TuiConfig`.
- In replay mode: `handle_key` blocks all text input (only scroll/fold/quit keys work); `render` shows a yellow `[REPLAY] <label>` banner in the header, hides the input bar entirely (layout changes from 3-area to 2-area).

**Binaries**
- `copper-ledge`: `--replay <dir>` and `--replay-speed <f>` flags; `run_replay()` function that feeds bundle events into TUI.
- `iron-peak`: `replay` subcommand with identical behaviour.
- Both existing `TuiConfig` constructions updated with `replay_label: None`.

**Web UI (TypeScript)**
- `session-parser.ts`: `SessionBundleParser.parse()` uses `DOMParser` to parse session.xml client-side and returns `TimestampedEvent[]`; unknown elements skipped.
- `useAgentSocket.ts`: detects `?replay=<path>` query param; connects to `/ws/replay?path=...` instead of `/ws`; `send()` is a no-op in replay mode; returns `isServerReplay`.
- `resolveWebSocketUrl.ts`: updated to accept optional `path` and `query` overrides.
- `App.tsx`: replay banner (amber), drag-and-drop drop zone accepting `session.xml` files, playback scrubber (range input with ⏮/⏭ buttons and timestamp display), hidden input bar in any replay mode.
<!-- SECTION:FINAL_SUMMARY:END -->
