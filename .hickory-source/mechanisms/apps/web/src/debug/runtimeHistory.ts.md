<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/web/src/debug/runtimeHistory.ts

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-3663d16e0736"></a>
## `apps/web/src/debug/runtimeHistory.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

<a id="f-3663d16e0736-runRepository"></a>
### `runRepository`

<storycode:copy id="f-3663d16e0736-runRepository">
import { observationText, type RuntimeRun } from "./runtimeObservations";
import type { WorkspaceStorage } from "../embed/storage";
import type { ObservationReading } from "./observations";

/** Reuses transactional workspace storage, with the same run wire model as Rust. */
export function runRepository(storage: WorkspaceStorage) {
  const path = (id: string) => {
    if (!/^[a-zA-Z0-9-]+$/.test(id)) throw Error("Invalid run identity");
    return `runs/${id}.json`;
  };
  const parse = (bytes: Uint8Array): RuntimeRun => {
    const run = JSON.parse(new TextDecoder().decode(bytes)) as RuntimeRun;
    if (run.schema_version !== 1 || !Array.isArray(run.events) || !Array.isArray(run.sources) || !run.sources.length) throw Error("Unsupported/corrupt runtime record");
    return run;
  };
  const verify = async (run: RuntimeRun) => {
    for (const source of run.sources) {
      const hash = await crypto.subtle.digest("SHA-256", new Uint8Array(source.bytes));
      if ([...new Uint8Array(hash)].map(b => b.toString(16).padStart(2, "0")).join("") !== source.revision) throw Error("Captured source digest mismatch");
    }
    if (new Set(run.events.map(event => event.id)).size !== run.events.length) throw Error("Duplicate event identity");
  };
  return {
    async list() {
      const runs = (await storage.snapshot()).filter(file => file.path.startsWith("runs/")).map(file => parse(file.bytes));
      for (const run of runs) await verify(run);
      return runs;
    },
    async save(run: RuntimeRun, pinUpdate = false) {
      await verify(run);
      const target = path(run.id);
      const previous = await storage.read(target);
      let candidate = run;
      if (previous) {
        const old = parse(previous.bytes);
        if (JSON.stringify(old.sources) !== JSON.stringify(run.sources) || old.started_unix_ms !== run.started_unix_ms ||
          JSON.stringify(run.events.slice(0, old.events.length)) !== JSON.stringify(old.events)) throw Error("Captured runtime evidence is immutable");
        if (old.status !== "running" && (run.status !== old.status || JSON.stringify(run.events) !== JSON.stringify(old.events))) throw Error("Finished run is immutable");
        if (old.truncated && !run.truncated) throw Error("Cannot erase truncation");
        candidate = { ...run, pinned: pinUpdate ? run.pinned : old.pinned };
      }
      const bytes = new TextEncoder().encode(JSON.stringify(candidate));
      if (bytes.length > 16 * 1024 * 1024) throw Error("Retained runtime record exceeds 16 MiB");
      await storage.write(target, bytes, previous?.revision ?? null);
      if (candidate.status !== "running") {
        const terminal = (await storage.snapshot()).filter(file => file.path.startsWith("runs/")).map(file => ({ file, run: parse(file.bytes) }))
          .filter(item => !item.run.pinned && item.run.status !== "running").sort((a,b) => a.run.started_unix_ms - b.run.started_unix_ms);
        let retainedBytes = terminal.reduce((sum, item) => sum + item.file.bytes.length, 0);
        let count = terminal.length;
        for (const item of terminal) if (count > 100 || retainedBytes > 256 * 1024 * 1024 || Date.now() - item.run.started_unix_ms > 30 * 86400000) {
          await storage.delete(item.file.path, item.file.revision); retainedBytes -= item.file.bytes.length; count--;
        }
      }
      return candidate;
    },
    async remove(id: string) { const old = await storage.read(path(id)); if (old) await storage.delete(old.path, old.revision); },
    close() { storage.close(); },
  };
}

</storycode:copy>
<a id="f-3663d16e0736-runtimeReading"></a>
### `runtimeReading`

