<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Observations read beside their captured source

For: Authors debugging generated code while navigating its document source.

Given a native DAP or browser interpreter session with captured UTF-8 source,
when its reading surface is expanded, then source excerpts and typed observations
share immutable revision references and evidence IDs. Requested breakpoints,
adapter-bound breakpoints and the actual paused statement are distinct facts.
Selecting a caller never moves the actual paused statement to that caller.
Values and explicit expression receipts identify their captured stop and frame;
invocation identity remains unknown when the provider cannot establish it.

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Read the detailed requirements

- [Requirements 1](requirements/01.md)


Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/debugging/source-anchored-observation-reading.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.


## Read the retained requirement detail

- [Requirement detail](requirements/01.md)
<storycode:include file="requirements/01.md" />

## Implementation

<a id="output-a714deb0aad7"></a>
## `apps/web/src/debug/ObservationReading.tsx`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [Observations read beside their captured source](contract.md).

<a id="f-a714deb0aad7-ObservationReading"></a>
### `ObservationReading`

<storycode:copy id="f-a714deb0aad7-ObservationReading">
import { useEffect, useId, useRef, useState } from "react";
import { useUiCommands, useUiRevision } from "../lib/uiActionBridge";
import { useUiReading } from "../lib/uiReading";
import { readingProviderAnnotations } from "./providerObservations";
import { selectObservations, type ObservationReading as Reading } from "./observations";

/** Both the human view and selected agent context consume these exact evidence IDs. */
export function ObservationReading({ reading, historical, changed = false }: { reading: Reading; historical: boolean; changed?: boolean }) {
  const owner = useRef(Symbol("observation-reading"));
  const [expanded, setExpanded] = useState(false);
  const [pinned, setPinned] = useState(false);
  useEffect(() => {
    selectObservations(reading, expanded || pinned, historical || changed, owner.current);
    return () => selectObservations(reading, false, historical, owner.current);
  }, [reading, expanded, pinned, historical, changed]);
  const parent = `source-observations:${useId()}`;
  const revision = useUiRevision(() => [reading, historical, changed, expanded, pinned]);
  useUiCommands([
    { key: `${parent}:open`, command: { parent, subject: parent, label: "Set source observation visibility", parameters: { open: { type: "boolean" } }, prerequisites: [],
      read: () => ({ revision: revision(), enabled: true, value: { open: expanded } }), execute: async args => { setExpanded(args.open as boolean); return { status: "completed", value: { open: args.open, context_selection_requested: true } }; } } },
    { key: `${parent}:pin`, command: { parent, subject: parent, label: "Set source observations pinned", parameters: { pinned: { type: "boolean" } }, prerequisites: ["Pinned evidence enters explicitly selected context; it remains captured/historical evidence"],
      read: () => ({ revision: revision(), enabled: true, value: { pinned } }), execute: async args => { setPinned(args.pinned as boolean); return { status: "completed", value: { pinned: args.pinned, context_selection_requested: true, provider_delivery_asserted: false } }; } } },
  ]);
  useUiReading("source observation disclosure", { expanded, pinned, scope: reading.scope, source_revision: reading.source.revision, historical, changed });
  return <><label><input type="checkbox" checked={pinned} onChange={event => setPinned(event.target.checked)} />Pin observations · {reading.scope.run} · {reading.scope.stop}</label><details data-observation-kind={reading.scope.stop === "runtime" ? "runtime" : "debugger"} className="debug-observations" open={expanded} onToggle={event => setExpanded(event.currentTarget.open)}>
    <summary>Source observations · {changed ? "source changed" : historical ? "historical capture" : reading.scope.stop === "runtime" ? "captured runtime" : "captured stop"}</summary>
    {expanded && <ObservationDetails reading={reading} historical={historical} changed={changed} />}
  </details></>;
}
</storycode:copy>
<a id="f-a714deb0aad7-ObservationDetails"></a>
### `ObservationDetails`

<storycode:copy id="f-a714deb0aad7-ObservationDetails">
function ObservationDetails({ reading, historical, changed }: { reading: Reading; historical: boolean; changed: boolean }) {
  const source = new TextDecoder().decode(new Uint8Array(reading.source.bytes));
  const providers = readingProviderAnnotations(reading);
  useUiReading("expanded source observations", {
    source_revision: reading.source.revision, scope: reading.scope, historical, changed,
    locations: reading.locations.map(location => ({ id: location.id, line: location.line, text: source.split("\n")[location.line],
      annotations: reading.annotations.filter(annotation => annotation.location === location.id) })),
    unmapped: reading.annotations.filter(annotation => annotation.location === null),
    providers: providers.map(annotation => ({ evidence: annotation.evidence, source_precision: annotation.source_precision,
      measurement_semantics: annotation.measurement_semantics ?? "not supplied", uncertainty: annotation.uncertainty ?? "not supplied" })),
    unsupported: reading.unsupported, scope_note: "Rendered captured evidence; not a current edit baseline, independent verification or proof of provider delivery",
  });
  return <>
    <p>Revision {reading.source.revision} · session {reading.scope.session ?? "unknown"} · run {reading.scope.run} · {reading.scope.stop === "runtime" ? "runtime observation batch; no debugger stop" : `stop ${reading.scope.stop}`}. Expanded evidence enters agent context.</p>
    {reading.locations.map(location => <div key={location.id} data-location={location.id}>
      <code>[{location.id}] {source.split("\n")[location.line]}</code>
      {reading.annotations.filter(a => a.location === location.id).map(a => <div key={a.evidence} data-evidence={a.evidence}>
        <strong>{a.payload.kind === "paused" ? "PAUSED HERE" : a.payload.kind.replaceAll("_", " ")}</strong>
        <code> {JSON.stringify(a.payload)}</code><small> · {a.evidence} · frame {a.scope.frame ?? "unknown"} · invocation {a.scope.invocation ?? "unknown"}</small>
      </div>)}
    </div>)}
    {reading.annotations.filter(a => a.location === null).map(a => <div key={a.evidence}>Unmapped · {a.evidence} · {JSON.stringify(a.payload)}</div>)}
    {providers.map(annotation => <small key={`provider-${annotation.evidence}`} data-provider-evidence={annotation.evidence}>
      {annotation.evidence} · source {annotation.source_precision} · measurement {annotation.measurement_semantics ?? "not supplied"} · uncertainty {annotation.uncertainty ?? "not supplied"}
    </small>)}
    <p>Unsupported: {reading.unsupported.join(", ")}</p>
  </>;
}
</storycode:copy>

### Assemble `apps/web/src/debug/ObservationReading.tsx`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/debug/ObservationReading.tsx"><storycode:paste select="#f-a714deb0aad7-ObservationReading"/><storycode:paste select="#f-a714deb0aad7-ObservationDetails"/></storycode:file>

<a id="output-d5d367dd354b"></a>
## `apps/web/src/debug/inlineEval.test.tsx`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [Observations read beside their captured source](contract.md).

<a id="f-d5d367dd354b-view"></a>
### `view`

<storycode:copy id="f-d5d367dd354b-view">
// Guarantee: docs/guarantees/debugging/source-anchored-observation-reading.md
import { act, cleanup, fireEvent, render, waitFor } from "@testing-library/react";
import { afterEach, expect, it, vi } from "vitest";
import { EditorState } from "@codemirror/state";
import { EditorView } from "@codemirror/view";
import { UiActions, type UiValue } from "../lib/uiActions";
import { UiActionContext } from "../lib/uiActionBridge";
import { useEditorActions } from "../editor/uiEditorActions";
import { inlineEvaluationOwner } from "./inlineEval";
import type { DebugSession } from "./useDebugger";
import { debugEditor, setPausedLine } from "./cmDebug";
let view: EditorView | undefined;
</storycode:copy>
<a id="f-d5d367dd354b-expression_statement"></a>
### `expression_statement`

<storycode:copy id="f-d5d367dd354b-expression_statement">
afterEach(() => { cleanup(); const host = view?.dom.parentElement; view?.destroy(); host?.remove(); });
</storycode:copy>
<a id="f-d5d367dd354b-setup"></a>
### `setup`

<storycode:copy id="f-d5d367dd354b-setup">
async function setup(query: (expression: string) => Promise<string | null>) {
  let owner = "stop-one";
  const watch = vi.fn(), host = document.body.appendChild(document.createElement("div"));
  const current = new EditorView({ parent: host, state: EditorState.create({ doc: "print(value)\n", extensions: debugEditor({ language: "python", onToggleBreakpoint: () => {}, onEvaluate: query, onAddWatch: watch, evaluationOwner: () => owner }) }) }); view = current;
  const registry = new UiActions({ workspace: "p", window: "inline-eval" }, () => true, async () => {});
  function Bridge() { useEditorActions(current, "inline-eval"); return null; }
  render(<UiActionContext.Provider value={registry}><Bridge /></UiActionContext.Provider>);
  await act(async () => current.dispatch({ effects: setPausedLine.of(0) }));
  const actions = () => registry.groups().groups.flatMap(group => registry.observe(group, 64).actions);
  await waitFor(() => expect(actions().some(action => action.label === "Set paused-line evaluation disclosure")).toBe(true));
  let id = 0;
  const run = async (label: string, args: Record<string, UiValue> = {}) => { let reply: unknown; await act(async () => { reply = await registry.act(registry.scope, { id: "acp:eval", kind: "agent" }, { handle: actions().find(action => action.label === label)!.handle, arguments: args, request_id: String(++id) }); }); return reply; };
  const read = async () => { const reply = await run("Read paused-line evaluation form bytes", { offset: 0, limit: 8192 }) as { outcome: { value: { bytes: number[] } } }; return JSON.parse(new TextDecoder().decode(new Uint8Array(reply.outcome.value.bytes))); };
  return { current, registry, actions, run, read, watch, replace: () => { owner = "stop-two"; current.dispatch({ effects: setPausedLine.of(0) }); } };
}
</storycode:copy>
<a id="f-d5d367dd354b-expression_statement-2"></a>
### `expression_statement`

<storycode:copy id="f-d5d367dd354b-expression_statement-2">
it("shares native/agent disclosure, draft, evaluation and watch without implicit value discovery", async () => {
  const query = vi.fn(async () => "private current value");
  const { current, actions, run, read, watch } = await setup(query);
  expect(actions().some(action => action.label === "Evaluate paused-line draft")).toBe(false);
  await run("Set paused-line evaluation disclosure", { open: true });
  const input = current.dom.querySelector<HTMLInputElement>(".cm-eval-input")!;
  await run("Set paused-line evaluation draft", { text: "value + 1" }); expect(input.value).toBe("value + 1");
  expect(await run("Evaluate paused-line draft")).toMatchObject({ outcome: { value: { evaluation_callback_completed: true, verified: false } } });
  expect(query).toHaveBeenCalledWith("value + 1");
  expect(JSON.stringify(actions())).not.toContain("private current value");
  expect(await read()).toMatchObject({ draft: "value + 1", result: "= private current value" });
  await act(async () => { fireEvent.input(input, { target: { value: "native expression" } }); fireEvent.keyDown(input, { key: "Enter", shiftKey: true }); });
  expect(watch).toHaveBeenCalledWith("native expression");
  await run("Watch paused-line evaluation draft"); expect(watch).toHaveBeenCalledTimes(2);
  await act(async () => fireEvent.keyDown(input, { key: "Escape" }));
  expect(actions().some(action => action.label === "Read paused-line evaluation form bytes")).toBe(false);
  await run("Set paused-line evaluation disclosure", { open: true });
  await run("Set paused-line evaluation draft", { text: "fresh draft" });
  await act(async () => fireEvent.input(input, { target: { value: "detached draft" } }));
  expect(await read()).toMatchObject({ draft: "fresh draft" });
  expect(current.state.doc.toString()).toBe("print(value)\n");
});
</storycode:copy>
<a id="f-d5d367dd354b-expression_statement-3"></a>
### `expression_statement`

