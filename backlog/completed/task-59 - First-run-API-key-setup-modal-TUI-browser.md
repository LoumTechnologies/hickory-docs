---
id: TASK-59
title: First-run API key setup modal (TUI + browser)
status: Done
assignee: []
created_date: '2026-05-21 12:07'
updated_date: '2026-05-21 12:24'
labels: []
dependencies: []
priority: high
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
When no API keys are configured, show a setup modal in both the TUI and browser UI that:
- Asks for API keys for Anthropic, OpenAI, and/or xAI
- Prominently emphasizes security: "Your keys never leave your machine. Hick's built-in secrets scanner detects and redacts credentials before they ever reach the LLM's context."
- Saves entered keys to ~/.hick/keys.toml
- Blocks agent startup until at least one key is provided

Architecture:
1. Add `UiEvent::AwaitingApiKey` + `UiCommand::ProvideApiKeys` to SDK
2. Create `hick-agent-tui/src/setup.rs` — blocking ratatui setup screen
3. Update web-ui types + add SetupModal React component
4. Restructure main.rs: channels + web server start BEFORE LLM build; setup coordination via AtomicBool
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 UiEvent::AwaitingApiKey and UiCommand::ProvideApiKeys exist in SDK with wire serialization
- [x] #2 TUI shows a fullscreen setup screen with security messaging and key fields when no keys configured
- [x] #3 Browser shows a modal with security pitch and provider key fields
- [x] #4 Either surface can complete setup; the other sees the transition
- [x] #5 Keys are saved to ~/.hick/keys.toml on completion
- [x] #6 cargo test --workspace -- --test-threads=1 passes
<!-- AC:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Implemented first-run API key setup modal across TUI and browser surfaces.

**SDK** (`hick-agent-sdk/src/ui_event.rs`):
- Added `UiEvent::AwaitingApiKey { node_id }` with wire serialization (`type: "awaiting_api_key"`)
- Added `UiCommand::ProvideApiKeys { anthropic, openai, xai }` 
- Added `AwaitingApiKey` arm to `conversation_tree.rs` (no-op, UI-only signal)

**TUI** (`hick-agent-tui/src/setup.rs`, new file):
- Full-screen ratatui setup form with three provider fields (masked input)
- Security callout block: "Your keys never leave your machine. Hick's built-in secrets scanner detects and redacts credentials before they ever reach the LLM's context window."
- Tab/↑↓ navigation, Enter to submit, Ctrl-C to quit
- `run_setup_tui(done_flag, result_out)` — exports from `lib.rs`

**Browser** (`web-ui/src/SetupModal.tsx`, new file; `web-ui/src/App.tsx`):
- `SetupModal` component with emerald security callout + three password fields
- Shown when any `awaiting_api_key` event is present in `visibleEvents`
- Submits `provide_api_keys` command via WebSocket

**main.rs** restructuring:
- UI channels (`ui_fanout_tx`, `ui_cmd_tx`) created before provider detection
- Web server started early (single-config only) so browser is live during setup
- Setup check: if no keys → emit `AwaitingApiKey`, run TUI blocking + poll `ui_cmd_rx` for browser command concurrently via `AtomicBool` coordination
- On completion: save keys to `~/.hick/keys.toml`, inject env vars, continue to LLM build
- Late web server start skipped when early start succeeded (`ws_started_early` flag)
- Added `save_user_keys()` helper

**agent_ui_bridge.rs**: Added `ProvideApiKeys` to the `None` arm (UI-only, not forwarded to agent)

All tests pass; web UI rebuilt.
<!-- SECTION:FINAL_SUMMARY:END -->
