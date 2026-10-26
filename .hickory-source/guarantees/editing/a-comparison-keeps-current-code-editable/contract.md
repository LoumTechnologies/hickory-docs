<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# A comparison keeps current code editable

For: Authors comparing current code with another revision while continuing to edit.

Given a literate reading or persistent document, comparing it with a Git
revision leaves the current code in the main editor's actual buffer. Added lines
update as it is edited. Removed lines are shown by a minus marker in the gutter. Clicking it opens a
floating previous-version preview with an explicit restore action on editable
comparisons. No historical rows take space in the text. Removed bytes enter the
current buffer only through explicit restoration. They do not affect saved text or language-server coordinates.

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Read the detailed requirements

- [Requirements 1](requirements/01.md)


Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/editing/a-comparison-keeps-current-code-editable.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.


## Read the retained requirement detail

- [Requirement detail](requirements/01.md)
<storycode:include file="requirements/01.md" />

## Implementation

<a id="output-d9ad7a131c57"></a>
## `apps/web/src/editor/comparison.controls.test.tsx`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [A comparison keeps current code editable](contract.md).

<a id="f-d9ad7a131c57-views"></a>
### `views`

<storycode:copy id="f-d9ad7a131c57-views">
// Guarantee: docs/guarantees/editing/a-comparison-keeps-current-code-editable.md
import { act, cleanup, render, waitFor } from "@testing-library/react";
import { afterEach, expect, it } from "vitest";
import { EditorState, Transaction } from "@codemirror/state";
import { EditorView } from "@codemirror/view";
import { UiActions, type UiValue } from "../lib/uiActions";
import { UiActionContext } from "../lib/uiActionBridge";
import { useEditorActions } from "./uiEditorActions";
import { comparisonField, setComparison } from "./comparison";
const views: EditorView[] = [];
</storycode:copy>
<a id="f-d9ad7a131c57-expression_statement"></a>
### `expression_statement`

<storycode:copy id="f-d9ad7a131c57-expression_statement">
afterEach(() => { cleanup(); for (const view of views.splice(0)) { const host = view.dom.parentElement; view.destroy(); host?.remove(); } });
</storycode:copy>
<a id="f-d9ad7a131c57-setup"></a>
### `setup`

<storycode:copy id="f-d9ad7a131c57-setup">
function setup(removed: string, editable = true) {
  const events: string[] = [], host = document.body.appendChild(document.createElement("div"));
  const view = new EditorView({ parent: host, state: EditorState.create({ doc: "same\n", extensions: [comparisonField, EditorView.updateListener.of(update => { for (const tr of update.transactions) { const event = tr.annotation(Transaction.userEvent); if (event) events.push(event); } })] }) });
  views.push(view); view.dispatch({ effects: setComparison.of({ base: `same\n${removed}`, editable }) });
  const registry = new UiActions({ workspace: "comparison", window: "one" }, () => true, async () => {});
  function Bridge() { useEditorActions(view, "comparison"); return null; }
  render(<UiActionContext.Provider value={registry}><Bridge /></UiActionContext.Provider>);
  const actions = () => registry.groups().groups.flatMap(group => registry.observe(group, 64).actions);
  let id = 0;
  const run = async (label: string, args: Record<string, UiValue> = {}) => { let result: unknown; await act(async () => { result = await registry.act(registry.scope, { id: "acp:comparison", kind: "agent" }, { handle: actions().find(action => action.label === label)!.handle, arguments: args, request_id: String(++id) }); }); return result; };
  return { view, events, registry, actions, run };
}
</storycode:copy>
<a id="f-d9ad7a131c57-expression_statement-2"></a>
### `expression_statement`

<storycode:copy id="f-d9ad7a131c57-expression_statement-2">
it("discloses bounded historical bytes only after opening and restores through the real buffer", async () => {
  const removed = "HIDDEN-REMOVED-SOURCE " + "λ".repeat(6000) + "\n";
  const { view, events, registry, actions, run } = setup(removed);
  expect(JSON.stringify(actions())).not.toContain("HIDDEN-REMOVED-SOURCE");
  expect(actions().some(action => action.label === "Read comparison removed-source bytes")).toBe(false);
  const markers = await run("Read comparison removed-line marker page", { first: 0, count: 64 }) as { outcome: { value: { markers: { position: number }[] } } };
  const position = markers.outcome.value.markers[0].position;
  await run("Open comparison removed-line preview", { position });
  expect(JSON.stringify(actions())).not.toContain("HIDDEN-REMOVED-SOURCE");
  const read = await run("Read comparison removed-source bytes", { offset: 0, limit: 8192 }) as { outcome: { value: { bytes: number[]; next: number } } };
  expect(read.outcome.value.bytes).toHaveLength(8192);
  expect(read.outcome.value.next).toBe(8192);
  expect(new TextDecoder().decode(new Uint8Array(read.outcome.value.bytes))).toContain("HIDDEN-REMOVED-SOURCE");
  const retired = actions().find(action => action.label === "Read comparison removed-source bytes")!;
  await run("Close comparison removed-line preview");
  expect(actions().some(action => action.label === "Read comparison removed-source bytes")).toBe(false);
  expect(await registry.act(registry.scope, { id: "acp:comparison", kind: "agent" }, { handle: retired.handle, arguments: { offset: 0, limit: 20 }, request_id: "closed-read" })).not.toMatchObject({ outcome: { status: "completed" } });
  await run("Open comparison removed-line preview", { position });
  expect(await run("Restore comparison removed-source bytes")).toMatchObject({ outcome: { status: "completed", value: { source_changed: true, saved: false, verified: false } } });
  expect(view.state.doc.toString()).toBe(`same\n${removed}`);
  expect(events).toEqual(["input.uiAgentRestore"]);
});
</storycode:copy>
<a id="f-d9ad7a131c57-expression_statement-3"></a>
### `expression_statement`