<storycode:copy id="f-d5d367dd354b-expression_statement-3">
it("refuses late values after source/stop replacement and does not steal another field's focus", async () => {
  let resolve!: (value: string) => void;
  const query = vi.fn(() => new Promise<string>(done => { resolve = done; }));
  const { current, registry, actions, run, replace } = await setup(query);
  const field = document.body.appendChild(document.createElement("input")); field.value = "unsubmitted"; field.focus();
  expect(await run("Set paused-line evaluation disclosure", { open: true })).toMatchObject({ outcome: { status: "requires_input" } });
  expect(document.activeElement).toBe(field); field.remove();
  await run("Set paused-line evaluation disclosure", { open: true });
  await run("Set paused-line evaluation draft", { text: "value" });
  const nativeInput = current.dom.querySelector<HTMLInputElement>(".cm-eval-input")!;
  const oldToggle = current.dom.querySelector(".cm-eval-toggle");
  let pending!: Promise<unknown>;
  await act(async () => { pending = registry.act(registry.scope, { id: "acp:eval", kind: "agent" }, { handle: actions().find(action => action.label === "Evaluate paused-line draft")!.handle, arguments: {}, request_id: "pending" }); });
  await waitFor(() => expect(query).toHaveBeenCalledTimes(1));
  await act(async () => replace());
  await act(async () => resolve("RETIRED VALUE"));
  expect(await pending).not.toMatchObject({ outcome: { status: "completed" } });
  expect(current.dom.textContent).not.toContain("RETIRED VALUE");
  await act(async () => fireEvent.keyDown(nativeInput, { key: "Enter" })); expect(query).toHaveBeenCalledTimes(1);
  expect(oldToggle).toBeNull();
  await run("Set paused-line evaluation disclosure", { open: true });
  const old = actions().find(action => action.label === "Set paused-line evaluation draft")!;
  await act(async () => current.dispatch({ changes: { from: 0, insert: "# changed\n" } }));
  expect(await registry.act(registry.scope, { id: "acp:eval", kind: "agent" }, { handle: old.handle, arguments: { text: "other" }, request_id: "old" })).not.toMatchObject({ outcome: { status: "completed" } });
});

</storycode:copy>
<a id="f-d5d367dd354b-expression_statement-4"></a>
### `expression_statement`

<storycode:copy id="f-d5d367dd354b-expression_statement-4">
it("keeps an evaluation owner stable for value replies and changes it for a new frame or stop", () => {
  const debug = { status: "paused", program: "test.py", selectedFrame: 1, frames: [], reading: { source: { revision: "source" }, scope: { run: "run", stop: "one" } } } as unknown as DebugSession;
  const owner = inlineEvaluationOwner(debug);
  expect(inlineEvaluationOwner({ ...debug, lastValue: { expression: "value", value: "reply", type: null }, watches: [] })).toBe(owner);
  expect(inlineEvaluationOwner({ ...debug, selectedFrame: 2 })).not.toBe(owner);
  expect(inlineEvaluationOwner({ ...debug, frames: [] })).not.toBe(owner);
  expect(inlineEvaluationOwner({ ...debug, status: "running" })).toBeNull();
});
</storycode:copy>
<a id="f-d5d367dd354b-expression_statement-5"></a>
### `expression_statement`

<storycode:copy id="f-d5d367dd354b-expression_statement-5">
it("retains cancellation uncertainty without adopting a value from the dispatched request", async () => {
  let resolve!: (value: string) => void;
  const query = vi.fn(() => new Promise<string>(done => { resolve = done; }));
  const { current, registry, actions, run, read } = await setup(query);
  await run("Set paused-line evaluation disclosure", { open: true });
  await run("Set paused-line evaluation draft", { text: "value" });
  let pending!: Promise<unknown>;
  await act(async () => { pending = registry.act(registry.scope, { id: "acp:eval", kind: "agent" }, { handle: actions().find(action => action.label === "Evaluate paused-line draft")!.handle, arguments: {}, request_id: "cancel" }); });
  await waitFor(() => expect(query).toHaveBeenCalledTimes(1));
  expect(await run("Set paused-line evaluation draft", { text: "next expression" })).toMatchObject({ outcome: { status: "completed" } });
  expect(await read()).toMatchObject({ draft: "next expression", requested_expression: "value", pending: true });
  expect(registry.cancel(registry.scope, { id: "acp:eval", kind: "agent" }, "cancel")).toBe(true);
  await act(async () => resolve("CANCELLED VALUE"));
  expect(await pending).not.toMatchObject({ outcome: { status: "completed" } });
  expect(current.dom.textContent).not.toContain("CANCELLED VALUE");
  expect(await read()).toMatchObject({ pending: false, error: expect.stringContaining("may have executed") });
});

</storycode:copy>
<a id="f-d5d367dd354b-expression_statement-6"></a>
### `expression_statement`

<storycode:copy id="f-d5d367dd354b-expression_statement-6">
it("allows explicit close while evaluation is pending and reports possible dispatch", async () => {
  let resolve!: (value: string) => void;
  const query = vi.fn(() => new Promise<string>(done => { resolve = done; }));
  const { registry, actions, run } = await setup(query);
  await run("Set paused-line evaluation disclosure", { open: true });
  await run("Set paused-line evaluation draft", { text: "value" });
  let pending!: Promise<unknown>;
  await act(async () => { pending = registry.act(registry.scope, { id: "acp:eval", kind: "agent" }, { handle: actions().find(action => action.label === "Evaluate paused-line draft")!.handle, arguments: {}, request_id: "closing" }); });
  await waitFor(() => expect(query).toHaveBeenCalledTimes(1));
  expect(await run("Set paused-line evaluation disclosure", { open: false })).toMatchObject({ outcome: { value: { possible_pending_request: true, evaluation_cancelled: false } } });
  await act(async () => resolve("CLOSED VALUE"));
  expect(await pending).not.toMatchObject({ outcome: { status: "completed" } });
  expect(actions().some(action => action.label === "Read paused-line evaluation form bytes")).toBe(false);
});
</storycode:copy>

### Assemble `apps/web/src/debug/inlineEval.test.tsx`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/debug/inlineEval.test.tsx"><storycode:paste select="#f-d5d367dd354b-view"/><storycode:paste select="#f-d5d367dd354b-expression_statement"/><storycode:paste select="#f-d5d367dd354b-setup"/><storycode:paste select="#f-d5d367dd354b-expression_statement-2"/><storycode:paste select="#f-d5d367dd354b-expression_statement-3"/><storycode:paste select="#f-d5d367dd354b-expression_statement-4"/><storycode:paste select="#f-d5d367dd354b-expression_statement-5"/><storycode:paste select="#f-d5d367dd354b-expression_statement-6"/></storycode:file>

<a id="output-4abce3fbcd42"></a>
## `apps/web/src/debug/observations.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [Observations read beside their captured source](contract.md).

<a id="f-4abce3fbcd42-ObservationScope"></a>
### `ObservationScope`

<storycode:copy id="f-4abce3fbcd42-ObservationScope">
import { readingProviderAnnotations, type ProviderAnnotation } from "./providerObservations";
/** Tool-produced evidence, never parsed from source text or display labels. */
export interface ObservationScope { session?: string | null; run: string; stop: string; frame: number | null; invocation: string | null }
</storycode:copy>
<a id="f-4abce3fbcd42-ObservationAnnotation"></a>
### `ObservationAnnotation`

<storycode:copy id="f-4abce3fbcd42-ObservationAnnotation">
export interface ObservationAnnotation {
  evidence: string; provider: string; payload_schema: string; location: string | null;
  scope: ObservationScope; payload: { kind: string; [key: string]: unknown };
}
</storycode:copy>
<a id="f-4abce3fbcd42-ObservationReading"></a>
### `ObservationReading`

<storycode:copy id="f-4abce3fbcd42-ObservationReading">
export interface ObservationReading {
  provider_annotations?: ProviderAnnotation[];
  source_aliases?: string[];
  schema_version: number;
  source: { id: string; uri: string; revision: string; bytes: number[] };
  scope: ObservationScope;
  locations: { id: string; revision: string; line: number; location: { source: string; bytes: { start: number; end: number } | null; precision: string } }[];
  annotations: ObservationAnnotation[]; unsupported: string[];
}
</storycode:copy>
<a id="f-4abce3fbcd42-SelectedObservations"></a>
### `SelectedObservations`

<storycode:copy id="f-4abce3fbcd42-SelectedObservations">
export interface SelectedObservations { reading: ObservationReading; state: "captured" | "historical" | "changed"; evidence: string[] }
</storycode:copy>
<a id="f-4abce3fbcd42-selected"></a>
### `selected`

Explicit expanded scope, independent of captured event volume or visible pixels.

<storycode:copy id="f-4abce3fbcd42-selected">
// Explicit expanded scope, independent of captured event volume or visible pixels.
const selected = new Map<symbol | string, SelectedObservations>();
</storycode:copy>
<a id="f-4abce3fbcd42-selectObservations"></a>
### `selectObservations`

<storycode:copy id="f-4abce3fbcd42-selectObservations">
export function selectObservations(reading: ObservationReading, expanded: boolean, historical: boolean, owner?: symbol) {
  const key = owner ?? reading.scope.run;
  if (!expanded) { selected.delete(key); return; }
  selected.set(key, { reading, state: historical ? "historical" : "captured", evidence: reading.annotations.map(a => a.evidence) });
}
</storycode:copy>
<a id="f-4abce3fbcd42-selectedObservations"></a>
### `selectedObservations`

