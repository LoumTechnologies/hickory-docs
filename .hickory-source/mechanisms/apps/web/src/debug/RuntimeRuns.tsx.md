<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/web/src/debug/RuntimeRuns.tsx

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-56919d2a9983"></a>
## `apps/web/src/debug/RuntimeRuns.tsx`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

<a id="f-56919d2a9983-useBrowserRuns"></a>
### `useBrowserRuns`

<storycode:copy id="f-56919d2a9983-useBrowserRuns">
import { ProfileEvidence } from "./ProfileEvidence";
import { RuntimeLog } from "./RuntimeLogView";
import { useEffect, useId, useMemo, useRef, useState } from "react";
import { useRuntimeActions } from "./uiRuntimeActions";
import { useUiCommands } from "../lib/uiActionBridge";
import type { EditorView } from "@codemirror/view";
import { StateEffect } from "@codemirror/state";
import { ObservationReading } from "./ObservationReading";
import { runtimeReading, runtimeText, runRepository } from "./runtimeHistory";
import type { RuntimeRun } from "./runtimeObservations";
import { runtimeEditor, setRuntimeReading } from "./cmRuntime";
import { openLocalStorage } from "../embed/storage";
import { request } from "../api/client";

export function useBrowserRuns(path: string) {
  const [runs, setRuns] = useState<RuntimeRun[]>([]);
  const [error, setError] = useState("");
  const repository = useMemo(() => openLocalStorage(`runtime:${path}`).then(runRepository), [path]);
  const gate = useRef(Promise.resolve());
  useEffect(() => {
    let mounted = true;
    repository.then(repo => repo.list()).then(saved => { if (mounted) setRuns(previous => [...saved.filter(run => !previous.some(p => p.id === run.id)), ...previous]); }).catch(e => setError(String(e)));
    return () => { mounted = false; };
  }, [repository]);
  function observe(run: RuntimeRun) {
    setRuns(previous => [...previous.filter(old => old.id !== run.id), { ...run, pinned: previous.find(old => old.id === run.id)?.pinned ?? run.pinned }]);
    if (run.status !== "running" || run.events.length % 100 === 0) {
      gate.current = gate.current.then(async () => { await (await repository).save(run); }).catch(e => setError(String(e)));
    }
  }
  async function pin(run: RuntimeRun) {
    const saved = await (await repository).save({ ...run, pinned: !run.pinned }, true);
    setRuns(previous => previous.map(old => old.id === run.id ? saved : old));
  }
  async function remove(run: RuntimeRun) {
    await (await repository).remove(run.id); setRuns(previous => previous.filter(old => old.id !== run.id));
  }
  return { runs, observe, pin, remove, error };
}

</storycode:copy>
<a id="f-56919d2a9983-RuntimeHistory"></a>
### `RuntimeHistory`

