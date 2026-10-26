<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/web/src/components/PrivatePredictionCoach.tsx

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-f32ded7fd60b"></a>
## `apps/web/src/components/PrivatePredictionCoach.tsx`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [Explicit UI action dispatch checks scope, state, policy and retries](../../../../../guarantees/agent/ui-action-dispatch/contract.md), [Keep a prediction separate from the answer it is tested against](../../../../../guarantees/verification/predictions-precede-managed-observation/contract.md), [Private edit proposals open review before writing](../../../../../guarantees/verification/private-proposals-wait-for-review/contract.md).

<a id="f-f32ded7fd60b-PrivatePredictionState"></a>
### `PrivatePredictionState`

<storycode:copy id="f-f32ded7fd60b-PrivatePredictionState">
import type { UiActor, UiOutcome } from "../lib/uiActions";
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
</storycode:copy>
<a id="f-f32ded7fd60b-object"></a>
### `object`

<storycode:copy id="f-f32ded7fd60b-object">
const object = (value: unknown): value is Record<string, unknown> => !!value && typeof value === "object" && !Array.isArray(value);
</storycode:copy>
<a id="f-f32ded7fd60b-only"></a>
### `only`

<storycode:copy id="f-f32ded7fd60b-only">
const only = (value: object, keys: string[]) => Object.keys(value).every(key => keys.includes(key));
</storycode:copy>
<a id="f-f32ded7fd60b-safeComparison"></a>
### `safeComparison`

<storycode:copy id="f-f32ded7fd60b-safeComparison">
function safeComparison(value: unknown): boolean {
  if (value == null) return true;
  return object(value) && only(value, ["summary", "dispatched"]) && typeof value.dispatched === "boolean" && object(value.summary)
    && only(value.summary, ["attempt", "scenario", "seal", "classification", "prediction_matched", "original_requirement_satisfied", "observed", "promoted", "current_certification"])
    && value.summary.current_certification === false;
}
</storycode:copy>
<a id="f-f32ded7fd60b-validatePrivatePredictionState"></a>
### `validatePrivatePredictionState`

<storycode:copy id="f-f32ded7fd60b-validatePrivatePredictionState">
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
</storycode:copy>
<a id="f-f32ded7fd60b-Agent"></a>
### `Agent`

<storycode:copy id="f-f32ded7fd60b-Agent">
type Agent = AcpState & { run?: { id: string; status: string; answer?: string } | null };
</storycode:copy>
<a id="f-f32ded7fd60b-base"></a>
### `base`

<storycode:copy id="f-f32ded7fd60b-base">
const base = "/api/prediction";
</storycode:copy>
<a id="f-f32ded7fd60b-PrivatePredictionCoach"></a>
### `PrivatePredictionCoach`

Host-bound branch of NativeWorkflowTour. No ordinary workspace/runtime reader mounts.

