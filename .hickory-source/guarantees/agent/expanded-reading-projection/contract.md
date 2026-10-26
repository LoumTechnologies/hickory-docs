<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Selected reading scope remains separate from complete edit state

For: Authors using an agent to change documents while retaining review control.

Given a workspace conversation, when the user selects documents in **Next context
projection**, the next capture includes their expanded source text or explicitly
pinned full text, independently of scrolling, focus and visibility. Unselected
open files contribute identities and edit snapshots, not model-visible text.
Fold state is retained across editor unmount/remount for the same source bytes.
An unavailable or changed fold state omits text until the editor is reopened or pinned.
Collapsing changes future captures; it never erases historical model exposure.

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Read the detailed requirements

- [Requirements 1](requirements/01.md)


Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/agent/expanded-reading-projection.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.


## Read the retained requirement detail

- [Requirement detail](requirements/01.md)
<storycode:include file="requirements/01.md" />

## Implementation

<a id="output-f139c8442934"></a>
## `apps/agent/tests/test_reading_projection.py`

Choose and deliver agent context while preserving the host protocol and the evidence behind a turn.

Related contracts: [Selected reading scope remains separate from complete edit state](contract.md).

<a id="f-f139c8442934-ReadingProjectionTests"></a>
### `ReadingProjectionTests`

<storycode:copy id="f-f139c8442934-ReadingProjectionTests">
import json
import unittest
from hickory_agent_worker.core import Agent, Config, reading_context


class ReadingProjectionTests(unittest.TestCase):
    # Guarantee: docs/guarantees/agent/expanded-reading-projection.md
    def test_projection_never_supplies_full_edit_snapshot(self):
        value = {'editors': {'buffers': [{'id': 'draft', 'content': 'secret folded body',
                  'observations': ['unselected']}], 'projection': {'version': 1,
                  'documents': [], 'observations': []}}}
        result = json.loads(reading_context(json.dumps(value)))
        self.assertEqual(result['editors']['buffers'], [{'id': 'draft'}])
        self.assertEqual(value['editors']['buffers'][0]['content'], 'secret folded body')

    def test_legacy_and_explicit_empty_scope_are_distinct(self):
        legacy = {'buffers': [{'content': 'legacy'}]}
        self.assertEqual(json.loads(reading_context(json.dumps(legacy))), legacy)
        self.assertEqual(reading_context('opaque adapter text'), 'opaque adapter text')


    def test_actual_model_request_contains_projection_without_complete_buffer(self):
        requests = []
        class Model:
            def complete_request(self, bundle):
                requests.append(bundle)
                return '{"done":"ok"}'
        class Tools:
            def complete(self, summary):
                return {'accepted': True}
        context = json.dumps({'editors': {'buffers': [{'id': 'draft', 'content': 'HIDDEN-BODY'}],
            'projection': {'version': 1, 'documents': [{'id': 'draft', 'revision': 'captured-revision',
            'segments': [{'start': 0, 'end': 7, 'text': 'VISIBLE', 'omitted': False}]}], 'observations': []}}})
        Agent(Model(), Tools(), None).run(Config('Task', context=context))
        parts = requests[0]['request']['parts']
        supplied = '\n'.join(p['text'] for p in parts)
        self.assertNotIn('HIDDEN-BODY', supplied)
        self.assertIn('VISIBLE', supplied)
        self.assertIn('captured-revision', supplied)
</storycode:copy>

### Assemble `apps/agent/tests/test_reading_projection.py`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/agent/tests/test_reading_projection.py"><storycode:paste select="#f-f139c8442934-ReadingProjectionTests"/></storycode:file>

<a id="output-abcd7546bb10"></a>
## `apps/web/src/debug/cmRuntime.controls.test.tsx`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [Selected reading scope remains separate from complete edit state](contract.md).

<a id="f-abcd7546bb10-view"></a>
### `view`