<storycode:copy id="f-56919d2a9983-RuntimeHistory">
export function RuntimeHistory({ runs, source, view, onPin, onDelete }: {
  runs: RuntimeRun[]; source?: string; view?: EditorView | null;
  onPin: (run: RuntimeRun) => Promise<void>; onDelete: (run: RuntimeRun) => Promise<void>;
}) {
  const [selected, setSelected] = useState("");
  const [invocation, setInvocation] = useState("");
  const [comparison, setComparison] = useState("");
  const [error, setError] = useState("");
  const sorted = [...runs].sort((a, b) => b.started_unix_ms - a.started_unix_ms || b.id.localeCompare(a.id));
  const latest = sorted[0];
  const run = sorted.find(item => item.id === selected) ?? latest;
  const compared = sorted.find(item => item.id === comparison);
  const reading = useMemo(() => run?.sources[0] ? runtimeReading(run, invocation) : null, [run, invocation]);
  const stale = !!run && source !== undefined && new TextDecoder().decode(new Uint8Array(run.sources[0]?.bytes ?? [])) !== source;
  useRuntimeActions(sorted, run, invocation, comparison, stale, { select: id => { setSelected(id); setInvocation(""); }, occurrence: setInvocation, compare: setComparison, pin: onPin, remove: onDelete });
  useEffect(() => {
    if (!view) return;
    if (!view.state.field(runtimeEditor, false)) view.dispatch({ effects: StateEffect.appendConfig.of(runtimeEditor) });
    view.dispatch({ effects: setRuntimeReading.of(stale ? null : reading) });
    return () => { if (view.dom.isConnected) view.dispatch({ effects: setRuntimeReading.of(null) }); };
  }, [view, reading, stale]);
  if (!run) return <p>No captured runtime runs yet. Capturing and selecting agent context are separate actions.</p>;
  const invocations = [...new Set(run.events.map(event => event.invocation ?? "unknown"))];
  async function act(action: () => Promise<void>) { try { await action(); setError(""); } catch (e) { setError(String(e)); } }
  function capture() {
    const bytes = JSON.stringify(run, null, 2);
    const url = URL.createObjectURL(new Blob([bytes], { type: "application/json" }));
    const link = document.createElement("a"); link.href = url; link.download = `runtime-${run!.id}.json`; link.click();
    setTimeout(() => URL.revokeObjectURL(url), 1000);
  }
  return <section aria-label="Runtime run history">
    <p>Initial display policy: latest run; explicit history/pin/compare. Selected historical runs stay selected until you choose latest.</p>
    <label>Runtime run <select value={selected} onChange={event => { setSelected(event.target.value); setInvocation(""); }}>
      <option value="">Latest run</option>{sorted.map(item => <option key={item.id} value={item.id}>{item.id} · {item.status}{item.pinned ? " · pinned" : ""}</option>)}
    </select></label>
    <p>Run {run.id} · {run.status} · {run.id === latest?.id ? "latest captured run" : "historical"}{stale ? " · source changed (stale)" : ""} · revision {run.sources[0]?.revision} {run.truncated ? "· truncated evidence" : ""}</p>
    {run.status === "running" && <p>Recorded running state; liveness after restart is unverified. This is not a success receipt.</p>}
    {run.detail && <p>{run.detail}</p>}
    {run.missing_artifacts.length > 0 && <p>Missing artifacts: {run.missing_artifacts.join(", ")}</p>}
    <button onClick={() => void act(() => onPin(run))}>{run.pinned ? "Unpin run" : "Pin run"}</button>
    <button onClick={() => void act(() => onDelete(run))}>Delete retained run</button>
    <button onClick={capture}>Capture run as document-owned evidence JSON</button>
    <p>Captures contain source/output: review before sharing. Browser records live in local storage; native records use .hick/runs. Authored Markdown is unchanged; existing output/ingested recording persistence is preserved.</p>
    <label>Invocation <select value={invocation} onChange={event => setInvocation(event.target.value)}>
      <option value="">All occurrences in this run ({run.events.length} events)</option>
      {invocations.map(id => <option key={id} value={id}>{id} · {run.events.filter(event => (event.invocation ?? "unknown") === id).length} events</option>)}
    </select></label>
    <details><summary>Selected occurrence output · {invocation || "all invocations"}</summary>
      <pre tabIndex={0} style={{ maxHeight: "18rem", overflow: "auto" }}>{runtimeText(run, invocation)}</pre>
    </details>
    <ProfileEvidence run={run} view={view} stale={stale} />
    <RuntimeLog key={run.id} run={invocation ? { ...run, events: run.events.filter(event => (event.invocation ?? "unknown") === invocation) } : run} />
    {reading && <ObservationReading key={`${run.id}:${invocation}`} reading={reading} historical={run.id !== latest?.id || run.status !== "running"} changed={stale} />}
    <label>Compare retained run <select value={comparison} onChange={event => setComparison(event.target.value)}>
      <option value="">No comparison</option>{sorted.filter(item => item.id !== run.id).map(item => <option key={item.id} value={item.id}>{item.id} · {item.status}</option>)}
    </select></label>
    {compared && <div aria-label="Separate run comparison"><p>Selected {run.id} ({run.status}) versus {compared.id} ({compared.status}); observations remain separate.</p>
      <pre tabIndex={0} style={{ maxHeight: "16rem", overflow: "auto" }}>{runtimeText(run)}</pre><pre tabIndex={0} style={{ maxHeight: "16rem", overflow: "auto" }}>{runtimeText(compared)}</pre></div>}
    {error && <p role="alert">{error}</p>}
  </section>;
}

</storycode:copy>
<a id="f-56919d2a9983-NativeRuntimeHistory"></a>
### `NativeRuntimeHistory`