<storycode:copy id="f-3663d16e0736-runtimeReading">
export function runtimeReading(run: RuntimeRun, invocation?: string): ObservationReading {
  const source = run.sources[0];
  let providerVersion = "unknown";
  try { providerVersion = JSON.parse(run.detail ?? "{}").provider_version ?? "unknown"; } catch { /* Legacy plain-text detail. */ }
  const locations: ObservationReading["locations"] = [];
  const selected = run.events.filter(event => !invocation || (event.invocation ?? "unknown") === invocation);
  const annotations = selected.map(event => {
    let location: string | null = null;
    const anchor = event.observation.kind === "exception" ? event.observation.throwing ?? event.emission : event.emission;
    if (anchor?.source === source.id && anchor.bytes) {
      location = `${source.revision}:byte:${anchor.bytes.start}`;
      if (!locations.some(item => item.id === location)) locations.push({ id: location, revision: source.revision,
        line: new TextDecoder().decode(new Uint8Array(source.bytes.slice(0, anchor.bytes.start))).split("\n").length - 1,
        location: { source: source.id, bytes: anchor.bytes, precision: anchor.precision } });
    }
    return { evidence: event.id, provider: event.provider, payload_schema: "hick.runtime-event.v1", location,
      scope: { session: run.id, run: run.id, stop: "runtime", frame: null, invocation: event.invocation },
      payload: { kind: event.observation.kind, event, run_status: run.status, truncated: run.truncated } };
  });
  return { schema_version: 1, source, scope: { session: run.id, run: run.id, stop: "runtime", frame: null, invocation: invocation ?? null },
    locations, annotations, provider_annotations: selected.map(event => {
      const annotation = annotations.find(item => item.evidence === event.id)!;
      const location = locations.find(item => item.id === annotation.location) ?? null;
      return { evidence: event.id, provider: event.provider, provider_version: providerVersion,
        payload_schema: `hick.${event.observation.kind}.v1`,
        origin: { kind: "runtime" as const, session: null, run: run.id, stop: null, frame: null, invocation: event.invocation },
        location, source_precision: location?.location.precision ?? "unknown",
        measurement_semantics: event.observation.kind === "profile" ? event.observation.aggregation : null,
        uncertainty: event.observation.kind === "profile" ? "Profiler overhead and timing variability are not estimated" : null,
        payload: event.observation };
    }), unsupported: ["model semantic support", ...(run.truncated ? ["complete event coverage (truncated)"] : [])] };
}

</storycode:copy>
<a id="f-3663d16e0736-runtimeText"></a>
### `runtimeText`

<storycode:copy id="f-3663d16e0736-runtimeText">
export function runtimeText(run: RuntimeRun, invocation?: string) {
  return run.events.filter(event => !invocation || (event.invocation ?? "unknown") === invocation).map(event =>
    `${event.id} · invocation ${event.invocation ?? "unknown"} · ${event.emission?.precision ?? "unknown location"}\n` +
    observationText(event.observation)).join("")
    .replaceAll("\r", "\\r").replaceAll("\u001b", "\\x1b");
}
</storycode:copy>

### Assemble `apps/web/src/debug/runtimeHistory.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/debug/runtimeHistory.ts"><storycode:paste select="#f-3663d16e0736-runRepository"/><storycode:paste select="#f-3663d16e0736-runtimeReading"/><storycode:paste select="#f-3663d16e0736-runtimeText"/></storycode:file>


## Exercise this mechanism

The test module below imports this implementation directly. Keep the assertions with the mechanism they exercise; passing reconstruction alone does not show those assertions ran.

<a id="output-b01fe4e29028"></a>
## `apps/web/src/debug/runtimeHistory.test.tsx`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [Explicit UI action dispatch checks scope, state, policy and retries](../../../../../guarantees/agent/ui-action-dispatch/contract.md), [Python observations stay confined and source bound](../../../../../guarantees/execution/python-observations-stay-confined/contract.md), [Python profiles keep function scope and measurement semantics](../../../../../guarantees/execution/python-profile-keeps-function-semantics/contract.md), [Runtime history keeps runs and invocation occurrences separate](../../../../../guarantees/execution/runtime-history-keeps-runs-and-occurrences-separate/contract.md).

<a id="f-b01fe4e29028-expression_statement"></a>
### `expression_statement`

<storycode:copy id="f-b01fe4e29028-expression_statement">
// @vitest-environment jsdom
// Guarantee: docs/guarantees/execution/runtime-history-keeps-runs-and-occurrences-separate.md
import { afterEach, expect, it, vi } from "vitest";
import { act, cleanup, fireEvent, render, screen, waitFor } from "@testing-library/react";
import { UiActions } from "../lib/uiActions";
import { UiActionContext } from "../lib/uiActionBridge";
import { EditorState } from "@codemirror/state";
import { EditorView } from "@codemirror/view";
import { createMemoryStorage } from "../embed/storage";
import { runRepository, runtimeReading } from "./runtimeHistory";
import { RuntimeHistory, NativeRuntimeHistory } from "./RuntimeRuns";
import { observationContext } from "./observations";
import type { RuntimeRun } from "./runtimeObservations";
import { request } from "../api/client";
vi.mock("../api/client", () => ({ request: vi.fn() }));
</storycode:copy>
<a id="f-b01fe4e29028-expression_statement-2"></a>
### `expression_statement`