<storycode:copy id="f-4abce3fbcd42-selectedObservations">
export function selectedObservations(path: string, content: string): SelectedObservations[] {
  const unique = new Map([...selected.values()].map(s => [JSON.stringify([s.reading.source.id, s.reading.source.revision, s.reading.scope.run, s.reading.scope.stop, s.reading.scope.frame, s.evidence]), s]));
  const normalized = (uri: string) => uri.replace(/^hick:\/\/\//, "");
  return [...unique.values()].filter(s => [s.reading.source.uri, ...(s.reading.source_aliases ?? [])].some(uri => normalized(uri) === normalized(path))).map(s => ({ ...s,
    state: new TextDecoder().decode(new Uint8Array(s.reading.source.bytes)) === content ? s.state : "changed" }));
}

</storycode:copy>
<a id="f-4abce3fbcd42-observationContext"></a>
### `observationContext`

Same source excerpts, coordinates and evidence as the expanded view, without a byte-array dump.

<storycode:copy id="f-4abce3fbcd42-observationContext">
/** Same source excerpts, coordinates and evidence as the expanded view, without a byte-array dump. */
export function observationContext(path: string, content: string) {
  return selectedObservations(path, content).map(selected => {
    const { reading } = selected;
    return { state: selected.state, evidence: selected.evidence, reading: {
      schema_version: reading.schema_version,
      source: { id: reading.source.id, uri: reading.source.uri, revision: reading.source.revision },
      scope: reading.scope, annotations: reading.annotations, provider_annotations: readingProviderAnnotations(reading).filter(item => selected.evidence.includes(item.evidence)), unsupported: reading.unsupported,
      excerpts: reading.locations.map(location => ({ location, text: location.location.bytes ?
        new TextDecoder().decode(new Uint8Array(reading.source.bytes.slice(location.location.bytes.start, location.location.bytes.end))) : null })),
    } };
  });
}

</storycode:copy>
<a id="f-4abce3fbcd42-observationSourceChanged"></a>
### `observationSourceChanged`

<storycode:copy id="f-4abce3fbcd42-observationSourceChanged">
export function observationSourceChanged(reading: ObservationReading | null | undefined, source: string | undefined): boolean {
  return !!reading && source !== undefined && new TextDecoder().decode(new Uint8Array(reading.source.bytes)) !== source;
}
</storycode:copy>

### Assemble `apps/web/src/debug/observations.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/debug/observations.ts"><storycode:paste select="#f-4abce3fbcd42-ObservationScope"/><storycode:paste select="#f-4abce3fbcd42-ObservationAnnotation"/><storycode:paste select="#f-4abce3fbcd42-ObservationReading"/><storycode:paste select="#f-4abce3fbcd42-SelectedObservations"/><storycode:paste select="#f-4abce3fbcd42-selected"/><storycode:paste select="#f-4abce3fbcd42-selectObservations"/><storycode:paste select="#f-4abce3fbcd42-selectedObservations"/><storycode:paste select="#f-4abce3fbcd42-observationContext"/><storycode:paste select="#f-4abce3fbcd42-observationSourceChanged"/></storycode:file>

<a id="output-a0a51eb651cb"></a>
## `apps/web/src/debug/providerObservations.test.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [Observations read beside their captured source](contract.md).

<a id="f-a0a51eb651cb-expression_statement"></a>
### `expression_statement`

<storycode:copy id="f-a0a51eb651cb-expression_statement">
// Guarantee: docs/guarantees/debugging/source-anchored-observation-reading.md
import { expect, it } from "vitest";
import { providerObservationContext, selectProviderReading, type ProviderReading } from "./providerObservations";
it("selects historical review identity without fabricating runtime scope", () => {
  const owner = Symbol();
  const reading: ProviderReading = { schema_version: 1, source: { id: "doc", uri: "hick:///x.py", revision: "head", bytes: [120] }, unsupported: [],
    annotations: [{ evidence: "thread-1", provider: "github", provider_version: "unknown", payload_schema: "review.v1",
      origin: { kind: "review", repository: "org/repo", pull_request: 2, thread: "t1", base_revision: "base", head_revision: "head", side: "RIGHT", author: "reviewer", resolved: false, outdated: true },
      location: null, source_precision: "unknown", measurement_semantics: null, uncertainty: "outdated", payload: { body: "Check this" } }] };
  expect(providerObservationContext("x.py", "x")).toEqual([]);
  selectProviderReading(owner, reading, true, true);
  const context = providerObservationContext("x.py", "changed")[0];
  expect(context.state).toBe("changed"); expect(context.evidence).toEqual(["thread-1"]);
  expect(context.reading.provider_annotations[0].origin).toEqual(reading.annotations[0].origin);
  expect(context.reading).not.toHaveProperty("scope");
  selectProviderReading(owner, reading, false, false);
});
</storycode:copy>

### Assemble `apps/web/src/debug/providerObservations.test.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/debug/providerObservations.test.ts"><storycode:paste select="#f-a0a51eb651cb-expression_statement"/></storycode:file>

<a id="output-a6f0ea9e8feb"></a>
## `crates/hick-observe/src/reading.rs`

Related contracts: [Observations read beside their captured source](contract.md), [Runtime observations retain their producing revision and occurrence](../../execution/revision-bound-runtime-observations/contract.md).

<a id="f-a6f0ea9e8feb-Scope"></a>
### `Scope`

Derived reading surfaces. Captured source and evidence remain in the existing Run.

<storycode:copy id="f-a6f0ea9e8feb-Scope">
//! Derived reading surfaces. Captured source and evidence remain in the existing Run.
use crate::{Event, Location, Observation, Precision, Run, Source};
use anyhow::{Result, ensure};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Scope {
    pub session: Option<String>,
    pub run: String,
    pub stop: String,
    pub frame: Option<i64>,
    /// Only supplied when the runtime actually identifies an invocation.
    pub invocation: Option<String>,
}
</storycode:copy>
<a id="f-a6f0ea9e8feb-DebugAnnotation"></a>
### `DebugAnnotation`

<storycode:copy id="f-a6f0ea9e8feb-DebugAnnotation">
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum DebugAnnotation {
    BreakpointRequested {
        breakpoint: String,
        state: String,
    },
    BreakpointBound {
        breakpoint: String,
    },
    Paused {
        reason: String,
    },
    Frame {
        name: String,
        depth: usize,
        reported_source: Option<String>,
        reported_line: Option<u32>,
    },
    Values {
        values: Vec<(String, String)>,
    },
    Expression {
        expression: String,
        value: String,
        receipt: String,
    },
    Exception {
        description: Option<String>,
    },
}
</storycode:copy>
<a id="f-a6f0ea9e8feb-Annotation"></a>
### `Annotation`

<storycode:copy id="f-a6f0ea9e8feb-Annotation">
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Annotation {
    pub evidence: String,
    pub provider: String,
    pub payload_schema: String,
    pub location: Option<String>,
    pub scope: Scope,
    pub payload: DebugAnnotation,
}
</storycode:copy>
<a id="f-a6f0ea9e8feb-LocationRef"></a>
### `LocationRef`

<storycode:copy id="f-a6f0ea9e8feb-LocationRef">
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LocationRef {
    pub id: String,
    pub revision: String,
    pub location: Location,
    /// Already mapped document coordinate, never a projected display row.
    pub line: u32,
}
</storycode:copy>
<a id="f-a6f0ea9e8feb-Reading"></a>
### `Reading`

<storycode:copy id="f-a6f0ea9e8feb-Reading">
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Reading {
    #[serde(default)]
    pub source_aliases: Vec<String>,
    pub schema_version: u32,
    pub source: Source,
    pub scope: Scope,
    pub locations: Vec<LocationRef>,
    pub annotations: Vec<Annotation>,
    pub unsupported: Vec<String>,
}
</storycode:copy>
<a id="f-a6f0ea9e8feb-impl-Reading"></a>
### `impl-Reading`

<storycode:copy id="f-a6f0ea9e8feb-impl-Reading">
impl Reading {
    pub fn new(source: Source, scope: Scope) -> Self {
        Self {
            schema_version: 1,
            source_aliases: vec![],
            source,
            scope,
            locations: vec![],
            annotations: vec![],
            unsupported: vec!["invocation identity".into(), "exception details".into()],
        }
    }
    pub fn location(&mut self, line: u32) -> Option<String> {
        if let Some(l) = self.locations.iter().find(|l| l.line == line) {
            return Some(l.id.clone());
        }
        let text = std::str::from_utf8(&self.source.bytes).ok()?;
        let mut start = 0;
        let mut found = None;
        for (index, part) in text.split_inclusive('\n').enumerate() {
            if index == line as usize {
                found = Some(start..start + part.trim_end_matches(['\r', '\n']).len());
                break;
            }
            start += part.len();
        }
        if text.ends_with('\n') && line as usize == text.bytes().filter(|b| *b == b'\n').count() {
            found = Some(text.len()..text.len());
        }
        let bytes = found?;
        let id = format!("{}:L{}", self.source.revision, line);
        self.locations.push(LocationRef {
            id: id.clone(),
            revision: self.source.revision.clone(),
            location: Location {
                source: self.source.id.clone(),
                block: None,
                bytes: Some(bytes),
                precision: Precision::AdapterReported,
            },
            line,
        });
        Some(id)
    }
    pub fn annotate(
        &mut self,
        run: &mut Run,
        line: Option<u32>,
        frame: Option<i64>,
        payload: DebugAnnotation,
    ) -> Result<()> {
        let location = line.and_then(|line| self.location(line));
        let mut scope = self.scope.clone();
        scope.frame = frame;
        let id = format!("{}:E{}", run.id, run.events.len());
        let annotation = Annotation {
            evidence: id.clone(),
            provider: "dap".into(),
            payload_schema: "hick.debug-annotation.v1".into(),
            location: location.clone(),
            scope,
            payload,
        };
        run.append(
            Event {
                id,
                provider: annotation.provider.clone(),
                block: None,
                invocation: annotation.scope.invocation.clone(),
                parent_invocation: None,
                thread: None,
                task: None,
                time: None,
                emission: location.and_then(|id| {
                    self.locations
                        .iter()
                        .find(|l| l.id == id)
                        .map(|l| l.location.clone())
                }),
                callers: vec![],
                observation: Observation::Debugger {
                    reason: "annotation".into(),
                    details: serde_json::to_value(&annotation)?,
                },
            },
            usize::MAX,
        )?;
        self.annotations.push(annotation);
        Ok(())
    }
    /// Action resolution is a lookup in a captured revision, never label parsing.
    pub fn resolve(
        &self,
        location: &str,
        scope: &Scope,
        current: &[u8],
        frame: Option<i64>,
    ) -> Result<u32> {
        ensure!(
            scope == &self.scope && scope.frame == frame,
            "stale stop or wrong frame"
        );
        ensure!(current == self.source.bytes, "source changed");
        let l = self
            .locations
            .iter()
            .find(|l| l.id == location)
            .ok_or_else(|| anyhow::anyhow!("unknown location handle"))?;
        ensure!(l.revision == self.source.revision, "stale source revision");
        Ok(l.line)
    }
    /// Explicit selection only. No automatic inclusion of every captured event.
    pub fn selected(&self, evidence: &[String]) -> Self {
        let mut result = self.clone();
        result
            .annotations
            .retain(|a| evidence.contains(&a.evidence));
        result.locations.retain(|l| {
            result
                .annotations
                .iter()
                .any(|a| a.location.as_ref() == Some(&l.id))
        });
        result
    }
}
</storycode:copy>
<a id="f-a6f0ea9e8feb-EvidenceOrigin"></a>
### `EvidenceOrigin`

Non-runtime providers do not fabricate a run or stop identity.

<storycode:copy id="f-a6f0ea9e8feb-EvidenceOrigin">
/// Non-runtime providers do not fabricate a run or stop identity.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum EvidenceOrigin {
    Runtime {
        session: Option<String>,
        run: String,
        stop: Option<String>,
        frame: Option<i64>,
        invocation: Option<String>,
    },
    Review {
        repository: String,
        pull_request: u64,
        thread: String,
        base_revision: String,
        head_revision: String,
        side: String,
        author: String,
        resolved: bool,
        outdated: bool,
    },
}
</storycode:copy>
<a id="f-a6f0ea9e8feb-ProviderAnnotation"></a>
### `ProviderAnnotation`

<storycode:copy id="f-a6f0ea9e8feb-ProviderAnnotation">
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProviderAnnotation {
    pub evidence: String,
    pub provider: String,
    pub provider_version: String,
    pub payload_schema: String,
    pub origin: EvidenceOrigin,
    /// None is honestly unmapped; source precision is not measurement uncertainty.
    pub location: Option<LocationRef>,
    pub source_precision: Precision,
    pub measurement_semantics: Option<String>,
    pub uncertainty: Option<String>,
    pub payload: serde_json::Value,
}
</storycode:copy>
<a id="f-a6f0ea9e8feb-tests"></a>
### `tests`