<storycode:copy id="f-56919d2a9983-NativeRuntimeHistory">
export function NativeRuntimeHistory({ source, view, path }: { path?: string; source?: string; view?: EditorView | null }) {
  const instance = useId(); const disclosure = useRef<HTMLDetailsElement>(null);
  const [running, setRunning] = useState(false);
  const [program, setProgram] = useState("");
  const [profile, setProfile] = useState(false);
  const [shown, setShown] = useState(false);
  const [runs, setRuns] = useState<RuntimeRun[]>([]);
  const [error, setError] = useState("");
  async function load() {
    const result = await request<{ runs: { state: string; run?: RuntimeRun; error?: string }[] }>("GET", "/api/runtime/runs");
    setRuns(result.runs.flatMap(item => item.run ? [item.run] : []));
    setError(result.runs.filter(item => item.state !== "present").map(item => `${item.state}: ${item.error ?? "missing"}`).join("; "));
  }
  async function capture(entry: string, profiling: boolean) {
    if (!path) throw Error("Open a saved document before executing Python");
    setRunning(true); setError("");
    try {
      const result = await request<{ run: RuntimeRun }>("POST", "/api/runtime/python", { document: path, program: entry, seconds: 10, profile: profiling });
      await load();
      return result.run;
    } catch (reason) { setError(String(reason)); throw reason; }
    finally { setRunning(false); }
  }
  const parent = `native-runtime:${path ?? "workspace"}:${instance}`;
  useUiCommands([
    { key: `${parent}/disclose`, command: { label: "Runtime observations", subject: parent, parent: "runtime", parameters: {}, prerequisites: [], read: () => ({ revision: String(shown), enabled: !!disclosure.current, value: shown }), execute: async () => {
      if (!disclosure.current) return { status: "unavailable", reason: "Runtime view unavailable" };
      disclosure.current.open = !disclosure.current.open; setShown(disclosure.current.open);
      if (disclosure.current.open) await load();
      return { status: "completed", value: { open: disclosure.current.open } };
    } } },
    ...(shown && path ? [{ key: `${parent}/capture`, command: { label: "Run saved Python confined", subject: parent, parent, parameters: { program: { type: "string" as const, maxLength: 4096 }, profile: { type: "boolean" as const } }, prerequisites: ["Saved source; confined Python runtime; bounded 10-second capture; no network; cancellation after launch is unsupported"], read: () => ({ revision: `${path}:${running}`, enabled: !running, reason: running ? "A capture is running" : undefined, value: { program, profile, capture_limit_seconds: 10 } }), execute: async (args: Record<string, import("../lib/uiActions").UiValue>) => {
      setProgram(String(args.program)); setProfile(Boolean(args.profile));
      const run = await capture(String(args.program), Boolean(args.profile));
      return { status: "completed" as const, value: { run: run.id, status: run.status, sources: run.sources.map(item => ({ id: item.id, uri: item.uri, revision: item.revision })), events: run.events.length, interpretation: "Inspect actual correctness output and profile metrics; completion alone is not verification" } };
    } } }] : []),
  ]);
  useEffect(() => { if (!shown) return; void load().catch(e => setError(String(e))); const timer = setInterval(() => void load().catch(e => setError(String(e))), 2000); return () => clearInterval(timer); }, [shown]);
  return <details ref={disclosure} onToggle={event => setShown(event.currentTarget.open)}><summary>Runtime observations</summary>{shown && path && <div>
    <label>Generated Python entry point (literate documents)<input value={program} onChange={event => setProgram(event.target.value)} /></label>
    <label><input type="checkbox" checked={profile} onChange={event => setProfile(event.target.checked)} />Profile main-thread functions with cProfile (trace hooks disabled)</label>
    <button disabled={running} onClick={() => void capture(program, profile).catch(() => {})}>Run saved Python confined</button>
    <p>{running ? "Capturing for at most 10 seconds; hiding this panel does not cancel execution." : "Executes saved application code in the cell sandbox, without network. Unsaved edits are not executed. Descriptor/native writes and catching sites are unsupported."}</p>
  </div>}{shown && <RuntimeHistory runs={path ? runs.filter(run => run.sources.some(item => item.uri.replace(/^hick:\/\/\//, "") === path)) : runs} source={source} view={view}
    onPin={async run => { await request("PATCH", `/api/runtime/runs/${encodeURIComponent(run.id)}`, { pinned: !run.pinned }); await load(); }}
    onDelete={async run => { await request("DELETE", `/api/runtime/runs/${encodeURIComponent(run.id)}`); await load(); }} />}{error && <p role="alert">{error}</p>}</details>;
}
</storycode:copy>

### Assemble `apps/web/src/debug/RuntimeRuns.tsx`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/debug/RuntimeRuns.tsx"><storycode:paste select="#f-56919d2a9983-useBrowserRuns"/><storycode:paste select="#f-56919d2a9983-RuntimeHistory"/><storycode:paste select="#f-56919d2a9983-NativeRuntimeHistory"/></storycode:file>

</storycode:doc>