<storycode:copy id="f-d9ad7a131c57-expression_statement-3">
it("refuses a stale native restore callback and keeps historical comparisons read-only", async () => {
  const { view, registry, actions, run } = setup("historical\n");
  await run("Open comparison removed-line preview", { position: 5 });
  const restoreAction = actions().find(action => action.label === "Restore comparison removed-source bytes")!;
  // Retain the actual native button across a source replacement.
  let button!: HTMLButtonElement;
  await waitFor(() => { button = [...document.querySelectorAll<HTMLButtonElement>(".comparison-removed button")].find(button => button.textContent === "Restore these lines")!; expect(button).toBeTruthy(); });
  await act(async () => view.dispatch({ changes: { from: 0, to: 4, insert: "changed" } }));
  const before = view.state.doc.toString();
  await act(async () => button.click());
  expect(view.state.doc.toString()).toBe(before);
  expect(await registry.act(registry.scope, { id: "acp:comparison", kind: "agent" }, { handle: restoreAction.handle, arguments: {}, request_id: "old-restore" })).not.toMatchObject({ outcome: { status: "completed" } });
  await act(async () => view.dispatch({ effects: setComparison.of({ base: `${before}historical\n`, editable: false }) }));
  const markers = await run("Read comparison removed-line marker page", { first: 0, count: 64 }) as { outcome: { value: { markers: { position: number }[] } } };
  await run("Open comparison removed-line preview", { position: markers.outcome.value.markers[0].position });
  expect(actions().some(action => action.label === "Read comparison removed-source bytes")).toBe(true);
  expect(actions().some(action => action.label === "Restore comparison removed-source bytes")).toBe(false);
});
</storycode:copy>

### Assemble `apps/web/src/editor/comparison.controls.test.tsx`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/editor/comparison.controls.test.tsx"><storycode:paste select="#f-d9ad7a131c57-views"/><storycode:paste select="#f-d9ad7a131c57-expression_statement"/><storycode:paste select="#f-d9ad7a131c57-setup"/><storycode:paste select="#f-d9ad7a131c57-expression_statement-2"/><storycode:paste select="#f-d9ad7a131c57-expression_statement-3"/></storycode:file>

## Implementation review

### Implementation

- [apps/web/src/editor/DocumentEditor.tsx](../../../mechanisms/apps/web/src/editor/DocumentEditor.tsx.md#output-aab82870e517)
- [apps/web/src/editor/comparison.ts](../../../mechanisms/apps/web/src/editor/comparison.ts.md#output-ceadb3379aac)
- [apps/web/src/lsp/representationMapping.ts](../../../mechanisms/apps/web/src/lsp/representationMapping.ts.md#output-237646581124)
- [apps/web/src/views/RepresentationTools.tsx](../../../mechanisms/apps/web/src/views/RepresentationTools.tsx.md#output-f5ea4f306ebf)
- [crates/hickory-cli/src/serve/revision.rs](../../../mechanisms/crates/hickory-cli/src/serve/revision.rs.md#output-84b229f3d940)

### Tests and fixtures

- [apps/web/e2e/literate-editor.spec.ts](../../../mechanisms/apps/web/e2e/literate-editor.spec.ts.md#output-648693644a28)
- [apps/web/src/editor/comparison.controls.test.tsx](contract.md#output-d9ad7a131c57)
- [apps/web/src/editor/comparison.test.ts](../../../mechanisms/apps/web/src/editor/comparison.ts.md#output-a13b9a534b46)
- [apps/web/src/lsp/cmLspFeatures.test.ts](../../editor-intelligence/save-can-format-first/contract.md#output-830f6d637cff)
- [apps/web/src/lsp/representationMapping.test.ts](../../../mechanisms/apps/web/src/lsp/representationMapping.ts.md#output-f3e0210e3ed7)
- [crates/hickory-cli/tests/literate_views.rs](../../../mechanisms/crates/hickory-cli/tests/literate_views.rs.md#output-3df513c37b3c)

### Evidence status

- Implementation review: not verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.
- Open evidence gaps: no-existing-code-or-test-reference-resolved.

</storycode:doc>