<storycode:copy id="f-a6f0ea9e8feb-tests">
#[cfg(test)]
mod tests {
    use super::*;
    fn fixture() -> (Run, Reading) {
        let source = Source::capture(
            "source".into(),
            "demo.md".into(),
            "prose\nBREAKPOINT HERE\nx += 1;\n".as_bytes().to_vec(),
        );
        let run = Run::new(vec![source.clone()], 1);
        let reading = Reading::new(
            source,
            Scope {
                session: Some("session".into()),
                run: run.id.clone(),
                stop: "S1".into(),
                frame: Some(7),
                invocation: None,
            },
        );
        (run, reading)
    }
    // Guarantee: docs/guarantees/debugging/source-anchored-observation-reading.md
    #[test]
    fn source_markers_do_not_create_evidence_and_relocated_breakpoints_stay_distinct() {
        let (mut run, mut reading) = fixture();
        assert!(reading.annotations.is_empty());
        reading
            .annotate(
                &mut run,
                Some(1),
                None,
                DebugAnnotation::BreakpointRequested {
                    breakpoint: "B1".into(),
                    state: "bound".into(),
                },
            )
            .unwrap();
        reading
            .annotate(
                &mut run,
                Some(2),
                None,
                DebugAnnotation::BreakpointBound {
                    breakpoint: "B1".into(),
                },
            )
            .unwrap();
        reading
            .annotate(
                &mut run,
                Some(2),
                Some(7),
                DebugAnnotation::Paused {
                    reason: "breakpoint".into(),
                },
            )
            .unwrap();
        assert_ne!(
            reading.annotations[0].location,
            reading.annotations[1].location
        );
        assert_eq!(
            reading.annotations[1].location,
            reading.annotations[2].location
        );
        assert_eq!(run.sources[0].bytes, reading.source.bytes);
        for (annotation, event) in reading.annotations.iter().zip(&run.events) {
            assert_eq!(annotation.evidence, event.id);
        }
        let selected = reading.selected(&[reading.annotations[2].evidence.clone()]);
        assert_eq!(selected.annotations.len(), 1);
        let json = serde_json::to_string(&selected).unwrap();
        assert_eq!(serde_json::from_str::<Reading>(&json).unwrap(), selected);
    }
    // Guarantee: docs/guarantees/debugging/source-anchored-observation-reading.md
    #[test]
    fn handles_reject_changed_source_repeated_stops_wrong_frames_and_fabrications() {
        let (_, mut reading) = fixture();
        let id = reading.location(2).unwrap();
        assert_eq!(
            reading
                .resolve(&id, &reading.scope, &reading.source.bytes, Some(7))
                .unwrap(),
            2
        );
        let mut stale = reading.scope.clone();
        stale.stop = "S0".into();
        assert!(
            reading
                .resolve(&id, &stale, &reading.source.bytes, Some(7))
                .is_err()
        );
        assert!(
            reading
                .resolve(&id, &reading.scope, &reading.source.bytes, Some(8))
                .is_err()
        );
        assert!(
            reading
                .resolve(&id, &reading.scope, b"edited", Some(7))
                .is_err()
        );
        assert!(
            reading
                .resolve(
                    "BREAKPOINT HERE",
                    &reading.scope,
                    &reading.source.bytes,
                    Some(7)
                )
                .is_err()
        );
        let mut other_run = reading.scope.clone();
        other_run.run = "other".into();
        assert!(
            reading
                .resolve(&id, &other_run, &reading.source.bytes, Some(7))
                .is_err()
        );
    }
    #[test]
    fn logs_profiles_and_review_payloads_keep_their_own_semantics() {
        use crate::{Clock, Interval};
        for payload in [
            Observation::Log {
                level: "info".into(),
                message: "hello".into(),
                fields: Default::default(),
            },
            Observation::Profile {
                metric: "cpu".into(),
                value: 0.5,
                unit: "ms".into(),
                interval: Interval {
                    start: Clock {
                        domain: "sampler".into(),
                        unit: "ms".into(),
                        value: 0.0,
                    },
                    end: Clock {
                        domain: "sampler".into(),
                        unit: "ms".into(),
                        value: 1.0,
                    },
                    scope: "function".into(),
                },
                aggregation: "sample estimate".into(),
                samples: Some(2),
                function: None,
            },
        ] {
            let bytes = serde_json::to_vec(&payload).unwrap();
            assert_eq!(
                serde_json::from_slice::<Observation>(&bytes).unwrap(),
                payload
            );
        }
        let review = ProviderAnnotation {
            evidence: "review:thread1".into(),
            provider: "github".into(),
            provider_version: "1".into(),
            payload_schema: "review.v1".into(),
            location: None,
            origin: EvidenceOrigin::Review {
                repository: "owner/repo".into(),
                pull_request: 66,
                thread: "thread1".into(),
                base_revision: "base".into(),
                head_revision: "head".into(),
                side: "right".into(),
                author: "reviewer".into(),
                resolved: false,
                outdated: true,
            },
            source_precision: Precision::Unknown,
            measurement_semantics: None,
            uncertainty: None,
            payload: serde_json::json!({"body":"untrusted review text"}),
        };
        let json = serde_json::to_value(&review).unwrap();
        assert!(json["origin"].get("run").is_none());
        assert_eq!(
            serde_json::from_value::<ProviderAnnotation>(json).unwrap(),
            review
        );
    }
}
</storycode:copy>
<a id="f-a6f0ea9e8feb-impl-Reading-2"></a>
### `impl-Reading`

<storycode:copy id="f-a6f0ea9e8feb-impl-Reading-2">
impl Reading {
    /// Source-adjacent textual equivalent with immutable byte lineage. Payload text is data.
    pub fn agent_projection(&self) -> serde_json::Value {
        let excerpts: Vec<_> = self
            .locations
            .iter()
            .map(|l| {
                let text = l
                    .location
                    .bytes
                    .as_ref()
                    .and_then(|r| self.source.bytes.get(r.clone()))
                    .and_then(|b| std::str::from_utf8(b).ok());
                serde_json::json!({"location":l,"text":text})
            })
            .collect();
        serde_json::json!({"schema_version":self.schema_version,"source":{"id":self.source.id,"uri":self.source.uri,"revision":self.source.revision},"scope":self.scope,"excerpts":excerpts,"annotations":self.annotations,"unsupported":self.unsupported})
    }
}
</storycode:copy>
<a id="f-a6f0ea9e8feb-impl-Run"></a>
### `impl-Run`

<storycode:copy id="f-a6f0ea9e8feb-impl-Run">
impl Run {
    /// Adapt every existing provider record to the common envelope, without a second store.
    pub fn provider_annotations(&self) -> Result<Vec<ProviderAnnotation>> {
        self.validate()?;
        let metadata = self
            .detail
            .as_deref()
            .and_then(|detail| serde_json::from_str::<serde_json::Value>(detail).ok());
        let version = metadata
            .as_ref()
            .and_then(|m| m.get("provider_version"))
            .and_then(|v| v.as_str())
            .unwrap_or("unknown");
        self.events
            .iter()
            .map(|event| {
                let location = event.emission.as_ref().and_then(|location| {
                    let source = self.sources.iter().find(|s| s.id == location.source)?;
                    let start = location.bytes.as_ref()?.start;
                    Some(LocationRef {
                        id: format!(
                            "{}:{}:bytes:{}:{}",
                            source.id,
                            source.revision,
                            start,
                            location.bytes.as_ref()?.end
                        ),
                        revision: source.revision.clone(),
                        location: location.clone(),
                        line: source.bytes[..start]
                            .iter()
                            .filter(|b| **b == b'\n')
                            .count() as u32,
                    })
                });
                let mut origin = EvidenceOrigin::Runtime {
                    session: None,
                    run: self.id.clone(),
                    stop: None,
                    frame: None,
                    invocation: event.invocation.clone(),
                };
                let (schema, semantics) = match &event.observation {
                    Observation::Log { .. } => ("hick.log.v1", None),
                    Observation::Output { .. } => ("hick.output.v1", None),
                    Observation::Profile { aggregation, .. } => {
                        ("hick.profile.v1", Some(aggregation.clone()))
                    }
                    Observation::Exception { .. } => ("hick.exception.v1", None),
                    Observation::Debugger { details, .. } => {
                        if let Ok(annotation) =
                            serde_json::from_value::<Annotation>(details.clone())
                        {
                            origin = EvidenceOrigin::Runtime {
                                session: annotation.scope.session,
                                run: self.id.clone(),
                                stop: Some(annotation.scope.stop),
                                frame: annotation.scope.frame,
                                invocation: annotation.scope.invocation,
                            };
                        }
                        ("hick.debugger.v1", None)
                    }
                };
                Ok(ProviderAnnotation {
                    evidence: event.id.clone(),
                    provider: event.provider.clone(),
                    provider_version: version.into(),
                    payload_schema: schema.into(),
                    origin,
                    source_precision: event
                        .emission
                        .as_ref()
                        .map(|l| l.precision.clone())
                        .unwrap_or(Precision::Unknown),
                    location,
                    measurement_semantics: semantics,
                    uncertainty: matches!(event.observation, Observation::Profile { .. }).then(
                        || "Profiler overhead and timing variability are not estimated".into(),
                    ),
                    payload: serde_json::to_value(&event.observation)?,
                })
            })
            .collect()
    }
}
</storycode:copy>
<a id="f-a6f0ea9e8feb-shared_fixture_tests"></a>
### `shared_fixture_tests`

<storycode:copy id="f-a6f0ea9e8feb-shared_fixture_tests">
#[cfg(test)]
mod shared_fixture_tests {
    use super::*;
    // Guarantee: docs/guarantees/debugging/source-anchored-observation-reading.md
    #[test]
    fn browser_fixture_is_the_rust_reading_and_agent_projection_contract() {
        let reading: Reading = serde_json::from_str(include_str!(
            "../../../apps/web/src/debug/fixtures/reading.json"
        ))
        .unwrap();
        Run::new(vec![reading.source.clone()], 0)
            .validate()
            .unwrap();
        let projection = reading.agent_projection();
        assert_eq!(projection["excerpts"][0]["text"], "value += 1;");
        assert_eq!(projection["annotations"][0]["evidence"], "opaque-E");
        let json = serde_json::to_value(&reading).unwrap();
        assert_eq!(serde_json::from_value::<Reading>(json).unwrap(), reading);
    }
}
</storycode:copy>
<a id="f-a6f0ea9e8feb-provider_projection_tests"></a>
### `provider_projection_tests`

<storycode:copy id="f-a6f0ea9e8feb-provider_projection_tests">
#[cfg(test)]
mod provider_projection_tests {
    use super::*;
    // Guarantee: docs/guarantees/debugging/source-anchored-observation-reading.md
    #[test]
    fn existing_run_records_share_the_provider_envelope_without_debugger_scope() {
        let source = Source::capture("doc".into(), "a.md".into(), b"x\n".to_vec());
        let mut run = Run::new(vec![source], 0);
        run.append(
            Event {
                id: "log-1".into(),
                provider: "logger".into(),
                block: None,
                invocation: None,
                parent_invocation: None,
                thread: None,
                task: None,
                time: None,
                emission: Some(Location {
                    source: "doc".into(),
                    block: None,
                    bytes: Some(0..1),
                    precision: Precision::Exact,
                }),
                callers: vec![],
                observation: Observation::Log {
                    level: "info".into(),
                    message: "hello".into(),
                    fields: Default::default(),
                },
            },
            10000,
        )
        .unwrap();
        let annotations = run.provider_annotations().unwrap();
        assert_eq!(annotations[0].evidence, "log-1");
        assert_eq!(annotations[0].source_precision, Precision::Exact);
        assert!(matches!(
            &annotations[0].origin,
            EvidenceOrigin::Runtime {
                stop: None,
                frame: None,
                ..
            }
        ));
        let json = serde_json::to_value(&annotations).unwrap();
        assert_eq!(
            serde_json::from_value::<Vec<ProviderAnnotation>>(json).unwrap(),
            annotations
        );
    }
}
</storycode:copy>

### Assemble `crates/hick-observe/src/reading.rs`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="crates/hick-observe/src/reading.rs"><storycode:paste select="#f-a6f0ea9e8feb-Scope"/><storycode:paste select="#f-a6f0ea9e8feb-DebugAnnotation"/><storycode:paste select="#f-a6f0ea9e8feb-Annotation"/><storycode:paste select="#f-a6f0ea9e8feb-LocationRef"/><storycode:paste select="#f-a6f0ea9e8feb-Reading"/><storycode:paste select="#f-a6f0ea9e8feb-impl-Reading"/><storycode:paste select="#f-a6f0ea9e8feb-EvidenceOrigin"/><storycode:paste select="#f-a6f0ea9e8feb-ProviderAnnotation"/><storycode:paste select="#f-a6f0ea9e8feb-tests"/><storycode:paste select="#f-a6f0ea9e8feb-impl-Reading-2"/><storycode:paste select="#f-a6f0ea9e8feb-impl-Run"/><storycode:paste select="#f-a6f0ea9e8feb-shared_fixture_tests"/><storycode:paste select="#f-a6f0ea9e8feb-provider_projection_tests"/></storycode:file>

<a id="output-d5f4bc355f22"></a>
## `crates/hickory-cli/src/debug_reading.rs`

Related contracts: [Observations read beside their captured source](contract.md).

<a id="f-d5f4bc355f22-impl-Live"></a>
### `impl-Live`

One producer for UI and MCP debugger readings; no execution policy.

