import { useEffect, useState } from "react";
import type { EditorView } from "@codemirror/view";
import { request } from "../api/client";
import { DebuggableDocument } from "../embed/DebuggableDocument";
import { PublishedSnapshot, decodeSnapshot, type ViewerSnapshot } from "../embed/PublishedSnapshot";
import { NativeRuntimeHistory } from "../debug/RuntimeRuns";
import type { RuntimeRun } from "../debug/runtimeObservations";
import { runtimeText } from "../debug/runtimeHistory";
import { APP_TOUR_PHASES } from "../tour/scenario";
import type { ReviewEvent } from "../lib/reviews";

type ObservationDelivery = { request: string; branch: string; evidence: string[] };
type Practice = { delivered_observations?: ObservationDelivery[]; active: boolean; id: string; path: string; source: string; source_hash: string; profile_source: string; meeting: string; doc_id: string; seed: string };
type History = { events: ReviewEvent[]; investigations?: { thread: string; state: string }[] };
type RunResult = { id: string; status: string; source_hash: string | null; blocks: { kind: string; status?: string; transcript?: unknown[] }[] };
export function NativeWorkflowTour({ onOpen, focused }: { onOpen?: (path: string, line?: number) => void; focused?: { path?: string; source?: string; view?: EditorView | null } }) {
  const [open, setOpen] = useState(false);
  const [info, setInfo] = useState<Practice | null>(null);
  const [created, setCreated] = useState<{ path: string; opened: boolean; open_error?: string } | null>(null);
  const [phase, setPhase] = useState(0);
  const [error, setError] = useState("");
  const [busy, setBusy] = useState(false);
  const [meeting, setMeeting] = useState<{ path: string; source: string; source_sha256: string; utterances?: { id: string; line: number }[] } | null>(null);
  const [history, setHistory] = useState<History>({ events: [] });
  const [ancestry, setAncestry] = useState<string[]>([]);
  const [pauseIds, setPauseIds] = useState<string[]>([]);
  const [pauseEvidence, setPauseEvidence] = useState("");
  const [counterexample, setCounterexample] = useState("");
  const [updated, setUpdated] = useState(false);
  const [runs, setRuns] = useState<RuntimeRun[]>([]);
  const [verified, setVerified] = useState<RunResult | null>(null);
  const [publication, setPublication] = useState<ViewerSnapshot | null>(null);
  const key = info?.active ? `hickory.aiEngineerTour.native:${info.id}` : null;
  useEffect(() => { if (key) localStorage.setItem(key, String(phase)); }, [key, phase]);
  async function refresh(initial = false) {
    const practice = await request<Practice>("GET", "/api/tour/practice");
    setInfo(practice);
    if (!practice.active) return;
    if (initial) { const saved = Number(localStorage.getItem(`hickory.aiEngineerTour.native:${practice.id}`)); setPhase(Number.isInteger(saved) && saved >= 0 && saved < APP_TOUR_PHASES.length ? saved : 0); }
    const review = await request<History>("GET", "/api/reviews"); setHistory(review);
    const captured = await request<{ runs: { state: string; run?: RuntimeRun }[] }>("GET", "/api/runtime/runs");
    setRuns(captured.runs.flatMap(r => r.state === "present" && r.run ? [r.run] : []));
    const turns = await request<{ turns: { id: string; parent_id?: string | null }[] }>("GET", "/api/docs/workspace/agent/turns");
    const branch = turns.turns.at(-1)?.id;
    const ancestors: string[] = []; let cursor = branch;
    while (cursor && !ancestors.includes(cursor)) { ancestors.push(cursor); cursor = turns.turns.find(t => t.id === cursor)?.parent_id ?? undefined; }
    setAncestry(ancestors);
    if (branch) {
      const context = await request<{ updates: { state: string }[] }>("POST", "/api/docs/workspace/agent/python/context", { branch });
      setUpdated(context.updates.some(u => u.state === "provider_confirmed"));
    }
  }
  async function act(action: () => Promise<void>) { setBusy(true); setError(""); try { await action(); } catch (e) { setError(String(e)); } finally { setBusy(false); } }
  const thread = [...history.events].reverse().find(e => e.target && (e.target.path === "deadline-spec.md" || e.target.path.endsWith("/deadline-spec.md")) && e.target.quote.includes("The deadline bounds"))?.thread;
  const resolved = [...history.events].reverse().find(e => e.thread === thread && (e.action === "resolve" || e.action === "reopen"))?.action === "resolve";
  const investigated = history.investigations?.some(i => i.thread === thread && i.state === "responded");
  const profiles = runs.filter(r => r.status === "completed" && r.sources.some(s => s.uri.replace(/^hick:\/\/\//, "") === "hotspot.md") && r.events.some(e => e.observation.kind === "profile") && runtimeText(r).includes("correctness: 15998000000"));
  const profileRevisions = profiles.flatMap(r => r.sources.filter(s => s.uri.replace(/^hick:\/\/\//, "") === "hotspot.md").map(s => s.revision));
  const profiledTwice = profiles.length >= 2 && new Set(profileRevisions).size >= 2;
  const supplied = new Set((info?.delivered_observations ?? []).filter(d => ancestry.includes(d.branch)).flatMap(d => d.evidence));
  const debugDelivered = pauseIds.some(id => supplied.has(id));
  const profileDelivered = profiles.some(r => r.events.some(e => e.observation.kind === "profile" && supplied.has(e.id)));
  const nativePassed = verified?.status === "ok" && verified.source_hash?.replace(/^sha256:/, "") === info?.source_hash && verified.blocks.some(b => b.kind === "exec") && verified.blocks.filter(b => b.kind === "exec").every(b => b.status === "ok");
  const published = publication?.verification?.status === "passed" && publication.source_hash === info?.source_hash;
  const gates = [!!meeting, !!thread, !!counterexample && !!pauseEvidence, updated && debugDelivered, !!investigated, !!nativePassed, profiledTwice && profileDelivered, !!resolved && !!nativePassed, !!published];
  async function runNative() {
    if (!info?.active) throw Error("Open the isolated practice repository first");
    const started = await request<{ run_id: string }>("POST", `/api/docs/${info.doc_id}/check`, {});
    for (let n = 0; n < 120; n++) {
      const result = await request<RunResult>("GET", `/api/runs/${started.run_id}`);
      if (result.status !== "running") { setVerified(result); await refresh(); return; }
      await new Promise(resolve => setTimeout(resolve, 500));
    }
    throw Error("Verification is still running; inspect the normal run panel. No success is assumed.");
  }
  return <div className="workflow-tour-launcher">
    <button className="btn" onClick={() => { setOpen(!open); if (!open) void act(() => refresh(true)); }}>{open ? "Close AI engineer tour" : "AI engineer tour"}</button>
    <section hidden={!open} className="workflow-tour workflow-tour--app" aria-label="AI engineer tour">
      <h2>Meeting → reviewed and verified change</h2>
      <p>Native practice uses a separate Git repository. Seeded meeting/proposal text is labelled; imports, review threads, model calls, context deliveries, runtime captures and checks use the product's real controls. Model calls require your configured provider and an explicit Send action.</p>
      {error && <p role="alert">{error}</p>}
      {!info?.active ? <>
        <button disabled={busy} onClick={() => void act(async () => setCreated(await request("POST", "/api/tour/practice", {})))}>Create and open an isolated practice repository</button>
        {created && <p>Created <code>{created.path}</code>. {created.opened ? "Opened in a separate native window." : `Open this folder in another Hickory window. ${created.open_error ?? ""}`}</p>}
      </> : <>
        <p>{info.seed} · repository <code>{info.path}</code></p>
        <p>Phase {phase + 1} of {APP_TOUR_PHASES.length}. Close the tour to use the main editor/review/agent controls; reopening resumes this phase. Refresh observations after acting. Reloaded execution steps require renewed evidence.</p>
        <h3>{APP_TOUR_PHASES[phase].title}</h3><p>{APP_TOUR_PHASES[phase].text}</p>
        <button disabled={busy} onClick={() => void act(() => refresh())}>Refresh observed product evidence</button>
        <button onClick={() => onOpen?.("deadline-spec.md", 13)}>Open the actual requirement and claim</button>
        {phase === 0 && <><pre>{info.meeting}</pre><button disabled={busy} onClick={() => void act(async () => setMeeting(await request("POST", "/api/meetings/import", { name: "deadline-meeting.vtt", text: info.meeting })))}>Import the seeded meeting through the real importer</button>{meeting && <p>Retained source <button onClick={() => onOpen?.(meeting.source, 1)}>{meeting.source}</button> · SHA-256 {meeting.source_sha256}. Use its utterance citation when linking R1. {meeting.utterances?.map(u => <button key={u.id} onClick={() => onOpen?.(meeting.source, u.line)}>{u.id} · source line {u.line}</button>)}</p>}</>}
        {phase === 1 && <p>Enable author colors in Lineage settings. The initial Git author is the fixture generator, not an authenticated human. Select the actual claim and use Comment on selection; retain the missed “including retries” qualification in the objection. Thread: {thread ?? "not yet observed"}.</p>}
        <div hidden={phase !== 2}><p>Bind a breakpoint and inspect an actual paused marker before continuing to the counterexample failure. Expanded/pinned observations use the same stable handles supplied to agent context. Reverse execution and restart-frame are unsupported here; retained-history browsing does not reverse execution.</p><DebuggableDocument path="deadline-spec.md" source={info.source} revision={info.source_hash} readOnly onObservation={reading => {
          if (reading.annotations.some(a => a.payload.kind === "paused") && reading.annotations.some(a => a.payload.kind === "breakpoint_bound") && new TextDecoder().decode(new Uint8Array(reading.source.bytes)) === info.source) { setPauseEvidence(reading.scope.stop); setPauseIds(reading.annotations.filter(a => a.payload.kind === "paused" || a.payload.kind === "breakpoint_bound").map(a => a.evidence)); }
        }} onRuntime={run => {
          if (run.status !== "running") {
            if (run.status === "failed" && runtimeText(run).includes("R1 failed: expected 100, got 160") && new TextDecoder().decode(new Uint8Array(run.sources[0]?.bytes ?? [])) === info.source) setCounterexample(run.id);
            void request("POST", "/api/runtime/runs", run).catch(e => setError(String(e)));
          }
        }} /></div>
        {phase === 3 && <p><button onClick={() => onOpen?.("AGENTS.md", 1)}>Open actual instruction source</button> In the workspace agent, ask it to read the requirement. Open AGENTS.md and add the missing “including retries” instruction, inspect Changed agent context, and queue that exact patch. Send a subsequent message to observe delivery. The gate reads a confirmed update receipt; it does not infer understanding. Confirmed update: {updated ? "observed" : "unavailable"}; exact paused/breakpoint evidence supplied on this branch: {debugDelivered ? "observed" : "unavailable"}. Pin source observations before sending.</p>}
        {phase === 4 && <p>Use the saved review thread's investigation selector to send to the workspace conversation. Inspect its actual response/session, proposed diff and edit approval. This uses your provider; the tour never silently calls a model or substitutes a recorded response. Response: {investigated ? "observed" : "unavailable"}.</p>}
        {phase === 5 && <><button disabled={busy} onClick={() => void act(runNative)}>Verify the saved corrected specification natively</button>{verified && <p>Actual run {verified.id}: {verified.status} · source {verified.source_hash ?? "unavailable"}. {nativePassed ? "Execution-cell witnesses passed for current source." : "Current-source pass not established."}</p>}<p>Inspect the agent pane's actual focused diff and approve it before running this check. A passing run does not resolve the human review thread.</p></>}
        {phase === 6 && <><button onClick={() => onOpen?.("hotspot.md", 11)}>Open the real profiling source</button><NativeRuntimeHistory path="hotspot.md" source={info.profile_source} view={focused?.path === "hotspot.md" ? focused.view : undefined} /><p>Completed profile source revisions with the correctness output: {new Set(profileRevisions).size}. Expand subtotal, pin selected evidence in next context, replace its loop with the arithmetic formula, then rerun correctness and profiling. Timing includes profiler overhead; no fixed speedup is promised. Exact profile evidence supplied on this conversation branch: {profileDelivered ? "observed" : "unavailable"}.</p></>}
        {phase === 7 && <p>Inspect the corrected source, exact revision check, log/emitter evidence and profiling comparison. Resolve the actual thread yourself in Review threads. Latest human action: {resolved ? "resolve" : "open/unavailable"}. The tour never resolves it automatically.</p>}
        {phase === 8 && <><a href="https://hickorydocs.com/snapshot.html?index=publications/index.json" target="_blank" rel="noreferrer">Open published verification</a><label>Load a reviewed published result JSON<input type="file" accept=".json" onChange={e => { const file = e.target.files?.[0]; if (file) void act(async () => { const value = JSON.parse(await file.text()); setPublication(await decodeSnapshot(value)); }); }} /></label>{publication && <><p>{published ? "Published pass matches the actual saved source hash." : "Publication is stale, failed, missing verification, or for different source bytes."}</p><PublishedSnapshot snapshot={publication} /></>}</>}
        <p role="status">Phase evidence: {gates[phase] ? "observed within the stated scope" : "not established"}. Human judgment and semantic correctness remain separate.</p>
        <div className="workflow-tour-controls"><button disabled={phase === 0} onClick={() => setPhase(p => p - 1)}>Back</button>{phase < APP_TOUR_PHASES.length - 1 && <button disabled={!gates[phase]} onClick={() => setPhase(p => p + 1)}>Continue</button>}<button onClick={() => { setPhase(0); setPauseEvidence(""); setCounterexample(""); setVerified(null); setMeeting(null); setPublication(null); }}>Restart walkthrough in this repository</button></div>
        {phase === APP_TOUR_PHASES.length - 1 && <p>Local evidence and the loaded publication are reviewable. Launch readiness is not established until the full native path and the deployed homepage build revision are independently witnessed. Preserve unavailable steps instead of calling them passed.</p>}
      </>}
      <button onClick={() => setOpen(false)}>Skip / close</button>
    </section>
  </div>;
}
