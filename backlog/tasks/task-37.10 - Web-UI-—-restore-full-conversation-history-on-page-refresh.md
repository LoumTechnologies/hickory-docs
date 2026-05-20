---
id: TASK-37.10
title: Web UI — restore full conversation history on page refresh
status: In Progress
assignee: []
created_date: '2026-05-19 17:36'
updated_date: '2026-05-20 01:17'
labels:
  - ui
  - web
dependencies: []
parent_task_id: TASK-37
priority: high
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
When the browser page is refreshed, the web UI currently starts with an empty event log and only displays events received after reconnection. The LLM retains full context (it reads from the session XML log), but the UI does not — so if the user types a message after refresh, the response is contextually correct but appears to come out of nowhere with no prior conversation visible.

## Problem

The WebSocket connection is stateless from the UI's perspective. On reconnect, the server only forwards new events. The client has no way to distinguish "first connect" from "reconnect after refresh", so it cannot request a history replay.

## Required Behaviour

On (re)connect, the server must replay all `UiEvent`s that have occurred in the current session before the client begins receiving live events. The client must render these replayed events identically to live events, resulting in a fully populated conversation view indistinguishable from a page that was never refreshed.

## Acceptance Criteria
<!-- AC:BEGIN -->
- Refreshing the browser mid-session shows the complete conversation history up to the point of refresh, followed by any new events without a visible seam.
- A user who types a message after refresh sees both the prior conversation and the new exchange in the UI.
- The history replay happens before any new live events are forwarded, so ordering is always correct.
- No duplicate events appear if the client reconnects while events are actively being emitted.

- [ ] #1 Refreshing the browser mid-session shows the complete prior conversation, not a blank slate.
- [ ] #2 New events received after refresh appear after the replayed history with no visible gap or duplicate.
- [ ] #3 History replay is ordered correctly even when events are actively emitted during reconnect.
- [ ] #4 The fix applies to the local web server; relay/remote mode is out of scope for this subtask.
<!-- AC:END -->



## Implementation Notes

The server-side session event buffer (in `hick-agent-sdk`'s web server) should accumulate all `UiEvent`s for the lifetime of the agent process. On each new WebSocket connection, replay the buffer in order before switching to live forwarding. A simple `Vec<UiEvent>` snapshot under a read lock is sufficient — no pagination needed at this stage.
<!-- SECTION:DESCRIPTION:END -->