<storycode:copy id="f-d5f4bc355f22-impl-Live">
//! One producer for UI and MCP debugger readings; no execution policy.
use super::Live;
impl Live {
    pub async fn capture_reading(
        &self,
        session: &str,
        reason: &str,
        frames: &[hick_dap::Frame],
        variables: &[hick_dap::Variable],
        selected_frame: Option<i64>,
    ) -> anyhow::Result<Option<hick_observe::reading::Reading>> {
        let Some(source) = self.session.observation_source.clone() else {
            return Ok(None);
        };
        use hick_observe::reading::{DebugAnnotation as A, Reading, Scope};
        let stopped = self.stopped.lock().await.clone();
        let reason = stopped
            .as_ref()
            .map(|s| s.reason.as_str())
            .unwrap_or(reason);
        let mut held = self.reading.lock().await;
        let mut run = self.observations.lock().await;
        let scope = Scope {
            session: Some(session.into()),
            run: run.id.clone(),
            stop: format!("{}:S{}", session, *self.stop_generation.lock().await),
            frame: selected_frame,
            invocation: None,
        };
        let mut reading = Reading::new(source, scope);
        reading.source_aliases = self.source_aliases.clone();
        if !self.session.capabilities().step_back {
            reading.unsupported.push("reverse execution".into());
        }
        if !self.session.capabilities().restart_frame {
            reading.unsupported.push("restart frame".into());
        }
        let mut add = |line, frame, payload| reading.annotate(&mut run, line, frame, payload);
        let result: anyhow::Result<()> = (|| {
            for bp in &self.session.breakpoint_statuses() {
                let breakpoint = format!("{session}:B{}", bp.line);
                add(
                    Some(bp.line),
                    None,
                    A::BreakpointRequested {
                        breakpoint: breakpoint.clone(),
                        state: match bp.state {
                            hick_dap::BindState::Bound => "bound",
                            hick_dap::BindState::Pending => "pending",
                            hick_dap::BindState::Refused => "refused",
                        }
                        .into(),
                    },
                )?;
                if bp.state.is_bound() {
                    add(
                        Some(bp.moved_to.unwrap_or(bp.line)),
                        None,
                        A::BreakpointBound { breakpoint },
                    )?;
                }
            }
            for (depth, frame) in frames.iter().enumerate() {
                add(
                    frame.line,
                    Some(frame.id),
                    A::Frame {
                        name: frame.name.clone(),
                        depth,
                        reported_source: frame.source.clone(),
                        reported_line: frame.source_line,
                    },
                )?;
            }
            let frame = frames.first().map(|f| f.id);
            let line = frames.first().and_then(|f| f.line);
            add(
                line,
                frame,
                A::Paused {
                    reason: reason.into(),
                },
            )?;
            add(
                frames
                    .iter()
                    .find(|f| Some(f.id) == selected_frame)
                    .and_then(|f| f.line),
                selected_frame,
                A::Values {
                    values: variables
                        .iter()
                        .map(|v| (v.name.clone(), v.value.clone()))
                        .collect(),
                },
            )?;
            if reason == "exception" {
                add(
                    line,
                    frame,
                    A::Exception {
                        description: stopped
                            .as_ref()
                            .and_then(|s| s.description.clone().or_else(|| s.text.clone())),
                    },
                )?;
            }
            Ok(())
        })();
        result?;
        // Re-reading one stop does not erase explicit evaluation receipts.
        if let Some(previous) = held.as_ref().filter(|p| p.scope.stop == reading.scope.stop) {
            for annotation in previous
                .annotations
                .iter()
                .filter(|a| matches!(a.payload, A::Expression { .. }))
            {
                if let Some(location) = annotation
                    .location
                    .as_ref()
                    .and_then(|id| previous.locations.iter().find(|l| &l.id == id))
                    && !reading.locations.iter().any(|l| l.id == location.id)
                {
                    reading.locations.push(location.clone());
                }
                reading.annotations.push(annotation.clone());
            }
        }
        *held = Some(reading.clone());
        Ok(Some(reading))
    }
    pub async fn record_expression(
        &self,
        expression: &str,
        value: &str,
        frame: Option<i64>,
    ) -> anyhow::Result<Option<hick_observe::reading::Reading>> {
        use hick_observe::reading::DebugAnnotation;
        let mut held = self.reading.lock().await;
        let Some(reading) = held.as_mut() else {
            return Ok(None);
        };
        let mut run = self.observations.lock().await;
        let location = reading
            .annotations
            .iter()
            .find(|a| matches!(a.payload, DebugAnnotation::Frame { .. }) && a.scope.frame == frame)
            .and_then(|a| a.location.as_ref());
        let line = location
            .and_then(|id| reading.locations.iter().find(|l| &l.id == id))
            .map(|l| l.line);
        let receipt = format!("{}:evaluation:{}", run.id, run.events.len());
        reading.annotate(
            &mut run,
            line,
            frame,
            DebugAnnotation::Expression {
                expression: expression.into(),
                value: value.into(),
                receipt,
            },
        )?;
        Ok(Some(reading.clone()))
    }
    pub async fn resolve_location(
        &self,
        location: &str,
        scope: &hick_observe::reading::Scope,
    ) -> anyhow::Result<u32> {
        let held = self.reading.lock().await;
        let reading = held
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("no current stop"))?;
        let current = if reading.source.uri.starts_with("hick-project:///") {
            reading.source.bytes.clone()
        } else {
            std::fs::read(&reading.source.uri)?
        };
        let thread = self
            .thread_id
            .lock()
            .await
            .ok_or_else(|| anyhow::anyhow!("no current stopped thread"))?;
        let actual_frame = self.session.stack(thread).await?.first().map(|f| f.id);
        reading.resolve(location, scope, &current, actual_frame)
    }
}
</storycode:copy>

### Assemble `crates/hickory-cli/src/debug_reading.rs`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="crates/hickory-cli/src/debug_reading.rs"><storycode:paste select="#f-d5f4bc355f22-impl-Live"/></storycode:file>

<a id="output-83c29f05f862"></a>
## `crates/hickory-cli/src/serve/debug_bridge/tests/tests.rs`

Related contracts: [Observations read beside their captured source](contract.md).

<a id="f-83c29f05f862-agent_action_fixture_uses_opaque_handles_without_display_line_arithmetic"></a>
### `agent_action_fixture_uses_opaque_handles_without_display_line_arithmetic`

<storycode:copy id="f-83c29f05f862-agent_action_fixture_uses_opaque_handles_without_display_line_arithmetic">
use super::*;
use hick_dap::Step;

// Guarantee: docs/guarantees/debugging/source-anchored-observation-reading.md
#[test]
fn agent_action_fixture_uses_opaque_handles_without_display_line_arithmetic() {
    let reading: hick_observe::reading::Reading = serde_json::from_str(include_str!(
        "../../../../../../apps/web/src/debug/fixtures/reading.json"
    ))
    .unwrap();
    let action = json!({"op":"located", "session":"debug-fixture", "location":reading.locations[0].id, "scope":reading.scope, "how":"jump"});
    assert!(action.get("line").is_none());
    let request: Request = serde_json::from_value(action).unwrap();
    match request {
        Request::Located {
            location, scope, ..
        } => {
            assert_eq!(
                reading
                    .resolve(&location, &scope, &reading.source.bytes, Some(7))
                    .unwrap(),
                1
            );
            let mut stale = scope;
            stale.stop = "old-stop".into();
            assert!(
                reading
                    .resolve(&location, &stale, &reading.source.bytes, Some(7))
                    .is_err()
            );
        }
        _ => panic!("expected located action"),
    }
}
</storycode:copy>
<a id="f-83c29f05f862-real_adapter_relocation_and_stale_actions_share_the_reading"></a>
### `real_adapter_relocation_and_stale_actions_share_the_reading`

<storycode:copy id="f-83c29f05f862-real_adapter_relocation_and_stale_actions_share_the_reading">
// Guarantee: docs/guarantees/debugging/source-anchored-observation-reading.md
#[cfg(unix)]
#[tokio::test(flavor = "multi_thread")]
async fn real_adapter_relocation_and_stale_actions_share_the_reading() {
    use hick_observe::reading::DebugAnnotation as A;
    let dir = tempfile::tempdir().unwrap();
    std::fs::create_dir(dir.path().join(".git")).unwrap();
    let cache = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../.hick-cache");
    if cache.join("adapters/python/bin/python3").exists() {
        std::os::unix::fs::symlink(cache, dir.path().join(".hick-cache")).unwrap();
    }
    if hick_dap::discover("python", dir.path()).is_none() {
        eprintln!("SKIPPED: install Python adapter with hick dap install python");
        return;
    }
    let source = "# Reading\n<hick:file path=\"main.py\">\nvalue = 1\n\n\ndef count(n):\n    return count(n - 1) if n else n\nprint(count(2))\n</hick:file>\n";
    let path = dir.path().join("demo.md");
    std::fs::write(&path, source).unwrap();
    let registry = Arc::new(Registry::new());
    let out = handle(
        &registry,
        dir.path(),
        Request::Start {
            request_id: Some("reading-start".into()),
            source_hash: Some(super::super::source_tests::digest(source.as_bytes())),
            source_hashes: Default::default(),
            source_test: None,
            doc: "hick:///demo.md".into(),
            breakpoints: vec![hick_dap::Breakpoint {
                line: 3,
                condition: None,
                hit_condition: None,
                log_message: None,
            }],
            program: Some("main.py".into()),
            project: None,
            profile: None,
            sources: Default::default(),
        },
    )
    .await;
    let (session, reading) = out
        .iter()
        .find_map(|r| match r {
            Response::Reading { session, reading } => Some((session.clone(), reading.clone())),
            _ => None,
        })
        .unwrap_or_else(|| panic!("missing reading: {out:?}"));
    let requested = reading
        .annotations
        .iter()
        .find(|a| matches!(a.payload, A::BreakpointRequested { .. }))
        .unwrap();
    let bound = reading
        .annotations
        .iter()
        .find(|a| matches!(a.payload, A::BreakpointBound { .. }))
        .unwrap();
    let paused = reading
        .annotations
        .iter()
        .find(|a| matches!(a.payload, A::Paused { .. }))
        .unwrap();
    assert_ne!(requested.location, bound.location);
    assert_eq!(bound.location, paused.location);
    assert_eq!(reading.source.bytes, source.as_bytes());
    let location = paused.location.clone().unwrap();
    let mut wrong_frame = reading.scope.clone();
    wrong_frame.frame = Some(-999);
    assert!(
        registry
            .get(&session)
            .await
            .unwrap()
            .resolve_location(&location, &wrong_frame)
            .await
            .is_err()
    );
    std::fs::write(&path, source.replace("value = 1", "value = 10")).unwrap();
    let changed = handle(
        &registry,
        dir.path(),
        Request::Located {
            session: session.clone(),
            location: location.clone(),
            scope: reading.scope.clone(),
            how: protocol::LocatedAction::Jump,
        },
    )
    .await;
    assert!(changed.iter().any(
        |r| matches!(r, Response::Failed { message, .. } if message.contains("source changed"))
    ));
    std::fs::write(&path, source).unwrap();
    let current = registry.get(&session).await.unwrap();
    let current_stop = *current.stop_generation.lock().await;
    let current_thread = *current.thread_id.lock().await;
    let current_frame = top_frame(&current).await;
    let _ = handle(
        &registry,
        dir.path(),
        Request::Step {
            session: session.clone(),
            how: hick_dap::Step::Over,
            frame: current_frame,
            stop: Some(current_stop),
            thread: current_thread,
            request_id: Some("reading-step".into()),
        },
    )
    .await;
    let stale = handle(
        &registry,
        dir.path(),
        Request::Located {
            session: session.clone(),
            location,
            scope: reading.scope,
            how: protocol::LocatedAction::RunTo,
        },
    )
    .await;
    assert!(stale.iter().any(|r| matches!(r, Response::Failed { .. })));
    registry.reap(&session).await;
}
</storycode:copy>
<a id="f-83c29f05f862-a_failed_build_reaches_the_client_before_the_failure"></a>
### `a_failed_build_reaches_the_client_before_the_failure`

A C# document whose build cannot succeed, and what the client is told.
The point is the ORDER and the SURVIVAL: the build's own output has to
reach the client on the failing path, before the failure, or a person
gets "building app.csproj failed" and none of the reasons.

