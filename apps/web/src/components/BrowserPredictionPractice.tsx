import "./privatePredictionCoach.css";
import { useEffect, useRef, useState } from "react";
import { useUiCommands, useUiRevision } from "../lib/uiActionBridge";
import { useUiReading } from "../lib/uiReading";
import type { UiCommand, UiOutcome } from "../lib/uiActions";
import { browserPredictionHistory, browserPredictionSource, inspectBrowserPrediction, parseBrowserInputs, promoteBrowserPrediction, recoverBrowserPrediction, runBrowserPrediction, savedBrowserPredictionInputs, sealBrowserPrediction, type BrowserSeal, type BrowserSummary } from "../tour/browserPrediction";

function PracticeSource({ source }: { source: string }) { useUiReading("browser prediction source", { source, edit_baseline: false }, undefined, "browser-prediction"); return <pre style={{ maxWidth: "100%", overflow: "auto" }}>{source}</pre>; }

/** Optional branch of the existing coach; no ordinary runtime/output reader mounts. */
export function BrowserPredictionPractice({ corrected, onResult, onInvalidate }: { corrected: boolean; onResult: (result: { corrected: boolean; requirement: boolean; matched: boolean; checks: boolean; original_unchanged: boolean; prior_failure: boolean }) => void; onInvalidate: () => void }) {
  const [initialInputs] = useState(savedBrowserPredictionInputs);
  const [attempts, setAttempts] = useState(() => JSON.stringify(initialInputs.attempts)), [budget, setBudget] = useState(() => String(initialInputs.budget));
  const [prediction, setPrediction] = useState(() => String(initialInputs.budget)), [rationale, setRationale] = useState("");
  useEffect(() => { try { localStorage.setItem("hickory.browserPredictionInputs.v1", JSON.stringify(parseBrowserInputs(attempts, budget))); } catch { /* Draft cache failure cannot admit execution or reset the private journal. */ } }, [attempts, budget]);
  const [sourceOpen, setSourceOpen] = useState(false);
  let source: string | null = null;
  try { source = browserPredictionSource(corrected, parseBrowserInputs(attempts, budget)); } catch { /* Invalid draft stays non-executable. */ }
  const [sealed, setSealed] = useState<BrowserSeal | null>(null), [history, setHistory] = useState<BrowserSummary[]>([]);
  const [busy, setBusy] = useState(false), [error, setError] = useState("");
  const [recovery, setRecovery] = useState<{ interrupted: number } | null>(null);
  const mounted = useRef(false), working = useRef(false), abort = useRef<AbortController | null>(null);
  const signature = JSON.stringify([corrected, attempts, budget, prediction, rationale]);
  const live = useRef(signature), epoch = useRef(0);
  if (live.current !== signature) { live.current = signature; epoch.current++; }
  const bound = useRef<{ signature: string; seal: BrowserSeal } | null>(null);
  const resultCallback = useRef(onResult); resultCallback.current = onResult;
  useEffect(() => { mounted.current = true; return () => { mounted.current = false; abort.current?.abort(); }; }, []);
  useEffect(() => { if (bound.current?.signature !== signature) { bound.current = null; setSealed(null); setHistory([]); setRecovery(null); abort.current?.abort(); } }, [signature]);
  const revision = useUiRevision(() => [signature, sealed?.receipt, history, busy, error, sourceOpen, recovery]);
  const current = () => !!sealed && bound.current?.seal === sealed && bound.current.signature === signature;
  const perform = async (effect: () => Promise<UiOutcome>): Promise<UiOutcome> => {
    if (working.current) return { status: "unavailable", reason: "A browser prediction operation is pending" };
    working.current = true; setBusy(true); setError("");
    try { return await effect(); }
    catch (cause) { const reason = cause instanceof Error ? cause.message : "Browser request unavailable"; if (mounted.current) setError(reason); return { status: "unavailable", reason }; }
    finally { working.current = false; if (mounted.current) setBusy(false); }
  };
  const parent = "browser-prediction";
  const attemptsRevision = useUiRevision(() => [corrected, attempts, busy]), budgetRevision = useUiRevision(() => [corrected, budget, busy]);
  const predictionRevision = useUiRevision(() => [corrected, prediction, busy]), rationaleRevision = useUiRevision(() => [corrected, rationale, busy]);
  const fields = [["attempts", "Edit browser prediction attempt durations", setAttempts, attemptsRevision], ["budget", "Edit browser prediction total budget", setBudget, budgetRevision], ["prediction", "Edit browser prediction expected duration", setPrediction, predictionRevision], ["rationale", "Edit browser prediction rationale", setRationale, rationaleRevision]] as const;
  const editField = (key: string, value: string): UiOutcome => {
    if (working.current) return { status: "unavailable", reason: "Wait for the pending operation" };
    const field = fields.find(field => field[0] === key);
    if (!field || [...value].length > (key === "rationale" ? 8192 : 1024)) return { status: "unavailable", reason: "Draft exceeds this field's supported bound" };
    bound.current = null; setSealed(null); setHistory([]); setRecovery(null); setError(""); onInvalidate(); field[2](value);
    return { status: "completed", value: { draft_changed: true, current_certification: false } };
  };
  const command = (key: string, label: string, enabled: boolean, execute: UiCommand["execute"], parameters: UiCommand["parameters"] = {}) => ({ key: `${parent}:${key}`, command: { parent, subject: parent, label, parameters, prerequisites: ["Browser-local exercise; target output withheld; user-controlled storage and incomplete exposure/environment knowledge cannot certify promotion"], read: () => ({ revision: revision(), enabled }), execute } });
  const sourceCommand = command("source", "Set browser prediction source disclosure", source !== null, async args => { setSourceOpen(Boolean(args.open)); return { status: "completed", value: { source_disclosed: Boolean(args.open), values: "withheld" } }; }, { open: { type: "boolean" } });
  sourceCommand.command.subject = `${parent}:source-disclosure`;
  const invoke = useUiCommands([
    sourceCommand,
    ...fields.map(([key, label, _write, readRevision]) => {
      const entry = command(key, label, !busy, async args => editField(key, String(args.value)), { value: { type: "string", maxLength: key === "rationale" ? 8192 : 1024 } });
      entry.command.subject = `${parent}:field:${key}`;
      entry.command.read = () => ({ revision: readRevision(), enabled: !busy });
      return entry;
    }),
    command("seal", "Seal browser prediction before execution", !busy, (_args, actor) => perform(async () => {
      const captured = signature, capturedEpoch = epoch.current, inputs = parseBrowserInputs(attempts, budget);
      const owns = () => mounted.current && live.current === captured && epoch.current === capturedEpoch;
      const seal = await sealBrowserPrediction(corrected, inputs, prediction, rationale, actor, owns);
      if (!owns()) return { status: "unavailable", reason: "Practice changed; the retained seal does not authorize the current practice" };
      bound.current = { signature: captured, seal }; setSealed(seal);
      const summaries = await browserPredictionHistory(seal, actor);
      if (!owns()) return { status: "unavailable", reason: "Practice changed; inspect the retained historical seal" };
      setHistory(summaries);
      return { status: "completed", value: { seal: seal.receipt.id, source_hash: seal.identity.source, inputs_hash: seal.identity.inputs, current_certification: false } };
    })),
    command("run", "Run sealed browser prediction", !busy && current(), (_args, actor, signal) => perform(async () => {
      if (!current() || !sealed) return { status: "unavailable", reason: "Seal the current practice first" };
      const captured = signature, capturedEpoch = epoch.current, seal = sealed, inputs = parseBrowserInputs(attempts, budget), cancel = new AbortController(); abort.current = cancel;
      const cancelCaller = () => cancel.abort(); if (signal.aborted) cancel.abort(); signal.addEventListener("abort", cancelCaller, { once: true });
      try {
        const summary = await runBrowserPrediction(seal, inputs, actor, cancel.signal, () => mounted.current && live.current === captured && epoch.current === capturedEpoch && bound.current?.seal === seal);
        if (!mounted.current || live.current !== captured || epoch.current !== capturedEpoch || bound.current?.seal !== seal) return { status: "unavailable", reason: "Practice changed; inspect its historical result" };
        const summaries = await browserPredictionHistory(seal, actor);
        if (!mounted.current || live.current !== captured || epoch.current !== capturedEpoch || bound.current?.seal !== seal) return { status: "unavailable", reason: "Practice changed; inspect historical results" };
        setHistory(summaries);
        resultCallback.current({ corrected, requirement: summary.original_requirement_satisfied === true, matched: summary.prediction_matched === true, checks: summary.independent_checks_accepted === true, original_unchanged: summary.original_expectation_unchanged === true, prior_failure: summaries.some(attempt => attempt.original_requirement_satisfied === false) });
        return { status: "completed", value: { attempt: summary.attempt, seal: seal.receipt.id, source_hash: seal.identity.source, inputs_hash: seal.identity.inputs, classification: summary.classification, prediction_matched: summary.prediction_matched, requirement_satisfied: summary.original_requirement_satisfied, current_certification: false, values: "withheld", independent_checks_accepted: summary.independent_checks_accepted === true, original_expectation_unchanged: summary.original_expectation_unchanged === true } };
      } finally { signal.removeEventListener("abort", cancelCaller); if (abort.current === cancel) abort.current = null; }
    })),
    command("history", "Read browser prediction history summaries", !busy && current(), (args, actor) => perform(async () => {
      if (!sealed || !current()) return { status: "unavailable", reason: "Seal the current practice to select its history" };
      const capturedEpoch = epoch.current, summaries = await browserPredictionHistory(sealed, actor);
      if (!mounted.current || capturedEpoch !== epoch.current || !current()) return { status: "unavailable", reason: "Practice changed; inspect its current history" };
      setHistory(summaries);
      const offset = Number(args.offset), limit = Number(args.limit);
      if (offset > summaries.length) return { status: "unavailable", reason: "Page outside the current history" };
      return { status: "completed", value: { attempts: summaries.slice(offset, offset + limit).map(value => ({ ...value })), offset, total: summaries.length, next_offset: offset + limit < summaries.length ? offset + limit : null, values: "withheld", current_certification: false } };
    }), { offset: { type: "number", integer: true, minimum: 0, maximum: 1000000 }, limit: { type: "number", integer: true, minimum: 1, maximum: 16 } }),
    command("recover", "Recover interrupted browser prediction attempts", !busy && current(), (_args, actor) => perform(async () => {
      if (!sealed || !current()) return { status: "unavailable", reason: "Select the current scenario with a seal before recovery" };
      const capturedEpoch = epoch.current, selected = sealed, recovered = await recoverBrowserPrediction(selected, actor);
      const summaries = await browserPredictionHistory(selected, actor);
      if (!mounted.current || capturedEpoch !== epoch.current || bound.current?.seal !== selected) return { status: "unavailable", reason: "Recovery settled historical attempts; inspect the current scenario" };
      setHistory(summaries); setRecovery({ interrupted: recovered.interrupted });
      return { status: "completed", value: { ...recovered, scenario: selected.scenario, scope: "Exclusive same-origin managed-runtime lease; interrupted/unknown evidence retained, no execution or fresh certification" } };
    })),
    command("promote", "Request browser prediction promotion", !busy && current() && history.length > 0, (_args, actor) => perform(async () => {
      if (!sealed || !current()) return { status: "unavailable", reason: "Select the current seal" };
      await promoteBrowserPrediction(sealed, history.at(-1)!.attempt, actor);
      return { status: "unavailable", reason: "Browser-local evidence cannot grant host certification" };
    })),
    command("inspect", "Request human inspection of browser prediction", !busy && current() && history.length > 0, (_args, actor) => {
      if (actor.kind !== "human") return Promise.resolve({ status: "requires_input", request: "browser-prediction-human-inspection", reason: "A human inspects separately; no target values or report URL are delivered to the predictor" });
      return perform(async () => {
        if (!sealed || !current()) return { status: "unavailable", reason: "Select the current seal" };
        const capturedEpoch = epoch.current;
        const report = window.open("about:blank", "_blank");
        if (!report) return { status: "requires_input", request: "browser-prediction-popup", reason: "Allow a separate human inspection window, then request inspection again" };
        report.opener = null;
        try {
          const selected = await inspectBrowserPrediction(history.at(-1)!.attempt, actor);
          if (!mounted.current || capturedEpoch !== epoch.current || !current()) { report.close(); return { status: "unavailable", reason: "Inspection owner changed; no values were delivered" }; }
          report.document.title = "Human-only browser prediction inspection";
          report.document.body.textContent = "Human-only inspection. Do not copy these held-out actuals into predictor context. Browser-local history is not authenticated host evidence.\n\n" + JSON.stringify(selected, null, 2);
          return { status: "completed", value: { human_exposure_recorded: true, inspection_window_requested: true, display_unverified: true } };
        } catch { report.close(); return { status: "unavailable", reason: "Inspection could not be retained; no values were delivered" }; }
      });
    }),
    // Distinct cancellation subject permits stopping an in-flight Run.
    { key: `${parent}:stop`, command: { parent, subject: `${parent}:stop`, label: "Stop browser prediction execution", parameters: {}, prerequisites: [], read: () => ({ revision: revision(), enabled: busy && !!abort.current }), execute: async () => { abort.current?.abort(); return { status: "completed", value: { cancellation_requested: true, settlement: "Inspect retained history after the worker stops" } }; } } },
  ]);
  // Human keystrokes change only a reversible draft, synchronously. The
  // registered agent command calls this same controller; durable seal/admission
  // remains the boundary before execution, not per-keystroke UI receipts.
  const field = (key: string, value: string) => { const result = editField(key, value); if (result.status === "unavailable") setError(result.reason); };
  const click = (key: string) => { void invoke(`${parent}:${key}`).catch(() => {}); };
  // Answer-free metadata only. Raw portable journals/runtime values are never readings.
  useUiReading("browser prediction practice", { attempts, budget, prediction, rationale, variant: corrected ? "recorded correction" : "recorded faulty proposal", sealed: current(), source_hash: sealed?.identity.source ?? null, history, error, recovery, operation_pending: busy, actual_values: "withheld", current_certification: false, scope: "Browser-local exercise; missing trustworthy prior-exposure/environment identity remains Unknown" }, undefined, "browser-prediction");
  return <section aria-label="Browser prediction practice">
    <h3>Predict with your own inputs</h3>
    <p>The implementation is a recorded example; execution is live. Seal before Run. Valid inputs restore after reload; re-enter your original prediction/rationale. Recovery settles interrupted attempts without replay. Answers stay withheld; human inspection opens separately. Browser-local storage and incomplete prior-exposure/environment knowledge remain Unknown and cannot earn host promotion.</p>
    <div className="private-prediction-fields">
      <label>Attempt durations (JSON, ms)<input maxLength={1024} disabled={busy} value={attempts} onChange={event => field("attempts", event.target.value)} /></label>
      <label>Total budget (ms)<input maxLength={1024} disabled={busy} value={budget} onChange={event => field("budget", event.target.value)} /></label>
      <label>Predicted elapsed duration (ms)<input maxLength={1024} disabled={busy} value={prediction} onChange={event => field("prediction", event.target.value)} /></label>
      <label>Prediction rationale<textarea disabled={busy} maxLength={8192} value={rationale} onChange={event => field("rationale", event.target.value)} /></label>
    </div>
    <button onClick={() => { void invoke(`${parent}:source`, { open: !sourceOpen }).catch(() => {}); }}>{sourceOpen ? "Hide prediction source" : "Inspect prediction source"}</button>
    {sourceOpen && source !== null && <PracticeSource source={source} />}
    <p>Use up to eight whole-millisecond durations from 0 to 1000ms and a whole-millisecond budget from 1 to 1000ms. Short-attempt and empty-sequence checks run independently.</p>
    <div className="private-prediction-controls">
      <button disabled={busy} onClick={() => click("seal")}>Seal browser prediction</button>
      <button disabled={busy || !current()} onClick={() => click("run")}>Run sealed prediction</button>
      <button disabled={!busy || !abort.current} onClick={() => click("stop")}>Stop prediction run</button>
      <button disabled={busy || !current()} onClick={() => click("recover")}>Recover interrupted attempts</button>
      <button disabled={busy || !current() || !history.length} onClick={() => click("promote")}>Request promotion</button>
      <button disabled={busy || !current() || !history.length} onClick={() => click("inspect")}>Inspect actuals as a human</button>
    </div>
    {error && <p role="alert">{error}</p>}
    {recovery && <p role="status">{recovery.interrupted} interrupted attempts settled. No execution replayed; evidence remains non-passing.</p>}
    <p role="status">{busy ? "Request pending" : current() ? "Current source/input prediction sealed" : "Seal the current inputs before execution"}. Target values withheld; host certification unavailable.</p>
    <ol aria-label="Retained browser prediction attempts">{history.map(attempt => <li key={attempt.attempt}>{attempt.classification} · prediction {String(attempt.prediction_matched)} · original requirement {String(attempt.original_requirement_satisfied)} · original expectation unchanged {String(attempt.original_expectation_unchanged)} · current certification {String(attempt.current_certification)}</li>)}</ol>
  </section>;
}
