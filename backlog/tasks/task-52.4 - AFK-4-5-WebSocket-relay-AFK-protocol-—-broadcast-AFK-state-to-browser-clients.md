---
id: TASK-52.4
title: 'AFK 4/5: WebSocket relay AFK protocol — broadcast AFK state to browser clients'
status: To Do
assignee: []
created_date: '2026-05-20 12:36'
labels:
  - afk
  - relay
  - websocket
  - protocol
dependencies:
  - TASK-52.2
parent_task_id: TASK-52
priority: medium
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Extend the relay protocol with AFK state messages so the browser web-UI knows when to show/hide its modal.\n\n**New message types** (JSON, text frame from agent → browser):\n```json\n{ \"type\": \"afk_entered\", \"entry_time\": \"<ISO8601>\", \"config\": { \"cloud_budget_tokens\": 50000 } }\n{ \"type\": \"afk_update\", \"stats\": { \"duration_secs\": 42, \"input_tokens\": 1200, \"cached_tokens\": 800, \"output_tokens\": 340, \"cost_usd\": 0.0023, \"models\": [\"llama3-8b\"] }, \"activity\": [ { \"turn\": 1, \"summary\": \"...\", \"model\": \"llama3-8b\" } ] }\n{ \"type\": \"afk_exited\", \"summary\": { ... same as afk_update.stats ... } }\n```\n\n**New message type** (browser → agent):\n```json\n{ \"type\": \"afk_dismiss\" }\n```\nThe relay forwards this back to the agent; the agent's relay bridge translates it to `UiCommand::AfkDismiss`.\n\n**Agent side**: the binary crate's relay bridge (in `iron-peak` / future `hick` binary) subscribes to `UiEvent::AfkEntered/AfkUpdate/AfkExited` and serialises them to the WebSocket session. No change to `hick-relay` itself — it stays content-agnostic.\n\n**Reconnect / late join**: when a browser connects to an active AFK session, the agent sends an immediate `afk_entered` followed by an `afk_update` so the browser modal appears without waiting for the next tick.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 afk_entered frame arrives at connected browser within 200 ms of AfkEntered event
- [ ] #2 afk_update frames arrive approximately every 2 s while AFK is active
- [ ] #3 afk_exited frame arrives when AFK mode ends
- [ ] #4 Browser that connects mid-AFK-session receives afk_entered + afk_update immediately on connect
- [ ] #5 afk_dismiss from browser causes agent to call AfkManager::exit() within 500 ms
- [ ] #6 Message schema documented in a comment in the relay bridge source file
<!-- AC:END -->