<storycode:copy id="f-83c29f05f862-a_failed_build_reaches_the_client_before_the_failure">
/// A C# document whose build cannot succeed, and what the client is told.
///
/// The point is the ORDER and the SURVIVAL: the build's own output has to
/// reach the client on the failing path, before the failure, or a person
/// gets "building app.csproj failed" and none of the reasons.
#[tokio::test]
async fn a_failed_build_reaches_the_client_before_the_failure() {
    let dir = tempfile::tempdir().unwrap();
    // A document that generates C# and no project file — the refusal
    // `build` makes without running anything, so this test needs no
    // .NET SDK to be meaningful.
    std::fs::write(
        dir.path().join("a.md"),
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n\
             <hick:doc xmlns:hick=\"http://www.hickorydocs.com/1.0\" weave=\"o.md\">\n\
             <hick:file path=\"Program.cs\">\n\
             class Program { static void Main() {} }\n\
             </hick:file>\n\
             </hick:doc>\n",
    )
    .unwrap();

    let registry = Arc::new(Registry::new());
    let responses = handle(
        &registry,
        dir.path(),
        Request::Start {
            request_id: Some("failure-start".into()),
            source_hash: Some(super::super::source_tests::digest(
                &std::fs::read(dir.path().join("a.md")).unwrap(),
            )),
            source_hashes: Default::default(),
            source_test: None,
            doc: "hick:///a.md".into(),
            breakpoints: vec![],
            program: None,
            project: None,
            profile: None,
            sources: Default::default(),
        },
    )
    .await;

    // Discovery runs before the build, so on a machine with no
    // netcoredbg the adapter is what is missing and the build never
    // gets a chance to speak. Either way the person is told something
    // they can act on, and that is what is asserted.
    let failure = responses
        .iter()
        .find_map(|r| match r {
            Response::Failed { message, .. } => Some(message.clone()),
            _ => None,
        })
        .expect("a C# document with no project file cannot start");
    assert!(
        failure.contains("no project file") || failure.contains("no debug adapter"),
        "the failure does not say what is missing: {failure}"
    );
    // And it says how to get out of it. Which sentence that is depends on
    // which of the two things was missing, and both are checked by
    // `dap_install`'s drift test — the point here is that a person is
    // never left with only the bad news.
    assert!(
        failure.contains("hick dap install csharp")
            || failure.contains("hick ingest")
            || failure.contains("hick:file"),
        "the failure says what is wrong but not what to do: {failure}"
    );
    // Whatever the build managed to say arrives BEFORE the failure.
    if let Some(build_at) = responses
        .iter()
        .position(|r| matches!(r, Response::Build { .. }))
    {
        let failed_at = responses
            .iter()
            .position(|r| matches!(r, Response::Failed { .. }))
            .unwrap();
        assert!(build_at < failed_at, "the build arrived after the failure");
    }
}
</storycode:copy>
<a id="f-83c29f05f862-a_build_line_is_shaped_like_a_transcript_event"></a>
### `a_build_line_is_shaped_like_a_transcript_event`

<storycode:copy id="f-83c29f05f862-a_build_line_is_shaped_like_a_transcript_event">
#[test]
fn a_build_line_is_shaped_like_a_transcript_event() {
    // Not because it IS a transcript — nothing here is recorded or
    // compared — but because the client already has a terminal that
    // renders exactly this shape.
    let since = std::time::Instant::now();
    let cmd = build_event(BuildOutput::Cmd("dotnet build".into()), since);
    assert_eq!(cmd["kind"], "cmd");
    assert_eq!(cmd["data"], "dotnet build");
    let out = build_event(BuildOutput::Out("Restored app.csproj".into()), since);
    assert_eq!(out["kind"], "out");
    // Lines arrive without their newline and the terminal needs one.
    assert_eq!(out["data"], "Restored app.csproj\n");
    // A note is this app talking, not the build tool, and it is indented
    // the way the spec writes it.
    let note = build_event(BuildOutput::Note("packages restore into x".into()), since);
    assert_eq!(note["data"], "  packages restore into x\n");
    let exit = build_event(BuildOutput::Exit(1), since);
    assert_eq!(exit["kind"], "exit");
    assert_eq!(exit["code"], 1);
}
</storycode:copy>
<a id="f-83c29f05f862-a_document_is_named_in_the_clients_scheme"></a>
### `a_document_is_named_in_the_clients_scheme`

<storycode:copy id="f-83c29f05f862-a_document_is_named_in_the_clients_scheme">
// Contract: .hickory-source/frontends/contract.md
#[test]
fn a_document_is_named_in_the_clients_scheme() {
    let folder = tempfile::tempdir().unwrap();
    let root = folder.path();
    std::fs::create_dir(root.join("docs")).unwrap();
    std::fs::write(root.join("docs/a.md"), "source").unwrap();
    assert_eq!(
        resolve(root, "hick:///docs/a.md").unwrap(),
        root.join("docs/a.md").canonicalize().unwrap()
    );
    let literal = "%2F λ?#.py";
    std::fs::write(root.join(literal), "value=1\n").unwrap();
    assert_eq!(
        resolve(root, "hick:///%252F%20%CE%BB%3F%23.py").unwrap(),
        root.join(literal).canonicalize().unwrap()
    );
    assert_eq!(
        resolve(root, literal).unwrap(),
        root.join(literal).canonicalize().unwrap()
    );
    for invalid in [
        "hick:///../outside",
        "hick:///%2e%2e/outside",
        "hick:///bad%",
        "hick:///bad%ZZ",
        "hick:///a?query",
        "hick:///a#fragment",
        "/docs/a.md",
        "file:///etc/passwd",
    ] {
        assert!(resolve(root, invalid).is_err(), "Accepted {invalid}");
    }
    #[cfg(unix)]
    {
        let outside = tempfile::NamedTempFile::new().unwrap();
        std::os::unix::fs::symlink(outside.path(), root.join("escape.py")).unwrap();
        assert!(resolve(root, "hick:///escape.py").is_err());
    }
}
</storycode:copy>
<a id="f-83c29f05f862-requests_parse_from_their_frame"></a>
### `requests_parse_from_their_frame`

