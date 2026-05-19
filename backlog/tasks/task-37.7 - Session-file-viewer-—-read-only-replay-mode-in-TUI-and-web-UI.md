---
id: TASK-37.7
title: Session file viewer — read-only replay mode in TUI and web UI
status: To Do
assignee: []
created_date: '2026-05-19 11:56'
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
- [ ] #1 SessionBundleReader in hick-agent-sdk parses session.xml and returns Vec<UiEvent>
- [ ] #2 --replay <dir> flag on copper-ledge and iron-peak launches TUI in read-only mode with the session events
- [ ] #3 TUI read-only mode hides the input bar and shows the REPLAY banner
- [ ] #4 Web UI accepts a ?replay=<path> URL parameter and streams the session over WebSocket
- [ ] #5 Web UI accepts drag-and-drop of a session.xml file and replays it client-side
- [ ] #6 Playback scrubber in web UI allows jumping to any timestamp
- [ ] #7 Unknown XML elements in the session file are skipped without error
- [ ] #8 cargo test --workspace passes
<!-- AC:END -->
