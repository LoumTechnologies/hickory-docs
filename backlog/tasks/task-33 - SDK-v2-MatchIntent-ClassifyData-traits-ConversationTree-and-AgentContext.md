---
id: TASK-33
title: 'SDK v2: MatchIntent/ClassifyData traits, ConversationTree, and AgentContext'
status: To Do
assignee: []
created_date: '2026-05-17 10:49'
updated_date: '2026-05-17 10:52'
labels:
  - sdk
  - architecture
  - refactor
milestone: m-1
dependencies: []
priority: high
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Refactor `hick-agent-sdk` to replace the monolithic `AgentRuntime` / `ReactLoop` design with a trait-based, composable architecture. This unlocks config-driven extension composition (TASK-27) and gives third-party extension authors stable, well-typed extension points.

## Motivation

Currently `ReactLoop` hard-codes response parsing (looking for `<hick:next>` tags and fenced code blocks) and security checks are scattered across `SinkGuard` / `AgentSecurityConfig`. There is no stable trait for "here is a new kind of LLM output syntax" or "here is a new category of sensitive data". Adding either requires forking `ReactLoop`.

## New primitives

### `MatchIntent` — extensible response parser

```rust
pub trait MatchIntent: Send + Sync {
    type IntentPayload: Send;

    /// Unique identifier used in config and routing.
    fn name(&self) -> &'static str;

    /// Inspect a completed LLM message and extract a typed payload if it matches.
    fn parse(&self, message: &Message) -> Option<Self::IntentPayload>;
}
```

Built-in impls live in extension crates:
- `hick-agent-script-first` → `ScriptBlockIntentParser` (fenced code block extraction)
- Future: `GraphqlBlockIntentParser`, `HickEditIntentParser`, etc.

### `ClassifyData` — background security / taint scanner

```rust
pub trait ClassifyData: Send + Sync {
    /// Canonical name for this classification (e.g. `"phi"`, `"secrets"`).
    fn classification_name(&self) -> &'static str;

    /// Return true if the message content matches this classification.
    fn scan(&self, message: &Message) -> bool;
}
```

Classifiers are registered on `ConversationTree`. The executor checks `history.get_active_classifications()` before running any tool — the loop author never writes security checks inline.

### `ConversationTree` — typed, taint-aware history

Replaces the flat `Vec<Message>` on `AgentRuntime`. Key additions:
- Each node carries a `HashSet<String>` of active data classifications (propagated from parent nodes).
- `.branch()` creates a child sub-tree that inherits parent classifications.
- `.merge(sub, MergeStrategy)` folds results back into the parent branch.
- `.is_idle()` / `.mark_idle()` track the daemon state.
- `.is_sub_branch()` returns `true` when this tree was created via `.branch()`.

`MergeStrategy` variants (v1): `SummarizeConformant` (LLM-summarize the sub-branch and append), `AppendRaw` (append sub-branch messages verbatim).

### `AgentContext` — dependency bag passed into the loop

```rust
pub struct AgentContext {
    pub llm: Box<dyn LlmClient>,
    pub router: IntentRouter,
    pub executor: Arc<dyn ToolExecutor>,
    pub io: Arc<dyn AgentIo>,
}
```

`IntentRouter` holds the registered `MatchIntent` impls and returns `LoopIntent`:

```rust
pub enum LoopIntent {
    CallTool(ToolData),
    Custom(String, Box<dyn Any + Send>),
    DelegateTask(String),
    TaskComplete(String),
    None,
}
```

The loop dispatches on `LoopIntent`; all security enforcement is inside `executor.run()`, not visible in the loop body.

### The canonical loop

```rust
fn run_agent_engine(ctx: &AgentContext, history: &mut ConversationTree, running: Arc<AtomicBool>) {
    while running.load(Ordering::SeqCst) {
        if history.is_idle() {
            history.push(Message::user(ctx.io.await_next_input_packet()));
        }
        let response = ctx.llm.chat(history).unwrap_or_else(|e| e.to_message());
        history.push(response.clone());
        match ctx.router.determine_intent(&response) {
            LoopIntent::CallTool(tool_data) => {
                history.push(ctx.executor.run(tool_data).unwrap_or_else(|e| e.to_message()));
            }
            LoopIntent::Custom(name, payload) => {
                history.push(ctx.executor.run_custom(&name, payload).unwrap_or_else(|e| e.to_message()));
            }
            LoopIntent::DelegateTask(prompt) => {
                let mut sub = history.branch();
                sub.push(Message::system(prompt));
                run_agent_engine(ctx, &mut sub, running.clone());
                history.merge(sub, MergeStrategy::SummarizeConformant).unwrap();
            }
            LoopIntent::TaskComplete(summary) => {
                ctx.io.print_status_update(&summary);
                if history.is_sub_branch() { return; } else { history.mark_idle(); }
            }
            LoopIntent::None => ctx.io.stream_chunk(&response),
        }
    }
}
```

## Binary crate migration (no shims)

`AgentRuntime` and the old `ReactLoop` public API are deleted — no compatibility shims. Both binary crates are updated in this task to speak the new API directly:

- **`copper-ledge`**: replace `ReactLoop::new(...).with_executor(...).with_sink_guard(...)` construction with `AgentContext` + `ConversationTree` + `IntentRouter` wiring a `ScriptBlockIntentParser`.
- **`iron-peak`**: same, plus register a `HickEditIntentParser` from `hick-agent-hick-edit`.

The TUI layer in both binaries is unchanged — it communicates over `AgentEvent`/`AgentCommand` channels which are independent of the runtime.

Updating the binaries is the integration test that the new API is ergonomic. The task is not done until both binaries compile and their existing tests pass against the new API.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 `MatchIntent`, `ClassifyData`, `ConversationTree`, `AgentContext`, `IntentRouter`, and `LoopIntent` are defined and exported from `hick-agent-sdk`.
- [ ] #2 `ConversationTree` tracks per-node data classifications; `.branch()` inherits parent classifications; `.merge(sub, strategy)` folds results back.
- [ ] #3 `IntentRouter` accepts a list of registered `MatchIntent` impls and returns `LoopIntent` by trying each parser in priority order.
- [ ] #4 The canonical `run_agent_engine` loop compiles and passes a unit test with a mock LLM, mock executor, and at least one custom `MatchIntent` impl.
- [ ] #5 `AgentRuntime` and the old `ReactLoop` public API are deleted — no shims remain in `hick-agent-sdk`.
- [ ] #6 `hick-agent-script-first`'s fenced-block parsing is extracted into a `ScriptBlockIntentParser` struct implementing `MatchIntent`; the old `ReactLoop` entry point is replaced.
- [ ] #7 `copper-ledge` is updated to build an `AgentContext` with `ScriptBlockIntentParser`; it compiles and all existing tests pass.
- [ ] #8 `iron-peak` is updated to build an `AgentContext` with both `ScriptBlockIntentParser` and `HickEditIntentParser`; it compiles and all existing tests pass.
- [ ] #9 All existing tests in `hick-agent-sdk` and `hick-agent-script-first` pass after the refactor.
<!-- AC:END -->
