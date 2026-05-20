---
id: TASK-37.6
title: Adaptive Cards renderer — extension UI protocol for web and TUI
status: In Progress
assignee: []
created_date: '2026-05-19 11:55'
updated_date: '2026-05-20 01:17'
labels:
  - web
  - frontend
  - sdk
dependencies:
  - TASK-37.5
  - TASK-37.2
parent_task_id: TASK-37
priority: medium
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Extensions can contribute rich UI elements to all surfaces via Microsoft Adaptive Cards. This task wires up the Adaptive Cards renderer in the web UI and adds a graceful TUI fallback, plus defines the extension-side protocol for emitting cards.

## What to build

### Extension protocol (Rust side)

Extensions emit an Adaptive Card by calling `session_log.record(LogEvent::Custom { xml_fragment, attachments: &[] })` where `xml_fragment` is:

```xml
<log:adaptive-card node-id="42" parent-id="37">
{ ...Adaptive Card JSON... }
</log:adaptive-card>
```

The `UiEvent` bridge (task-37.1) reads this XML fragment and emits `UiEvent::AdaptiveCard { node_id, parent_id, json }`. Add a helper function in `hick-agent-sdk` to make this ergonomic:

```rust
pub fn emit_adaptive_card(log: &dyn SessionLog, id_gen: &IdGen, parent_id: NodeId, card_json: &str);
```

### Web UI renderer

In the React app, detect `UiEvent::AdaptiveCard` nodes in the conversation tree. Render them using `@microsoft/adaptivecards` (already installed in task-37.4):

```tsx
import * as AdaptiveCards from "@microsoft/adaptivecards";

function AdaptiveCardNode({ json }: { json: string }) {
  const ref = useRef<HTMLDivElement>(null);
  useEffect(() => {
    const card = new AdaptiveCards.AdaptiveCard();
    card.parse(JSON.parse(json));
    ref.current?.replaceChildren(card.render());
  }, [json]);
  return <div ref={ref} className="adaptive-card-host" />;
}
```

Apply a minimal host config that matches the app's dark theme (card background, font family, accent color).

### TUI fallback

In `hick-agent-tui`, `UiEvent::AdaptiveCard` nodes are rendered as a gray block showing the card's `body` text fields extracted from the JSON (no layout, just content). Cards with no readable text fields show a placeholder: `[Adaptive Card — view in web UI]`.

### Example card (for testing)

Add a test in `hick-agent-sdk` that calls `emit_adaptive_card` and verifies the resulting `xml_fragment` round-trips correctly through the `UiEvent` bridge to produce a well-formed `UiEvent::AdaptiveCard`.

## Constraints

- The AC renderer must not crash on malformed card JSON — catch parse errors and show an error placeholder
- The `@microsoft/adaptivecards` package version must be pinned in `package.json`
- Do not use the Adaptive Cards designer or host config editor — just the core renderer
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 emit_adaptive_card() helper exists in hick-agent-sdk and produces a well-formed log:adaptive-card XML fragment
- [ ] #2 UiEvent::AdaptiveCard is produced by the bridge when the XML fragment is detected
- [ ] #3 Web UI renders Adaptive Cards using @microsoft/adaptivecards in the conversation tree
- [ ] #4 Malformed card JSON shows an error placeholder instead of crashing
- [ ] #5 TUI renders the text content of AC card body fields; shows placeholder for cards with no readable text
- [ ] #6 A round-trip test in hick-agent-sdk verifies emit → bridge → UiEvent::AdaptiveCard
- [ ] #7 cargo test --workspace passes
<!-- AC:END -->
