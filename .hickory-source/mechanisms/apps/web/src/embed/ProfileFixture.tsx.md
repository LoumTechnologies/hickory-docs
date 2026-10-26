<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/web/src/embed/ProfileFixture.tsx

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-0c342a8c9778"></a>
## `apps/web/src/embed/ProfileFixture.tsx`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

<a id="f-0c342a8c9778-ProfileFixture"></a>
### `ProfileFixture`

Explicit recorded-profile host. Importing a record never launches its program.

<storycode:copy id="f-0c342a8c9778-ProfileFixture">
// Explicit recorded-profile host. Importing a record never launches its program.
import { useEffect, useRef, useState } from "react";
import type { EditorView } from "@codemirror/view";
import { DocumentEmbed } from "./DocumentEmbed";
import { RuntimeHistory } from "../debug/RuntimeRuns";
import { observationContext } from "../debug/observations";
import type { RuntimeRun } from "../debug/runtimeObservations";
import { decodeRuntimeRun } from "../debug/decodeRuntimeRun";
import { useUiCommands, useUiRevision } from "../lib/uiActionBridge";
import { useUiReading } from "../lib/uiReading";
export function ProfileFixture() {
  const [run, setRun] = useState<RuntimeRun | null>(null), [source, setSource] = useState("");
  const [view, setView] = useState<EditorView | null>(null), [context, setContext] = useState("");
  const [error, setError] = useState(""), [busy, setBusy] = useState(false);
  const owner = useRef(0);
  const editorOwner = useRef({ view, generation: 0 });
  if (editorOwner.current.view !== view) editorOwner.current = { view, generation: editorOwner.current.generation + 1 };
  useEffect(() => () => { owner.current++; }, []);
  const revision = useUiRevision(() => [run, source, editorOwner.current.generation, context, busy, owner.current]);
  useUiReading("retained profile import status", { run: run?.id ?? null, source, context, busy, error, execution: "recorded import only", current_certification: false });
  const visible = (commands: { key: string; command: import("../lib/uiActions").UiCommand }[]) => run ? commands : commands.filter(command => command.key === "profile-fixture:import");
  const invoke = useUiCommands(visible([
    { key: "profile-fixture:import", command: { label: "Import retained profile JSON", parent: "profile-fixture", subject: "profile-fixture:record", parameters: { json: { type: "string", maxLength: 2 * 1024 * 1024 } }, prerequisites: ["Explicit captured runtime record; source bytes and hash must match. Producer/execution are not authenticated"], read: () => ({ revision: revision(), enabled: !busy }), execute: async args => {
      const captured = ++owner.current; setBusy(true); setError("");
      try { const record = await decodeRuntimeRun(String(args.json)); if (captured !== owner.current) return { status: "unavailable", reason: "A newer profile import superseded this read" }; setRun(record); setSource(new TextDecoder("utf-8", { fatal: true, ignoreBOM: true }).decode(new Uint8Array(record.sources[0].bytes))); setContext(""); return { status: "completed", value: { loaded: true, historical: true, executed: false, current_certification: false } }; }
      catch (reason) { if (captured === owner.current) setError(String(reason)); return { status: "unavailable", reason: String(reason) }; }
      finally { if (captured === owner.current) setBusy(false); }
    } } },
    { key: "profile-fixture:context", command: { label: "Inspect selected profile context", parent: "profile-fixture", subject: "profile-fixture:context", parameters: {}, prerequisites: [], read: () => ({ revision: revision(), enabled: !!run && !busy }), execute: async () => { const reading = JSON.stringify(observationContext(run!.sources[0].uri, source)); setContext(reading); return { status: "completed", value: { context: reading, recorded: true } }; } } },
    { key: "profile-fixture:edit", command: { label: "Edit after captured profile", parent: "profile-fixture", subject: "profile-fixture:record", parameters: {}, prerequisites: [], read: () => ({ revision: revision(), enabled: !!run && !busy }), execute: async () => { setSource(source + "\n# edited after capture"); return { status: "completed", value: { edited: true, capture_retained: true, executed: false } }; } } },
  ]));
  const act = (key: string, args = {}) => { const captured = owner.current; void invoke(`profile-fixture:${key}`, args).catch(reason => { if (owner.current === captured) setError(String(reason)); }); };
  return <main><h1>Retained profile replay</h1><p>This browser does not execute Python. Load an actual saved native capture; source and metrics remain captured evidence. Pin/remove affect this page only.</p>
    {error && <p role="alert">{error}</p>}
    <label>Retained profile record<input type="file" accept=".json" disabled={busy} onChange={event => {
      const file = event.target.files?.[0]; if (!file) return;
      const captured = ++owner.current;
      if (file.size > 2 * 1024 * 1024) { setError("Recorded runtime import exceeds 2 MiB"); return; }
      void file.text().then(json => { if (captured === owner.current) act("import", { json }); }).catch(reason => { if (captured === owner.current) setError(String(reason)); });
    }} /></label>
    {run && <><DocumentEmbed path={run.sources[0].uri} revision={source} source={source} onChange={event => { owner.current++; setBusy(false); setSource(event.source); }} onViewReady={setView} />
      <RuntimeHistory runs={[run]} source={source} view={view} onPin={async selected => { setRun(current => current === selected ? { ...current, pinned: !current.pinned } : current); }} onDelete={async selected => { if (run === selected) { owner.current++; setBusy(false); setRun(null); setSource(""); setContext(""); } }} />
      <button disabled={busy} onClick={() => act("context")}>Inspect selected profile context</button>
      <output aria-label="Selected profile context">{context}</output>
      <button disabled={busy} onClick={() => act("edit")}>Edit after captured profile</button></>}
  </main>;
}
</storycode:copy>

