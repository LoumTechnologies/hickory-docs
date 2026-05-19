---
id: TASK-37.1
title: Unified UiEvent/UiCommand + ConversationTree model in hick-agent-sdk
status: Done
assignee: []
created_date: '2026-05-19 11:54'
updated_date: '2026-05-19 13:01'
labels:
  - sdk
  - ui
dependencies: []
parent_task_id: TASK-37
priority: high
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Currently each extension crate (hick-agent-script-first, hick-agent-engine) defines its own `AgentEvent`/`AgentCommand` enums, and each binary crate hardwires to those types. This prevents any shared UI layer. This task defines the canonical UI event model in `hick-agent-sdk` that all UIs — TUI, local web, and remote relay — will consume.

## What to build

### `UiEvent` enum (in `hick-agent-sdk::ui_event`)

A UI-facing event superset of today's per-extension `AgentEvent`. Every variant carries a `NodeId` so the conversation tree can be built from the stream. Key variants:

- `SessionStarted { node_id, model, sandbox_image, provider_info }`
- `UserTurn { node_id, parent_id, text }`
- `Thinking { node_id, parent_id }`
- `Token { node_id, text }` — streaming token delta
- `AssistantDone { node_id, text }` — full response
- `ActionStart { node_id, parent_id, lang, code, collapsed_by_default: true }`
- `ActionEnd { node_id, stdout, stderr, exit_code, collapsed_by_default: true }`
- `SubAgentSpawned { node_id, parent_id, label }` — signals a tree branch
- `SubAgentDone { node_id }` — closes the branch
- `McpCall { node_id, parent_id, server, method }` — collapsed by default
- `McpResponse { node_id, server, method }` — collapsed by default
- `SecurityViolation { node_id, description }`
- `NetworkApprovalRequest { node_id, host, suggestions, response_tx }` — same approval flow as today
- `AdaptiveCard { node_id, parent_id, json }` — extension-contributed card
- `Info { node_id, text }` — status messages
- `Error { node_id, text }`
- `Done { node_id }` — turn complete
- `Interrupted { node_id }`

### `UiCommand` enum (in `hick-agent-sdk::ui_event`)

- `Message(String)` — user submitted a prompt
- `DirectCommand(String)` — `! <shell>` pass-through
- `Cancel` — interrupt the current operation
- `Quit`
- `ExpandNode(NodeId)` / `CollapseNode(NodeId)` — for UIs that track collapse state server-side

### `NodeId` type

A `u64` counter (or newtype over it), incremented atomically by an `IdGen` helper. Serializes to/from string for JSON/XML transport.

### `ConversationTree`

An append-only in-memory tree built by calling `ConversationTree::apply(UiEvent)`. Each node stores: `id`, `parent_id`, `kind`, `timestamp`, `collapsed_by_default`, and opaque content. The tree is the single render model used by all UIs. Children of a node are ordered by insertion; nodes with no parent are roots.

### Bridge adapters

Add bridge functions/impls in the extension crates to convert their existing `AgentEvent` → `UiEvent` and `UiCommand` → `AgentCommand`. This keeps the extension internals unchanged while enabling the shared UI layer.

## Files to create/modify

- `hick-agent-sdk/src/ui_event.rs` — `UiEvent`, `UiCommand`, `NodeId`, `IdGen`
- `hick-agent-sdk/src/conversation_tree.rs` — `ConversationTree`, `ConversationNode`, `NodeKind`
- `hick-agent-sdk/src/lib.rs` — add `pub use` for new types
- `hick-agent-script-first/src/events.rs` — add `AgentEvent::to_ui_event()` bridge
- `hick-agent-engine/src/events.rs` — same bridge if applicable

## Constraints

- `UiEvent` must be `Clone + Send + Sync`.
- `NodeId` must be serializable as a plain integer in both XML attributes and JSON.
- The `NetworkApprovalRequest` variant carries a `oneshot::Sender` — this is the same approval channel used today; it is not serialized for remote transport (remote relay uses a separate approval round-trip).
- No UI rendering code in this task — just the data model.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 UiEvent, UiCommand, NodeId, IdGen are defined in hick-agent-sdk::ui_event and re-exported from lib.rs
- [x] #2 Every UiEvent variant carries a NodeId; branching variants (UserTurn, ActionStart, SubAgentSpawned, etc.) carry a parent_id
- [x] #3 ConversationTree::apply(UiEvent) builds the in-memory tree; children are ordered by insertion
- [x] #4 collapsed_by_default is true on ActionStart, ActionEnd, McpCall, McpResponse variants
- [x] #5 AgentEvent → UiEvent bridge is implemented in hick-agent-script-first
- [x] #6 cargo test --workspace passes
<!-- AC:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Implemented `hick-agent-sdk` modules `ui_event` (`UiEvent`, `UiCommand`, `NodeId`, `IdGen`, `UiNetworkApproval`) and `conversation_tree` (`ConversationTree::apply`, ordered children, tests). Added `hick-agent-script-first::agent_ui_bridge` with `AgentUiBridge`, `AgentEvent::into_ui_events`, and `ui_command_to_agent` / `agent_command_to_ui`. Removed orphan duplicate `hick-agent-engine/src/events.rs` (crate already re-exported script-first events via `lib.rs`).
<!-- SECTION:NOTES:END -->