<storycode:copy id="f-b01fe4e29028-expression_statement-2">
afterEach(() => { cleanup(); vi.mocked(request).mockReset(); });
</storycode:copy>
<a id="f-b01fe4e29028-fixture"></a>
### `fixture`

<storycode:copy id="f-b01fe4e29028-fixture">
async function fixture(id = "run-one"): Promise<RuntimeRun> {
  const bytes = [...new TextEncoder().encode("# 🦀\nconsole.log('hello');\n")];
  const revision = [...new Uint8Array(await crypto.subtle.digest("SHA-256", new Uint8Array(bytes)))].map(byte => byte.toString(16).padStart(2, "0")).join("");
  return { schema_version: 1, id, started_unix_ms: Date.now(), status: "completed", pinned: false,
    sources: [{ id: "document", uri: "example.md", revision, bytes }], truncated: false, missing_artifacts: [], detail: null,
    events: [{ id: `${id}-E1`, provider: "browser-interpreter", block: "main.js", invocation: "call-1", parent_invocation: null,
      thread: null, task: null, time: null, emission: { source: "document", block: "main.js", bytes: { start: 7, end: 8 }, precision: "exact" }, callers: [],
      observation: { kind: "output", stream: "stdout", category: "log", bytes: [...new TextEncoder().encode("hello\n")] } }] };
}
</storycode:copy>
<a id="f-b01fe4e29028-expression_statement-3"></a>
### `expression_statement`

<storycode:copy id="f-b01fe4e29028-expression_statement-3">
it("retains immutable evidence, explicit pins and digest validation", async () => {
  const storage = createMemoryStorage(); const repository = runRepository(storage); const run = await fixture();
  await repository.save(run); await repository.save({ ...run, pinned: true }, true);
  expect((await repository.list())[0].pinned).toBe(true);
  await expect(repository.save({ ...run, events: [] })).rejects.toThrow("immutable");
  const corrupted = structuredClone(run); corrupted.sources[0].bytes[0] = 65;
  await expect(repository.save(corrupted)).rejects.toThrow("digest");
  await repository.remove(run.id); expect(await repository.list()).toEqual([]);
  await storage.write("runs/tampered.json", new TextEncoder().encode(JSON.stringify(corrupted)), null);
  await expect(repository.list()).rejects.toThrow("digest");
});
</storycode:copy>
<a id="f-b01fe4e29028-expression_statement-4"></a>
### `expression_statement`

<storycode:copy id="f-b01fe4e29028-expression_statement-4">
it("selects latest failure separately, preserves source positions, and supplies exact selected event IDs", async () => {
  const first = await fixture(); const failed = await fixture("run-two"); failed.started_unix_ms++; failed.status = "failed"; failed.truncated = true;
  const source = new TextDecoder().decode(new Uint8Array(first.sources[0].bytes));
  const host = document.createElement("div"); document.body.append(host);
  const view = new EditorView({ state: EditorState.create({ doc: source }), parent: host });
  const pin = vi.fn(async () => undefined);
  const rendered = render(<RuntimeHistory runs={[first, failed]} source={source} view={view} onPin={pin} onDelete={vi.fn()} />);
  expect(screen.getByText(/^Run run-two · failed/)).toBeTruthy();
  expect(view.state.doc.toString()).toBe(source);
  expect(host.querySelector(".runtime-inline")).toBeTruthy();
  fireEvent.change(screen.getByLabelText("Runtime run"), { target: { value: first.id } });
  expect(screen.getByText(/run-one · completed · historical/)).toBeTruthy();
  fireEvent.click(screen.getByRole("button", { name: "Pin run" })); await waitFor(() => expect(pin).toHaveBeenCalledWith(first));
  const details = screen.getByText(/Source observations/).closest("details")!;
  details.open = true; fireEvent(details, new Event("toggle"));
  await waitFor(() => expect(observationContext("example.md", source)[0]?.evidence).toEqual([first.events[0].id]));
  fireEvent.change(screen.getByLabelText("Compare retained run"), { target: { value: failed.id } });
  expect(screen.getByLabelText("Separate run comparison")).toBeTruthy();
  rendered.rerender(<RuntimeHistory runs={[first, failed]} source={source + "edited"} view={view} onPin={pin} onDelete={vi.fn()} />);
  expect(screen.getByText(/source changed \(stale\)/)).toBeTruthy();
  expect(host.querySelector(".runtime-inline")).toBeNull();
  expect(runtimeReading(first).annotations[0].evidence).toBe(first.events[0].id);
  view.destroy(); host.remove();
});