### Assemble `apps/web/src/embed/ProfileFixture.tsx`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/embed/ProfileFixture.tsx"><storycode:paste select="#f-0c342a8c9778-ProfileFixture"/></storycode:file>


## Exercise this mechanism

The test module below imports this implementation directly. Keep the assertions with the mechanism they exercise; passing reconstruction alone does not show those assertions ran.

<a id="output-a43a8523337a"></a>
## `apps/web/src/embed/ProfileFixture.test.tsx`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

<a id="f-a43a8523337a-expression_statement"></a>
### `expression_statement`

<storycode:copy id="f-a43a8523337a-expression_statement">
// Guarantees: agent/ui-action-dispatch.md; execution/python-profile-keeps-function-semantics.md
import { act, cleanup, fireEvent, render, screen } from "@testing-library/react";
import { afterEach, expect, it, vi } from "vitest";
import record from "../debug/fixtures/python-profile.json";
import { UiActions } from "../lib/uiActions";
import { UiActionContext } from "../lib/uiActionBridge";
import { ProfileFixture } from "./ProfileFixture";
vi.mock("./DocumentEmbed", () => ({ DocumentEmbed: ({ source }: { source: string }) => <pre data-testid="source">{source}</pre> }));
</storycode:copy>
<a id="f-a43a8523337a-expression_statement-2"></a>
### `expression_statement`

<storycode:copy id="f-a43a8523337a-expression_statement-2">
vi.mock("../debug/RuntimeRuns", () => ({ RuntimeHistory: ({ runs, onPin, onDelete }: { runs: { pinned: boolean }[]; onPin: (run: unknown) => Promise<void>; onDelete: (run: unknown) => Promise<void> }) => <><output data-testid="pinned">{String(runs[0].pinned)}</output><button onClick={() => void onPin(runs[0])}>Pin run</button><button onClick={() => void onDelete(runs[0])}>Remove run</button></> }));
</storycode:copy>
<a id="f-a43a8523337a-expression_statement-3"></a>
### `expression_statement`

<storycode:copy id="f-a43a8523337a-expression_statement-3">
afterEach(cleanup);
</storycode:copy>
<a id="f-a43a8523337a-expression_statement-4"></a>
### `expression_statement`

<storycode:copy id="f-a43a8523337a-expression_statement-4">
it("imports through the shared controller, retains the valid record after refusal, and applies local pin/remove", async () => {
  const registry = new UiActions({ workspace: "recorded", window: "profile" }, () => true, async () => {});
  render(<UiActionContext.Provider value={registry}><ProfileFixture /></UiActionContext.Provider>);
  const load = async (json: string) => {
    const target = registry.observe("profile-fixture").actions.find(action => action.label === "Import retained profile JSON")!;
    let result: unknown;
    await act(async () => { result = await registry.act(registry.scope, { id: "parent", kind: "agent" }, { handle: target.handle, arguments: { json }, request_id: crypto.randomUUID() }); });
    return result;
  };
  expect(await load(JSON.stringify(record))).toMatchObject({ status: "result", outcome: { status: "completed", value: { loaded: true, executed: false, historical: true } } });
  const source = screen.getByTestId("source").textContent;
  const invalid = structuredClone(record); invalid.sources[0].bytes[0] ^= 1;
  expect(await load(JSON.stringify(invalid))).toMatchObject({ status: "result", outcome: { status: "unavailable" } });
  expect(screen.getByRole("alert").textContent).toContain("source hash");
  expect(screen.getByTestId("source").textContent).toBe(source);
  fireEvent.click(screen.getByRole("button", { name: "Pin run" }));
  expect(screen.getByTestId("pinned").textContent).toBe("true");
  fireEvent.click(screen.getByRole("button", { name: "Remove run" }));
  expect(screen.queryByTestId("source")).toBeNull();
});
</storycode:copy>

### Assemble `apps/web/src/embed/ProfileFixture.test.tsx`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/embed/ProfileFixture.test.tsx"><storycode:paste select="#f-a43a8523337a-expression_statement"/><storycode:paste select="#f-a43a8523337a-expression_statement-2"/><storycode:paste select="#f-a43a8523337a-expression_statement-3"/><storycode:paste select="#f-a43a8523337a-expression_statement-4"/></storycode:file>


</storycode:doc>
