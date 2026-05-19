---
id: TASK-37.11
title: Agent panes — named markdown side-panels for persistent agent-driven context
status: To Do
assignee: []
created_date: '2026-05-19 17:57'
labels:
  - web
  - tui
  - sdk
  - mobile-ready
dependencies:
  - TASK-37.5
  - TASK-37.7
parent_task_id: TASK-37
priority: medium
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Add a named markdown pane system that lets agent extensions push persistent context panels (source documents, status, plots, etc.) alongside the conversation tree — without any platform-specific knowledge in the agent code.

## Design principles

- The agent controls **content only** — never layout, never chrome. The shell decides where panes appear and how many to show at once.
- **Gradual, reversible**: emitting zero panes produces the normal UI. Each pane is independently opt-in. There is no "eject" point.
- **Cross-platform by construction**: the event carries markdown text (works in TUI and replay); images are served as local files (works in browser and mobile).
- **No hick-specific knowledge in any frontend**: iron-peak's hick-edit extension simply writes a hick XML file as a fenced code block in markdown — the UI sees only generic markdown.

## `.agents/panes/` directory convention

A new well-known project directory: `<project_dir>/.agents/panes/`.

- Agent extensions write pane content files here: `source.md`, `status.md`, etc.
- Image files (`.png`, `.svg`, `.jpg`) referenced by those markdown files live alongside them.
- Markdown files use **relative paths** for images: `![plot](./plot.png)`.
- The directory is gitignored by default (it is ephemeral agent output, not source).

This is the agent's entire API for pane content — write a file, the UI updates.

## `PaneContent` UiEvent

A new variant added to `UiEvent` in `hick-agent-sdk`:

```rust
PaneContent {
    node_id: NodeId,
    /// Stable pane identity — matches the markdown filename without extension.
    /// e.g. "source", "status", "diff"
    name: String,
    /// Display label shown in the pane header by the shell.
    title: String,
    /// Sanitized markdown text.  No raw HTML permitted.
    /// Carried in the event for TUI rendering and session replay.
    /// Images are referenced by relative path (resolved via /pane-files/).
    markdown: String,
}
```

Multiple events with the same `name` **replace** previous content (not append). Empty `markdown` clears the pane.

## Web server: `/pane-files/` route

Add a static-file route to `WebServer` that serves files from `<project_dir>/.agents/panes/`:

- `GET /pane-files/{path}` — serves the file at `.agents/panes/{path}`.
- Strict path containment: reject any path that escapes `.agents/panes/` (no `..` traversal).
- Correct `Content-Type` headers for `.png`, `.svg`, `.jpg`, `.gif`, `.webp`.
- The browser resolves `./plot.png` in pane markdown against this base URL.

## Pane watcher (hick-agent-sdk or binary crates)

A `PaneWatcher` that monitors `.agents/panes/*.md` for changes using `notify` (or polling fallback) and emits `PaneContent` events into the UI broadcast channel. Binary crates (copper-ledge, iron-peak) start the watcher at agent startup.

On startup the watcher does an initial scan so existing pane files are loaded before the first user turn.

## TUI rendering

- The TUI layout gains an optional right-hand pane column when `PaneContent` events have been received.
- Toggle visibility with a key (e.g. `Tab`).
- Markdown is rendered as styled text: headings bold+underline, fenced code blocks with a border, bullet lists with `•`, images replaced by `[image: alt text]`.
- Multiple panes are shown as a vertical stack or navigable with `[` / `]`.

## Web UI rendering

- Named panes appear as tabs or an accordion in the sidebar (or a dedicated right panel).
- Markdown rendered with a sanitizing renderer (no raw HTML passthrough).
- Images fetched from `/pane-files/` — the browser resolves relative paths automatically once the base URL is set.
- Unknown pane names render as generic panels using `title` as the label.

## Session bundle / replay integration

When a session ends (or periodically), snapshot `.agents/panes/` into the bundle:

```
<bundle-dir>/
  session.xml
  panes/
    source.md
    plot.png
```

The replay web server serves `/pane-files/` from `<bundle-dir>/panes/` instead of the live project directory, so replayed sessions show the pane content as it was at recording time.

## Mobile notes (not implemented here)

- The web server must bind to a LAN-reachable address (not just `127.0.0.1`) for mobile clients to fetch `/pane-files/` images. That is a separate configuration concern.
- SwiftUI can render the markdown text from `PaneContent` events directly; images are fetched over the local network from `/pane-files/`.
- No changes to the event model are needed for mobile support.

## Out of scope

- Arbitrary HTML in pane content (explicitly disallowed — sanitize it out).
- Agent control of pane layout or position.
- More than one level of subdirectory in `.agents/panes/` (keep it flat for now).
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 PaneContent UiEvent variant added to hick-agent-sdk with name, title, markdown fields
- [ ] #2 PaneWatcher monitors .agents/panes/*.md and emits PaneContent events on create/modify/delete
- [ ] #3 copper-ledge and iron-peak start PaneWatcher at agent startup and perform initial scan
- [ ] #4 Web server serves GET /pane-files/{path} from <project_dir>/.agents/panes/ with path containment
- [ ] #5 Web UI renders named panes as tabs/panels in the sidebar with sanitized markdown and image support
- [ ] #6 TUI renders pane markdown as styled text; images replaced by [image: alt text]; toggle with Tab
- [ ] #7 Unknown pane names render as generic panels (no frontend changes required for new pane names)
- [ ] #8 Empty markdown in a PaneContent event clears/hides the pane
- [ ] #9 Session bundle snapshots .agents/panes/ at session end; replay serves pane files from the snapshot
- [ ] #10 No raw HTML passthrough in any pane renderer
- [ ] #11 cargo test --workspace passes
<!-- AC:END -->
