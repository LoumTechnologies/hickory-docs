import { useCallback, useEffect, useRef, useState } from "react";
import { request } from "../api/client";
import type { AcpState } from "../api/acp";
import { captureUiWindow, useUiCommands, useUiRevision } from "../lib/uiActionBridge";
import { useUiReading } from "../lib/uiReading";
import { PredictionEvidence, validatePredictionHistory, type PredictionHistory } from "../embed/PredictionHistory";
import { AnchoredCoach } from "./AnchoredCoach";
import { CodeEditor } from "./ComparisonEditor";
import { AgentChanges } from "./AgentChanges";
export interface PrivatePredictionState {
  document: string; source: string; source_hash: string; current: boolean;
  question: { requirement_reference: string }; target: unknown; environment_scope: string;
  actual_values: "withheld"; application_integration: "incomplete"; runtime_revalidation?: boolean;
}
export function validatePrivatePredictionState(value: unknown): asserts value is PrivatePredictionState {
  const state = value as Partial<PrivatePredictionState> | null;
  if (!state || typeof state !== "object" || typeof state.document !== "string" || !state.document
    || typeof state.source !== "string" || typeof state.source_hash !== "string" || !/^[a-f0-9]{64}$/.test(state.source_hash)
    || (state.runtime_revalidation !== undefined && typeof state.runtime_revalidation !== "boolean")
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
  const [prompt, setPrompt] = useState("Investigate the host requirement using captured source. Seal your prediction and rationale before Verify. If it fails, explain and propose a correction for human review, then reseal the unchanged expectation and verify. Keep held-out target answers withheld.");
  const [provider, setProvider] = useState(""), [model, setModel] = useState("");
  const [promotion, setPromotion] = useState<{ source_hash: string; checked_at: string; scope: string; summary: unknown } | null>(null);
  const [pendingForms, setPendingForms] = useState(0);
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
    owner.current = true;
    const poll = () => void refresh().catch(reason => { if (owner.current) setSyncError(String(reason)); });
    poll(); const timer = setInterval(poll, 1000);
    return () => { owner.current = false; sequence.current++; clearInterval(timer); };
  }, [refresh]);
  const running = agent?.run?.status === "running";
  const attempts = evidence?.history.attempts ?? [];
  const latestAttempt = attempts.at(-1);
  const latestSeal = evidence?.history.ordering.find(entry => entry.event.id === latestAttempt?.seal)?.event.seal as { identity?: { source?: string } } | undefined;
  const canPromote = !!state.runtime_revalidation && latestSeal?.identity?.source === state.source_hash && !!latestAttempt && ["first_attempt_verified", "informed_prediction_verified", "verified_after_correction", "verified_after_prior_attempt"].includes(String(latestAttempt.classification));
  useEffect(() => { if (promotion && promotion.source_hash !== state.source_hash) setPromotion(null); }, [state.source_hash, promotion]);
  const repaired = attempts.some(attempt => attempt.classification === "verified_after_correction");
  const failed = attempts.some(attempt => ["mismatch", "requirement_failed", "execution_rejected"].includes(String(attempt.classification)));
  const pending = agent?.edits?.changes.some(change => change.status === "pending");
  const stage = repaired ? "Compare retained evidence" : pending ? "Review the proposed correction" : failed ? "Delegate the correction" : "Delegate prediction and investigation";
  const target = repaired ? "#private-evidence" : pending ? "#private-review" : "#private-delegate";
  useUiReading("private prediction session status", { state, running, stage, request: { prompt, provider, model }, attempts: attempts.length, error, syncError, promotion_at_request: promotion, current_certification: false });
  const revision = useUiRevision(() => [state, agent?.run, prompt, provider, model, busy, open, pendingForms]);
  const promptRevision = useUiRevision(() => [initial.document, prompt, busy, running]);
  const providerRevision = useUiRevision(() => [initial.document, provider, busy, running]);
  const modelRevision = useUiRevision(() => [initial.document, model, busy, running]);
  const invoke = useUiCommands([
    ...([
      ["prompt", "Edit private investigation request", 32768, promptRevision, setPrompt],
      ["provider", "Choose private investigation provider", 128, providerRevision, setProvider],
      ["model", "Choose private investigation model", 1024, modelRevision, setModel],
    ] as const).map(([field, label, maxLength, fieldRevision, setValue]) => ({ key: `private:${field}`, command: {
      label, parent: "private-prediction", subject: `${initial.document}:${field}`,
      parameters: { value: { type: "string" as const, maxLength } }, prerequisites: [],
      read: () => ({ revision: fieldRevision(), enabled: !busy && !running }),
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
    { key: "private:coach", command: { label: "Set private coach visibility", parent: "private-prediction", subject: initial.document, parameters: { open: { type: "boolean" } }, prerequisites: [], read: () => ({ revision: revision(), enabled: true }), execute: async args => { const visible = Boolean(args.open); setOpen(visible); localStorage.setItem(key, visible ? "open" : "dismissed"); return { status: "completed", value: {} }; } } },
  ]);
  const setRequest = (field: "prompt" | "provider" | "model", value: string) => {
    setPendingForms(count => count + 1);
    void invoke(`private:${field}`, { value }).catch(reason => { if (owner.current) setError(String(reason)); })
      .finally(() => { if (owner.current) setPendingForms(count => count - 1); });
  };
  return <main style={{ maxWidth: 1100, margin: "auto", padding: 24 }}>
    <h1>Predict, investigate and review</h1>
    <p>{state.question.requirement_reference}</p>
    <p>Target answers and runtime diagnostics stay withheld. Current certification is unavailable until host revalidation and promotion. {state.environment_scope}</p>
    {error && <p role="alert">{error}</p>}{syncError && <p role="alert">{syncError}</p>}
    <section id="private-delegate"><h2>Delegate to the agent</h2>
      <label>Investigation request<textarea maxLength={32768} value={prompt} disabled={busy || running} onChange={event => setRequest("prompt", event.target.value)} /></label>
      <label>Provider<input maxLength={128} value={provider} disabled={busy || running} onChange={event => setRequest("provider", event.target.value)} placeholder="Configured default" /></label>
      <label>Model<input maxLength={1024} value={model} disabled={busy || running} onChange={event => setRequest("model", event.target.value)} placeholder="Provider default" /></label>
      <button disabled={busy || !!pendingForms || running || !prompt.trim() || state.current !== true} onClick={() => void invoke("private:start")}>Delegate investigation</button>
      <button disabled={!running} onClick={() => void invoke("private:stop")}>Stop investigation</button>
      <button onClick={() => void invoke("private:coach", { open: true })}>Resume coach</button>
      <p role="status">{agent?.run?.status ?? "Ready for a request"}</p><p>{agent?.run?.answer}</p>
    </section>
    <section><h2>Captured source</h2><p>SHA-256: {state.source_hash}</p><CodeEditor path={state.source} item={{ id: initial.document, label: "Captured private source", kind: "authored", text: source }} editable={false} onChange={() => {}} /></section>
    <section id="private-review"><AgentChanges doc={`private:${initial.document}`} state={agent} running={!!running} update={value => setAgent(value as Agent)} submitDecision={async body => {
      const actual = await request<Agent>("POST", `${base}/agent/decision`, { id: body.id, accepted: body.accepted, ...(body.new_text !== undefined ? { new_text: body.new_text } : {}) });
      await refresh(); return actual;
    }} /></section>
    <section id="private-evidence"><button disabled={!canPromote || busy || running || state.current !== true} onClick={() => void invoke("private:promote")}>Revalidate and promote</button>
      {promotion && <p role="status">Host revalidation at {promotion.checked_at} for source {promotion.source_hash}. {promotion.scope}</p>}{evidence ? <PredictionEvidence value={evidence} sourceHash={state.source_hash} /> : <p>No retained execution attempt yet. The agent must seal a prediction before execution.</p>}</section>
    {open && <AnchoredCoach target={target} title={stage} onDismiss={() => void invoke("private:coach", { open: false })}>
      <p>{pending ? "Review the exact proposed source diff. Accept or reject it yourself; the agent cannot approve its own correction." : repaired ? "The repair passed the unchanged prediction. The original failure stays in history. Use Revalidate and promote when the host supplies a supported runtime and fixed input binding; the button remains unavailable otherwise." : failed ? "The comparison failed. Ask the agent to reason from the requirement and captured source, propose a reviewed repair, then repeat the unchanged prediction." : "Delegate the investigation. The agent records its own prediction before the existing runner and debugger compare it privately."}</p>
      <p>Future-stop debugger coaching and full application integration remain in progress.</p>
    </AnchoredCoach>}
  </main>;
}