<storycode:copy id="f-83c29f05f862-requests_parse_from_their_frame">
#[test]
fn requests_parse_from_their_frame() {
    let mut frame = vec![CHANNEL_DEBUG];
    frame.extend_from_slice(br#"{"op":"start","doc":"hick:///a.md","breakpoints":[{"line":7}]}"#);
    let request = request_of(&frame).expect("parses");
    match request {
        Request::Start {
            doc,
            breakpoints,
            program,
            project,
            ..
        } => {
            assert!(project.is_none());
            assert_eq!(doc, "hick:///a.md");
            assert_eq!(breakpoints[0].line, 7);
            // Optional fields are absent, not zero.
            assert!(breakpoints[0].condition.is_none());
            // No program named: the session falls back to the first
            // debuggable file, which is what a one-file document wants.
            assert!(program.is_none());
        }
        other => panic!("wrong request: {other:?}"),
    }
}
</storycode:copy>
<a id="f-83c29f05f862-a_step_names_the_session_apis_own_verb"></a>
### `a_step_names_the_session_apis_own_verb`

<storycode:copy id="f-83c29f05f862-a_step_names_the_session_apis_own_verb">
#[test]
fn a_step_names_the_session_apis_own_verb() {
    let mut frame = vec![CHANNEL_DEBUG];
    frame.extend_from_slice(br#"{"op":"step","session":"dbg-0","how":"drop_frame"}"#);
    match request_of(&frame).expect("parses") {
        Request::Step { how, .. } => assert_eq!(how, Step::DropFrame),
        other => panic!("wrong request: {other:?}"),
    }
}
</storycode:copy>
<a id="f-83c29f05f862-a_response_frame_is_tagged_with_the_debug_channel"></a>
### `a_response_frame_is_tagged_with_the_debug_channel`

<storycode:copy id="f-83c29f05f862-a_response_frame_is_tagged_with_the_debug_channel">
#[test]
fn a_response_frame_is_tagged_with_the_debug_channel() {
    let frame = frame_of(&Response::Ended {
        session: "dbg-0".into(),
    });
    assert_eq!(frame[0], CHANNEL_DEBUG);
    let value: Value = serde_json::from_slice(&frame[1..]).unwrap();
    assert_eq!(value["event"], "ended");
    assert_eq!(value["session"], "dbg-0");
}
</storycode:copy>
<a id="f-83c29f05f862-a_failure_carries_words_rather_than_a_code"></a>
### `a_failure_carries_words_rather_than_a_code`

<storycode:copy id="f-83c29f05f862-a_failure_carries_words_rather_than_a_code">
#[test]
fn a_failure_carries_words_rather_than_a_code() {
    // The UI shows this text; a code would need a second table nobody
    // would keep in step.
    let frame = frame_of(&Response::Failed {
        scope_retired: false,
        request_id: None,
        doc: None,
        session: Some("dbg-1".into()),
        message: "that line is prose, not code".into(),
        about: Some("breakpoints".into()),
        lines: vec![4],
        offer_install: None,
    });
    let value: Value = serde_json::from_slice(&frame[1..]).unwrap();
    assert_eq!(value["event"], "failed");
    assert!(value["message"].as_str().unwrap().contains("prose"));
    // And says what it was about, so the app can show it on the
    // breakpoint rather than in a banner that outlives its cause.
    assert_eq!(value["about"], "breakpoints");
    assert_eq!(value["lines"][0], 4);
}
</storycode:copy>
<a id="f-83c29f05f862-a_failure_about_nothing_in_particular_carries_no_lines"></a>
### `a_failure_about_nothing_in_particular_carries_no_lines`

<storycode:copy id="f-83c29f05f862-a_failure_about_nothing_in_particular_carries_no_lines">
#[test]
fn a_failure_about_nothing_in_particular_carries_no_lines() {
    // `about`/`lines` are omitted rather than sent empty: a UI that keys
    // on their presence should not have to know two spellings of absent.
    let frame = frame_of(&Response::Failed {
        scope_retired: false,
        request_id: None,
        doc: None,
        session: None,
        message: "no such session".into(),
        about: None,
        lines: Vec::new(),
        offer_install: None,
    });
    let value: Value = serde_json::from_slice(&frame[1..]).unwrap();
    assert!(value.get("about").is_none(), "{value}");
    assert!(value.get("lines").is_none(), "{value}");
}
</storycode:copy>
<a id="f-83c29f05f862-the_channel_carries_no_way_to_edit_anything"></a>
### `the_channel_carries_no_way_to_edit_anything`

<storycode:copy id="f-83c29f05f862-the_channel_carries_no_way_to_edit_anything">
#[test]
fn the_channel_carries_no_way_to_edit_anything() {
    // The isolation guarantee, checked against the protocol itself: if an
    // edit verb ever appears here, this fails.
    let described = describe();
    let requests = described["requests"].as_array().unwrap();
    for request in requests {
        let name = request.as_str().unwrap();
        assert!(
            !name.contains("edit") && !name.contains("write") && !name.contains("save"),
            "the debug channel grew a way to write: {name}"
        );
    }
}
</storycode:copy>

### Assemble `crates/hickory-cli/src/serve/debug_bridge/tests/tests.rs`

The selections below preserve the original byte order. Narrative reading order does not change the output.

### Select only an adapter-reported thread at its current stop

This opt-in witness launches a real multithreaded Python program. It selects the
worker, evaluates its own frame, rejects an earlier stop and a foreign thread,
then selects main and continues to completion without changing source. Ignored
by default means unexecuted unless explicitly invoked with `--ignored`; set
`HICKORY_THREAD_TEST_VENV` to a virtualenv containing debugpy, or prepare
`target/thread-debug-env`. The separate host witnesses must still establish
menus, retained editor state and obsolete reply rejection.

The captured-literal witness registers two percent-distinct project names in
memory and starts the canonical captured source URI through the real debugger
service. It checks the paused frame URI/hash, scoped value 41 and live conditional
breakpoint receipt, then stops the owned session. Host directory entries and
retained project bytes must remain unchanged; debugging may use only its private
scratch clone, without materializing the retained comparison context. The
explicit ignored test executed successfully on 2026-11-16 with the shared
`target/thread-debug-env` virtualenv (one test, zero skipped). Its first execution
exposed the primary captured-address decoder still using a raw URI split; the
subsequent shared primary mapping fix passed this actual service/DAP path. This
does not establish frontend pointer, keyboard or accessibility acceptance.

<storycode:copy id="debugger-thread-stop-witness">
#[cfg(unix)]
#[tokio::test(flavor = "multi_thread")]
#[ignore = "requires debugpy in HICKORY_THREAD_TEST_VENV or target/thread-debug-env"]
async fn real_threads_select_current_stack_and_refuse_stale_scope_without_source_writes() {
    async fn ask(registry: &Arc<Registry>, root: &std::path::Path, value: Value) -> Vec<Value> {
        handle(registry, root, serde_json::from_value(value).unwrap())
            .await
            .into_iter()
            .map(|response| serde_json::to_value(response).unwrap())
            .collect()
    }
    fn event<'a>(rows: &'a [Value], kind: &str) -> &'a Value {
        rows.iter()
            .find(|row| row["event"] == kind)
            .unwrap_or_else(|| panic!("Missing {kind}: {rows:?}"))
    }
    let root = tempfile::tempdir().unwrap();
    std::fs::create_dir(root.path().join(".git")).unwrap();
    let venv = std::env::var_os("HICKORY_THREAD_TEST_VENV")
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../target/thread-debug-env")
        });
    assert!(
        venv.join("bin/python").exists(),
        "Prepare the explicit debugpy test virtualenv first"
    );
    std::os::unix::fs::symlink(
        std::fs::canonicalize(venv).unwrap(),
        root.path().join(".venv"),
    )
    .unwrap();
    let source = "import threading\nready = threading.Event()\nrelease = threading.Event()\ndef worker():\n    marker = 41\n    ready.set()\n    while not release.is_set():\n        marker += 0\n    marker += 1\nthread = threading.Thread(target=worker, name='hickory-worker')\nthread.start()\nready.wait()\nvalue = 7\nrelease.set()\nthread.join()\nprint(value)\n";
    std::fs::write(root.path().join("main.py"), source).unwrap();
    let registry = Arc::new(Registry::new());
    let breakpoint = source
        .lines()
        .position(|line| line == "release.set()")
        .unwrap();
    let started = ask(
        &registry,
        root.path(),
        json!({"op":"start","doc":"hick:///main.py","request_id":"launch-threads","source_hash":super::super::source_tests::digest(source.as_bytes()),"breakpoints":[{"line":breakpoint}]}),
    )
    .await;
    let session = event(&started, "started")["session"]
        .as_str()
        .unwrap()
        .to_owned();
    let stopped = event(&started, "stopped");
    let stop = stopped["stop"].as_u64().unwrap();
    let main = stopped["thread"].as_i64().unwrap();
    let main_frame = stopped["frames"][0]["id"].as_i64().unwrap();
    let live = registry.get(&session).await.unwrap();
    let mut delivered = false;
    let listed = handle_with(
        &registry,
        root.path(),
        serde_json::from_value(json!({"op":"threads","session":session,"request_id":"list-1"}))
            .unwrap(),
        |responses| {
            assert!(
                live.operation.try_lock().is_err(),
                "Response queueing lost the session operation guard"
            );
            assert!(!responses.is_empty());
            delivered = true;
        },
    )
    .await
    .into_iter()
    .map(|response| serde_json::to_value(response).unwrap())
    .collect::<Vec<_>>();
    assert!(delivered);
    let list = event(&listed, "threads");
    assert_eq!(list["stop"], stop);
    assert_eq!(list["selected"], main);
    assert_eq!(list["request_id"], "list-1");
    let worker = list["threads"]
        .as_array()
        .unwrap()
        .iter()
        .find(|thread| thread["name"] == "hickory-worker")
        .unwrap()["id"]
        .as_i64()
        .unwrap();
    assert_ne!(worker, main);
    for (requested, generation) in [(worker, stop - 1), (i64::MAX, stop)] {
        let refused = ask(&registry, root.path(), json!({"op":"select_thread","session":session,"thread":requested,"stop":generation,"request_id":"refused"})).await;
        assert_eq!(event(&refused, "failed")["request_id"], "refused");
        assert_eq!(
            *registry.get(&session).await.unwrap().thread_id.lock().await,
            Some(main)
        );
    }
    let selected = ask(&registry, root.path(), json!({"op":"select_thread","session":session,"thread":worker,"stop":stop,"request_id":"select-worker"})).await;
    let paused = event(&selected, "stopped");
    assert_eq!(paused["thread"], worker);
    let worker_stop = paused["stop"].as_u64().unwrap();
    assert!(worker_stop > stop);
    let worker_frame = paused["frames"]
        .as_array()
        .unwrap()
        .iter()
        .find(|frame| frame["name"] == "worker")
        .unwrap()["id"]
        .as_i64()
        .unwrap();
    assert_eq!(event(&selected, "threads")["request_id"], "select-worker");
    let locals = ask(&registry, root.path(), json!({"op":"state","session":session,"frame":worker_frame,"stop":worker_stop,"thread":worker,"request_id":"worker-locals"})).await;
    let locals = event(&locals, "frame_variables");
    assert_eq!(locals["request_id"], "worker-locals");
    assert_eq!(locals["frame"], worker_frame);
    assert_eq!(locals["thread"], worker);
    assert_eq!(locals["stop"], worker_stop);
    assert!(
        locals["variables"]
            .as_array()
            .unwrap()
            .iter()
            .any(|value| value["name"] == "marker" && value["value"] == "41")
    );
    let foreign = ask(&registry, root.path(), json!({"op":"eval","session":session,"expression":"value","frame":main_frame,"context":"watch","stop":worker_stop,"thread":worker,"request_id":"foreign-frame"})).await;
    assert_eq!(event(&foreign, "failed")["request_id"], "foreign-frame");
    let value = ask(&registry, root.path(), json!({"op":"eval","session":session,"expression":"marker","frame":worker_frame,"context":"watch","stop":worker_stop,"thread":worker,"request_id":"worker-value"})).await;
    let value = event(&value, "value");
    assert_eq!(value["value"], "41");
    assert_eq!(value["stop"], worker_stop);
    assert_eq!(value["thread"], worker);
    assert_eq!(value["frame"], worker_frame);
    assert_eq!(value["request_id"], "worker-value");
    // Contract: .hickory-source/frontends/contract.md. A reported root can
    // expand only under the frame that observed it; descendants join that scope.
    let composite=ask(&registry,root.path(),json!({"op":"eval","session":session,"expression":"{'nested': {'answer': marker}}","frame":worker_frame,"context":"watch","stop":worker_stop,"thread":worker,"request_id":"composite"})).await;
    let reference = event(&composite, "value")["reference"].as_i64().unwrap();
    assert!(reference > 0);
    let unknown=ask(&registry,root.path(),json!({"op":"children","session":session,"reference":i32::MAX,"frame":worker_frame,"stop":worker_stop,"thread":worker,"request_id":"unknown-reference"})).await;
    assert_eq!(event(&unknown, "failed")["request_id"], "unknown-reference");
    assert_eq!(event(&unknown, "failed")["scope_retired"], false);
    let expanded=ask(&registry,root.path(),json!({"op":"children","session":session,"reference":reference,"frame":worker_frame,"stop":worker_stop,"thread":worker,"request_id":"nested-root"})).await;
    let expanded = event(&expanded, "children");
    assert_eq!(expanded["frame"], worker_frame);
    let nested = expanded["variables"]
        .as_array()
        .unwrap()
        .iter()
        .find(|row| {
            row["name"]
                .as_str()
                .is_some_and(|name| name.contains("nested"))
        })
        .unwrap()["variables_reference"]
        .as_i64()
        .unwrap();
    let answer=ask(&registry,root.path(),json!({"op":"children","session":session,"reference":nested,"frame":worker_frame,"stop":worker_stop,"thread":worker,"request_id":"nested-answer"})).await;
    assert!(
        event(&answer, "children")["variables"]
            .as_array()
            .unwrap()
            .iter()
            .any(|row| row["name"]
                .as_str()
                .is_some_and(|name| name.contains("answer"))
                && row["value"] == "41")
    );
    let unscoped=ask(&registry,root.path(),json!({"op":"eval","session":session,"expression":"globals().__setitem__('forbidden_probe', True)","frame":worker_frame,"context":"repl","request_id":"unscoped-effect"})).await;
    assert_eq!(event(&unscoped, "failed")["request_id"], "unscoped-effect");
    let probe=ask(&registry,root.path(),json!({"op":"eval","session":session,"expression":"'forbidden_probe' in globals()","frame":worker_frame,"context":"watch","stop":worker_stop,"thread":worker,"request_id":"probe-absent"})).await;
    assert_eq!(event(&probe, "value")["value"], "False");
    let hash = super::super::source_tests::digest(source.as_bytes());
    let baseline = serde_json::to_value(
        registry
            .get(&session)
            .await
            .unwrap()
            .session
            .breakpoint_statuses(),
    )
    .unwrap();
    for (id, revision, line) in [
        ("wrong-source", "0".repeat(64), breakpoint as u32),
        ("outside-source", hash.clone(), u32::MAX),
    ] {
        let refused=ask(&registry,root.path(),json!({"op":"breakpoints","session":session,"source_hash":revision,"request_id":id,"breakpoints":[{"line":line}]})).await;
        assert_eq!(event(&refused, "failed")["request_id"], id);
        assert_eq!(
            serde_json::to_value(
                registry
                    .get(&session)
                    .await
                    .unwrap()
                    .session
                    .breakpoint_statuses()
            )
            .unwrap(),
            baseline
        );
    }
    std::fs::write(
        root.path().join("main.py"),
        format!("{source}# changed saved bytes\n"),
    )
    .unwrap();
    let changed=ask(&registry,root.path(),json!({"op":"breakpoints","session":session,"source_hash":hash,"request_id":"changed-saved","breakpoints":[{"line":breakpoint}]})).await;
    assert_eq!(event(&changed, "failed")["request_id"], "changed-saved");
    std::fs::write(root.path().join("main.py"), source).unwrap();
    let stale = ask(&registry, root.path(), json!({"op":"eval","session":session,"expression":"marker","frame":worker_frame,"context":"watch","stop":stop,"thread":worker,"request_id":"old-value"})).await;
    assert_eq!(event(&stale, "failed")["request_id"], "old-value");
    let selected = ask(&registry, root.path(), json!({"op":"select_thread","session":session,"thread":main,"stop":worker_stop,"request_id":"select-main"})).await;
    let main_pause = event(&selected, "stopped");
    assert_eq!(main_pause["thread"], main);
    let main_stop = main_pause["stop"].as_u64().unwrap();
    let main_frame = main_pause["frames"]
        .as_array()
        .unwrap()
        .iter()
        .find(|row| row["name"] == "<module>")
        .unwrap()["id"]
        .as_i64()
        .unwrap();
    let retired=ask(&registry,root.path(),json!({"op":"children","session":session,"reference":reference,"frame":main_frame,"stop":main_stop,"thread":main,"request_id":"retired-reference"})).await;
    assert_eq!(event(&retired, "failed")["request_id"], "retired-reference");
    let stale_move=ask(&registry,root.path(),json!({"op":"step","session":session,"how":"continue","stop":worker_stop,"thread":worker,"frame":worker_frame,"request_id":"stale-resume"})).await;
    assert_eq!(event(&stale_move, "failed")["request_id"], "stale-resume");
    let done = ask(
        &registry,
        root.path(),
        json!({"op":"step","session":session,"how":"continue","stop":main_stop,"thread":main,"frame":main_frame,"request_id":"finish-worker"}),
    )
    .await;
    assert!(
        done.iter().any(|row| row["event"] == "finished"),
        "{done:?}"
    );
    assert_eq!(
        std::fs::read_to_string(root.path().join("main.py")).unwrap(),
        source
    );
}