<storycode:copy id="f-abcd7546bb10-view">
// Guarantee: docs/guarantees/agent/expanded-reading-projection.md
import { act, cleanup, render } from "@testing-library/react";
import { afterEach, expect, it, vi } from "vitest";
import { EditorState } from "@codemirror/state";
import { EditorView } from "@codemirror/view";
import { UiActions, type UiValue } from "../lib/uiActions";
import { UiActionContext } from "../lib/uiActionBridge";
import { useEditorActions } from "../editor/uiEditorActions";
import { runtimeEditor, setRuntimeReading } from "./cmRuntime";
import { selectedObservations, type ObservationReading } from "./observations";
import { RUNTIME_REVEAL } from "./runtimeNavigation";
let view: EditorView | undefined;
</storycode:copy>
<a id="f-abcd7546bb10-expression_statement"></a>
### `expression_statement`

<storycode:copy id="f-abcd7546bb10-expression_statement">
afterEach(() => { cleanup(); const host = view?.dom.parentElement; view?.destroy(); host?.remove(); });
</storycode:copy>
<a id="f-abcd7546bb10-expression_statement-2"></a>
### `expression_statement`

<storycode:copy id="f-abcd7546bb10-expression_statement-2">
it("shares native inline disclosure/log navigation, reads only explicit expansion and retires source owners", async () => {
  const source = "code\n";
  const scope = { run: "inline-run", stop: "runtime", frame: null, invocation: null };
  const reading: ObservationReading = { schema_version: 1, source: { id: "inline.md", uri: "inline.md", revision: "captured", bytes: [...new TextEncoder().encode(source)] }, scope,
    locations: [{ id: "L", revision: "captured", line: 1, location: { source: "inline.md", bytes: { start: 0, end: 4 }, precision: "exact" } }],
    annotations: [{ evidence: "event-one", provider: "runtime", payload_schema: "runtime/v1", location: "L", scope, payload: { kind: "runtime", event: { observation: { kind: "log", level: "info", message: "CAPTURED BODY", fields: {} } } } }], unsupported: ["not verification"] };
  const host = document.body.appendChild(document.createElement("div"));
  const current = new EditorView({ parent: host, state: EditorState.create({ doc: source, extensions: runtimeEditor }) }); view = current;
  const registry = new UiActions({ workspace: "p", window: "inline" }, () => true, async () => {});
  function Bridge() { useEditorActions(current, "inline"); return null; }
  render(<UiActionContext.Provider value={registry}><Bridge /></UiActionContext.Provider>);
  const actions = () => registry.groups().groups.flatMap(group => registry.observe(group, 64).actions);
  let id = 0;
  const run = async (label: string, args: Record<string, UiValue>) => { let result: unknown; await act(async () => { result = await registry.act(registry.scope, { id: "acp:inline", kind: "agent" }, { handle: actions().find(action => action.label === label)!.handle, arguments: args, request_id: String(++id) }); }); return result; };
  const disclose = "Set inline runtime annotation disclosure", read = "Read expanded inline runtime annotation bytes", reveal = "Show inline runtime event in execution log";
  await act(async () => current.dispatch({ effects: setRuntimeReading.of(reading) }));
  expect(JSON.stringify(actions())).not.toContain("CAPTURED BODY");
  expect(actions().some(action => action.label === read)).toBe(false);
  expect(selectedObservations("inline.md", source)).toEqual([]);
  await run(disclose, { index: 0, open: true });
  const native = current.dom.querySelector<HTMLDetailsElement>("details.runtime-inline")!;
  native.scrollIntoView = vi.fn();
  expect(native.open).toBe(true);
  expect(selectedObservations("inline.md", source)[0]).toMatchObject({ evidence: ["event-one"] });
  const reply = await run(read, { index: 0, offset: 0, limit: 8192 }) as { outcome: { value: { bytes: number[] } } };
  expect(new TextDecoder().decode(new Uint8Array(reply.outcome.value.bytes))).toContain("CAPTURED BODY");
  const navigation = vi.fn(); window.addEventListener(RUNTIME_REVEAL, navigation);
  try {
    expect(await run(reveal, { index: 0, event: 0 })).toMatchObject({ outcome: { value: { navigation_requested: true } } });
    await act(async () => native.querySelector<HTMLButtonElement>("button")!.click());
    expect(navigation).toHaveBeenCalledTimes(2);
    expect(native.scrollIntoView).toHaveBeenCalledTimes(2);
    await act(async () => { native.open = false; native.dispatchEvent(new Event("toggle")); });
    expect(actions().some(action => action.label === read)).toBe(false);
    expect(selectedObservations("inline.md", source)).toEqual([]);
    await run(disclose, { index: 0, open: true });
    const old = actions().find(action => action.label === read)!;
    const oldButton = native.querySelector<HTMLButtonElement>("button")!;
    await act(async () => current.dispatch({ changes: { from: 0, insert: "new " } }));
    expect(selectedObservations("inline.md", source)).toEqual([]);
    expect(await registry.act(registry.scope, { id: "acp:inline", kind: "agent" }, { handle: old.handle, arguments: { index: 0, offset: 0, limit: 8192 }, request_id: "stale" })).not.toMatchObject({ outcome: { status: "completed" } });
    await act(async () => oldButton.click());
    expect(navigation).toHaveBeenCalledTimes(2);
  } finally { window.removeEventListener(RUNTIME_REVEAL, navigation); }
});
</storycode:copy>