<storycode:copy id="f-f32ded7fd60b-PrivatePredictionCoach">
/** Host-bound branch of NativeWorkflowTour. No ordinary workspace/runtime reader mounts. */
export function PrivatePredictionCoach({ initial }: { initial: PrivatePredictionState }) {
  const [state, setState] = useState(initial), [source, setSource] = useState("");
  const [agent, setAgent] = useState<Agent | null>(null), [evidence, setEvidence] = useState<PredictionHistory | null>(null);
  const [prompt, setPrompt] = useState((initial.retained_plan ? "Use the host retained-stop plan: seal the initial debugger prediction before launch, then seal the future-stop prediction while paused before Continue. After reviewed repair repeat the same expectations for both stops and the combined output check. " : "") + "Investigate the host requirement using captured source. Seal your prediction and rationale before Verify. If it fails, explain and propose a correction for human review, then reseal the unchanged expectation and verify. Keep held-out target answers withheld.");
  const [acpAgent, setAcpAgent] = useState("");
  const [provider, setProvider] = useState(""), [model, setModel] = useState("");
  const [promotion, setPromotion] = useState<{ source_hash: string; checked_at: string; scope: string; summary: unknown } | null>(null);
  const [prediction, setPrediction] = useState('{"kind":"equals","value":null}'), [rationale, setRationale] = useState("");
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
  useUiReading("private prediction session status", { state, running, stage, request: { prompt, provider, model, acp_agent: acpAgent }, attempts: attempts.length, error, syncError, promotion_at_request: promotion, current_certification: false });
  const revision = useUiRevision(() => [state, agent?.run, prompt, provider, model, acpAgent, prediction, rationale, busy, open]);
  const promptRevision = useUiRevision(() => [initial.document, prompt, busy, running]);
  const acpRevision = useUiRevision(() => [initial.document, acpAgent, busy, running]);
  const providerRevision = useUiRevision(() => [initial.document, provider, busy, running]);
  const modelRevision = useUiRevision(() => [initial.document, model, busy, running]);
  const predictionRevision = useUiRevision(() => [initial.document, prediction, busy]);
  const rationaleRevision = useUiRevision(() => [initial.document, rationale, busy]);
  const requestFields = [
    ["prompt", "Edit private investigation request", 32768, promptRevision, setPrompt],
    ["provider", "Choose private investigation provider", 128, providerRevision, setProvider],
    ["acp", "Choose private ACP adapter", 128, acpRevision, setAcpAgent],
    ["model", "Choose private investigation model", 1024, modelRevision, setModel],
    ["prediction", "Edit private debugger predicate", 32768, predictionRevision, setPrediction],
    ["rationale", "Edit private debugger rationale", 8192, rationaleRevision, setRationale],
  ] as const;
  const editRequest = (field: string, value: string): UiOutcome => {
    const entry = requestFields.find(entry => entry[0] === field);
    if (!entry || busy || (running && !["prediction", "rationale"].includes(field))) return { status: "unavailable", reason: "The current private form is unavailable" };
    if (value.length > entry[2]) return { status: "unavailable", reason: "Draft exceeds the field bound" };
    entry[4](value);
    return { status: "completed", value: { draft_changed: true, execution_started: false } };
  };
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
  const closing = useRef(false), closeOwner = useRef(false);
  useEffect(() => { closeOwner.current = true; return () => { closeOwner.current = false; }; }, []);
  const closePrivateWindow = async (actor: UiActor): Promise<UiOutcome> => {
    if (actor.kind === "agent") return { status: "requires_input", request: "private-window-close", reason: "A human ends and closes the isolated practice; the active predictor cannot release its own window/quarantine" };
    if (!ended) { const reason = "End private practice before closing this window. Stop any investigation first; its retained history will remain protected."; setError(reason); return { status: "requires_input", request: "private-end-before-close", reason }; }
    if (closing.current) return { status: "unavailable", reason: "A native close request is pending" };
    closing.current = true; setBusy(true); setError("");
    try {
      const actual = await request<{ ok: boolean }>("POST", "/api/window/close", {});
      if (actual.ok !== true) throw Error("Native host did not acknowledge this close request");
      return { status: "completed", value: { host_acknowledged: true, window_destruction_unverified: true } };
    } catch { const reason = "This host could not confirm window closure. Use its browser/OS close control; the private history and root remain protected."; if (closeOwner.current) setError(reason); return { status: "requires_input", request: "private-window-host-close", reason }; }
    finally { closing.current = false; if (closeOwner.current) setBusy(false); }
  };
  const latestClose = useRef(closePrivateWindow); latestClose.current = closePrivateWindow;
  useEffect(() => {
    const close = () => { void latestClose.current({ id: "local-user", kind: "human" }); };
    window.addEventListener("hickory-window-close-request", close);
    return () => window.removeEventListener("hickory-window-close-request", close);
  }, []);
  const invoke = useUiCommands([
    { key: "private:close-window", command: { label: "Close private practice window", parent: "private-window", subject: initial.document, parameters: {}, prerequisites: ["Human-owned lifecycle; End must finish before host close. Window closure is independent of an expired retired project router."],
      read: () => ({ revision: JSON.stringify([ended, busy, closing.current]), enabled: !busy && !closing.current }), execute: (_args, actor) => closePrivateWindow(actor) } },
    ...(ended ? [] : available([
    ...requestFields.map(([field, label, maxLength, fieldRevision]) => ({ key: `private:${field}`, command: {
      label, parent: "private-prediction", subject: `${initial.document}:${field}`,
      parameters: { value: { type: "string" as const, maxLength } }, prerequisites: [],
      read: () => ({ revision: fieldRevision(), enabled: !busy && (["prediction", "rationale"].includes(field) || !running) }),
      execute: async (args: Record<string, import("../lib/uiActions").UiValue>) => editRequest(field, String(args.value)),
    } })),
    { key: "private:start", command: { label: "Delegate private investigation", parent: "private-prediction", subject: initial.document, parameters: {}, prerequisites: ["Configured Python provider/runtime or explicit fresh-context ACP adapter; exact startup-bound private workspace"],
      read: () => ({ revision: revision(), enabled: !busy && !running && !!prompt.trim() && state.current === true }), execute: async (_args, actor) => {
        if (actor.kind === "agent") return { status: "requires_input", request: "private-investigation", reason: "A human starts the provider conversation; the active agent must not recursively delegate itself." };
        setBusy(true); setError("");
        try { const ui_window = await captureUiWindow(); const actual = await request<Agent>("POST", `${base}/agent`, { prompt, ...(ui_window ? { ui_window } : {}), ...(acpAgent ? { acp_agent: acpAgent } : {}), ...(provider ? { provider } : {}), ...(model ? { model } : {}) }); if (owner.current) setAgent(actual); return { status: "completed", value: { dispatched: true, verification: "not established" } }; }
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
    { key: "private:seal-retained", command: { label: "Seal debugger prediction", parent: "private-prediction", subject: initial.document, parameters: {}, prerequisites: ["Trusted initial/future question, approved source and current configured runtime"], read: () => ({ revision: revision(), enabled: !!state.retained_plan && !busy && !!rationale.trim() && ["not_started", "paused_needs_prediction", "expired"].includes(state.retained_debugger?.phase ?? "") }), execute: async () => {
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
  ])),
  ]);
  // Reversible human draft edits must update synchronously. Agent commands use
  // this same controller with intent/result receipts; Delegate/Seal remain the
  // authoritative persisted boundaries before provider/runtime execution.
  const setRequest = (field: string, value: string) => {
    const result = editRequest(field, value);
    if (result.status === "unavailable") setError(result.reason);
  };
  if (ended) return <main className="private-prediction-coach"><h1>Private practice ended</h1><p>The prediction history remains retained and this root remains private. Reopening the practice starts a fresh context and revalidates its approved source and runtime.</p>
    {error && <p role="alert">{error}</p>}
    {/* The project lease is retired: human host close must not depend on its receipt endpoint. */}
    <button disabled={busy || closing.current} onClick={() => { void latestClose.current({ id: "local-user", kind: "human" }); }}>Close window</button>
  </main>;
  return <main className="private-prediction-coach">
    <h1>Predict, investigate and review</h1>
    <p>{state.question.requirement_reference}</p>
    <p>Target answers and runtime diagnostics stay withheld. Current certification is unavailable until host revalidation and promotion. {state.environment_scope}</p>
    {error && <p role="alert">{error}</p>}{syncError && <p role="alert">{syncError}</p>}
    <section id="private-delegate"><h2>Delegate to the agent</h2>
      <div className="private-prediction-fields"><label className="private-prediction-request">Investigation request<textarea maxLength={32768} value={prompt} disabled={busy || running} onChange={event => setRequest("prompt", event.target.value)} /></label>
      <label>Provider<input maxLength={128} value={provider} disabled={busy || running} onChange={event => setRequest("provider", event.target.value)} placeholder="Configured default" /></label>
      <label>Model<input maxLength={1024} value={model} disabled={busy || running} onChange={event => setRequest("model", event.target.value)} placeholder="Provider default" /></label>
      <label className="private-prediction-request">ACP adapter (optional)<input maxLength={128} value={acpAgent} disabled={busy || running} onChange={event => setRequest("acp", event.target.value)} placeholder="Leave blank for confined Python" /></label></div>
      <p>ACP requires an installed adapter with explicit private arguments and HTTP MCP support. Clear Provider and Model when choosing ACP. Its fresh managed context has no client filesystem, terminal or ordinary history. External harness internals are outside this managed boundary.</p>
      <div className="private-prediction-controls">
      <button data-private-coach="delegate" disabled={busy || running || !prompt.trim() || state.current !== true} onClick={() => void invoke("private:start")}>Delegate investigation</button>
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
      <button data-private-coach="seal" disabled={busy || !rationale.trim() || !["not_started", "paused_needs_prediction", "expired"].includes(state.retained_debugger?.phase ?? "")} onClick={() => void invoke("private:seal-retained")}>Seal debugger prediction</button>
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
</storycode:copy>

### Assemble `apps/web/src/components/PrivatePredictionCoach.tsx`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/components/PrivatePredictionCoach.tsx"><storycode:paste select="#f-f32ded7fd60b-PrivatePredictionState"/><storycode:paste select="#f-f32ded7fd60b-object"/><storycode:paste select="#f-f32ded7fd60b-only"/><storycode:paste select="#f-f32ded7fd60b-safeComparison"/><storycode:paste select="#f-f32ded7fd60b-validatePrivatePredictionState"/><storycode:paste select="#f-f32ded7fd60b-Agent"/><storycode:paste select="#f-f32ded7fd60b-base"/><storycode:paste select="#f-f32ded7fd60b-PrivatePredictionCoach"/></storycode:file>


## Exercise this mechanism

The test module below imports this implementation directly. Keep the assertions with the mechanism they exercise; passing reconstruction alone does not show those assertions ran.

<a id="output-4b3612758c98"></a>
## `apps/web/src/components/PrivatePredictionCoach.test.tsx`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [Explicit UI action dispatch checks scope, state, policy and retries](../../../../../guarantees/agent/ui-action-dispatch/contract.md), [Keep a prediction separate from the answer it is tested against](../../../../../guarantees/verification/predictions-precede-managed-observation/contract.md), [Private edit proposals open review before writing](../../../../../guarantees/verification/private-proposals-wait-for-review/contract.md).

<a id="f-4b3612758c98-expression_statement"></a>
### `expression_statement`

<storycode:copy id="f-4b3612758c98-expression_statement">
// Guarantee: docs/guarantees/verification/predictions-precede-managed-observation.md
import { act, cleanup, fireEvent, render, screen, waitFor } from "@testing-library/react";
import { afterEach, expect, it, vi } from "vitest";
import { request } from "../api/client";
import { UiActions } from "../lib/uiActions";
import { UiActionContext } from "../lib/uiActionBridge";
import { PrivatePredictionCoach, validatePrivatePredictionState, type PrivatePredictionState } from "./PrivatePredictionCoach";
vi.mock("../api/client", () => ({ request: vi.fn() }));
</storycode:copy>
<a id="f-4b3612758c98-expression_statement-2"></a>
### `expression_statement`

<storycode:copy id="f-4b3612758c98-expression_statement-2">
vi.mock("./ComparisonEditor", () => ({ CodeEditor: ({ item }: { item: { text: string } }) => <pre>{item.text}</pre> }));
</storycode:copy>
<a id="f-4b3612758c98-expression_statement-3"></a>
### `expression_statement`

<storycode:copy id="f-4b3612758c98-expression_statement-3">
afterEach(() => { cleanup(); vi.clearAllMocks(); localStorage.clear(); });
</storycode:copy>
<a id="f-4b3612758c98-initial"></a>
### `initial`

<storycode:copy id="f-4b3612758c98-initial">
const initial: PrivatePredictionState = { document: "bound", source: "doc.md", source_hash: "a".repeat(64), current: true, question: { requirement_reference: "A single total budget" }, target: {}, environment_scope: "Host startup identities only", actual_values: "withheld", application_integration: "incomplete" };
</storycode:copy>
<a id="f-4b3612758c98-responses"></a>
### `responses`

<storycode:copy id="f-4b3612758c98-responses">
function responses() {
  vi.mocked(request).mockImplementation(async (method, path) => {
    if (path.endsWith("/state")) return initial;
    if (path.endsWith("/tool")) return { ok: true, source: "captured source only" };
    if (path.endsWith("/evidence")) return { evidence: null };
    if (path.endsWith("/agent")) return { backend: "python", ready: true, edits: { mode: "review", changes: [] }, run: method === "POST" ? { id: "run", status: "running" } : null };
    throw Error(`Unexpected surface ${path}`);
  });
}
</storycode:copy>
<a id="f-4b3612758c98-expression_statement-4"></a>
### `expression_statement`

<storycode:copy id="f-4b3612758c98-expression_statement-4">
it("uses only quarantined readers and starts a provider request only after explicit delegation", async () => {
  responses(); render(<PrivatePredictionCoach initial={initial} />);
  await screen.findByText("captured source only");
  expect(vi.mocked(request).mock.calls.some(call => call[0] === "POST" && call[1].endsWith("/agent"))).toBe(false);
  expect(vi.mocked(request).mock.calls.every(call => call[1].startsWith("/api/prediction/"))).toBe(true);
  fireEvent.click(screen.getByRole("button", { name: "Delegate investigation" }));
  await waitFor(() => expect(vi.mocked(request).mock.calls.some(call => call[0] === "POST" && call[1] === "/api/prediction/agent")).toBe(true));
  expect(screen.getByText(/Current certification is unavailable/)).toBeTruthy();
});
</storycode:copy>
<a id="f-4b3612758c98-expression_statement-5"></a>
### `expression_statement`

<storycode:copy id="f-4b3612758c98-expression_statement-5">
it("requires human input for recursive provider delegation and preserves that request boundary", async () => {
  responses();
  const registry = new UiActions({ workspace: "private", window: "one" }, () => true, async () => {});
  render(<UiActionContext.Provider value={registry}><PrivatePredictionCoach initial={initial} /></UiActionContext.Provider>);
  await screen.findByText("captured source only");
  const command = registry.observe("private-prediction").actions.find(action => action.label === "Delegate private investigation")!;
  let result: unknown;
  await act(async () => { result = await registry.act(registry.scope, { id: "predictor", kind: "agent" }, { handle: command.handle, arguments: {}, request_id: "recursive" }); });
  expect(result).toMatchObject({ status: "result", outcome: { status: "requires_input" } });
  expect(vi.mocked(request).mock.calls.some(call => call[0] === "POST" && call[1].endsWith("/agent"))).toBe(false);
});

</storycode:copy>
<a id="f-4b3612758c98-expression_statement-6"></a>
### `expression_statement`

<storycode:copy id="f-4b3612758c98-expression_statement-6">
it("rejects malformed host metadata before rendering source-reading controls", () => {
  expect(() => validatePrivatePredictionState({ ...initial, source: { Unix: [100, 111, 99] } })).toThrow("Invalid private prediction startup response");
  expect(() => validatePrivatePredictionState({ ...initial, actual_values: "available" })).toThrow();
  expect(() => validatePrivatePredictionState({ ...initial, actual: "unexpected held-out value" })).toThrow();
  expect(() => validatePrivatePredictionState({ ...initial, retained_debugger: null })).toThrow("Invalid private prediction startup response");
  expect(() => validatePrivatePredictionState({ ...initial, retained_debugger: { phase: "complete", actual_values: "withheld", last: { summary: { current_certification: false }, dispatched: true, actual: "unexpected" } } })).toThrow();
});

</storycode:copy>
<a id="f-4b3612758c98-expression_statement-7"></a>
### `expression_statement`

<storycode:copy id="f-4b3612758c98-expression_statement-7">
it("retains consecutive agent provider/model edits with separate persisted intents", async () => {
  responses();
  const waiting: (() => void)[] = [];
  const registry = new UiActions({ workspace: "private", window: "forms" }, () => true, receipt =>
    receipt.stage === "requested" && receipt.action.label.startsWith("Choose private investigation")
      ? new Promise<void>(resolve => { waiting.push(resolve); }) : Promise.resolve());
  render(<UiActionContext.Provider value={registry}><PrivatePredictionCoach initial={initial} /></UiActionContext.Provider>);
  await screen.findByText("captured source only");
  const commands = registry.observe("private-prediction").actions;
  const provider = commands.find(command => command.label === "Choose private investigation provider")!;
  const model = commands.find(command => command.label === "Choose private investigation model")!;
  let edits: Promise<unknown>[] = [];
  await act(async () => { edits = [
    registry.act(registry.scope, { id: "predictor", kind: "agent" }, { handle: provider.handle, arguments: { value: "openrouter" }, request_id: "provider" }),
    registry.act(registry.scope, { id: "predictor", kind: "agent" }, { handle: model.handle, arguments: { value: "anthropic/claude-sonnet-4.5" }, request_id: "model" }),
  ]; await Promise.resolve(); });
  await waitFor(() => expect(waiting).toHaveLength(2));
  await act(async () => { waiting.forEach(release => release()); await Promise.all(edits); });
  expect((screen.getByRole("textbox", { name: "Provider" }) as HTMLInputElement).value).toBe("openrouter");
  expect((screen.getByRole("textbox", { name: "Model" }) as HTMLInputElement).value).toBe("anthropic/claude-sonnet-4.5");
  fireEvent.click(screen.getByRole("button", { name: "Delegate investigation" }));
  await waitFor(() => expect(vi.mocked(request).mock.calls.find(call => call[0] === "POST" && call[1].endsWith("/agent"))?.[2]).toMatchObject({ provider: "openrouter", model: "anthropic/claude-sonnet-4.5" }));
});

</storycode:copy>
<a id="f-4b3612758c98-expression_statement-8"></a>
### `expression_statement`

<storycode:copy id="f-4b3612758c98-expression_statement-8">
it("requires a human to end and retains a real host lifecycle receipt before retiring the coach", async () => {
  responses();
  const ordinary = vi.mocked(request).getMockImplementation()!;
  vi.mocked(request).mockImplementation(async (method, path, body) => path.endsWith("/end")
    ? { ended: true, history_retained: true, root_still_private: true }
    : ordinary(method, path, body));
  const registry = new UiActions({ workspace: "private", window: "ending" }, () => true, async () => {});
  render(<UiActionContext.Provider value={registry}><PrivatePredictionCoach initial={initial} /></UiActionContext.Provider>);
  await screen.findByText("captured source only");
  const command = registry.observe("private-prediction").actions.find(action => action.label === "End private practice")!;
  let result: unknown;
  await act(async () => { result = await registry.act(registry.scope, { id: "predictor", kind: "agent" }, { handle: command.handle, arguments: {}, request_id: "end-own-quarantine" }); });
  expect(result).toMatchObject({ status: "result", outcome: { status: "requires_input" } });
  expect(vi.mocked(request).mock.calls.some(call => call[1].endsWith("/end"))).toBe(false);
  fireEvent.click(screen.getByRole("button", { name: "End private practice" }));
  await screen.findByRole("heading", { name: "Private practice ended" });
  expect(screen.queryByRole("button", { name: "Delegate investigation" })).toBeNull();
  expect(registry.observe("private-prediction").actions).toHaveLength(0);
});

</storycode:copy>
<a id="f-4b3612758c98-expression_statement-9"></a>
### `expression_statement`

<storycode:copy id="f-4b3612758c98-expression_statement-9">
it("shares agent sealing and human Continue through the host-bound retained controller", async () => {
  let current: PrivatePredictionState = { ...initial, retained_plan: { initial: { requirement_reference: "Initial allowance" }, next: { requirement_reference: "Future allowance" } }, retained_debugger: { phase: "not_started", actual_values: "withheld", last: null } };
  const calls: unknown[] = [];
  vi.mocked(request).mockImplementation(async (_method, path, body) => {
    if (path.endsWith("/state")) return current;
    if (path.endsWith("/tool")) return { ok: true, source: "captured source only" };
    if (path.endsWith("/evidence")) return { evidence: null };
    if (path.endsWith("/agent")) return { backend: "python", ready: true, edits: { mode: "review", changes: [] } };
    if (path.endsWith("/debugger")) {
      calls.push(body);
      const seal = (body as { name: string }).name === "seal_debugger_prediction";
      current = { ...current, retained_debugger: { phase: seal ? "sealed" : "paused_needs_prediction", actual_values: "withheld", last: null } };
      return { isError: false, structuredContent: seal ? { sealed: "initial", executed: false } : { dispatched: true, summary: { classification: "first_attempt_verified" } } };
    }
    throw Error(`Unexpected ${path}`);
  });
  const registry = new UiActions({ workspace: "private", window: "retained" }, () => true, async () => {});
  render(<UiActionContext.Provider value={registry}><PrivatePredictionCoach initial={current} /></UiActionContext.Provider>);
  await screen.findByText("captured source only");
  expect((screen.getByRole("button", { name: "Launch or continue private debugger" }) as HTMLButtonElement).disabled).toBe(true);
  fireEvent.change(screen.getByRole("textbox", { name: "Debugger predicate JSON" }), { target: { value: '{"kind":"equals","value":100}' } });
  fireEvent.change(screen.getByRole("textbox", { name: "Debugger prediction rationale" }), { target: { value: "Initial budget before launch" } });
  await waitFor(() => expect((screen.getByRole("button", { name: "Seal debugger prediction" }) as HTMLButtonElement).disabled).toBe(false));
  const seal = registry.observe("private-prediction").actions.find(action => action.label === "Seal debugger prediction")!;
  await act(async () => { await registry.act(registry.scope, { id: "predictor", kind: "agent" }, { handle: seal.handle, arguments: {}, request_id: "seal-initial" }); });
  expect(calls[0]).toEqual({ name: "seal_debugger_prediction", arguments: { expected_source_hash: initial.source_hash, predicted: { kind: "equals", value: 100 }, rationale: "Initial budget before launch" } });
  fireEvent.click(screen.getByRole("button", { name: "Launch or continue private debugger" }));
  await waitFor(() => expect(calls).toHaveLength(2));
  expect(calls[1]).toEqual({ name: "continue_debugger_prediction", arguments: {} });
  await screen.findByText(/Phase: paused_needs_prediction/);
});


</storycode:copy>
<a id="f-4b3612758c98-expression_statement-10"></a>
### `expression_statement`

<storycode:copy id="f-4b3612758c98-expression_statement-10">
it("dispatches an explicitly chosen private ACP adapter without inventing a passing result", async () => {
  responses(); render(<PrivatePredictionCoach initial={initial} />);
  await screen.findByText("captured source only");
  fireEvent.change(screen.getByRole("textbox", { name: "ACP adapter (optional)" }), { target: { value: "isolated-qa" } });
  await waitFor(() => expect((screen.getByRole("button", { name: "Delegate investigation" }) as HTMLButtonElement).disabled).toBe(false));
  fireEvent.click(screen.getByRole("button", { name: "Delegate investigation" }));
  await waitFor(() => expect(vi.mocked(request).mock.calls.find(call => call[0] === "POST" && call[1].endsWith("/agent"))?.[2]).toMatchObject({ acp_agent: "isolated-qa" }));
  expect(screen.getByText(/Current certification is unavailable/)).toBeTruthy();
});

</storycode:copy>
<a id="f-4b3612758c98-expression_statement-11"></a>
### `expression_statement`

<storycode:copy id="f-4b3612758c98-expression_statement-11">
it("requires End before native close and closes after End independently of retired receipts", async () => {
  responses();
  const ordinary = vi.mocked(request).getMockImplementation()!;
  vi.mocked(request).mockImplementation(async (method, path, body) => {
    if (path.endsWith("/end")) return { ended: true, history_retained: true, root_still_private: true };
    if (path === "/api/window/close") return { ok: true };
    return ordinary(method, path, body);
  });
  let retired = false;
  const registry = new UiActions({ workspace: "private", window: "close" }, () => true, async () => { if (retired) throw Error("retired receipt endpoint"); });
  render(<UiActionContext.Provider value={registry}><PrivatePredictionCoach initial={initial} /></UiActionContext.Provider>);
  await screen.findByText("captured source only");
  await act(async () => { window.dispatchEvent(new Event("hickory-window-close-request")); });
  expect(screen.getByText(/End private practice before closing/)).toBeTruthy();
  expect(vi.mocked(request).mock.calls.some(call => call[1] === "/api/window/close")).toBe(false);
  const command = registry.observe("private-window").actions[0];
  let result: unknown;
  await act(async () => { result = await registry.act(registry.scope, { kind: "agent", id: "predictor" }, { handle: command.handle, arguments: {}, request_id: "close-quarantine" }); });
  expect(result).toMatchObject({ outcome: { status: "requires_input" } });
  fireEvent.click(screen.getByRole("button", { name: "End private practice" }));
  await screen.findByRole("heading", { name: "Private practice ended" });
  retired = true;
  fireEvent.click(screen.getByRole("button", { name: "Close window" }));
  await waitFor(() => expect(request).toHaveBeenCalledWith("POST", "/api/window/close", {}));
});

</storycode:copy>
<a id="f-4b3612758c98-expression_statement-12"></a>
### `expression_statement`

<storycode:copy id="f-4b3612758c98-expression_statement-12">
it("reports uncertain native closure without disclosing host errors or retrying automatically", async () => {
  responses();
  const ordinary = vi.mocked(request).getMockImplementation()!;
  vi.mocked(request).mockImplementation(async (method, path, body) => {
    if (path.endsWith("/end")) return { ended: true, history_retained: true, root_still_private: true };
    if (path === "/api/window/close") throw Error("private-host-secret");
    return ordinary(method, path, body);
  });
  const registry = new UiActions({ workspace: "private", window: "failed-close" }, () => true, async () => {});
  const rendered = render(<UiActionContext.Provider value={registry}><PrivatePredictionCoach initial={initial} /></UiActionContext.Provider>);
  await screen.findByText("captured source only");
  fireEvent.click(screen.getByRole("button", { name: "End private practice" }));
  await screen.findByRole("heading", { name: "Private practice ended" });
  await act(async () => { window.dispatchEvent(new Event("hickory-window-close-request")); });
  await screen.findByText(/This host could not confirm window closure/);
  expect(screen.queryByText(/private-host-secret/)).toBeNull();
  expect(vi.mocked(request).mock.calls.filter(call => call[1] === "/api/window/close")).toHaveLength(1);
  expect((screen.getByRole("button", { name: "Close window" }) as HTMLButtonElement).disabled).toBe(false);
  rendered.unmount();
  window.dispatchEvent(new Event("hickory-window-close-request"));
  expect(vi.mocked(request).mock.calls.filter(call => call[1] === "/api/window/close")).toHaveLength(1);
});

</storycode:copy>
<a id="f-4b3612758c98-expression_statement-13"></a>
### `expression_statement`

<storycode:copy id="f-4b3612758c98-expression_statement-13">
// Guarantee: docs/guarantees/verification/private-proposals-wait-for-review.md
it("retains rapid human typing while draft-receipt persistence is delayed", async () => {
  responses();
  const release: (() => void)[] = [];
  const registry = new UiActions({ workspace: "private", window: "typing" }, () => true, receipt =>
    receipt.stage === "requested" && receipt.action.label === "Choose private investigation provider"
      ? new Promise<void>(resolve => release.push(resolve)) : Promise.resolve());
  render(<UiActionContext.Provider value={registry}><PrivatePredictionCoach initial={initial} /></UiActionContext.Provider>);
  await screen.findByText("captured source only");
  const provider = screen.getByRole("textbox", { name: "Provider" }) as HTMLInputElement;
  let typed = "";
  try {
    for (const letter of "deepseek") {
      typed += letter;
      fireEvent.change(provider, { target: { value: provider.value + letter } });
      expect(provider.value).toBe(typed);
    }
    expect(vi.mocked(request).mock.calls.some(call => call[0] === "POST" && call[1].endsWith("/agent"))).toBe(false);
    fireEvent.click(screen.getByRole("button", { name: "Delegate investigation" }));
    await waitFor(() => expect(vi.mocked(request).mock.calls.find(call => call[0] === "POST" && call[1].endsWith("/agent"))?.[2]).toMatchObject({ provider: "deepseek" }));
  } finally { await act(async () => { release.forEach(done => done()); }); }
});
</storycode:copy>

### Assemble `apps/web/src/components/PrivatePredictionCoach.test.tsx`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/components/PrivatePredictionCoach.test.tsx"><storycode:paste select="#f-4b3612758c98-expression_statement"/><storycode:paste select="#f-4b3612758c98-expression_statement-2"/><storycode:paste select="#f-4b3612758c98-expression_statement-3"/><storycode:paste select="#f-4b3612758c98-initial"/><storycode:paste select="#f-4b3612758c98-responses"/><storycode:paste select="#f-4b3612758c98-expression_statement-4"/><storycode:paste select="#f-4b3612758c98-expression_statement-5"/><storycode:paste select="#f-4b3612758c98-expression_statement-6"/><storycode:paste select="#f-4b3612758c98-expression_statement-7"/><storycode:paste select="#f-4b3612758c98-expression_statement-8"/><storycode:paste select="#f-4b3612758c98-expression_statement-9"/><storycode:paste select="#f-4b3612758c98-expression_statement-10"/><storycode:paste select="#f-4b3612758c98-expression_statement-11"/><storycode:paste select="#f-4b3612758c98-expression_statement-12"/><storycode:paste select="#f-4b3612758c98-expression_statement-13"/></storycode:file>


</storycode:doc>