#[cfg(unix)]
#[tokio::test(flavor = "multi_thread")]
#[ignore = "requires debugpy in HICKORY_THREAD_TEST_VENV or target/thread-debug-env"]
async fn real_running_launch_exposes_session_for_stop_and_live_breakpoints() {
    let root = tempfile::tempdir().unwrap();
    std::fs::create_dir(root.path().join(".git")).unwrap();
    let venv = std::env::var_os("HICKORY_THREAD_TEST_VENV")
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../target/thread-debug-env")
        });
    std::os::unix::fs::symlink(
        std::fs::canonicalize(venv).unwrap(),
        root.path().join(".venv"),
    )
    .unwrap();
    let source = "import time\nwhile True:\n    time.sleep(0.01)\n";
    std::fs::write(root.path().join("running.py"), source).unwrap();
    let registry = Arc::new(Registry::new());
    let (tx, mut receiver) = mpsc::unbounded_channel();
    let running = {
        let registry = registry.clone();
        let path = root.path().to_path_buf();
        tokio::spawn(async move {
            dispatch(
                &registry,
                &path,
                serde_json::from_value(
                    json!({"op":"start","doc":"hick:///running.py","request_id":"launch-running","source_hash":super::super::source_tests::digest(source.as_bytes()),"breakpoints":[]}),
                )
                .unwrap(),
                &tx,
            )
            .await;
        })
    };
    let session = tokio::time::timeout(Duration::from_secs(15), async {
        loop {
            let frame = receiver
                .recv()
                .await
                .expect("Debug launch closed before reporting its session");
            let response: Value = serde_json::from_slice(&frame[1..]).unwrap();
            assert_ne!(response["event"], "failed", "{response}");
            if response["event"] == "started" {
                assert_eq!(response["request_id"], "launch-running");
                break response["session"].as_str().unwrap().to_owned();
            }
        }
    })
    .await
    .expect("A running program hid its real session until its first stop");
    let breakpoints = tokio::time::timeout(Duration::from_secs(10), handle(&registry, root.path(), serde_json::from_value(json!({"op":"breakpoints","session":session,"source_hash":super::super::source_tests::digest(source.as_bytes()),"request_id":"live-conditional","breakpoints":[{"line":2,"condition":"False"}]})).unwrap())).await.expect("Live breakpoints waited for the program to stop");
    assert!(
        breakpoints
            .iter()
            .any(|response| matches!(response, Response::Breakpoints { .. })),
        "{breakpoints:?}"
    );
    let ended = tokio::time::timeout(
        Duration::from_secs(10),
        handle(
            &registry,
            root.path(),
            serde_json::from_value(json!({"op":"stop","session":session})).unwrap(),
        ),
    )
    .await
    .expect("Stop could not interrupt the running launch");
    assert!(
        ended
            .iter()
            .any(|response| matches!(response, Response::Ended { .. })),
        "{ended:?}"
    );
    tokio::time::timeout(Duration::from_secs(10), running)
        .await
        .expect("The initial stop waiter survived adapter shutdown")
        .unwrap();
    assert!(registry.get(&session).await.is_err());
    assert_eq!(
        std::fs::read_to_string(root.path().join("running.py")).unwrap(),
        source
    );
}

#[cfg(unix)]
#[tokio::test(flavor = "multi_thread")]
#[ignore = "requires debugpy in HICKORY_THREAD_TEST_VENV or target/thread-debug-env"]
async fn real_captured_literal_source_round_trips_through_debugpy_without_host_writes() {
    async fn ask(registry: &Arc<Registry>, root: &std::path::Path, value: Value) -> Vec<Value> {
        tokio::time::timeout(
            Duration::from_secs(30),
            handle(registry, root, serde_json::from_value(value).unwrap()),
        )
        .await
        .expect("Captured-source debugger request did not settle")
        .into_iter()
        .map(|response| serde_json::to_value(response).unwrap())
        .collect()
    }
    fn event<'a>(rows: &'a [Value], kind: &str) -> &'a Value {
        rows.iter()
            .find(|row| row["event"] == kind)
            .unwrap_or_else(|| panic!("Missing {kind}: {rows:?}"))
    }
    fn host_entries(root: &std::path::Path) -> Vec<std::ffi::OsString> {
        let mut entries = std::fs::read_dir(root)
            .unwrap()
            .map(|entry| entry.unwrap().file_name())
            .collect::<Vec<_>>();
        entries.sort();
        entries
    }
    let host_root = tempfile::tempdir().unwrap();
    std::fs::create_dir(host_root.path().join(".git")).unwrap();
    let venv = std::env::var_os("HICKORY_THREAD_TEST_VENV")
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../target/thread-debug-env")
        });
    assert!(
        venv.join("bin/python").exists(),
        "Prepare the explicit debugpy test virtualenv first"
    );
    let environment = std::fs::canonicalize(venv).unwrap();
    std::os::unix::fs::symlink(&environment, host_root.path().join(".venv")).unwrap();
    let baseline = host_entries(host_root.path());
    let primary = "literal %2F #.py";
    let distinct = "literal %252F #.py";
    let source = "marker = 41\nmarker += 0\nprint(marker)\n";
    let other = "raise RuntimeError('Wrong captured percent spelling executed')\n";
    let hash = super::super::source_tests::digest(source.as_bytes());
    let mut tree = hick_lang::ProjectTree::default();
    tree.insert(
        primary,
        hick_lang::ProjectEntry::file(source.as_bytes().to_vec()),
    )
    .unwrap();
    tree.insert(
        distinct,
        hick_lang::ProjectEntry::file(other.as_bytes().to_vec()),
    )
    .unwrap();
    let projects = Arc::new(hick_lsp::ProjectContexts::default());
    let capture = "a1b2c3d4e5f60718";
    projects.register(capture, tree).unwrap();
    let uri = hick_lsp::ProjectContexts::source_uri(capture, primary).unwrap();
    let other_uri = hick_lsp::ProjectContexts::source_uri(capture, distinct).unwrap();
    assert_ne!(uri, other_uri);
    assert!(uri.contains("%252F%20%23.py"), "{uri}");
    assert!(other_uri.contains("%25252F%20%23.py"), "{other_uri}");
    let (_, path, context) = projects
        .get(&tower_lsp::lsp_types::Url::parse(&uri).unwrap())
        .unwrap();
    assert_eq!(path, primary);
    assert!(!context.is_materialized());
    let registry = Arc::new(Registry::with_projects(
        projects.clone(),
        host_root.path().to_path_buf(),
    ));
    let started=ask(&registry,host_root.path(),json!({"op":"start","doc":uri,"request_id":"captured-literal-launch","source_hash":hash,"breakpoints":[{"line":1}]})).await;
    let launch = event(&started, "started");
    assert_eq!(launch["doc"], uri);
    assert_eq!(launch["request_id"], "captured-literal-launch");
    assert_eq!(launch["source_hash"], hash);
    let session = launch["session"].as_str().unwrap().to_owned();
    let stopped = event(&started, "stopped");
    let stop = stopped["stop"].as_u64().unwrap();
    let thread = stopped["thread"].as_i64().unwrap();
    let frame = stopped["frames"]
        .as_array()
        .unwrap()
        .iter()
        .find(|frame| frame["name"] == "<module>")
        .unwrap();
    assert_eq!(frame["source"], uri, "{frame}");
    assert_eq!(frame["source_hash"], hash, "{frame}");
    assert_ne!(frame["source"], other_uri);
    assert_eq!(frame["source_line"], 1);
    assert_eq!(stopped["line"], 1);
    let frame_id = frame["id"].as_i64().unwrap();
    let evaluated=ask(&registry,host_root.path(),json!({"op":"eval","session":session,"expression":"marker","context":"watch","frame":frame_id,"stop":stop,"thread":thread,"request_id":"captured-literal-value"})).await;
    let value = event(&evaluated, "value");
    assert_eq!(value["value"], "41");
    assert_eq!(value["stop"], stop);
    assert_eq!(value["thread"], thread);
    assert_eq!(value["frame"], frame_id);
    assert_eq!(value["request_id"], "captured-literal-value");
    let changed=ask(&registry,host_root.path(),json!({"op":"breakpoints","session":session,"source_hash":hash,"request_id":"captured-literal-condition","breakpoints":[{"line":1,"condition":"marker == 41"}]})).await;
    let receipt = event(&changed, "breakpoints");
    assert_eq!(receipt["request_id"], "captured-literal-condition");
    assert_eq!(receipt["source_hash"], hash);
    assert!(
        receipt["breakpoints"]
            .as_array()
            .unwrap()
            .iter()
            .any(|mark| mark["line"] == 1 && mark["state"] == "bound"),
        "{receipt}"
    );
    let ended = ask(
        &registry,
        host_root.path(),
        json!({"op":"stop","session":session}),
    )
    .await;
    assert_eq!(event(&ended, "ended")["session"], session);
    assert!(registry.get(&session).await.is_err());
    assert!(
        !context.is_materialized(),
        "Debugging must not materialize the retained comparison context"
    );
    assert_eq!(
        context.tree.get(primary).unwrap().bytes.as_ref(),
        source.as_bytes()
    );
    assert_eq!(
        context.tree.get(distinct).unwrap().bytes.as_ref(),
        other.as_bytes()
    );
    assert_eq!(host_entries(host_root.path()), baseline);
    assert_eq!(
        std::fs::read_link(host_root.path().join(".venv")).unwrap(),
        environment
    );
    assert!(!host_root.path().join(primary).exists());
    assert!(!host_root.path().join(distinct).exists());
}
</storycode:copy>

<storycode:file path="crates/hickory-cli/src/serve/debug_bridge/tests/tests.rs"><storycode:paste select="#f-83c29f05f862-agent_action_fixture_uses_opaque_handles_without_display_line_arithmetic"/><storycode:paste select="#f-83c29f05f862-real_adapter_relocation_and_stale_actions_share_the_reading"/><storycode:paste select="#f-83c29f05f862-a_failed_build_reaches_the_client_before_the_failure"/><storycode:paste select="#f-83c29f05f862-a_build_line_is_shaped_like_a_transcript_event"/><storycode:paste select="#f-83c29f05f862-a_document_is_named_in_the_clients_scheme"/><storycode:paste select="#f-83c29f05f862-requests_parse_from_their_frame"/><storycode:paste select="#f-83c29f05f862-a_step_names_the_session_apis_own_verb"/><storycode:paste select="#f-83c29f05f862-a_response_frame_is_tagged_with_the_debug_channel"/><storycode:paste select="#f-83c29f05f862-a_failure_carries_words_rather_than_a_code"/><storycode:paste select="#f-83c29f05f862-a_failure_about_nothing_in_particular_carries_no_lines"/><storycode:paste select="#f-83c29f05f862-the_channel_carries_no_way_to_edit_anything"/><storycode:paste select="#debugger-thread-stop-witness"/></storycode:file>

## Implementation review

### Implementation

- [apps/web/playwright.embed.config.ts](../../../mechanisms/apps/web/playwright.embed.config.ts.md#output-6fcf90daa368)
- [apps/web/src/debug/ObservationReading.tsx](contract.md#output-a714deb0aad7)
- [apps/web/src/debug/browser/runtime.ts](../../../mechanisms/apps/web/src/debug/browser/runtime.ts.md#output-6c24fa24bf81)
- [apps/web/src/debug/browser/transport.ts](../../../mechanisms/apps/web/src/debug/browser/transport.ts.md#output-4357b3feff1c)
- [apps/web/src/debug/observations.ts](contract.md#output-4abce3fbcd42)
- [apps/web/src/views/WorkspaceChat.tsx](../../agent/unsent-responses-survive-restart/contract.md#output-c910953d9e7f)
- [crates/hick-dap/src/mapping.rs](../../../mechanisms/crates/hick-dap/src/mapping.rs.md#output-32f353d4cad8)
- [crates/hick-observe/src/reading.rs](contract.md#output-a6f0ea9e8feb)
- [crates/hickory-cli/src/debug_reading.rs](contract.md#output-d5f4bc355f22)
- [crates/hickory-cli/src/python_agent.rs](../../../mechanisms/crates/hickory-cli/src/python_agent.rs.md#output-609d98dd1fad)
- [crates/hickory-cli/src/serve/debug_bridge.rs](../../../mechanisms/crates/hickory-cli/src/serve/debug_bridge.rs.md#output-b6fc115ecefd)

### Tests and fixtures

- [apps/web/e2e/browserEmbedding.spec.ts](../../../mechanisms/apps/web/e2e/browserEmbedding.spec.ts.md#output-31019d3567c7)
- [apps/web/src/debug/ObservationReading.test.tsx](../../../mechanisms/apps/web/src/debug/ObservationReading.test.tsx.md#output-ec33c3e05a60)
- [apps/web/src/debug/browser/runtime.test.ts](../../../mechanisms/apps/web/src/debug/browser/runtime.ts.md#output-de529cc60d5d)
- [apps/web/src/debug/fixtures/reading.json](files/apps/web/src/debug/fixtures/reading.json.md)
- [apps/web/src/debug/inlineEval.test.tsx](contract.md#output-d5d367dd354b)
- [apps/web/src/debug/providerObservations.test.ts](contract.md#output-a0a51eb651cb)
- [crates/hickory-cli/src/serve/debug_bridge/tests/tests.rs](contract.md#output-83c29f05f862)

### Evidence status

- Implementation review: not verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.


## Supporting literal inputs

- [`apps/web/src/debug/fixtures/reading.json`](files/apps/web/src/debug/fixtures/reading.json.md)
<storycode:include file="files/apps/web/src/debug/fixtures/reading.json.md" />
</storycode:doc>