### Assemble `apps/web/src/debug/cmRuntime.controls.test.tsx`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/debug/cmRuntime.controls.test.tsx"><storycode:paste select="#f-abcd7546bb10-view"/><storycode:paste select="#f-abcd7546bb10-expression_statement"/><storycode:paste select="#f-abcd7546bb10-expression_statement-2"/></storycode:file>

<a id="output-cd54a8e364bd"></a>
## `apps/web/src/lib/contextProjection.test.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [Selected reading scope remains separate from complete edit state](contract.md), [Select model context without saving open buffers](../the-agent-sees-the-open-editors/contract.md).

<a id="f-cd54a8e364bd-expression_statement"></a>
### `expression_statement`

<storycode:copy id="f-cd54a8e364bd-expression_statement">
import { expect, it, vi } from "vitest";
// @ts-expect-error Node runtime crypto is available to Vitest without browser Node typings.
import { webcrypto } from "node:crypto";
import { EditorState } from "@codemirror/state";
import { codeFolding, foldEffect } from "@codemirror/language";
import { projectDocument } from "./contextProjection";
import type { EditorView } from "@codemirror/view";
vi.stubGlobal("crypto", webcrypto);
</storycode:copy>
<a id="f-cd54a8e364bd-expression_statement-2"></a>
### `expression_statement`

<storycode:copy id="f-cd54a8e364bd-expression_statement-2">
// Guarantee: docs/guarantees/agent/expanded-reading-projection.md
it("preserves Unicode source coordinates, duplicate passages and omitted ranges while pins bypass folds", async () => {
  const content = "α\nsame\nhidden\nsame";
  const state = EditorState.create({ doc: content, extensions: [codeFolding()] }).update({ effects: foldEffect.of({ from: 6, to: 13 }) }).state;
  const view = { state } as EditorView;
  const result = await projectDocument("unsaved", content, view);
  expect(result.segments).toEqual([
    { start: 0, end: 7, line_start: 1, line_end: 2, text: "α\nsame", omitted: false },
    { start: 7, end: 14, line_start: 2, line_end: 3, text: "", omitted: true },
    { start: 14, end: 19, line_start: 3, line_end: 4, text: "\nsame", omitted: false },
  ]);
  const pinned = await projectDocument("unsaved", content, view, true);
  expect(pinned.segments.map(s => s.text).join("")).toBe(content);
  expect(pinned.revision).toBe(result.revision);
  expect((await projectDocument("other", content)).id).toBe("other");
  expect((await projectDocument("unsaved", content + "edit")).revision).not.toBe(result.revision);
});

