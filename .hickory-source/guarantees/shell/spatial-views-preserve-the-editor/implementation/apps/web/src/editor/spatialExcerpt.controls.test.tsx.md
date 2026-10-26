<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/web/src/editor/spatialExcerpt.controls.test.tsx

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-c25f2be0aa63"></a>
## `apps/web/src/editor/spatialExcerpt.controls.test.tsx`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [Spatial Views Preserve The Editor](../../../../../contract.md).

<a id="f-c25f2be0aa63-view"></a>
### `view`

<storycode:copy id="f-c25f2be0aa63-view">
// Guarantee: docs/guarantees/shell/spatial-views-preserve-the-editor.md
import { act, cleanup, render } from "@testing-library/react";
import { afterEach, expect, it } from "vitest";
import { EditorState } from "@codemirror/state";
import { EditorView } from "@codemirror/view";
import { UiActions } from "../lib/uiActions";
import { UiActionContext } from "../lib/uiActionBridge";
import { useEditorActions } from "./uiEditorActions";
import { showSpatialExcerpt } from "./spatialExcerpt";
let view: EditorView | undefined;
</storycode:copy>
<a id="f-c25f2be0aa63-expression_statement"></a>
### `expression_statement`

<storycode:copy id="f-c25f2be0aa63-expression_statement">
afterEach(() => { cleanup(); const host = view?.dom.parentElement; view?.destroy(); host?.remove(); });
</storycode:copy>
<a id="f-c25f2be0aa63-expression_statement-2"></a>
### `expression_statement`

<storycode:copy id="f-c25f2be0aa63-expression_statement-2">
it("shares actual excerpt edges without reading hidden source or changing text and selection", async () => {
  const source = Array.from({ length: 40 }, (_, index) => `line ${index + 1}`).join("\n");
  const host = document.body.appendChild(document.createElement("div"));
  const current = new EditorView({ parent: host, state: EditorState.create({ doc: source }) }); view = current;
  const focal = current.state.doc.line(20).from;
  current.dispatch({ selection: { anchor: focal + 2 } });
  const selection = current.state.selection.toJSON();
  const registry = new UiActions({ workspace: "p", window: "excerpt" }, () => true, async () => {});
  function Bridge() { useEditorActions(current, "excerpt"); return null; }
  render(<UiActionContext.Provider value={registry}><Bridge /></UiActionContext.Provider>);
  const actions = () => registry.groups().groups.flatMap(group => registry.observe(group, 64).actions);
  let id = 0;
  const run = async (label: string) => { let result: unknown; await act(async () => { result = await registry.act(registry.scope, { id: "acp:excerpt", kind: "agent" }, { handle: actions().find(action => action.label === label)!.handle, arguments: {}, request_id: String(++id) }); }); return result; };
  expect(actions().some(action => action.label.includes("spatial excerpt"))).toBe(false);
  await act(async () => showSpatialExcerpt(current, focal, true));
  expect(JSON.stringify(actions())).not.toContain("line 20");
  expect(await run("Read spatial excerpt extent")).toMatchObject({ outcome: { value: { first: 18, last: 23, focal: 20, content_read: false } } });
  const oldButton = current.dom.querySelector<HTMLButtonElement>('[aria-label="Show three more lines above"]')!;
  expect(await run("Show three more spatial excerpt lines above")).toMatchObject({ outcome: { value: { first: 15, source_changed: false } } });
  await act(async () => oldButton.click());
  expect(await run("Read spatial excerpt extent")).toMatchObject({ outcome: { value: { first: 15 } } });
  expect(await run("Hide three spatial excerpt lines above")).toMatchObject({ outcome: { value: { first: 18 } } });
  expect(actions().some(action => action.label === "Hide three spatial excerpt lines above")).toBe(false);
  const retired = actions().find(action => action.label === "Read spatial excerpt extent")!;
  await act(async () => showSpatialExcerpt(current, null));
  expect(actions().some(action => action.label.includes("spatial excerpt"))).toBe(false);
  expect(await registry.act(registry.scope, { id: "acp:excerpt", kind: "agent" }, { handle: retired.handle, arguments: {}, request_id: "retired" })).not.toMatchObject({ outcome: { status: "completed" } });
  expect(current.state.doc.toString()).toBe(source);
  expect(current.state.selection.toJSON()).toEqual(selection);
});
</storycode:copy>

### Assemble `apps/web/src/editor/spatialExcerpt.controls.test.tsx`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/editor/spatialExcerpt.controls.test.tsx"><storycode:paste select="#f-c25f2be0aa63-view"/><storycode:paste select="#f-c25f2be0aa63-expression_statement"/><storycode:paste select="#f-c25f2be0aa63-expression_statement-2"/></storycode:file>

</storycode:doc>