</storycode:copy>
<a id="f-b01fe4e29028-expression_statement-5"></a>
### `expression_statement`

<storycode:copy id="f-b01fe4e29028-expression_statement-5">
// Guarantee: docs/guarantees/execution/python-observations-stay-confined.md
it("explicit native launch executes the saved path and refreshes retained evidence", async () => {
  const run = await fixture(); run.sources[0].uri = "example.md";
  let captured = false;
  vi.mocked(request).mockImplementation(async (method) => {
    if (method === "POST") { captured = true; return { run }; }
    return { runs: captured ? [{ state: "present", run }] : [] };
  });
  render(<NativeRuntimeHistory path="example.md" source="unsaved content" />);
  const details = screen.getByText("Runtime observations").closest("details")!;
  details.open = true; fireEvent(details, new Event("toggle"));
  await screen.findByRole("button", { name: "Run saved Python confined" });
  fireEvent.change(screen.getByLabelText(/Generated Python entry/), { target: { value: "main.py" } });
  fireEvent.click(screen.getByRole("button", { name: "Run saved Python confined" }));
  await waitFor(() => expect(request).toHaveBeenCalledWith("POST", "/api/runtime/python", { document: "example.md", program: "main.py", seconds: 10, profile: false }));
  await screen.findByText(/Run run-one · completed/);
  expect(screen.getByText(/Unsaved edits are not executed/)).toBeTruthy();
});

</storycode:copy>
<a id="f-b01fe4e29028-expression_statement-6"></a>
### `expression_statement`

<storycode:copy id="f-b01fe4e29028-expression_statement-6">
// Guarantee: docs/guarantees/agent/ui-action-dispatch.md
it("agent profiling invokes native capture and returns the saved-source receipt without certifying unsaved edits", async () => {
  const run = await fixture("profile-saved-source"); run.sources[0].uri = "example.md";
  let captured = false;
  vi.mocked(request).mockImplementation(async method => {
    if (method === "POST") { captured = true; return { run }; }
    return { runs: captured ? [{ state: "present", run }] : [] };
  });
  const registry = new UiActions({ workspace: "profile", window: "native" }, () => true, async () => {});
  render(<UiActionContext.Provider value={registry}><NativeRuntimeHistory path="example.md" source="unsaved source is different" /></UiActionContext.Provider>);
  const descriptor = (label: string) => registry.groups().groups.flatMap(group => registry.observe(group, 64).actions).find(action => action.label === label)!;
  const actor = { id: "python:profile-loop", kind: "agent" as const };
  await act(async () => {
    const disclosure = descriptor("Runtime observations");
    await registry.act(registry.scope, actor, { handle: disclosure.handle, arguments: {}, request_id: "disclose" });
  });
  await screen.findByRole("button", { name: "Run saved Python confined" });
  await act(async () => {
    const capture = descriptor("Run saved Python confined");
    const result = await registry.act(registry.scope, actor, { handle: capture.handle, arguments: { program: "main.py", profile: true }, request_id: "profile" });
    expect(result).toMatchObject({ status: "result", outcome: { status: "completed", value: { run: run.id, sources: [{ revision: run.sources[0].revision }], interpretation: "Inspect actual correctness output and profile metrics; completion alone is not verification" } } });
  });
  expect(request).toHaveBeenCalledWith("POST", "/api/runtime/python", { document: "example.md", program: "main.py", seconds: 10, profile: true });
  await screen.findByText(/source changed \(stale\)/);
  expect(screen.getByText(/Unsaved edits are not executed/)).toBeTruthy();
});
</storycode:copy>

### Assemble `apps/web/src/debug/runtimeHistory.test.tsx`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/debug/runtimeHistory.test.tsx"><storycode:paste select="#f-b01fe4e29028-expression_statement"/><storycode:paste select="#f-b01fe4e29028-expression_statement-2"/><storycode:paste select="#f-b01fe4e29028-fixture"/><storycode:paste select="#f-b01fe4e29028-expression_statement-3"/><storycode:paste select="#f-b01fe4e29028-expression_statement-4"/><storycode:paste select="#f-b01fe4e29028-expression_statement-5"/><storycode:paste select="#f-b01fe4e29028-expression_statement-6"/></storycode:file>


</storycode:doc>