</storycode:copy>
<a id="f-cd54a8e364bd-expression_statement-3"></a>
### `expression_statement`

<storycode:copy id="f-cd54a8e364bd-expression_statement-3">
// Guarantee: docs/guarantees/agent/expanded-reading-projection.md
it("retains fold intent across editor unmounts and refuses stale source coordinates", async () => {
  const { EditorView } = await import("@codemirror/view");
  const { rememberReadingState, restoreReadingState, retainedReadingState } = await import("./contextProjection");
  const content = "head\nhidden\ntail";
  const first = new EditorView({ state: EditorState.create({ doc: content, extensions: [codeFolding()] }) });
  first.dispatch({ effects: foldEffect.of({ from: 4, to: 11 }) });
  const original = await projectDocument("a", content, first);
  rememberReadingState("unique-document", first);
  first.destroy();
  expect(await projectDocument("a", content, retainedReadingState("unique-document"))).toEqual(original);
  const second = new EditorView({ state: EditorState.create({ doc: content, extensions: [codeFolding()] }) });
  try {
    restoreReadingState("unique-document", second);
    expect(await projectDocument("a", content, second)).toEqual(original);
    const stale = await projectDocument("a", "changed unsaved source", retainedReadingState("unique-document"));
    expect(stale.selection).toBe("unavailable");
    expect(stale.segments.every(s => s.omitted)).toBe(true);
  } finally { second.destroy(); }
});
</storycode:copy>

### Assemble `apps/web/src/lib/contextProjection.test.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/lib/contextProjection.test.ts"><storycode:paste select="#f-cd54a8e364bd-expression_statement"/><storycode:paste select="#f-cd54a8e364bd-expression_statement-2"/><storycode:paste select="#f-cd54a8e364bd-expression_statement-3"/></storycode:file>

## Implementation review

### Implementation

- [apps/web/src/debug/cmRuntime.ts](../../../mechanisms/apps/web/src/debug/cmRuntime.ts.md#output-2517e693c335)
- [apps/web/src/lib/contextProjection.ts](../../../mechanisms/apps/web/src/lib/contextProjection.ts.md#output-5194716bada7)
- [crates/hick-literate/src/session_receipts.rs](../../../mechanisms/crates/hick-literate/src/session_receipts.rs.md#output-4e6c982edf60)
- [crates/hickory-cli/src/serve/agent_context.rs](../../../mechanisms/crates/hickory-cli/src/serve/agent_context.rs.md#output-792dad4f8311)

### Tests and fixtures

- [apps/agent/tests/test_reading_projection.py](contract.md#output-f139c8442934)
- [apps/web/src/debug/ObservationReading.test.tsx](../../../mechanisms/apps/web/src/debug/ObservationReading.test.tsx.md#output-ec33c3e05a60)
- [apps/web/src/debug/cmRuntime.controls.test.tsx](contract.md#output-abcd7546bb10)
- [apps/web/src/lib/contextProjection.test.ts](contract.md#output-cd54a8e364bd)
- [apps/web/src/views/WorkspaceChat.test.tsx](../../../mechanisms/apps/web/src/views/WorkspaceChat.test.tsx.md#output-41ee32c19327)

### Focused witnesses

- `preserves Unicode source coordinates, duplicate passages and omitted ranges while pins bypass folds` in `apps/web/src/lib/contextProjection.test.ts`.
- `retains fold intent across editor unmounts and refuses stale source coordinates` in `apps/web/src/lib/contextProjection.test.ts`.

### Evidence status

- Implementation review: not verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.
- Open evidence gaps: no-existing-code-or-test-reference-resolved.

</storycode:doc>
