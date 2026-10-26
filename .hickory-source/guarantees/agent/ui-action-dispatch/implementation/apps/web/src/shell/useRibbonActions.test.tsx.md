<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/web/src/shell/useRibbonActions.test.tsx

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-2089413b681b"></a>
## `apps/web/src/shell/useRibbonActions.test.tsx`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [Explicit UI action dispatch checks scope, state, policy and retries](../../../../../contract.md).

<a id="f-2089413b681b-expression_statement"></a>
### `expression_statement`

<storycode:copy id="f-2089413b681b-expression_statement">
import { useState } from "react";
import { act, cleanup, fireEvent, render, screen } from "@testing-library/react";
import { afterEach, expect, it, vi } from "vitest";
import { EditorState } from "@codemirror/state";
import { EditorView } from "@codemirror/view";
import { UiActions } from "../lib/uiActions";
import { UiActionContext } from "../lib/uiActionBridge";
import { registerReviewEditor, REVIEW_TARGET } from "../lib/reviews";
import { useRibbonActions, type RibbonActionShape } from "./useRibbonActions";
afterEach(cleanup);
</storycode:copy>
<a id="f-2089413b681b-expression_statement-2"></a>
### `expression_statement`

<storycode:copy id="f-2089413b681b-expression_statement-2">
// Guarantee: docs/guarantees/agent/ui-action-dispatch.md
it("reveals a measured connection before shared navigation/review and refuses changed source", async () => {
  const host = document.createElement("div"); document.body.append(host);
  const view = new EditorView({ parent: host, state: EditorState.create({ doc: "claim\n" }) });
  const unregister = registerReviewEditor(view, "note.md");
  const shape: RibbonActionShape = { key: "one", family: "lineage", target: { kind: "document", path: "upstream.md", span: [0, 5] }, hl: [{ view, from: 0, to: 5 }], states: [view.state] };
  const navigate = vi.fn(), reviews: unknown[] = [];
  const listener = (event: Event) => reviews.push((event as CustomEvent).detail);
  window.addEventListener(REVIEW_TARGET, listener);
  const registry = new UiActions({ workspace: "p", window: "w" }, () => true, async () => {});
  let id = 0;
  const action = (label: string) => registry.groups().groups.flatMap(group => registry.observe(group).actions).find(action => action.label === label)!;
  const run = (label: string) => registry.act(registry.scope, { id: "agent", kind: "agent" }, { handle: action(label).handle, arguments: {}, request_id: String(++id) });
  function Harness() {
    const [visible, setVisible] = useState(false);
    const invoke = useRibbonActions([shape], () => visible, (_shape, value) => setVisible(value), navigate);
    return <button onClick={() => invoke(shape, "navigate")}>Follow native ribbon</button>;
  }
  const rendered = render(<UiActionContext.Provider value={registry}><Harness /></UiActionContext.Provider>);
  try {
    expect(action("Follow lineage connection 1").enabled).toBe(false);
    expect(await run("Follow lineage connection 1")).toMatchObject({ status: "disabled" });
    await act(async () => { await run("Reveal lineage connection 1"); });
    expect(await run("Follow lineage connection 1")).toMatchObject({ outcome: { value: { navigation_requested: true } } });
    await act(async () => fireEvent.click(screen.getByRole("button", { name: "Follow native ribbon" })));
    expect(navigate).toHaveBeenCalledTimes(2);
    expect(navigate).toHaveBeenLastCalledWith(shape);
    await act(async () => { expect(await run("Review lineage connection 1")).toMatchObject({ outcome: { value: { review_requested: true, human_resolution: false } } }); });
    expect(reviews).toMatchObject([{ path: "note.md", quote: "claim", kind: "relationship", relationship: { target: shape.target } }]);
    const old = action("Follow lineage connection 1");
    act(() => view.dispatch({ changes: { from: 0, to: 5, insert: "other" } }));
    expect(await registry.act(registry.scope, { id: "agent", kind: "agent" }, { handle: old.handle, arguments: {}, request_id: "old-source" })).toMatchObject({ status: "stale" });
    expect(action("Follow lineage connection 1").enabled).toBe(false);
    expect(navigate).toHaveBeenCalledTimes(2);
  } finally { rendered.unmount(); unregister(); view.destroy(); host.remove(); window.removeEventListener(REVIEW_TARGET, listener); }
});
</storycode:copy>

### Assemble `apps/web/src/shell/useRibbonActions.test.tsx`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/shell/useRibbonActions.test.tsx"><storycode:paste select="#f-2089413b681b-expression_statement"/><storycode:paste select="#f-2089413b681b-expression_statement-2"/></storycode:file>

</storycode:doc>
