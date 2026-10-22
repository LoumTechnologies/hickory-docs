import { PrivatePredictionCoach, type PrivatePredictionState } from "./PrivatePredictionCoach";
import { useEffect, useId, useState } from "react";
import type { EditorView } from "@codemirror/view";
import { request } from "../api/client";
import { PublishedSnapshot, decodeSnapshot, type ViewerSnapshot } from "../embed/PublishedSnapshot";
import { useUiCommands, useUiRevision } from "../lib/uiActionBridge";
import type { UiCommand, UiValue } from "../lib/uiActions";
import { capturedDebugReadings, onDebugReading } from "../debug/readingCapture";
import type { ReviewEvent } from "../lib/reviews";
import { AnchoredCoach } from "./AnchoredCoach";
import { COACH_STEPS, INVESTIGATION_TASK, CORRECTION_TASK, reconciledCoachStep } from "../tour/scenario";

type Execution = { id: string; status: string; started_at?: string; source_hash: string | null; error?: string | null; blocks: { kind: string; status?: string; transcript?: unknown[] }[] };
type Practice = { active: boolean; id: string; path: string; source: string; source_hash: string; seed_source_hash: string; seed: string; agent_requests?: { request: string; branch: string; state: string }[]; agent_edits?: { source_hash: string; branch: string }[]; executions?: Execution[]; debug_evidence?: Pause[] };
type Turn = { id: string; parent_id?: string | null; status: string };
type Pause = { run: string; stop: string; evidence: string[] };
const key = (id: string) => `hickory.aiEngineerCoach.v2:${id}`;
const counterexample = (run: Execution, hash: string) => run.status === "failed" && run.source_hash?.replace(/^sha256:/, "") === hash && `${run.error ?? ""}${JSON.stringify(run.blocks.map(block => block.transcript ?? []))}`.includes("R1 failed: expected 100, got 160");
type NativeWorkflowTourProps = { onOpen?: (path: string, line?: number) => void; focused?: { path?: string; source?: string; view?: EditorView | null }; privateSession?: PrivatePredictionState };
export function NativeWorkflowTour(props: NativeWorkflowTourProps) {
  return props.privateSession ? <PrivatePredictionCoach key={props.privateSession.document} initial={props.privateSession} /> : <OrdinaryWorkflowTour {...props} />;
}
function OrdinaryWorkflowTour({ onOpen }: NativeWorkflowTourProps) {
  const instance = useId(); const parent = `coach:${instance}`;
  const [open, setOpen] = useState(false);
  const [info, setInfo] = useState<Practice | null>(null);
  const [created, setCreated] = useState<{ path: string; opened: boolean; open_error?: string } | null>(null);
  const [phase, setPhase] = useState(0);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState("");
  const [events, setEvents] = useState<ReviewEvent[]>([]);
  const [turns, setTurns] = useState<Turn[]>([]);
  const [pause, setPause] = useState<Pause | null>(null);
  const [publication, setPublication] = useState<ViewerSnapshot | null>(null);
  const [skipped, setSkipped] = useState<number[]>([]);
  const [task, setTask] = useState("");
  const thread = [...events].reverse().find(event => event.actor === "user-declared" && event.target && event.target.path.endsWith("deadline-spec.md") && event.target.quote.includes("The deadline bounds"))?.thread;
  const resolved = [...events].reverse().find(event => event.actor === "user-declared" && event.thread === thread && ["resolve", "reopen"].includes(event.action))?.action === "resolve";
  const branch = turns.at(-1)?.id;
  const ancestors = new Set<string>(); let cursor = branch;
  while (cursor && !ancestors.has(cursor)) { ancestors.add(cursor); cursor = turns.find(turn => turn.id === cursor)?.parent_id ?? undefined; }
  const delegated = !!info?.agent_requests?.some(receipt => receipt.state === "provider_confirmed" && ancestors.has(receipt.branch) && turns.some(turn => turn.id === receipt.branch && turn.status === "ok"));
  const failed = info?.executions?.find(run => counterexample(run, info.seed_source_hash));
  const corrected = !!info?.agent_edits?.some(edit => edit.source_hash === info.source_hash && ancestors.has(edit.branch));
  const verified = info?.executions?.filter(run => run.source_hash?.replace(/^sha256:/, "") === info.source_hash).sort((a,b) => (a.started_at ?? "").localeCompare(b.started_at ?? "")).at(-1);
  const passed = verified?.status === "ok" && verified.blocks.some(block => block.kind === "exec") && verified.blocks.filter(block => block.kind === "exec").every(block => block.status === "ok");
  const published = publication?.verification?.status === "passed" && publication.source_hash === info?.source_hash;
  const gates = [delegated, !!failed && !!pause, !!thread, !!passed && corrected && info?.source_hash !== info?.seed_source_hash, !!resolved && !!passed, !!published];
  useEffect(() => { if (info?.active) localStorage.setItem(key(info.id), String(phase)); }, [info?.id, info?.active, phase]);
  useEffect(() => {
    function observe() {
      if (!info?.active) return;
      const reading = capturedDebugReadings().find(reading => reading.source.uri.replace(/^hick:\/\/\//, "") === "deadline-spec.md" && reading.source.revision.replace(/^sha256:/, "") === info.seed_source_hash && reading.annotations.some(item => item.payload.kind === "paused") && reading.annotations.some(item => item.payload.kind === "breakpoint_bound"));
      if (reading) setPause({ run: reading.scope.run, stop: reading.scope.stop, evidence: reading.annotations.filter(item => ["paused", "breakpoint_bound"].includes(item.payload.kind)).map(item => item.evidence) });
    }
    observe(); return onDebugReading(observe);
  }, [info?.id, info?.seed_source_hash, info?.active]);
  async function refresh(resume = false) {
    const practice = await request<Practice>("GET", "/api/tour/practice");
    if (!practice.active) { setInfo(practice); return; }
    const saved = Number(localStorage.getItem(key(practice.id)));
    const [review, conversation] = await Promise.all([
      request<{ events: ReviewEvent[] }>("GET", "/api/reviews"),
      request<{ turns: Turn[] }>("GET", "/api/docs/workspace/agent/turns"),
    ]);
    setInfo(practice); setEvents(review.events); setTurns(conversation.turns);
    if (!resume) setPause(previous => practice.debug_evidence?.find(item => item.run && item.stop && item.evidence?.length) ?? previous);
    if (resume) {
      // Reload never turns a saved step number into evidence. Reconcile the
      // provider gate now; later steps remain gated by current receipts.
      const lineage = new Set<string>(); let current = conversation.turns.at(-1)?.id;
      while (current && !lineage.has(current)) { lineage.add(current); current = conversation.turns.find(turn => turn.id === current)?.parent_id ?? undefined; }
      const confirmed = !!practice.agent_requests?.some(receipt => receipt.state === "provider_confirmed" && lineage.has(receipt.branch) && conversation.turns.some(turn => turn.id === receipt.branch && turn.status === "ok"));
      const failure = practice.executions?.some(run => counterexample(run, practice.seed_source_hash));
      const observedPause = practice.debug_evidence?.find(item => item.run && item.stop && item.evidence?.length);
      const observedThread = [...review.events].reverse().find(event => event.actor === "user-declared" && event.target?.path.endsWith("deadline-spec.md") && event.target.quote.includes("The deadline bounds"))?.thread;
      const observedResolution = [...review.events].reverse().find(event => event.actor === "user-declared" && event.thread === observedThread && ["resolve", "reopen"].includes(event.action))?.action === "resolve";
      const last = practice.executions?.filter(run => run.source_hash?.replace(/^sha256:/, "") === practice.source_hash).sort((a,b) => (a.started_at ?? "").localeCompare(b.started_at ?? "")).at(-1);
      const pass = last?.status === "ok" && last.blocks.some(block => block.kind === "exec") && last.blocks.filter(block => block.kind === "exec").every(block => block.status === "ok");
      const correction = !!practice.agent_edits?.some(edit => edit.source_hash === practice.source_hash && lineage.has(edit.branch));
      setPhase(reconciledCoachStep(saved, [confirmed, !!failure && !!observedPause, !!observedThread, !!pass && correction && practice.source_hash !== practice.seed_source_hash, !!pass && observedResolution, false]));
      setPause(observedPause ?? null); setPublication(null); setSkipped([]);
    }
  }
  async function perform(action: () => Promise<UiValue>): Promise<import("../lib/uiActions").UiOutcome> {
    setBusy(true); setError("");
    try { return { status: "completed", value: await action() }; }
    catch (reason) { setError(String(reason)); throw reason; }
    finally { setBusy(false); }
  }
  async function loadPublication(text: string) {
    const snapshot = await decodeSnapshot(JSON.parse(text)); setPublication(snapshot);
    return { source_hash: snapshot.source_hash, reported_status: snapshot.verification?.status ?? "not_run", revision: snapshot.verification?.revision ?? null, interpretation: "Loaded artifact declaration; inspect its reviewed publication origin and matching source" };
  }
  const revision = useUiRevision(() => [open, info, phase, busy, gates, skipped, error, task, publication, pause, events, turns, created]);
  const commands: { key: string; command: UiCommand }[] = [];
  function add(id: string, label: string, enabled: boolean, execute: UiCommand["execute"], parameters: UiCommand["parameters"] = {}) {
    if (id !== "toggle" && !open) return;
    if (!["toggle", "read", "dismiss", "create", "create-private"].includes(id) && !info?.active) return;
    if (["create", "create-private"].includes(id) && info?.active) return;
    if (id === "task" && ![0, 3].includes(phase)) return;
    if (id === "source" && phase !== 1) return;
    if (id === "publication" && phase !== 5) return;
    if (["next", "skip"].includes(id) && phase === COACH_STEPS.length - 1) return;
    commands.push({ key: `${parent}/${id}`, command: { label, subject: parent, parent, parameters, prerequisites: [], read: () => ({ revision: revision(), enabled: enabled && (!busy || id === "dismiss"), reason: busy && id !== "dismiss" ? "Coach is refreshing" : !enabled ? "Required product evidence is not established" : undefined, value: (id === "toggle" ? { open } : { open, step: phase + 1, total: COACH_STEPS.length, evidence: !!gates[phase] }) as UiValue }), execute } });
  }
  add("toggle", open ? "Close AI engineer tour" : "AI engineer tour", true, async () => perform(async () => { if (!open) await refresh(true); setOpen(!open); return { open: !open }; }));
  add("refresh", "Refresh observed product evidence", !!info?.active, async () => perform(async () => { await refresh(); return { refreshed: true, verification: "See actual receipts; refresh is not execution" }; }));
  add("create", "Create and open an isolated practice repository", !info?.active, async () => perform(async () => { const result = await request<{ path: string; opened: boolean; open_error?: string }>("POST", "/api/tour/practice", {}); setCreated(result); return JSON.parse(JSON.stringify(result)); }));
  add("create-private", "Create a private prediction practice", !info?.active, async (_args, actor) => {
    if (actor.kind === "agent") return { status: "requires_input", request: "create-private-practice", reason: "A human creates the isolated private practice and authorizes its host execution environment; the predictor cannot reset its own history." };
    setBusy(true); setError("");
    try {
      const result = await request<{ path?: string; opened?: boolean; open_error?: string; created: boolean; requires_input?: { request: string; reason: string } }>("POST", "/api/tour/private-practice", {});
      if (result.requires_input) { setError(result.requires_input.reason); return { status: "requires_input", ...result.requires_input }; }
      if (!result.created || !result.path) return { status: "unavailable", reason: "Private practice creation did not return a host resource" };
      setCreated({ path: result.path, opened: result.opened === true, open_error: result.open_error });
      return { status: "completed", value: JSON.parse(JSON.stringify(result)) };
    } catch (reason) { setError(String(reason)); return { status: "unavailable", reason: String(reason) }; }
    finally { setBusy(false); }
  });
  add("back", "Back", !!info?.active && phase > 0, async () => { setPhase(phase - 1); return { status: "completed", value: { step: phase } }; });
  add("next", "Continue", !!info?.active && !!gates[phase] && phase < COACH_STEPS.length - 1, async () => { setPhase(phase + 1); return { status: "completed", value: { step: phase + 2, completed_operation: false } }; });
  add("skip", "Skip this step", !!info?.active && phase < COACH_STEPS.length - 1, async () => { setSkipped([...new Set([...skipped, phase])]); setPhase(phase + 1); return { status: "completed", value: { skipped: phase + 1, evidence_established: false } }; });
  add("restart", "Restart guidance", !!info?.active, async () => { setPhase(0); setSkipped([]); setPause(null); setPublication(null); return { status: "completed", value: { step: 1, retained_history: true, repository_reset: false } }; });
  add("task", phase < 3 ? "Copy investigation task" : "Copy correction and verification task", !!info?.active && [0, 3].includes(phase), async (_args, actor) => {
    const text = phase < 3 ? INVESTIGATION_TASK : CORRECTION_TASK; setTask(text);
    if (actor.kind === "human" && navigator.clipboard) await navigator.clipboard.writeText(text);
    return { status: "completed", value: { text, provider_request_sent: false } };
  });
  add("publication", "Compare reviewed published JSON", !!info?.active && phase === 5, async args => perform(async () => loadPublication(String(args.json))), { json: { type: "string", maxLength: 262144 } });
  add("dismiss", "Dismiss practice guidance", true, async () => { setOpen(false); return { status: "completed", value: { open: false, retained_history: true } }; });
  add("source", "Open actual source and debugger", !!info?.active && phase === 1 && !!onOpen, async () => { onOpen?.("deadline-spec.md", 13); return { status: "completed", value: { navigation_requested: "deadline-spec.md", line: 13, debugger_started: false } }; });
  add("read", "Read current practice guidance bytes", true, async args => {
    const bytes = new TextEncoder().encode(JSON.stringify({ customer: "Two 80ms attempts must share a 100ms total deadline, including retries", deterministic_model: true, step: phase + 1, guidance: COACH_STEPS[phase], gates, skipped: skipped.map(step => step + 1), error, task, practice: info ? { active: info.active, path: info.path, source_hash: info.source_hash, seed_source_hash: info.seed_source_hash, seed: info.seed } : null, created, evidence: { delegated, failed_run: failed?.id ?? null, pause, thread: thread ?? null, corrected, verified_run: verified?.id ?? null, passed: !!passed, resolved, publication_matches: !!published }, launch_readiness: "Native/provider recording and released/deployed witnesses are required separately" }));
    const offset = args.offset as number, end = Math.min(offset + (args.limit as number), bytes.length);
    if (offset > bytes.length) return { status: "unavailable", reason: "Offset outside current guidance" };
    return { status: "completed", value: { bytes: Array.from(bytes.slice(offset, end)), encoding: "JSON UTF-8", offset, total_bytes: bytes.length, next_offset: end < bytes.length ? end : null } };
  }, { offset: { type: "number", minimum: 0, maximum: 1000000000, integer: true }, limit: { type: "number", minimum: 1, maximum: 8192, integer: true } });
  const invoke = useUiCommands(commands);
  const click = (id: string, args: Record<string, UiValue> = {}) => { void invoke(`${parent}/${id}`, args).catch(reason => setError(String(reason))); };
  const target = phase === 0 || phase === 3 ? '[data-coach-target="agent"]' : phase === 2 || phase === 4 ? '[data-coach-target="review"]' : '.cm-card-rail';
  return <div className="workflow-tour-launcher">
    <button className="btn" disabled={busy} onClick={() => click("toggle")}>{open ? "Close AI engineer tour" : "AI engineer tour"}</button>
    {open && <AnchoredCoach target={target} title="AI engineer tour" onDismiss={() => click("dismiss")}>
      <section className="workflow-tour workflow-tour--app" aria-label="Agent-first practice">
        <p><strong>Customer brief:</strong> A client retries outbound requests. Two 80ms attempts must share a 100ms total deadline, while short attempts and an empty sequence retain their behavior. The missed qualification is “including retries.” This deterministic model does not test network or OS timing.</p>
        {error && <p role="alert">{error}</p>}
        {!info?.active ? <>
          <p>Start in a separate Git repository. Meeting text and the original mistake are labelled fixtures; subsequent provider calls, debugging, edits and checks must be real.</p>
          <button disabled={busy} onClick={() => click("create")}>Create and open an isolated practice repository</button>
          <button disabled={busy} onClick={() => click("create-private")}>Create a private prediction practice</button>
          <p>Private practice uses host-sealed predictions and withheld debugger/output comparisons. It pauses other engine surfaces until End and requires an explicitly selected local Python/DAP environment. Creation starts no model request or target execution.</p>
          {created && <p>Created <code>{created.path}</code>. {created.opened ? "Opened in a separate native window." : `Open this folder in another Hickory window. ${created.open_error ?? ""}`}</p>}
        </> : <>
          <p>{info.seed} · <code>{info.path}</code></p>
          <p>Step {phase + 1} of {COACH_STEPS.length}. Dismiss to use the workspace; reopening reconciles actual receipts. Missing witnesses stay unestablished.</p>
          <h3>{COACH_STEPS[phase].title}</h3><p>{COACH_STEPS[phase].text}</p>
          {phase === 0 && <><button onClick={() => click("task")}>Copy investigation task</button><p>Paste into the actual workspace agent, inspect the provider, then Send. No model request is sent by the coach. Confirmed request and completed response: {delegated ? "observed" : "not established"}.</p></>}
          {phase === 1 && <><button onClick={() => click("source")}>Open actual source and debugger</button><p>Source-bound failure: {failed?.id ?? "not established"}. Bound paused evidence: {pause ? `${pause.run} / ${pause.stop}` : "not established"}. Ask the agent to use the registered debugger controls and inspect the actual paused marker. Historical browsing is not reverse execution.</p></>}
          {phase === 2 && <p>Decide whether the deadline includes retries. Select the actual “The deadline bounds…” claim and record the missed qualification with the existing Comment on selection control. Saved review thread: {thread ?? "not established"}. The coach does not create or resolve a human review for you.</p>}
          {phase === 3 && <><button onClick={() => click("task")}>Copy correction and verification task</button><p>Review the agent's actual preview before accepting it. Delegate Verify saved document through its shared action handle. Receipt: {verified?.id ?? "not established"} · {verified?.status ?? "unavailable"}; current-root pass: {passed ? "observed" : "not established"}. A model completion is not a check.</p></>}
          {phase === 4 && <p>Inspect the focused diff, actual source/run identity and independent checks, then resolve the real review thread yourself. Current resolution: {resolved ? "resolved" : "open/unavailable"}. Profiling is optional when it helps this customer problem, using the normal runtime controls.</p>}
          {phase === 5 && <><a href="https://hickorydocs.com/snapshot.html?index=publications/index.json" target="_blank" rel="noreferrer">Inspect reviewed published verification</a><label>Load reviewed publication JSON<input type="file" accept=".json" onChange={event => { const file = event.target.files?.[0]; if (file) void file.text().then(json => click("publication", { json })).catch(reason => setError(String(reason))); }} /></label>{publication && <><p>{published ? "Loaded published pass matches saved source bytes; inspect its origin and tested revision." : "Publication is stale, failed, unavailable or for different source bytes."}</p><PublishedSnapshot snapshot={publication} /></>}</>}
          {task && <details open><summary>Delegation text — review before sending</summary><pre>{task}</pre></details>}
          <p role="status">Step evidence: {gates[phase] ? "observed within the stated scope" : "not established"}.{skipped.length > 0 && ` Skipped steps: ${skipped.map(step => step + 1).join(", ")}; these are not passed.`}</p>
          <button disabled={busy} onClick={() => click("refresh")}>Refresh observed product evidence</button>
          <div className="workflow-tour-controls"><button disabled={phase === 0 || busy} onClick={() => click("back")}>Back</button>{phase < COACH_STEPS.length - 1 && <><button disabled={!gates[phase] || busy} onClick={() => click("next")}>Continue</button><button disabled={busy} onClick={() => click("skip")}>Skip this step</button></>}<button disabled={busy} onClick={() => click("restart")}>Restart guidance</button></div>
          <p>Retained history is not reset. Reload requires missing paused/publication evidence to be re-established. Recording, native/provider execution and deployed revision still need their actual witnesses; the coach never declares launch readiness.</p>
        </>}
        <button onClick={() => click("dismiss")}>Skip / close</button>
      </section>
    </AnchoredCoach>}
  </div>;
}
