import { useCallback, useEffect, useRef, useState } from "react";
import { request } from "../api/client";
import type { AcpState } from "../api/acp";
import { captureUiWindow, useUiCommands, useUiRevision } from "../lib/uiActionBridge";
import { useUiReading } from "../lib/uiReading";
import { PredictionEvidence, validatePredictionHistory, type PredictionHistory } from "../embed/PredictionHistory";
import { AnchoredCoach } from "./AnchoredCoach";
import { CodeEditor } from "./ComparisonEditor";
import { AgentChanges } from "./AgentChanges";
import "./privatePredictionCoach.css";
export interface PrivatePredictionState {
  document: string; source: string; source_hash: string; current: boolean;
  question: { requirement_reference: string }; target: unknown; environment_scope: string;
  retained_plan?: { initial: { requirement_reference: string }; next: { requirement_reference: string } } | null;
  retained_debugger?: { phase: "not_started" | "sealed" | "paused_needs_prediction" | "complete" | "expired"; actual_values: "withheld"; last: unknown };
  actual_values: "withheld"; application_integration: "incomplete"; runtime_revalidation?: boolean;
}
const object = (value: unknown): value is Record<string, unknown> => !!value && typeof value === "object" && !Array.isArray(value);
const only = (value: object, keys: string[]) => Object.keys(value).every(key => keys.includes(key));
function safeComparison(value: unknown): boolean {
  if (value == null) return true;
  return object(value) && only(value, ["summary", "dispatched"]) && typeof value.dispatched === "boolean" && object(value.summary)
    && only(value.summary, ["attempt", "scenario", "seal", "classification", "prediction_matched", "original_requirement_satisfied", "observed", "promoted", "current_certification"])
    && value.summary.current_certification === false;
}
export function validatePrivatePredictionState(value: unknown): asserts value is PrivatePredictionState {
  const state = value as Partial<PrivatePredictionState> | null;
  if (!state || !object(state) || !only(state, ["document", "source", "source_hash", "sealed_source_hash", "current", "actor", "question", "target", "identities", "actual_values", "runtime_revalidation", "environment_scope", "supported", "application_integration", "retained_debugger", "retained_plan"]) || typeof state.document !== "string" || !state.document
    || typeof state.source !== "string" || typeof state.source_hash !== "string" || !/^[a-f0-9]{64}$/.test(state.source_hash)
    || (state.runtime_revalidation !== undefined && typeof state.runtime_revalidation !== "boolean")
    || (state.retained_debugger !== undefined && (!state.retained_debugger || !object(state.retained_debugger) || !only(state.retained_debugger, ["phase", "actual_values", "last"]) || !safeComparison(state.retained_debugger.last) || state.retained_debugger.actual_values !== "withheld" || !["not_started", "sealed", "paused_needs_prediction", "complete", "expired"].includes(state.retained_debugger.phase)))
    || (state.retained_plan != null && (typeof state.retained_plan.initial?.requirement_reference !== "string" || typeof state.retained_plan.next?.requirement_reference !== "string"))
    || typeof state.current !== "boolean" || state.actual_values !== "withheld" || state.application_integration !== "incomplete"
    || !state.question || typeof state.question.requirement_reference !== "string" || typeof state.environment_scope !== "string") {
    throw Error("Invalid private prediction startup response");
  }
}
type Agent = AcpState & { run?: { id: string; status: string; answer?: string } | null };
const base = "/api/prediction";
/** Host-bound branch of NativeWorkflowTour. No ordinary workspace/runtime reader mounts. */
export function PrivatePredictionCoach({ initial }: { initial: PrivatePredictionState }) {
  const [state, setState] = useState(initial), [source, setSource] = useState("");
  const [agent, setAgent] = useState<Agent | null>(null), [evidence, setEvidence] = useState<PredictionHistory | null>(null);
  const [prompt, setPrompt] = useState((initial.retained_plan ? "Use the host retained-stop plan: seal the initial debugger prediction before launch, then seal the future-stop prediction while paused before Continue. After reviewed repair repeat the same expectations for both stops and the combined output check. " : "") + "Investigate the host requirement using captured source. Seal your prediction and rationale before Verify. If it fails, explain and propose a correction for human review, then reseal the unchanged expectation and verify. Keep held-out target answers withheld.");
  const [provider, setProvider] = useState(""), [model, setModel] = useState("");
  const [promotion, setPromotion] = useState<{ source_hash: string; checked_at: string; scope: string; summary: unknown } | null>(null);
  const [prediction, setPrediction] = useState('{"kind":"equals","value":null}'), [rationale, setRationale] = useState("");
  const [pendingForms, setPendingForms] = useState(0);
  const [ended, setEnded] = useState(false);
  const [busy, setBusy] = useState(false), [error, setError] = useState(""), [syncError, setSyncError] = useState("");
  const key = `hickory.privateCoach:${initial.document}`;
  const [open, setOpen] = useState(() => localStorage.getItem(key) !== "dismissed");
  const owner = useRef(true), sequence = useRef(0);
  const refresh = useCallback(async () => {
    const captured = ++sequence.current;
    const [next, read, run, selected] = await Promise.all([
      request<PrivatePredictionState>("GET", `${base}/state`),
      request<{ ok: boolean; source: string | null }>("POST", `${base}/tool`, { name: "read_doc", args: [] }),
      request<Agent>("GET", `${base}/agent`),
      request<{ evidence: PredictionHistory | null }>("GET", `${base}/evidence`),
    ]);
    if (!owner.current || captured !== sequence.current) return;
    validatePrivatePredictionState(next);
    if (next.document !== initial.document || next.actual_values !== "withheld") throw Error("Private session identity changed; reopen this window.");
    if (selected.evidence) validatePredictionHistory(selected.evidence);
    setState(next); setAgent(run); setEvidence(selected.evidence);
    if (read.ok && read.source !== null) setSource(read.source);
    else { setSource(""); throw Error("Captured source is unavailable or changed outside the session. Host recovery is required."); }
    setSyncError("");
  }, [initial.document]);
  useEffect(() => {
    if (ended) return;
    owner.current = true;
    const poll = () => void refresh().catch(reason => { if (owner.current) setSyncError(String(reason)); });
    poll(); const timer = setInterval(poll, 1000);
    return () => { owner.current = false; sequence.current++; clearInterval(timer); };
  }, [refresh, ended]);
  const running = agent?.run?.status === "running";
  const attempts = evidence?.history.attempts ?? [];
  const latestAttempt = state.retained_plan ? [...attempts].reverse().find(attempt => evidence?.history.ordering.some(entry => entry.event.id === attempt.seal && (entry.event.seal as { target?: { kind?: string } } | undefined)?.target?.kind === "combined")) : attempts.at(-1);
  const latestSeal = evidence?.history.ordering.find(entry => entry.event.id === latestAttempt?.seal)?.event.seal as { identity?: { source?: string } } | undefined;
  const canPromote = !!state.runtime_revalidation && latestSeal?.identity?.source === state.source_hash && !!latestAttempt && ["first_attempt_verified", "informed_prediction_verified", "verified_after_correction", "verified_after_prior_attempt"].includes(String(latestAttempt.classification));
  useEffect(() => { if (promotion && promotion.source_hash !== state.source_hash) setPromotion(null); }, [state.source_hash, promotion]);
  const repaired = state.retained_plan ? latestAttempt?.classification === "verified_after_correction" : attempts.some(attempt => attempt.classification === "verified_after_correction");
  const needsFuture = state.retained_debugger?.phase === "paused_needs_prediction";
  const failed = attempts.some(attempt => ["mismatch", "requirement_failed", "execution_rejected"].includes(String(attempt.classification)));
  const pending = agent?.edits?.changes.some(change => change.status === "pending");
  const stage = pending ? "Review the proposed correction" : needsFuture ? "Predict before Continue" : repaired ? "Compare retained evidence" : failed ? "Delegate the correction" : "Delegate prediction and investigation";
  const target = pending ? "#private-review" : needsFuture ? '[data-private-coach="seal"]' : repaired ? '[data-private-coach="promote"]' : '[data-private-coach="delegate"]';
  useUiReading("private prediction session status", { state, running, stage, request: { prompt, provider, model }, attempts: attempts.length, error, syncError, promotion_at_request: promotion, current_certification: false });
  const revision = useUiRevision(() => [state, agent?.run, prompt, provider, model, prediction, rationale, busy, open, pendingForms]);
  const promptRevision = useUiRevision(() => [initial.document, prompt, busy, running]);
  const providerRevision = useUiRevision(() => [initial.document, provider, busy, running]);
  const modelRevision = useUiRevision(() => [initial.document, model, busy, running]);
  const predictionRevision = useUiRevision(() => [initial.document, prediction, busy]);
  const rationaleRevision = useUiRevision(() => [initial.document, rationale, busy]);
  const retainedCommand = async (name: string, args: unknown) => {
    setBusy(true); setError("");
    try {
      const actual = await request<{ isError: boolean; structuredContent?: Record<string, import("../lib/uiActions").UiValue> }>("POST", `${base}/debugger`, { name, arguments: args });
      await refresh();
      if (!actual.structuredContent) throw Error("Retained debugger request refused; inspect source/runtime, host plan and seal ordering.");
      return { status: "completed" as const, value: { ...actual.structuredContent, non_passing: actual.isError } };
    } catch (reason) { setError(String(reason)); return { status: "unavailable" as const, reason: String(reason) }; }
    finally { setBusy(false); }
  };
  const available = (commands: { key: string; command: import("../lib/uiActions").UiCommand }[]) => commands.filter(item => !!state.retained_plan || !["private:prediction", "private:rationale", "private:seal-retained", "private:continue-retained"].includes(item.key));
  const invoke = useUiCommands(ended ? [] : available([
    ...([
      ["prompt", "Edit private investigation request", 32768, promptRevision, setPrompt],
      ["provider", "Choose private investigation provider", 128, providerRevision, setProvider],
      ["model", "Choose private investigation model", 1024, modelRevision, setModel],
      ["prediction", "Edit private debugger predicate", 32768, predictionRevision, setPrediction],
      ["rationale", "Edit private debugger rationale", 8192, rationaleRevision, setRationale],
    ] as const).map(([field, label, maxLength, fieldRevision, setValue]) => ({ key: `private:${field}`, command: {
      label, parent: "private-prediction", subject: `${initial.document}:${field}`,
      parameters: { value: { type: "string" as const, maxLength } }, prerequisites: [],
      read: () => ({ revision: fieldRevision(), enabled: !busy && (["prediction", "rationale"].includes(field) || !running) }),
      execute: async (args: Record<string, import("../lib/uiActions").UiValue>) => { setValue(String(args.value)); return { status: "completed" as const, value: {} }; },
    } })),
    { key: "private:start", command: { label: "Delegate private investigation", parent: "private-prediction", subject: initial.document, parameters: {}, prerequisites: ["Configured provider and confined Python runtime; exact startup-bound private workspace"],
      read: () => ({ revision: revision(), enabled: !busy && !pendingForms && !running && !!prompt.trim() && state.current === true }), execute: async (_args, actor) => {
        if (actor.kind === "agent") return { status: "requires_input", request: "private-investigation", reason: "A human starts the provider conversation; the active agent must not recursively delegate itself." };
        setBusy(true); setError("");
        try { const ui_window = await captureUiWindow(); const actual = await request<Agent>("POST", `${base}/agent`, { prompt, ...(ui_window ? { ui_window } : {}), ...(provider ? { provider } : {}), ...(model ? { model } : {}) }); if (owner.current) setAgent(actual); return { status: "completed", value: { dispatched: true, verification: "not established" } }; }
        catch (reason) { if (owner.current) setError(String(reason)); return { status: "unavailable", reason: String(reason) }; }
        finally { if (owner.current) setBusy(false); }
      } } },
    { key: "private:promote", command: { label: "Revalidate and promote private prediction", parent: "private-prediction", subject: initial.document, parameters: {}, prerequisites: ["Host-bound inputs and a current supported runtime profile; original prediction and requirement both pass"],
      read: () => ({ revision: revision(), enabled: canPromote && !busy && !running && state.current === true }), execute: async () => {
        if (!latestAttempt) return { status: "unavailable", reason: "No retained passing attempt" };
        setBusy(true); setError("");
        try { const actual = await request<NonNullable<typeof promotion>>("POST", `${base}/promote`, { attempt: latestAttempt.attempt }); if (owner.current) setPromotion(actual); return { status: "completed", value: { checked_at: actual.checked_at, source_hash: actual.source_hash, scope: actual.scope, historical_request_result: true } }; }
        catch (reason) { if (owner.current) setError(String(reason)); return { status: "unavailable", reason: String(reason) }; }
        finally { if (owner.current) setBusy(false); }
      } } },
    { key: "private:stop", command: { label: "Stop private investigation", parent: "private-prediction", subject: initial.document, parameters: {}, prerequisites: [], read: () => ({ revision: revision(), enabled: !!running }), execute: async () => { await request("POST", `${base}/agent/stop`, {}); await refresh(); return { status: "completed", value: { stop_requested: true } }; } } },
    { key: "private:seal-retained", command: { label: "Seal debugger prediction", parent: "private-prediction", subject: initial.document, parameters: {}, prerequisites: ["Trusted initial/future question, approved source and current configured runtime"], read: () => ({ revision: revision(), enabled: !!state.retained_plan && !busy && !pendingForms && !!rationale.trim() && ["not_started", "paused_needs_prediction", "expired"].includes(state.retained_debugger?.phase ?? "") }), execute: async () => {
      let predicted: unknown; try { predicted = JSON.parse(prediction); } catch { return { status: "unavailable", reason: "Enter a supported predicate as JSON" }; }
      return retainedCommand("seal_debugger_prediction", { expected_source_hash: state.source_hash, predicted, rationale });
    } } },
    { key: "private:continue-retained", command: { label: "Launch or continue private debugger", parent: "private-prediction", subject: initial.document, parameters: {}, prerequisites: ["A sealed prediction for this initial or future stop"], read: () => ({ revision: revision(), enabled: !!state.retained_plan && !busy && state.retained_debugger?.phase === "sealed" }), execute: async () => retainedCommand("continue_debugger_prediction", {}) } },
    { key: "private:end", command: { label: "End private practice", parent: "private-prediction", subject: initial.document, parameters: {}, prerequisites: ["Stop the investigation, wait for verification, and close other clients of this practice"], read: () => ({ revision: revision(), enabled: !busy && !running }), execute: async (_args, actor) => {
      if (actor.kind === "agent") return { status: "requires_input", request: "end-private-practice", reason: "A human ends the isolated practice. The active predictor cannot release its own quarantine." };
      setBusy(true); setError("");
      try { const actual = await request<{ ended: boolean; history_retained: boolean; root_still_private: boolean }>("POST", `${base}/end`, {}); if (actual.ended !== true || actual.history_retained !== true || actual.root_still_private !== true) throw Error("Invalid private lifecycle receipt"); setEnded(true); return { status: "completed", value: { ended: true, history_retained: true, root_still_private: true, context_reusable: false } }; }
      catch (reason) { setError(String(reason)); return { status: "unavailable", reason: String(reason) }; }
      finally { setBusy(false); }
    } } },
    { key: "private:coach", command: { label: "Set private coach visibility", parent: "private-prediction", subject: initial.document, parameters: { open: { type: "boolean" } }, prerequisites: [], read: () => ({ revision: revision(), enabled: true }), execute: async args => { const visible = Boolean(args.open); setOpen(visible); localStorage.setItem(key, visible ? "open" : "dismissed"); return { status: "completed", value: {} }; } } },
  ]));
  const setRequest = (field: "prompt" | "provider" | "model" | "prediction" | "rationale", value: string) => {
    setPendingForms(count => count + 1);
    void invoke(`private:${field}`, { value }).catch(reason => { if (owner.current) setError(String(reason)); })
      .finally(() => { if (owner.current) setPendingForms(count => count - 1); });
  };
  if (ended) return <main className="private-prediction-coach"><h1>Private practice ended</h1><p>The prediction history remains retained and this root remains private. Close this window. Reopening the practice starts a fresh context and revalidates its approved source and runtime.</p></main>;
  return <main className="private-prediction-coach">
    <h1>Predict, investigate and review</h1>
    <p>{state.question.requirement_reference}</p>
    <p>Target answers and runtime diagnostics stay withheld. Current certification is unavailable until host revalidation and promotion. {state.environment_scope}</p>
    {error && <p role="alert">{error}</p>}{syncError && <p role="alert">{syncError}</p>}
    <section id="private-delegate"><h2>Delegate to the agent</h2>
      <div className="private-prediction-fields"><label className="private-prediction-request">Investigation request<textarea maxLength={32768} value={prompt} disabled={busy || running} onChange={event => setRequest("prompt", event.target.value)} /></label>
      <label>Provider<input maxLength={128} value={provider} disabled={busy || running} onChange={event => setRequest("provider", event.target.value)} placeholder="Configured default" /></label>
      <label>Model<input maxLength={1024} value={model} disabled={busy || running} onChange={event => setRequest("model", event.target.value)} placeholder="Provider default" /></label></div>
      <div className="private-prediction-controls">
      <button data-private-coach="delegate" disabled={busy || !!pendingForms || running || !prompt.trim() || state.current !== true} onClick={() => void invoke("private:start")}>Delegate investigation</button>
      <button disabled={!running} onClick={() => void invoke("private:stop")}>Stop investigation</button>
      <button onClick={() => void invoke("private:coach", { open: true })}>Resume coach</button>
      <button disabled={busy || running} onClick={() => void invoke("private:end")}>End private practice</button></div>
      <p role="status">{agent?.run?.status ?? "Ready for a request"}</p><p>{agent?.run?.answer}</p>
    </section>
    <section><h2>Captured source</h2><p>SHA-256: {state.source_hash}</p><CodeEditor path={state.source} item={{ id: initial.document, label: "Captured private source", kind: "authored", text: source }} editable={false} onChange={() => {}} /></section>
    {state.retained_plan && <section id="private-debugger"><h2>Predict the next debugger stop</h2>
      <p>Before launch: {state.retained_plan.initial.requirement_reference}. Before Continue: {state.retained_plan.next.requirement_reference}.</p>
      <p>Phase: {state.retained_debugger?.phase ?? "unavailable"}. Target values stay withheld. A paused capture expires after two minutes; a new sequence retains the earlier history. Output verification runs separately.</p>
      <div className="private-prediction-fields"><label>Debugger predicate JSON<textarea maxLength={32768} value={prediction} disabled={busy} onChange={event => setRequest("prediction", event.target.value)} /></label>
      <label>Debugger prediction rationale<textarea maxLength={8192} value={rationale} disabled={busy} onChange={event => setRequest("rationale", event.target.value)} /></label></div>
      <div className="private-prediction-controls">
      <button data-private-coach="seal" disabled={busy || !!pendingForms || !rationale.trim() || !["not_started", "paused_needs_prediction", "expired"].includes(state.retained_debugger?.phase ?? "")} onClick={() => void invoke("private:seal-retained")}>Seal debugger prediction</button>
      <button disabled={busy || state.retained_debugger?.phase !== "sealed"} onClick={() => void invoke("private:continue-retained")}>Launch or continue private debugger</button></div>
    </section>}
    <section id="private-review"><AgentChanges doc={`private:${initial.document}`} state={agent} running={!!running} update={value => setAgent(value as Agent)} submitDecision={async body => {
      const actual = await request<Agent>("POST", `${base}/agent/decision`, { id: body.id, accepted: body.accepted, ...(body.new_text !== undefined ? { new_text: body.new_text } : {}) });
      await refresh(); return actual;
    }} /></section>
    <section id="private-evidence"><button data-private-coach="promote" disabled={!canPromote || busy || running || state.current !== true} onClick={() => void invoke("private:promote")}>Revalidate and promote</button>
      {promotion && <p role="status">Host revalidation at {promotion.checked_at} for source {promotion.source_hash}. {promotion.scope}</p>}{evidence ? <PredictionEvidence value={evidence} sourceHash={state.source_hash} /> : <p>No retained execution attempt yet. The agent must seal a prediction before execution.</p>}</section>
    {open && <AnchoredCoach target={target} title={stage} onDismiss={() => void invoke("private:coach", { open: false })}>
      <p>{pending ? "Review the exact proposed source diff. Accept or reject it yourself; the agent cannot approve its own correction." : needsFuture ? "Before resuming the retained process, seal the next prediction against this source and paused context. Prior verdicts remain visible; selected target values stay withheld." : repaired ? "The repair passed the unchanged prediction. The original failure stays in history. Use Revalidate and promote when the host supplies a supported runtime and fixed input binding; the button remains unavailable otherwise." : failed ? "The comparison failed. Ask the agent to reason from the requirement and captured source, propose a reviewed repair, then repeat the unchanged prediction." : "Delegate the investigation. The agent records its own prediction before the existing runner and debugger compare it privately."}</p>
      <p>{state.retained_plan ? "Seal before launch, then predict the future stop before Continue. Prior visible verdicts and paused identity remain in the retained evidence." : "This host has not supplied a retained future-stop question plan."} Full application integration remains in progress.</p>
    </AnchoredCoach>}
  </main>;
}
