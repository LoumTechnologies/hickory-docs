<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Browser-local prediction practice uses the shared gate

For: Reviewers deciding what an output, recording or prediction actually establishes.

The optional browser branch of the existing workflow coach uses
`hick-observe::prediction::Ledger` compiled into the shipped parser/ledger WASM.
It does not implement a second comparison or promotion engine. The portable
journal uses the same validation, seal, admission, original requirement,
classification, interruption, actor-scoped history and promotion methods as the
native private ledger. Native private-file ledgers cannot export raw portable
snapshots through this API.

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Read the detailed requirements

- [Requirements 1](requirements/01.md)


Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/verification/browser-predictions-use-the-shared-gate.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.


## Read the retained requirement detail

- [Requirement detail](requirements/01.md)
<storycode:include file="requirements/01.md" />

## Implementation

<a id="output-49c7b898ffd6"></a>
## `apps/web/src/components/BrowserPredictionPractice.tsx`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [Browser-local prediction practice uses the shared gate](contract.md).

<a id="f-49c7b898ffd6-PracticeSource"></a>
### `PracticeSource`

<storycode:copy id="f-49c7b898ffd6-PracticeSource">
import "./privatePredictionCoach.css";
import { useEffect, useRef, useState } from "react";
import { useUiCommands, useUiRevision } from "../lib/uiActionBridge";
import { useUiReading } from "../lib/uiReading";
import type { UiCommand, UiOutcome } from "../lib/uiActions";
import { browserPredictionHistory, browserPredictionSource, inspectBrowserPrediction, parseBrowserInputs, promoteBrowserPrediction, recoverBrowserPrediction, runBrowserPrediction, savedBrowserPredictionInputs, sealBrowserPrediction, type BrowserSeal, type BrowserSummary } from "../tour/browserPrediction";

function PracticeSource({ source }: { source: string }) { useUiReading("browser prediction source", { source, edit_baseline: false }, undefined, "browser-prediction"); return <pre style={{ maxWidth: "100%", overflow: "auto" }}>{source}</pre>; }

</storycode:copy>
<a id="f-49c7b898ffd6-BrowserPredictionPractice"></a>
### `BrowserPredictionPractice`

Optional branch of the existing coach; no ordinary runtime/output reader mounts.

<storycode:copy id="f-49c7b898ffd6-BrowserPredictionPractice">
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
</storycode:copy>

### Assemble `apps/web/src/components/BrowserPredictionPractice.tsx`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/components/BrowserPredictionPractice.tsx"><storycode:paste select="#f-49c7b898ffd6-PracticeSource"/><storycode:paste select="#f-49c7b898ffd6-BrowserPredictionPractice"/></storycode:file>

<a id="output-39bbc44cf019"></a>
## `apps/web/src/components/WorkflowTour.tsx`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [The engineer tour distinguishes replay from execution](../../authoring/the-engineer-tour-distinguishes-replay-from-execution/contract.md), [Browser-local prediction practice uses the shared gate](contract.md).

<a id="f-39bbc44cf019-NativeWorkflowTour"></a>
### `NativeWorkflowTour`

<storycode:copy id="f-39bbc44cf019-NativeWorkflowTour">
import { BrowserPredictionPractice } from "./BrowserPredictionPractice";
import { UiLink } from "./UiLink";
import { lazy, Suspense, useContext, useRef, useLayoutEffect } from "react";
const NativeWorkflowTour = lazy(() => import("./NativeWorkflowTour").then(m => ({ default: m.NativeWorkflowTour })));
</storycode:copy>
<a id="f-39bbc44cf019-import_statement"></a>
### `import_statement`

<storycode:copy id="f-39bbc44cf019-import_statement">
import type { EditorView } from "@codemirror/view";
</storycode:copy>
<a id="f-39bbc44cf019-import_statement-2"></a>
### `import_statement`

<storycode:copy id="f-39bbc44cf019-import_statement-2">
import { emit } from "../analytics/events";
</storycode:copy>
<a id="f-39bbc44cf019-import_statement-3"></a>
### `import_statement`

<storycode:copy id="f-39bbc44cf019-import_statement-3">
import { useEffect, useState } from "react";
</storycode:copy>
<a id="f-39bbc44cf019-import_statement-4"></a>
### `import_statement`

<storycode:copy id="f-39bbc44cf019-import_statement-4">
import { UiActionContext, useUiCommands, useUiRevision } from "../lib/uiActionBridge";
</storycode:copy>
<a id="f-39bbc44cf019-import_statement-5"></a>
### `import_statement`

<storycode:copy id="f-39bbc44cf019-import_statement-5">
import { useUiReading } from "../lib/uiReading";
</storycode:copy>
<a id="f-39bbc44cf019-import_statement-6"></a>
### `import_statement`

<storycode:copy id="f-39bbc44cf019-import_statement-6">
import type { UiCommand, UiValue } from "../lib/uiActions";
</storycode:copy>
<a id="f-39bbc44cf019-import_statement-7"></a>
### `import_statement`

<storycode:copy id="f-39bbc44cf019-import_statement-7">
import { DebuggableDocument } from "../embed/DebuggableDocument";
</storycode:copy>
<a id="f-39bbc44cf019-import_statement-8"></a>
### `import_statement`

<storycode:copy id="f-39bbc44cf019-import_statement-8">
import { scenarioSource, TOUR_STEPS } from "../tour/scenario";
</storycode:copy>
<a id="f-39bbc44cf019-import_statement-9"></a>
### `import_statement`

<storycode:copy id="f-39bbc44cf019-import_statement-9">
import "./WorkflowTour.css";
</storycode:copy>
<a id="f-39bbc44cf019-KEY"></a>
### `KEY`

<storycode:copy id="f-39bbc44cf019-KEY">
const KEY = "hickory.aiEngineerTour";
</storycode:copy>
<a id="f-39bbc44cf019-WorkflowTour"></a>
### `WorkflowTour`

<storycode:copy id="f-39bbc44cf019-WorkflowTour">
export function WorkflowTour(props: { surface: "site" | "app"; predictionOnly?: boolean; onPredictionPractice?: () => void; onOpen?: (path: string, line?: number) => void; focused?: { path?: string; source?: string; view?: EditorView | null } }) {
  return props.surface === "app" ? <Suspense fallback={<p>Loading native workflow…</p>}><NativeWorkflowTour onOpen={props.onOpen} focused={props.focused} /></Suspense> : <BrowserWorkflowTour surface="site" predictionOnly={props.predictionOnly} onPredictionPractice={props.onPredictionPractice} />;
}
</storycode:copy>
<a id="f-39bbc44cf019-BrowserWorkflowTour"></a>
### `BrowserWorkflowTour`

<storycode:copy id="f-39bbc44cf019-BrowserWorkflowTour">
function BrowserWorkflowTour({ surface, predictionOnly = false, onPredictionPractice }: { surface: "site" | "app"; predictionOnly?: boolean; onPredictionPractice?: () => void }) {
  const [open, setOpen] = useState(predictionOnly);
  const [step, setStep] = useState(() => { if (predictionOnly) return 3; const saved = Number(localStorage.getItem(KEY)); return Number.isInteger(saved) && saved >= 0 && saved < TOUR_STEPS.length ? saved : 0; });
  const [view, setView] = useState<EditorView | null>(null);
  const [objection, setObjection] = useState("");
  const [failed, setFailed] = useState(false);
  const [passed, setPassed] = useState(false);
  const [practice, setPractice] = useState(predictionOnly);
  const registry = useContext(UiActionContext), restricted = useRef<typeof registry>(null);
  const invalidatePractice = () => { setFailed(false); setPassed(false); };
  const confinePractice = () => {
    if (registry && restricted.current !== registry) {
      registry.confine((key, command) => command.parent === "browser-prediction" || ["browser-tour:visibility", "browser-tour:back", "browser-tour:next", "browser-tour:restart", "browser-tour:close", "browser-tour:objection", "browser-tour:prediction-practice", "browser-tour:download", "browser-tour:publication"].includes(key));
      restricted.current = registry;
    }
  };
  useLayoutEffect(() => { if (practice || restricted.current) confinePractice(); }, [registry, practice]);
  const showPractice = (value: boolean) => { if (value) { confinePractice(); if (onPredictionPractice) { onPredictionPractice(); return; } } setPractice(predictionOnly || value); invalidatePractice(); };
  useEffect(() => { localStorage.setItem(KEY, String(step)); }, [step]);
  useEffect(() => { if (open && surface === "site") emit({ name: "demo_engaged", demo_id: "ai-engineer-tour", step: String(step) }); }, [open, step, surface]);
  const corrected = step >= 4;
  const source = scenarioSource(corrected);
  useEffect(() => {
    if (!view || practice) return;
    const needle = step >= 3 ? "function elapsed" : step === 1 ? "The deadline bounds" : "Meeting evidence";
    const anchor = view.state.doc.toString().indexOf(needle);
    if (anchor >= 0) view.dispatch({ selection: { anchor }, scrollIntoView: true });
  }, [view, step, practice]);
  const canContinue = step === 2 ? !!objection.trim() : step === 3 ? failed : step === 4 ? passed : true;
  const restart = () => { setStep(predictionOnly ? 3 : 0); setFailed(false); setPassed(false); setObjection(""); setPractice(predictionOnly); };
  const download = () => {
    const url = URL.createObjectURL(new Blob([scenarioSource(true)], { type: "text/markdown" }));
    const link = document.createElement("a"); link.href = url; link.download = "deadline-spec.md"; link.click(); setTimeout(() => URL.revokeObjectURL(url), 1000);
  };
  const revision = useUiRevision(() => [open, step, objection, failed, passed, practice]);
  useUiReading("browser workflow tour", open ? { open, step, title: TOUR_STEPS[step].title, text: TOUR_STEPS[step].text, objection, can_continue: canContinue,
    counterexample_observed: failed, correction_observed: passed, browser_prediction_practice: practice, current_certification: false, proposal: "recorded example", execution: "live browser", human_attestation: false, agent_scope: restricted.current ? "Prediction-only managed interface for this window lifetime; raw reports and ordinary commands unavailable; prior context/environment knowledge Unknown" : "Ordinary browser example" } : { open, current_certification: false, agent_scope: restricted.current || predictionOnly ? "Prediction-only managed interface remains confined for this window lifetime" : "Ordinary browser example" }, undefined, restricted.current || predictionOnly ? "browser-prediction" : undefined);
  const command = (key: string, label: string, enabled: boolean, effect: (args: Record<string, UiValue>) => void, parameters: UiCommand["parameters"] = {}) => ({ key: `browser-tour:${key}`, command: {
    label, parent: "browser-tour", subject: "browser-tour", parameters, prerequisites: [], read: () => ({ revision: revision(), enabled }),
    execute: async (args: Record<string, UiValue>) => { effect(args); return { status: "completed" as const, value: { controller_update_requested: true, human_attestation: false, execution_certified: false } }; },
  } });
  const invoke = useUiCommands([
    command("visibility", "Set browser workflow tour visibility", true, args => setOpen(args.open as boolean), { open: { type: "boolean" } }),
    ...(open ? [{ key: "browser-tour:prediction-practice", command: { label: "Set browser prediction practice visibility", parent: "browser-tour", subject: "browser-tour", parameters: { open: { type: "boolean" as const } }, prerequisites: ["Entering permanently confines this managed agent window; hiding cannot reopen ordinary capabilities"], read: () => ({ revision: revision(), enabled: true }), execute: async (args: Record<string, UiValue>) => {
      if (predictionOnly && !args.open) return { status: "requires_input" as const, request: "browser-prediction-fresh-window", reason: "Use a new page and fresh provider context for ordinary work; this prediction window cannot release its own confinement" };
      showPractice(Boolean(args.open)); return { status: "completed" as const, value: { prediction_requested: true, ordinary_capabilities_reopened: false } };
    } } }, command("back", "Previous browser workflow step", step > 0, () => setStep(step - 1)),
      command("next", "Continue browser workflow step", canContinue && step < TOUR_STEPS.length - 1, () => setStep(step + 1)),
      command("restart", "Restart browser workflow tour", true, restart), command("close", "Skip and close browser workflow tour", true, () => setOpen(false)),
      command("download", practice ? "Download default browser workflow executable spec" : "Download browser workflow executable spec", true, download),
      ...(predictionOnly && step === 5 ? [{ key: "browser-tour:publication", command: { label: "Request separately selected public prediction evidence", parent: "browser-tour", subject: "browser-tour", parameters: {}, prerequisites: [], read: () => ({ revision: revision(), enabled: true }), execute: async () => ({ status: "requires_input" as const, request: "browser-prediction-human-publication", reason: "A human selects matching public evidence in a separate context. No raw report, answer or publication URL is delivered to this predictor" }) } }] : []),
      ...(step === 2 ? [command("objection", "Set browser workflow practice objection", true, args => setObjection(args.text as string), { text: { type: "string", maxLength: 65536 } })] : [])] : []),
  ]);
  const invokePublication = () => invoke("browser-tour:publication").catch(() => {});
  return <div className="workflow-tour-launcher">
    {(restricted.current || predictionOnly) && <p>This page stays in prediction practice, even if you close the coach. For ordinary agent work, open a new page with fresh context. History remains retained.</p>}
    <button className="btn" onClick={() => setOpen(!open)}>{open ? "Close AI engineer tour" : "AI engineer tour"}</button>
    {open && <section className={`workflow-tour${surface === "app" ? " workflow-tour--app" : ""}`} aria-label="AI engineer tour">
      <p>Step {step + 1} of {TOUR_STEPS.length} · {surface === "site" ? "Browser tour" : "Isolated practice document"}</p>
      <h2>{TOUR_STEPS[step].title}</h2><p>{TOUR_STEPS[step].text}</p>
      <p className="muted">The agent proposal and correction are recorded examples. Code execution below is live in your browser. Your project is not modified.</p>
      {step === 2 && <label>Your objection<textarea value={objection} onChange={event => setObjection(event.target.value)} /></label>}
      {step >= 4 && <pre aria-label="Correction diff">- var allowed = budget;{"\n"}+ var allowed = Math.max(0, budget - total);</pre>}
      <div className="workflow-tour-controls">
        <button disabled={step === 0} onClick={() => setStep(step - 1)}>Back</button>
        {step < TOUR_STEPS.length - 1 && <button disabled={!canContinue} onClick={() => setStep(step + 1)}>Continue</button>}
        <button onClick={restart}>Restart tour</button>
        <button onClick={() => setOpen(false)}>Skip / close</button>
        <button onClick={download}>{practice ? "Download default executable spec" : "Download executable spec"}</button>
      </div>
      {!predictionOnly && <label><input type="checkbox" checked={practice} onChange={event => showPractice(event.target.checked)} />Try input-changing prediction practice (browser-local, no API key)</label>}
      {practice ? <BrowserPredictionPractice corrected={corrected} onInvalidate={invalidatePractice} onResult={result => { if (!result.corrected && !result.requirement) setFailed(true); if (result.corrected && result.requirement && result.matched && result.checks && result.original_unchanged && result.prior_failure) setPassed(true); }} /> : <DebuggableDocument key={corrected ? "corrected" : "original"} path="deadline-spec.md" source={source} revision={corrected ? "corrected-example" : "original-example"} readOnly onViewReady={setView}
        onExecutionResult={result => { if (result.revision === "original-example" && result.exitCode !== 0) setFailed(true); if (result.revision === "corrected-example" && result.exitCode === 0) setPassed(true); }} />}
      {step === 3 && !failed && <p role="status">Run the original code to observe the counterexample.</p>}
      {step === 4 && !passed && <p role="status">Run the corrected code to verify the checks.</p>}
      {step === 5 && !predictionOnly && <p>Complete native workflow: import a transcript, challenge the real claim, explicitly update stale agent context, send a real investigation, inspect source-linked runtime/debug evidence, profile Python and rerun correctness, then resolve the thread yourself. The desktop tour creates a separate practice Git repository. These native actions and actual model calls are not simulated by this homepage. <UiLink href="./snapshot.html?index=publications/index.json">Inspect published verification</UiLink>.</p>}
      {predictionOnly && step === 5 && <p>These browser comparisons do not supply a real provider recording, human approval or host certification. Use a fresh native private practice for the supported authoritative flow. <button onClick={() => { void invokePublication(); }}>Request separately selected evidence</button></p>}
      {step === 5 && <p>Local verification: <code>hick weave deadline-spec.md</code>, then <code>hick test deadline-spec.md</code> (Node.js required). In the workspace, enable author colors in Lineage settings; use Review threads to retain an objection and its resolution.</p>}
    </section>}
  </div>;
}
</storycode:copy>

### Assemble `apps/web/src/components/WorkflowTour.tsx`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/components/WorkflowTour.tsx"><storycode:paste select="#f-39bbc44cf019-NativeWorkflowTour"/><storycode:paste select="#f-39bbc44cf019-import_statement"/><storycode:paste select="#f-39bbc44cf019-import_statement-2"/><storycode:paste select="#f-39bbc44cf019-import_statement-3"/><storycode:paste select="#f-39bbc44cf019-import_statement-4"/><storycode:paste select="#f-39bbc44cf019-import_statement-5"/><storycode:paste select="#f-39bbc44cf019-import_statement-6"/><storycode:paste select="#f-39bbc44cf019-import_statement-7"/><storycode:paste select="#f-39bbc44cf019-import_statement-8"/><storycode:paste select="#f-39bbc44cf019-import_statement-9"/><storycode:paste select="#f-39bbc44cf019-KEY"/><storycode:paste select="#f-39bbc44cf019-WorkflowTour"/><storycode:paste select="#f-39bbc44cf019-BrowserWorkflowTour"/></storycode:file>

<a id="output-4412ea75a6ea"></a>
## `apps/web/src/editor/generated/hick-lang/hick_lang.d.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [Browser-local prediction practice uses the shared gate](contract.md).

<a id="f-4412ea75a6ea-browser_prediction"></a>
### `browser_prediction`

tslint:disable
eslint-disable
Private browser-host transition. Actor and actual result arguments must be
supplied by trusted adapters, never forwarded from model invocation fields.
Storage is user-controlled, not a native private directory or authenticated
evidence store. Snapshot persistence must succeed before admitted dispatch.

<storycode:copy id="f-4412ea75a6ea-browser_prediction">
/* tslint:disable */
/* eslint-disable */

/**
 * Private browser-host transition. Actor and actual result arguments must be
 * supplied by trusted adapters, never forwarded from model invocation fields.
 * Storage is user-controlled, not a native private directory or authenticated
 * evidence store. Snapshot persistence must succeed before admitted dispatch.
 */
export function browser_prediction(snapshot: string, command: string): string;

</storycode:copy>
<a id="f-4412ea75a6ea-evaluate_project"></a>
### `evaluate_project`

Pure complete-project evaluation. Input/output entries are byte arrays,
preserving binary assets and metadata; no browser filesystem is consulted.

<storycode:copy id="f-4412ea75a6ea-evaluate_project">
/**
 * Pure complete-project evaluation. Input/output entries are byte arrays,
 * preserving binary assets and metadata; no browser filesystem is consulted.
 */
export function evaluate_project(entry: string, sources_json: string, input_json: string): string;

</storycode:copy>
<a id="f-4412ea75a6ea-latest_debug_project"></a>
### `latest_debug_project`

The same exact-source selection used by native document launches.

<storycode:copy id="f-4412ea75a6ea-latest_debug_project">
/**
 * The same exact-source selection used by native document launches.
 */
export function latest_debug_project(versions_json: string, displayed_json: string, path: string): string;

</storycode:copy>
<a id="f-4412ea75a6ea-literal_files"></a>
### `literal_files`

Versioned non-executing materialization of literal files, including lineage.
Missing semantics are a diagnostic, never a successful partial program.

<storycode:copy id="f-4412ea75a6ea-literal_files">
/**
 * Versioned non-executing materialization of literal files, including lineage.
 * Missing semantics are a diagnostic, never a successful partial program.
 */
export function literal_files(source: string): string;

</storycode:copy>
<a id="f-4412ea75a6ea-project_position"></a>
### `project_position`

<storycode:copy id="f-4412ea75a6ea-project_position">
export function project_position(local_json: string, endpoint_json: string, path: string, byte: number): string;

</storycode:copy>
<a id="f-4412ea75a6ea-project_transition"></a>
### `project_transition`

One deterministic transition renderer for all comparison surfaces.

<storycode:copy id="f-4412ea75a6ea-project_transition">
/**
 * One deterministic transition renderer for all comparison surfaces.
 */
export function project_transition(old_json: string, new_json: string): string;

</storycode:copy>
<a id="f-4412ea75a6ea-regenerate_project_patch"></a>
### `regenerate_project_patch`

Exact shared patch regeneration; every downstream act is checked before returning source.

<storycode:copy id="f-4412ea75a6ea-regenerate_project_patch">
/**
 * Exact shared patch regeneration; every downstream act is checked before returning source.
 */
export function regenerate_project_patch(entry: string, sources_json: string, input_json: string, id: string, new_json: string): string;

</storycode:copy>
<a id="f-4412ea75a6ea-reverse_project_edit"></a>
### `reverse_project_edit`

<storycode:copy id="f-4412ea75a6ea-reverse_project_edit">
export function reverse_project_edit(entry: string, sources_json: string, input_json: string, endpoint_json: string, edit_json: string): string;

</storycode:copy>
<a id="f-4412ea75a6ea-structure"></a>
### `structure`

The structure of a document — `hick_lang::Structure` as JSON.
Offsets are byte offsets into `source`'s UTF-8; the caller converts them
to whatever its strings index by.

<storycode:copy id="f-4412ea75a6ea-structure">
/**
 * The structure of a document — `hick_lang::Structure` as JSON.
 *
 * Offsets are byte offsets into `source`'s UTF-8; the caller converts them
 * to whatever its strings index by.
 */
export function structure(source: string): string;

</storycode:copy>
<a id="f-4412ea75a6ea-InitInput"></a>
### `InitInput`

<storycode:copy id="f-4412ea75a6ea-InitInput">
export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

</storycode:copy>
<a id="f-4412ea75a6ea-InitOutput"></a>
### `InitOutput`

<storycode:copy id="f-4412ea75a6ea-InitOutput">
export interface InitOutput {
    readonly memory: WebAssembly.Memory;
    readonly browser_prediction: (a: number, b: number, c: number, d: number, e: number) => void;
    readonly evaluate_project: (a: number, b: number, c: number, d: number, e: number, f: number, g: number) => void;
    readonly latest_debug_project: (a: number, b: number, c: number, d: number, e: number, f: number, g: number) => void;
    readonly literal_files: (a: number, b: number, c: number) => void;
    readonly project_position: (a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number) => void;
    readonly project_transition: (a: number, b: number, c: number, d: number, e: number) => void;
    readonly regenerate_project_patch: (a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number, i: number, j: number, k: number) => void;
    readonly reverse_project_edit: (a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number, i: number, j: number, k: number) => void;
    readonly structure: (a: number, b: number, c: number) => void;
    readonly __wbindgen_export: (a: number) => void;
    readonly __wbindgen_add_to_stack_pointer: (a: number) => number;
    readonly __wbindgen_export2: (a: number, b: number) => number;
    readonly __wbindgen_export3: (a: number, b: number, c: number, d: number) => number;
    readonly __wbindgen_export4: (a: number, b: number, c: number) => void;
}

</storycode:copy>
<a id="f-4412ea75a6ea-SyncInitInput"></a>
### `SyncInitInput`

<storycode:copy id="f-4412ea75a6ea-SyncInitInput">
export type SyncInitInput = BufferSource | WebAssembly.Module;

</storycode:copy>
<a id="f-4412ea75a6ea-initSync"></a>
### `initSync`

Instantiates the given `module`, which can either be bytes or
a precompiled `WebAssembly.Module`.
@param {{ module: SyncInitInput }} module - Passing `SyncInitInput` directly is deprecated.
@returns {InitOutput}

<storycode:copy id="f-4412ea75a6ea-initSync">
/**
 * Instantiates the given `module`, which can either be bytes or
 * a precompiled `WebAssembly.Module`.
 *
 * @param {{ module: SyncInitInput }} module - Passing `SyncInitInput` directly is deprecated.
 *
 * @returns {InitOutput}
 */
export function initSync(module: { module: SyncInitInput } | SyncInitInput): InitOutput;

</storycode:copy>
<a id="f-4412ea75a6ea-__wbg_init"></a>
### `__wbg_init`

If `module_or_path` is {RequestInfo} or {URL}, makes a request and
for everything else, calls `WebAssembly.instantiate` directly.
@param {{ module_or_path: InitInput | Promise<InitInput> }} module_or_path - Passing `InitInput` directly is deprecated.
@returns {Promise<InitOutput>}

<storycode:copy id="f-4412ea75a6ea-__wbg_init">
/**
 * If `module_or_path` is {RequestInfo} or {URL}, makes a request and
 * for everything else, calls `WebAssembly.instantiate` directly.
 *
 * @param {{ module_or_path: InitInput | Promise<InitInput> }} module_or_path - Passing `InitInput` directly is deprecated.
 *
 * @returns {Promise<InitOutput>}
 */
export default function __wbg_init (module_or_path?: { module_or_path: InitInput | Promise<InitInput> } | InitInput | Promise<InitInput>): Promise<InitOutput>;
</storycode:copy>

### Assemble `apps/web/src/editor/generated/hick-lang/hick_lang.d.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/editor/generated/hick-lang/hick_lang.d.ts"><storycode:paste select="#f-4412ea75a6ea-browser_prediction"/><storycode:paste select="#f-4412ea75a6ea-evaluate_project"/><storycode:paste select="#f-4412ea75a6ea-latest_debug_project"/><storycode:paste select="#f-4412ea75a6ea-literal_files"/><storycode:paste select="#f-4412ea75a6ea-project_position"/><storycode:paste select="#f-4412ea75a6ea-project_transition"/><storycode:paste select="#f-4412ea75a6ea-regenerate_project_patch"/><storycode:paste select="#f-4412ea75a6ea-reverse_project_edit"/><storycode:paste select="#f-4412ea75a6ea-structure"/><storycode:paste select="#f-4412ea75a6ea-InitInput"/><storycode:paste select="#f-4412ea75a6ea-InitOutput"/><storycode:paste select="#f-4412ea75a6ea-SyncInitInput"/><storycode:paste select="#f-4412ea75a6ea-initSync"/><storycode:paste select="#f-4412ea75a6ea-__wbg_init"/></storycode:file>

<a id="output-c53582515263"></a>
## `apps/web/src/editor/generated/hick-lang/hick_lang.js`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [Browser-local prediction practice uses the shared gate](contract.md).

<a id="f-c53582515263-browser_prediction"></a>
### `browser_prediction`

@ts-self-types="./hick_lang.d.ts"
Private browser-host transition. Actor and actual result arguments must be
supplied by trusted adapters, never forwarded from model invocation fields.
Storage is user-controlled, not a native private directory or authenticated
evidence store. Snapshot persistence must succeed before admitted dispatch.
@param {string} snapshot
@param {string} command
@returns {string}

<storycode:copy id="f-c53582515263-browser_prediction">
/* @ts-self-types="./hick_lang.d.ts" */

/**
 * Private browser-host transition. Actor and actual result arguments must be
 * supplied by trusted adapters, never forwarded from model invocation fields.
 * Storage is user-controlled, not a native private directory or authenticated
 * evidence store. Snapshot persistence must succeed before admitted dispatch.
 * @param {string} snapshot
 * @param {string} command
 * @returns {string}
 */
export function browser_prediction(snapshot, command) {
    let deferred3_0;
    let deferred3_1;
    try {
        const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
        const ptr0 = passStringToWasm0(snapshot, wasm.__wbindgen_export2, wasm.__wbindgen_export3);
        const len0 = WASM_VECTOR_LEN;
        const ptr1 = passStringToWasm0(command, wasm.__wbindgen_export2, wasm.__wbindgen_export3);
        const len1 = WASM_VECTOR_LEN;
        wasm.browser_prediction(retptr, ptr0, len0, ptr1, len1);
        var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
        var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
        deferred3_0 = r0;
        deferred3_1 = r1;
        return getStringFromWasm0(r0, r1);
    } finally {
        wasm.__wbindgen_add_to_stack_pointer(16);
        wasm.__wbindgen_export4(deferred3_0, deferred3_1, 1);
    }
}

</storycode:copy>
<a id="f-c53582515263-evaluate_project"></a>
### `evaluate_project`

Pure complete-project evaluation. Input/output entries are byte arrays,
preserving binary assets and metadata; no browser filesystem is consulted.
@param {string} entry
@param {string} sources_json
@param {string} input_json
@returns {string}

<storycode:copy id="f-c53582515263-evaluate_project">
/**
 * Pure complete-project evaluation. Input/output entries are byte arrays,
 * preserving binary assets and metadata; no browser filesystem is consulted.
 * @param {string} entry
 * @param {string} sources_json
 * @param {string} input_json
 * @returns {string}
 */
export function evaluate_project(entry, sources_json, input_json) {
    let deferred4_0;
    let deferred4_1;
    try {
        const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
        const ptr0 = passStringToWasm0(entry, wasm.__wbindgen_export2, wasm.__wbindgen_export3);
        const len0 = WASM_VECTOR_LEN;
        const ptr1 = passStringToWasm0(sources_json, wasm.__wbindgen_export2, wasm.__wbindgen_export3);
        const len1 = WASM_VECTOR_LEN;
        const ptr2 = passStringToWasm0(input_json, wasm.__wbindgen_export2, wasm.__wbindgen_export3);
        const len2 = WASM_VECTOR_LEN;
        wasm.evaluate_project(retptr, ptr0, len0, ptr1, len1, ptr2, len2);
        var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
        var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
        deferred4_0 = r0;
        deferred4_1 = r1;
        return getStringFromWasm0(r0, r1);
    } finally {
        wasm.__wbindgen_add_to_stack_pointer(16);
        wasm.__wbindgen_export4(deferred4_0, deferred4_1, 1);
    }
}

</storycode:copy>
<a id="f-c53582515263-latest_debug_project"></a>
### `latest_debug_project`

The same exact-source selection used by native document launches.
@param {string} versions_json
@param {string} displayed_json
@param {string} path
@returns {string}

<storycode:copy id="f-c53582515263-latest_debug_project">
/**
 * The same exact-source selection used by native document launches.
 * @param {string} versions_json
 * @param {string} displayed_json
 * @param {string} path
 * @returns {string}
 */
export function latest_debug_project(versions_json, displayed_json, path) {
    let deferred4_0;
    let deferred4_1;
    try {
        const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
        const ptr0 = passStringToWasm0(versions_json, wasm.__wbindgen_export2, wasm.__wbindgen_export3);
        const len0 = WASM_VECTOR_LEN;
        const ptr1 = passStringToWasm0(displayed_json, wasm.__wbindgen_export2, wasm.__wbindgen_export3);
        const len1 = WASM_VECTOR_LEN;
        const ptr2 = passStringToWasm0(path, wasm.__wbindgen_export2, wasm.__wbindgen_export3);
        const len2 = WASM_VECTOR_LEN;
        wasm.latest_debug_project(retptr, ptr0, len0, ptr1, len1, ptr2, len2);
        var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
        var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
        deferred4_0 = r0;
        deferred4_1 = r1;
        return getStringFromWasm0(r0, r1);
    } finally {
        wasm.__wbindgen_add_to_stack_pointer(16);
        wasm.__wbindgen_export4(deferred4_0, deferred4_1, 1);
    }
}

</storycode:copy>
<a id="f-c53582515263-literal_files"></a>
### `literal_files`

Versioned non-executing materialization of literal files, including lineage.
Missing semantics are a diagnostic, never a successful partial program.
@param {string} source
@returns {string}

<storycode:copy id="f-c53582515263-literal_files">
/**
 * Versioned non-executing materialization of literal files, including lineage.
 * Missing semantics are a diagnostic, never a successful partial program.
 * @param {string} source
 * @returns {string}
 */
export function literal_files(source) {
    let deferred2_0;
    let deferred2_1;
    try {
        const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
        const ptr0 = passStringToWasm0(source, wasm.__wbindgen_export2, wasm.__wbindgen_export3);
        const len0 = WASM_VECTOR_LEN;
        wasm.literal_files(retptr, ptr0, len0);
        var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
        var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
        deferred2_0 = r0;
        deferred2_1 = r1;
        return getStringFromWasm0(r0, r1);
    } finally {
        wasm.__wbindgen_add_to_stack_pointer(16);
        wasm.__wbindgen_export4(deferred2_0, deferred2_1, 1);
    }
}

</storycode:copy>
<a id="f-c53582515263-project_position"></a>
### `project_position`

@param {string} local_json
@param {string} endpoint_json
@param {string} path
@param {number} byte
@returns {string}

<storycode:copy id="f-c53582515263-project_position">
/**
 * @param {string} local_json
 * @param {string} endpoint_json
 * @param {string} path
 * @param {number} byte
 * @returns {string}
 */
export function project_position(local_json, endpoint_json, path, byte) {
    let deferred4_0;
    let deferred4_1;
    try {
        const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
        const ptr0 = passStringToWasm0(local_json, wasm.__wbindgen_export2, wasm.__wbindgen_export3);
        const len0 = WASM_VECTOR_LEN;
        const ptr1 = passStringToWasm0(endpoint_json, wasm.__wbindgen_export2, wasm.__wbindgen_export3);
        const len1 = WASM_VECTOR_LEN;
        const ptr2 = passStringToWasm0(path, wasm.__wbindgen_export2, wasm.__wbindgen_export3);
        const len2 = WASM_VECTOR_LEN;
        wasm.project_position(retptr, ptr0, len0, ptr1, len1, ptr2, len2, byte);
        var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
        var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
        deferred4_0 = r0;
        deferred4_1 = r1;
        return getStringFromWasm0(r0, r1);
    } finally {
        wasm.__wbindgen_add_to_stack_pointer(16);
        wasm.__wbindgen_export4(deferred4_0, deferred4_1, 1);
    }
}

</storycode:copy>
<a id="f-c53582515263-project_transition"></a>
### `project_transition`

One deterministic transition renderer for all comparison surfaces.
@param {string} old_json
@param {string} new_json
@returns {string}

<storycode:copy id="f-c53582515263-project_transition">
/**
 * One deterministic transition renderer for all comparison surfaces.
 * @param {string} old_json
 * @param {string} new_json
 * @returns {string}
 */
export function project_transition(old_json, new_json) {
    let deferred3_0;
    let deferred3_1;
    try {
        const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
        const ptr0 = passStringToWasm0(old_json, wasm.__wbindgen_export2, wasm.__wbindgen_export3);
        const len0 = WASM_VECTOR_LEN;
        const ptr1 = passStringToWasm0(new_json, wasm.__wbindgen_export2, wasm.__wbindgen_export3);
        const len1 = WASM_VECTOR_LEN;
        wasm.project_transition(retptr, ptr0, len0, ptr1, len1);
        var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
        var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
        deferred3_0 = r0;
        deferred3_1 = r1;
        return getStringFromWasm0(r0, r1);
    } finally {
        wasm.__wbindgen_add_to_stack_pointer(16);
        wasm.__wbindgen_export4(deferred3_0, deferred3_1, 1);
    }
}

</storycode:copy>
<a id="f-c53582515263-regenerate_project_patch"></a>
### `regenerate_project_patch`

Exact shared patch regeneration; every downstream act is checked before returning source.
@param {string} entry
@param {string} sources_json
@param {string} input_json
@param {string} id
@param {string} new_json
@returns {string}

<storycode:copy id="f-c53582515263-regenerate_project_patch">
/**
 * Exact shared patch regeneration; every downstream act is checked before returning source.
 * @param {string} entry
 * @param {string} sources_json
 * @param {string} input_json
 * @param {string} id
 * @param {string} new_json
 * @returns {string}
 */
export function regenerate_project_patch(entry, sources_json, input_json, id, new_json) {
    let deferred6_0;
    let deferred6_1;
    try {
        const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
        const ptr0 = passStringToWasm0(entry, wasm.__wbindgen_export2, wasm.__wbindgen_export3);
        const len0 = WASM_VECTOR_LEN;
        const ptr1 = passStringToWasm0(sources_json, wasm.__wbindgen_export2, wasm.__wbindgen_export3);
        const len1 = WASM_VECTOR_LEN;
        const ptr2 = passStringToWasm0(input_json, wasm.__wbindgen_export2, wasm.__wbindgen_export3);
        const len2 = WASM_VECTOR_LEN;
        const ptr3 = passStringToWasm0(id, wasm.__wbindgen_export2, wasm.__wbindgen_export3);
        const len3 = WASM_VECTOR_LEN;
        const ptr4 = passStringToWasm0(new_json, wasm.__wbindgen_export2, wasm.__wbindgen_export3);
        const len4 = WASM_VECTOR_LEN;
        wasm.regenerate_project_patch(retptr, ptr0, len0, ptr1, len1, ptr2, len2, ptr3, len3, ptr4, len4);
        var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
        var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
        deferred6_0 = r0;
        deferred6_1 = r1;
        return getStringFromWasm0(r0, r1);
    } finally {
        wasm.__wbindgen_add_to_stack_pointer(16);
        wasm.__wbindgen_export4(deferred6_0, deferred6_1, 1);
    }
}

</storycode:copy>
<a id="f-c53582515263-reverse_project_edit"></a>
### `reverse_project_edit`

@param {string} entry
@param {string} sources_json
@param {string} input_json
@param {string} endpoint_json
@param {string} edit_json
@returns {string}

<storycode:copy id="f-c53582515263-reverse_project_edit">
/**
 * @param {string} entry
 * @param {string} sources_json
 * @param {string} input_json
 * @param {string} endpoint_json
 * @param {string} edit_json
 * @returns {string}
 */
export function reverse_project_edit(entry, sources_json, input_json, endpoint_json, edit_json) {
    let deferred6_0;
    let deferred6_1;
    try {
        const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
        const ptr0 = passStringToWasm0(entry, wasm.__wbindgen_export2, wasm.__wbindgen_export3);
        const len0 = WASM_VECTOR_LEN;
        const ptr1 = passStringToWasm0(sources_json, wasm.__wbindgen_export2, wasm.__wbindgen_export3);
        const len1 = WASM_VECTOR_LEN;
        const ptr2 = passStringToWasm0(input_json, wasm.__wbindgen_export2, wasm.__wbindgen_export3);
        const len2 = WASM_VECTOR_LEN;
        const ptr3 = passStringToWasm0(endpoint_json, wasm.__wbindgen_export2, wasm.__wbindgen_export3);
        const len3 = WASM_VECTOR_LEN;
        const ptr4 = passStringToWasm0(edit_json, wasm.__wbindgen_export2, wasm.__wbindgen_export3);
        const len4 = WASM_VECTOR_LEN;
        wasm.reverse_project_edit(retptr, ptr0, len0, ptr1, len1, ptr2, len2, ptr3, len3, ptr4, len4);
        var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
        var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
        deferred6_0 = r0;
        deferred6_1 = r1;
        return getStringFromWasm0(r0, r1);
    } finally {
        wasm.__wbindgen_add_to_stack_pointer(16);
        wasm.__wbindgen_export4(deferred6_0, deferred6_1, 1);
    }
}

</storycode:copy>
<a id="f-c53582515263-structure"></a>
### `structure`

The structure of a document — `hick_lang::Structure` as JSON.
Offsets are byte offsets into `source`'s UTF-8; the caller converts them
to whatever its strings index by.
@param {string} source
@returns {string}

<storycode:copy id="f-c53582515263-structure">
/**
 * The structure of a document — `hick_lang::Structure` as JSON.
 *
 * Offsets are byte offsets into `source`'s UTF-8; the caller converts them
 * to whatever its strings index by.
 * @param {string} source
 * @returns {string}
 */
export function structure(source) {
    let deferred2_0;
    let deferred2_1;
    try {
        const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
        const ptr0 = passStringToWasm0(source, wasm.__wbindgen_export2, wasm.__wbindgen_export3);
        const len0 = WASM_VECTOR_LEN;
        wasm.structure(retptr, ptr0, len0);
        var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
        var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
        deferred2_0 = r0;
        deferred2_1 = r1;
        return getStringFromWasm0(r0, r1);
    } finally {
        wasm.__wbindgen_add_to_stack_pointer(16);
        wasm.__wbindgen_export4(deferred2_0, deferred2_1, 1);
    }
}
</storycode:copy>
<a id="f-c53582515263-__wbg_get_imports"></a>
### `__wbg_get_imports`

<storycode:copy id="f-c53582515263-__wbg_get_imports">
function __wbg_get_imports() {
    const import0 = {
        __proto__: null,
        __wbg_getRandomValues_ceb34d8ffce7e87f: function() { return handleError(function (arg0, arg1) {
            globalThis.crypto.getRandomValues(getArrayU8FromWasm0(arg0, arg1));
        }, arguments); },
        __wbindgen_object_drop_ref: function(arg0) {
            takeObject(arg0);
        },
    };
    return {
        __proto__: null,
        "./hick_lang_bg.js": import0,
    };
}

</storycode:copy>
<a id="f-c53582515263-addHeapObject"></a>
### `addHeapObject`

<storycode:copy id="f-c53582515263-addHeapObject">
function addHeapObject(obj) {
    if (heap_next === heap.length) heap.push(heap.length + 1);
    const idx = heap_next;
    heap_next = heap[idx];

    heap[idx] = obj;
    return idx;
}

</storycode:copy>
<a id="f-c53582515263-dropObject"></a>
### `dropObject`

<storycode:copy id="f-c53582515263-dropObject">
function dropObject(idx) {
    if (idx < 1028) return;
    heap[idx] = heap_next;
    heap_next = idx;
}

</storycode:copy>
<a id="f-c53582515263-getArrayU8FromWasm0"></a>
### `getArrayU8FromWasm0`

<storycode:copy id="f-c53582515263-getArrayU8FromWasm0">
function getArrayU8FromWasm0(ptr, len) {
    ptr = ptr >>> 0;
    return getUint8ArrayMemory0().subarray(ptr / 1, ptr / 1 + len);
}

</storycode:copy>
<a id="f-c53582515263-cachedDataViewMemory0"></a>
### `cachedDataViewMemory0`

<storycode:copy id="f-c53582515263-cachedDataViewMemory0">
let cachedDataViewMemory0 = null;
</storycode:copy>
<a id="f-c53582515263-getDataViewMemory0"></a>
### `getDataViewMemory0`

<storycode:copy id="f-c53582515263-getDataViewMemory0">
function getDataViewMemory0() {
    if (cachedDataViewMemory0 === null || cachedDataViewMemory0.buffer.detached === true || (cachedDataViewMemory0.buffer.detached === undefined && cachedDataViewMemory0.buffer !== wasm.memory.buffer)) {
        cachedDataViewMemory0 = new DataView(wasm.memory.buffer);
    }
    return cachedDataViewMemory0;
}

</storycode:copy>
<a id="f-c53582515263-getStringFromWasm0"></a>
### `getStringFromWasm0`

<storycode:copy id="f-c53582515263-getStringFromWasm0">
function getStringFromWasm0(ptr, len) {
    return decodeText(ptr >>> 0, len);
}

</storycode:copy>
<a id="f-c53582515263-cachedUint8ArrayMemory0"></a>
### `cachedUint8ArrayMemory0`

<storycode:copy id="f-c53582515263-cachedUint8ArrayMemory0">
let cachedUint8ArrayMemory0 = null;
</storycode:copy>
<a id="f-c53582515263-getUint8ArrayMemory0"></a>
### `getUint8ArrayMemory0`

<storycode:copy id="f-c53582515263-getUint8ArrayMemory0">
function getUint8ArrayMemory0() {
    if (cachedUint8ArrayMemory0 === null || cachedUint8ArrayMemory0.byteLength === 0) {
        cachedUint8ArrayMemory0 = new Uint8Array(wasm.memory.buffer);
    }
    return cachedUint8ArrayMemory0;
}

</storycode:copy>
<a id="f-c53582515263-getObject"></a>
### `getObject`

<storycode:copy id="f-c53582515263-getObject">
function getObject(idx) { return heap[idx]; }

</storycode:copy>
<a id="f-c53582515263-handleError"></a>
### `handleError`

<storycode:copy id="f-c53582515263-handleError">
function handleError(f, args) {
    try {
        return f.apply(this, args);
    } catch (e) {
        wasm.__wbindgen_export(addHeapObject(e));
    }
}

</storycode:copy>
<a id="f-c53582515263-heap"></a>
### `heap`

<storycode:copy id="f-c53582515263-heap">
let heap = new Array(1024).fill(undefined);
</storycode:copy>
<a id="f-c53582515263-expression_statement"></a>
### `expression_statement`

<storycode:copy id="f-c53582515263-expression_statement">
heap.push(undefined, null, true, false);

</storycode:copy>
<a id="f-c53582515263-heap_next"></a>
### `heap_next`

<storycode:copy id="f-c53582515263-heap_next">
let heap_next = heap.length;

</storycode:copy>
<a id="f-c53582515263-passStringToWasm0"></a>
### `passStringToWasm0`

<storycode:copy id="f-c53582515263-passStringToWasm0">
function passStringToWasm0(arg, malloc, realloc) {
    if (realloc === undefined) {
        const buf = cachedTextEncoder.encode(arg);
        const ptr = malloc(buf.length, 1) >>> 0;
        getUint8ArrayMemory0().subarray(ptr, ptr + buf.length).set(buf);
        WASM_VECTOR_LEN = buf.length;
        return ptr;
    }

    let len = arg.length;
    let ptr = malloc(len, 1) >>> 0;

    const mem = getUint8ArrayMemory0();

    let offset = 0;

    for (; offset < len; offset++) {
        const code = arg.charCodeAt(offset);
        if (code > 0x7F) break;
        mem[ptr + offset] = code;
    }
    if (offset !== len) {
        if (offset !== 0) {
            arg = arg.slice(offset);
        }
        ptr = realloc(ptr, len, len = offset + arg.length * 3, 1) >>> 0;
        const view = getUint8ArrayMemory0().subarray(ptr + offset, ptr + len);
        const ret = cachedTextEncoder.encodeInto(arg, view);

        offset += ret.written;
        ptr = realloc(ptr, len, offset, 1) >>> 0;
    }

    WASM_VECTOR_LEN = offset;
    return ptr;
}

</storycode:copy>
<a id="f-c53582515263-takeObject"></a>
### `takeObject`

<storycode:copy id="f-c53582515263-takeObject">
function takeObject(idx) {
    const ret = getObject(idx);
    dropObject(idx);
    return ret;
}

</storycode:copy>
<a id="f-c53582515263-cachedTextDecoder"></a>
### `cachedTextDecoder`

<storycode:copy id="f-c53582515263-cachedTextDecoder">
let cachedTextDecoder = new TextDecoder('utf-8', { ignoreBOM: true, fatal: true });
</storycode:copy>
<a id="f-c53582515263-expression_statement-2"></a>
### `expression_statement`

<storycode:copy id="f-c53582515263-expression_statement-2">
cachedTextDecoder.decode();
</storycode:copy>
<a id="f-c53582515263-MAX_SAFARI_DECODE_BYTES"></a>
### `MAX_SAFARI_DECODE_BYTES`

<storycode:copy id="f-c53582515263-MAX_SAFARI_DECODE_BYTES">
const MAX_SAFARI_DECODE_BYTES = 2146435072;
</storycode:copy>
<a id="f-c53582515263-numBytesDecoded"></a>
### `numBytesDecoded`

<storycode:copy id="f-c53582515263-numBytesDecoded">
let numBytesDecoded = 0;
</storycode:copy>
<a id="f-c53582515263-decodeText"></a>
### `decodeText`

<storycode:copy id="f-c53582515263-decodeText">
function decodeText(ptr, len) {
    numBytesDecoded += len;
    if (numBytesDecoded >= MAX_SAFARI_DECODE_BYTES) {
        cachedTextDecoder = new TextDecoder('utf-8', { ignoreBOM: true, fatal: true });
        cachedTextDecoder.decode();
        numBytesDecoded = len;
    }
    return cachedTextDecoder.decode(getUint8ArrayMemory0().subarray(ptr, ptr + len));
}

</storycode:copy>
<a id="f-c53582515263-cachedTextEncoder"></a>
### `cachedTextEncoder`

<storycode:copy id="f-c53582515263-cachedTextEncoder">
const cachedTextEncoder = new TextEncoder();

</storycode:copy>
<a id="f-c53582515263-if_statement"></a>
### `if_statement`

<storycode:copy id="f-c53582515263-if_statement">
if (!('encodeInto' in cachedTextEncoder)) {
    cachedTextEncoder.encodeInto = function (arg, view) {
        const buf = cachedTextEncoder.encode(arg);
        view.set(buf);
        return {
            read: arg.length,
            written: buf.length
        };
    };
}

</storycode:copy>
<a id="f-c53582515263-WASM_VECTOR_LEN"></a>
### `WASM_VECTOR_LEN`

<storycode:copy id="f-c53582515263-WASM_VECTOR_LEN">
let WASM_VECTOR_LEN = 0;

</storycode:copy>
<a id="f-c53582515263-wasmModule"></a>
### `wasmModule`

<storycode:copy id="f-c53582515263-wasmModule">
let wasmModule, wasmInstance, wasm;
</storycode:copy>
<a id="f-c53582515263-__wbg_finalize_init"></a>
### `__wbg_finalize_init`

<storycode:copy id="f-c53582515263-__wbg_finalize_init">
function __wbg_finalize_init(instance, module) {
    wasmInstance = instance;
    wasm = instance.exports;
    wasmModule = module;
    cachedDataViewMemory0 = null;
    cachedUint8ArrayMemory0 = null;
    return wasm;
}

</storycode:copy>
<a id="f-c53582515263-__wbg_load"></a>
### `__wbg_load`

<storycode:copy id="f-c53582515263-__wbg_load">
async function __wbg_load(module, imports) {
    if (typeof Response === 'function' && module instanceof Response) {
        if (typeof WebAssembly.instantiateStreaming === 'function') {
            try {
                return await WebAssembly.instantiateStreaming(module, imports);
            } catch (e) {
                const validResponse = module.ok && expectedResponseType(module.type);

                if (validResponse && module.headers.get('Content-Type') !== 'application/wasm') {
                    console.warn("`WebAssembly.instantiateStreaming` failed because your server does not serve Wasm with `application/wasm` MIME type. Falling back to `WebAssembly.instantiate` which is slower. Original error:\n", e);

                } else { throw e; }
            }
        }

        const bytes = await module.arrayBuffer();
        return await WebAssembly.instantiate(bytes, imports);
    } else {
        const instance = await WebAssembly.instantiate(module, imports);

        if (instance instanceof WebAssembly.Instance) {
            return { instance, module };
        } else {
            return instance;
        }
    }

    function expectedResponseType(type) {
        switch (type) {
            case 'basic': case 'cors': case 'default': return true;
        }
        return false;
    }
}

</storycode:copy>
<a id="f-c53582515263-initSync"></a>
### `initSync`

<storycode:copy id="f-c53582515263-initSync">
function initSync(module) {
    if (wasm !== undefined) return wasm;


    if (module !== undefined) {
        if (Object.getPrototypeOf(module) === Object.prototype) {
            ({module} = module)
        } else {
            console.warn('using deprecated parameters for `initSync()`; pass a single object instead')
        }
    }

    const imports = __wbg_get_imports();
    if (!(module instanceof WebAssembly.Module)) {
        module = new WebAssembly.Module(module);
    }
    const instance = new WebAssembly.Instance(module, imports);
    return __wbg_finalize_init(instance, module);
}

</storycode:copy>
<a id="f-c53582515263-__wbg_init"></a>
### `__wbg_init`

<storycode:copy id="f-c53582515263-__wbg_init">
async function __wbg_init(module_or_path) {
    if (wasm !== undefined) return wasm;


    if (module_or_path !== undefined) {
        if (Object.getPrototypeOf(module_or_path) === Object.prototype) {
            ({module_or_path} = module_or_path)
        } else {
            console.warn('using deprecated parameters for the initialization function; pass a single object instead')
        }
    }

    if (module_or_path === undefined) {
        module_or_path = new URL('hick_lang_bg.wasm', import.meta.url);
    }
    const imports = __wbg_get_imports();

    if (typeof module_or_path === 'string' || (typeof Request === 'function' && module_or_path instanceof Request) || (typeof URL === 'function' && module_or_path instanceof URL)) {
        module_or_path = fetch(module_or_path);
    }

    const { instance, module } = await __wbg_load(await module_or_path, imports);

    return __wbg_finalize_init(instance, module);
}

</storycode:copy>
<a id="f-c53582515263-export_statement"></a>
### `export_statement`

<storycode:copy id="f-c53582515263-export_statement">
export { initSync, __wbg_init as default };
</storycode:copy>

### Assemble `apps/web/src/editor/generated/hick-lang/hick_lang.js`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/editor/generated/hick-lang/hick_lang.js"><storycode:paste select="#f-c53582515263-browser_prediction"/><storycode:paste select="#f-c53582515263-evaluate_project"/><storycode:paste select="#f-c53582515263-latest_debug_project"/><storycode:paste select="#f-c53582515263-literal_files"/><storycode:paste select="#f-c53582515263-project_position"/><storycode:paste select="#f-c53582515263-project_transition"/><storycode:paste select="#f-c53582515263-regenerate_project_patch"/><storycode:paste select="#f-c53582515263-reverse_project_edit"/><storycode:paste select="#f-c53582515263-structure"/><storycode:paste select="#f-c53582515263-__wbg_get_imports"/><storycode:paste select="#f-c53582515263-addHeapObject"/><storycode:paste select="#f-c53582515263-dropObject"/><storycode:paste select="#f-c53582515263-getArrayU8FromWasm0"/><storycode:paste select="#f-c53582515263-cachedDataViewMemory0"/><storycode:paste select="#f-c53582515263-getDataViewMemory0"/><storycode:paste select="#f-c53582515263-getStringFromWasm0"/><storycode:paste select="#f-c53582515263-cachedUint8ArrayMemory0"/><storycode:paste select="#f-c53582515263-getUint8ArrayMemory0"/><storycode:paste select="#f-c53582515263-getObject"/><storycode:paste select="#f-c53582515263-handleError"/><storycode:paste select="#f-c53582515263-heap"/><storycode:paste select="#f-c53582515263-expression_statement"/><storycode:paste select="#f-c53582515263-heap_next"/><storycode:paste select="#f-c53582515263-passStringToWasm0"/><storycode:paste select="#f-c53582515263-takeObject"/><storycode:paste select="#f-c53582515263-cachedTextDecoder"/><storycode:paste select="#f-c53582515263-expression_statement-2"/><storycode:paste select="#f-c53582515263-MAX_SAFARI_DECODE_BYTES"/><storycode:paste select="#f-c53582515263-numBytesDecoded"/><storycode:paste select="#f-c53582515263-decodeText"/><storycode:paste select="#f-c53582515263-cachedTextEncoder"/><storycode:paste select="#f-c53582515263-if_statement"/><storycode:paste select="#f-c53582515263-WASM_VECTOR_LEN"/><storycode:paste select="#f-c53582515263-wasmModule"/><storycode:paste select="#f-c53582515263-__wbg_finalize_init"/><storycode:paste select="#f-c53582515263-__wbg_load"/><storycode:paste select="#f-c53582515263-initSync"/><storycode:paste select="#f-c53582515263-__wbg_init"/><storycode:paste select="#f-c53582515263-export_statement"/></storycode:file>

<a id="output-3b08e5ec33ba"></a>
## `apps/web/src/editor/generated/hick-lang/hick_lang_bg.wasm.d.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [Browser-local prediction practice uses the shared gate](contract.md).

<a id="f-3b08e5ec33ba-memory"></a>
### `memory`

<storycode:copy id="f-3b08e5ec33ba-memory">
/* tslint:disable */
/* eslint-disable */
export const memory: WebAssembly.Memory;
</storycode:copy>
<a id="f-3b08e5ec33ba-browser_prediction"></a>
### `browser_prediction`

<storycode:copy id="f-3b08e5ec33ba-browser_prediction">
export const browser_prediction: (a: number, b: number, c: number, d: number, e: number) => void;
</storycode:copy>
<a id="f-3b08e5ec33ba-evaluate_project"></a>
### `evaluate_project`

<storycode:copy id="f-3b08e5ec33ba-evaluate_project">
export const evaluate_project: (a: number, b: number, c: number, d: number, e: number, f: number, g: number) => void;
</storycode:copy>
<a id="f-3b08e5ec33ba-latest_debug_project"></a>
### `latest_debug_project`

<storycode:copy id="f-3b08e5ec33ba-latest_debug_project">
export const latest_debug_project: (a: number, b: number, c: number, d: number, e: number, f: number, g: number) => void;
</storycode:copy>
<a id="f-3b08e5ec33ba-literal_files"></a>
### `literal_files`

<storycode:copy id="f-3b08e5ec33ba-literal_files">
export const literal_files: (a: number, b: number, c: number) => void;
</storycode:copy>
<a id="f-3b08e5ec33ba-project_position"></a>
### `project_position`

<storycode:copy id="f-3b08e5ec33ba-project_position">
export const project_position: (a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number) => void;
</storycode:copy>
<a id="f-3b08e5ec33ba-project_transition"></a>
### `project_transition`

<storycode:copy id="f-3b08e5ec33ba-project_transition">
export const project_transition: (a: number, b: number, c: number, d: number, e: number) => void;
</storycode:copy>
<a id="f-3b08e5ec33ba-regenerate_project_patch"></a>
### `regenerate_project_patch`

<storycode:copy id="f-3b08e5ec33ba-regenerate_project_patch">
export const regenerate_project_patch: (a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number, i: number, j: number, k: number) => void;
</storycode:copy>
<a id="f-3b08e5ec33ba-reverse_project_edit"></a>
### `reverse_project_edit`

<storycode:copy id="f-3b08e5ec33ba-reverse_project_edit">
export const reverse_project_edit: (a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number, i: number, j: number, k: number) => void;
</storycode:copy>
<a id="f-3b08e5ec33ba-structure"></a>
### `structure`

<storycode:copy id="f-3b08e5ec33ba-structure">
export const structure: (a: number, b: number, c: number) => void;
</storycode:copy>
<a id="f-3b08e5ec33ba-__wbindgen_export"></a>
### `__wbindgen_export`

<storycode:copy id="f-3b08e5ec33ba-__wbindgen_export">
export const __wbindgen_export: (a: number) => void;
</storycode:copy>
<a id="f-3b08e5ec33ba-__wbindgen_add_to_stack_pointer"></a>
### `__wbindgen_add_to_stack_pointer`

<storycode:copy id="f-3b08e5ec33ba-__wbindgen_add_to_stack_pointer">
export const __wbindgen_add_to_stack_pointer: (a: number) => number;
</storycode:copy>
<a id="f-3b08e5ec33ba-__wbindgen_export2"></a>
### `__wbindgen_export2`

<storycode:copy id="f-3b08e5ec33ba-__wbindgen_export2">
export const __wbindgen_export2: (a: number, b: number) => number;
</storycode:copy>
<a id="f-3b08e5ec33ba-__wbindgen_export3"></a>
### `__wbindgen_export3`

<storycode:copy id="f-3b08e5ec33ba-__wbindgen_export3">
export const __wbindgen_export3: (a: number, b: number, c: number, d: number) => number;
</storycode:copy>
<a id="f-3b08e5ec33ba-__wbindgen_export4"></a>
### `__wbindgen_export4`

<storycode:copy id="f-3b08e5ec33ba-__wbindgen_export4">
export const __wbindgen_export4: (a: number, b: number, c: number) => void;
</storycode:copy>

### Assemble `apps/web/src/editor/generated/hick-lang/hick_lang_bg.wasm.d.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/editor/generated/hick-lang/hick_lang_bg.wasm.d.ts"><storycode:paste select="#f-3b08e5ec33ba-memory"/><storycode:paste select="#f-3b08e5ec33ba-browser_prediction"/><storycode:paste select="#f-3b08e5ec33ba-evaluate_project"/><storycode:paste select="#f-3b08e5ec33ba-latest_debug_project"/><storycode:paste select="#f-3b08e5ec33ba-literal_files"/><storycode:paste select="#f-3b08e5ec33ba-project_position"/><storycode:paste select="#f-3b08e5ec33ba-project_transition"/><storycode:paste select="#f-3b08e5ec33ba-regenerate_project_patch"/><storycode:paste select="#f-3b08e5ec33ba-reverse_project_edit"/><storycode:paste select="#f-3b08e5ec33ba-structure"/><storycode:paste select="#f-3b08e5ec33ba-__wbindgen_export"/><storycode:paste select="#f-3b08e5ec33ba-__wbindgen_add_to_stack_pointer"/><storycode:paste select="#f-3b08e5ec33ba-__wbindgen_export2"/><storycode:paste select="#f-3b08e5ec33ba-__wbindgen_export3"/><storycode:paste select="#f-3b08e5ec33ba-__wbindgen_export4"/></storycode:file>

<a id="output-88aa201d7149"></a>
## `apps/web/src/embed/StaticUiSurface.tsx`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [Browser-local prediction practice uses the shared gate](contract.md).

<a id="f-88aa201d7149-STATIC_UI_CHANNEL"></a>
### `STATIC_UI_CHANNEL`

<storycode:copy id="f-88aa201d7149-STATIC_UI_CHANNEL">
import { useEffect, useState, type ReactNode } from "react";
import { UiActionContext } from "../lib/uiActionBridge";
import { createEmbedUiSession } from "./uiHostSession";

export const STATIC_UI_CHANNEL = "hickory-static-ui";

</storycode:copy>
<a id="f-88aa201d7149-connectStaticUiParent"></a>
### `connectStaticUiParent`

Explicit parent attachment; static pages never probe a local engine.

<storycode:copy id="f-88aa201d7149-connectStaticUiParent">
/** Explicit parent attachment; static pages never probe a local engine. */
export function connectStaticUiParent(target: Window, parentOrigin: string, session: ReturnType<typeof createEmbedUiSession>) {
  if (new URL(parentOrigin).origin !== parentOrigin || parentOrigin === "null") throw Error("A concrete parent origin is required");
  let disposed = false;
  const send = (event: string, data: unknown, id?: string) => {
    if (!disposed) target.parent.postMessage({ channel: STATIC_UI_CHANNEL, version: 1, event, data, id }, parentOrigin);
  };
  const listener = (event: MessageEvent) => {
    if (disposed || event.source !== target.parent || event.origin !== parentOrigin) return;
    const message = event.data;
    if (!message || message.channel !== STATIC_UI_CHANNEL || message.version !== 1 || message.op !== "ui") return;
    if (typeof message.id !== "string" || !message.id.length || message.id.length > 128) return;
    void session.invoke(message.data).then(result => send("ui-result", { scope: session.scope, result }, message.id),
      error => send("error", { scope: session.scope, message: String(error) }, message.id));
  };
  target.addEventListener("message", listener);
  send("ready", { scope: session.scope, actor: "embed-parent agent", nativeExecution: false, receiptStorage: "parent must persist ui-receipt events" });
  return { receipt: (value: unknown) => send("ui-receipt", value), dispose() { disposed = true; target.removeEventListener("message", listener); } };
}

</storycode:copy>
<a id="f-88aa201d7149-StaticUiSurface"></a>
### `StaticUiSurface`

<storycode:copy id="f-88aa201d7149-StaticUiSurface">
export function StaticUiSurface({ children }: { children: ReactNode }) {
  const [session, setSession] = useState<ReturnType<typeof createEmbedUiSession> | null>(null);
  useEffect(() => {
    let bridge: ReturnType<typeof connectStaticUiParent> | undefined;
    const current = createEmbedUiSession(receipt => bridge?.receipt(receipt));
    const origin = new URL(location.href).searchParams.get("parentOrigin");
    try { if (origin && window.parent !== window) bridge = connectStaticUiParent(window, origin, current); }
    catch { /* Invalid attachment cannot grant authority; ordinary viewing remains available. */ }
    setSession(current);
    return () => { bridge?.dispose(); current.dispose(); };
  }, []);
  return <UiActionContext.Provider value={session?.registry ?? null}>{children}</UiActionContext.Provider>;
}
</storycode:copy>

### Assemble `apps/web/src/embed/StaticUiSurface.tsx`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/embed/StaticUiSurface.tsx"><storycode:paste select="#f-88aa201d7149-STATIC_UI_CHANNEL"/><storycode:paste select="#f-88aa201d7149-connectStaticUiParent"/><storycode:paste select="#f-88aa201d7149-StaticUiSurface"/></storycode:file>

<a id="output-2aaf7d46e33e"></a>
## `apps/web/src/tour/browserPrediction.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [Browser-local prediction practice uses the shared gate](contract.md).

<a id="f-2aaf7d46e33e-BROWSER_PREDICTION_KEY"></a>
### `BROWSER_PREDICTION_KEY`

Host-private adapter. Never return a portable snapshot, output or raw runtime
diagnostics to ui_act/ui_observe or the ordinary browser runtime history.

<storycode:copy id="f-2aaf7d46e33e-BROWSER_PREDICTION_KEY">
// Host-private adapter. Never return a portable snapshot, output or raw runtime
// diagnostics to ui_act/ui_observe or the ordinary browser runtime history.
import { browserPredictionTransition, loadHickLang, hickLangArtifactHash } from "../editor/hickLang";
import { browserRuntimeAssetHash, createBrowserDebugger } from "../debug/browser/transport";
import type { RuntimeRun } from "../debug/runtimeObservations";
import type { UiActor } from "../lib/uiActions";
import { scenarioSource } from "./scenario";
export const BROWSER_PREDICTION_KEY = "hickory.browserPrediction.v1";
</storycode:copy>
<a id="f-2aaf7d46e33e-BrowserInputs"></a>
### `BrowserInputs`

<storycode:copy id="f-2aaf7d46e33e-BrowserInputs">
export type BrowserInputs = { attempts: number[]; budget: number };
</storycode:copy>
<a id="f-2aaf7d46e33e-BrowserIdentity"></a>
### `BrowserIdentity`

<storycode:copy id="f-2aaf7d46e33e-BrowserIdentity">
export type BrowserIdentity = { source: string; inputs: string; dependencies: Record<string, string>; environment: Record<string, string> };
</storycode:copy>
<a id="f-2aaf7d46e33e-BrowserSeal"></a>
### `BrowserSeal`

<storycode:copy id="f-2aaf7d46e33e-BrowserSeal">
export type BrowserSeal = { receipt: { id: string; generation: number }; scenario: string; identity: BrowserIdentity; actor: UiActor; source: string };
</storycode:copy>
<a id="f-2aaf7d46e33e-BrowserSummary"></a>
### `BrowserSummary`

<storycode:copy id="f-2aaf7d46e33e-BrowserSummary">
export type BrowserSummary = { attempt: string; classification: string; prediction_matched: boolean | null; original_requirement_satisfied: boolean | null; current_certification: boolean; original_expectation_unchanged?: boolean; independent_checks_accepted?: boolean };
</storycode:copy>
<a id="f-2aaf7d46e33e-hash"></a>
### `hash`

<storycode:copy id="f-2aaf7d46e33e-hash">
export async function hash(text: string) { return [...new Uint8Array(await crypto.subtle.digest("SHA-256", new TextEncoder().encode(text)))].map(value => value.toString(16).padStart(2, "0")).join(""); }
</storycode:copy>
<a id="f-2aaf7d46e33e-parseBrowserInputs"></a>
### `parseBrowserInputs`

<storycode:copy id="f-2aaf7d46e33e-parseBrowserInputs">
export function parseBrowserInputs(attempts: string, budget: string): BrowserInputs {
  const values: unknown = JSON.parse(attempts), limit = Number(budget);
  if (!Array.isArray(values) || values.length > 8 || values.some(value => typeof value !== "number" || !Number.isInteger(value) || value < 0 || value > 1000) || !budget.trim() || !Number.isInteger(limit) || limit <= 0 || limit > 1000) throw Error("Use up to eight whole-millisecond durations from 0 to 1000ms and a whole-millisecond budget from 1 to 1000ms");
  return { attempts: values, budget: limit };
}
</storycode:copy>
<a id="f-2aaf7d46e33e-savedBrowserPredictionInputs"></a>
### `savedBrowserPredictionInputs`

Restore question inputs only, never a foreign prediction/rationale or actual.

<storycode:copy id="f-2aaf7d46e33e-savedBrowserPredictionInputs">
/** Restore question inputs only, never a foreign prediction/rationale or actual. */
export function savedBrowserPredictionInputs(): BrowserInputs {
  try {
    const value = JSON.parse(localStorage.getItem("hickory.browserPredictionInputs.v1") ?? "null");
    if (!value || typeof value !== "object" || Object.keys(value).some(key => !["attempts", "budget"].includes(key)) || typeof value.budget !== "number") throw Error("Invalid cached inputs");
    return parseBrowserInputs(JSON.stringify(value.attempts), String(value.budget));
  } catch { return { attempts: [80, 80], budget: 100 }; }
}
</storycode:copy>
<a id="f-2aaf7d46e33e-browserPredictionSource"></a>
### `browserPredictionSource`

<storycode:copy id="f-2aaf7d46e33e-browserPredictionSource">
export function browserPredictionSource(corrected: boolean, inputs: BrowserInputs) {
  const example = scenarioSource(corrected);
  const implementation = example.slice(example.indexOf("function elapsed"), example.indexOf("var actual"));
  return `# Browser-local retry prediction\n\nOne total budget including retries. The implementation is a recorded example; execution is live.\n\n<hick:file path="deadline.js">\n${implementation}console.log(JSON.stringify({target: elapsed(${JSON.stringify(inputs.attempts)}, ${inputs.budget}), short: elapsed([20, 30], ${inputs.budget}), empty: elapsed([], ${inputs.budget})}));\n</hick:file>\n`;
}
</storycode:copy>
<a id="f-2aaf7d46e33e-transition"></a>
### `transition`

<storycode:copy id="f-2aaf7d46e33e-transition">
async function transition<T>(command: Record<string, unknown>, generation = false, current?: () => boolean): Promise<T> {
  await loadHickLang();
  if (!navigator.locks) throw Error("This browser lacks the storage lease required for prediction practice; use a supported browser or native private practice");
  try { return await navigator.locks.request("hickory-browser-prediction-journal", async () => {
    if (current && !current()) throw Error("Practice changed");
    const previous = localStorage.getItem(BROWSER_PREDICTION_KEY) ?? "";
    const operation = generation ? { ...command, generation: previous ? JSON.parse(previous).entries.length : 0 } : command;
    const result = browserPredictionTransition(previous, operation);
    if (result.error || typeof result.snapshot !== "string") throw Error("Browser prediction operation refused; retained history, identity or ordering is unavailable");
    // Synchronous durable admission before dispatch. Quota/permission failure
    // prevents execution; corrupt/missing history is never silently reset.
    localStorage.setItem(BROWSER_PREDICTION_KEY, result.snapshot);
    return result.value as T;
  }); } catch { throw Error("Browser prediction history could not be validated or persisted; execution is refused"); }
}
</storycode:copy>
<a id="f-2aaf7d46e33e-sealBrowserPrediction"></a>
### `sealBrowserPrediction`

<storycode:copy id="f-2aaf7d46e33e-sealBrowserPrediction">
export async function sealBrowserPrediction(corrected: boolean, inputs: BrowserInputs, prediction: string, rationale: string, actor: UiActor, current: () => boolean): Promise<BrowserSeal> {
  const expected = Number(prediction);
  if (!prediction.trim() || !Number.isFinite(expected) || expected < 0 || expected > 8000 || !rationale.trim() || new TextEncoder().encode(rationale).length > 8192) throw Error("Enter a finite prediction from 0 to 8000ms and a rationale before sealing");
  const source = browserPredictionSource(corrected, inputs);
  const [sourceHash, inputsHash, worker, mechanism] = await Promise.all([hash(source), hash(JSON.stringify(inputs)), browserRuntimeAssetHash(), hickLangArtifactHash()]);
  if (!current()) throw Error("Practice changed before sealing; inspect it again");
  const identity = { source: sourceHash, inputs: inputsHash, dependencies: { mechanism }, environment: { worker_entry: worker } };
  const value = await transition<{ receipt: BrowserSeal["receipt"]; scenario: string }>({ operation: "seal", actor, seal: {
    scenario: { family: "retry-total-budget-browser-v1", inputs, requirement_reference: "R1: one total budget including retries", requirement: { kind: "number_at_most", value: inputs.budget }, identity_known: false },
    identity, target: { kind: "output", execution: "deadline-output" }, predicted: { kind: "equals", value: expected }, rationale,
  } }, true, current);
  return { ...value, actor, identity, source };
}
</storycode:copy>
<a id="f-2aaf7d46e33e-browserPredictionHistory"></a>
### `browserPredictionHistory`

<storycode:copy id="f-2aaf7d46e33e-browserPredictionHistory">
export async function browserPredictionHistory(seal: BrowserSeal, actor: UiActor): Promise<BrowserSummary[]> {
  const history = await transition<{ attempts: BrowserSummary[] }>({ operation: "history", current: seal.identity, actor, scenarios: [seal.scenario] });
  return history.attempts.map(value => ({ attempt: value.attempt, classification: value.classification, prediction_matched: value.prediction_matched, original_requirement_satisfied: value.original_requirement_satisfied, current_certification: false, original_expectation_unchanged: value.original_expectation_unchanged === true }));
}
</storycode:copy>
<a id="f-2aaf7d46e33e-checkRuntimeAssets"></a>
### `checkRuntimeAssets`

<storycode:copy id="f-2aaf7d46e33e-checkRuntimeAssets">
async function checkRuntimeAssets(seal: BrowserSeal) {
  const [worker, mechanism] = await Promise.all([browserRuntimeAssetHash(), hickLangArtifactHash()]);
  if (worker !== seal.identity.environment.worker_entry || mechanism !== seal.identity.dependencies.mechanism) throw Error("Browser runtime assets changed; reseal before execution. Imported dependency/browser closure remains unverified");
}
</storycode:copy>
<a id="f-2aaf7d46e33e-runBrowserPrediction"></a>
### `runBrowserPrediction`

<storycode:copy id="f-2aaf7d46e33e-runBrowserPrediction">
export async function runBrowserPrediction(seal: BrowserSeal, inputs: BrowserInputs, actor: UiActor, signal: AbortSignal, current: () => boolean): Promise<BrowserSummary> {
  if (seal.actor.kind !== actor.kind || seal.actor.id !== actor.id || !current() || signal.aborted) throw Error("The current predictor must run its own current seal");
  await checkRuntimeAssets(seal);
  // One same-origin runtime lease. Another page cannot recover or race a live
  // practice run. Journal transitions use their separate short storage lease.
  return navigator.locks.request("hickory-browser-prediction-runtime", { ifAvailable: true }, async lease => {
    if (!lease) throw Error("Another browser prediction runtime lease is active; inspect it before retrying");
    if (!current() || signal.aborted) throw Error("Practice changed before admission");
    const admission = await transition<{ receipt: { id: string }; dispatch: boolean }>({ operation: "admit", seal: seal.receipt.id, request: crypto.randomUUID(), identity: seal.identity, actor });
    if (!admission.dispatch) throw Error("This admission already dispatched; inspect retained history");
    let backend: ReturnType<typeof createBrowserDebugger> | undefined;
    try {
      if (!current() || signal.aborted) throw Error("Practice changed before dispatch");
      const { run, output } = await new Promise<{ run: RuntimeRun; output: string }>((resolve, reject) => {
        let output = "", settled = false;
        const finish = (error?: boolean, run?: RuntimeRun) => { if (settled) return; settled = true; clearTimeout(timer); signal.removeEventListener("abort", abort); if (error || !run) reject(Error("Browser execution unavailable; diagnostics remain withheld")); else resolve({ run, output }); };
        const abort = () => finish(true);
        const timer = setTimeout(() => finish(true), 10000);
        signal.addEventListener("abort", abort, { once: true });
        backend = createBrowserDebugger({ readDocument: () => ({ source: seal.source, revision: seal.identity.source }), onOutput: text => { if (output.length + text.length > 65536) finish(true); else output += text; }, onRuntime: run => { if (run.status !== "running") finish(run.status !== "completed", run); } });
        backend.client.on(event => { if (event.event === "failed" || event.event === "stopped") finish(true); });
        backend.client.start("browser-prediction.md", [], "deadline.js");
      });
      await checkRuntimeAssets(seal);
      if (!current() || signal.aborted || run.sources.find(source => source.id === "document")?.revision !== seal.identity.source || run.truncated || run.missing_artifacts.length) throw Error("Browser result identity unavailable");
      const actual: unknown = JSON.parse(output.trim());
      if (!actual || typeof actual !== "object") throw Error("Browser result unavailable");
      const fields = actual as Record<string, unknown>;
      if (Object.keys(fields).sort().join(",") !== "empty,short,target" || [fields.target, fields.short, fields.empty].some(value => typeof value !== "number" || !Number.isFinite(value))) throw Error("Browser result unavailable");
      const accepted = fields.short === Math.min(50, inputs.budget) && fields.empty === 0 && Number(fields.target) >= 0;
      const summary = await transition<BrowserSummary>({ operation: "finish", attempt: admission.receipt.id, execution_accepted: accepted, result: { identity: seal.identity, target: { kind: "output", execution: "deadline-output" }, actual: fields.target, evidence: { runtime: run.id, source: seal.identity.source, short: fields.short, empty: fields.empty, scope: "Live interpreter; browser-local history, incomplete environment/prior-exposure knowledge" } } });
      return { ...summary, current_certification: false, independent_checks_accepted: accepted };
    } catch {
      // The worker is terminated before an interrupted result is recorded. An
      // uncertain/crashed page remains non-passing rather than auto-reexecuting.
      backend?.dispose();
      await transition({ operation: "interrupt", attempt: admission.receipt.id }).catch(() => {});
      throw Error("Browser execution could not be retained; values/diagnostics remain withheld. Inspect history before retrying");
    } finally { backend?.dispose(); }
  });
}
</storycode:copy>
<a id="f-2aaf7d46e33e-promoteBrowserPrediction"></a>
### `promoteBrowserPrediction`

<storycode:copy id="f-2aaf7d46e33e-promoteBrowserPrediction">
export async function promoteBrowserPrediction(seal: BrowserSeal, attempt: string, actor: UiActor): Promise<BrowserSummary> {
  try { await transition<BrowserSummary>({ operation: "promote", attempt, current: seal.identity, actor }); } catch { throw Error("Promotion blocked: browser-local prior exposure and full environment identity are Unknown; passing comparisons cannot grant host certification. Use a fresh native private practice for an authoritative check"); }
  throw Error("User-controlled browser history cannot grant host certification");
}
</storycode:copy>
<a id="f-2aaf7d46e33e-inspectBrowserPrediction"></a>
### `inspectBrowserPrediction`

<storycode:copy id="f-2aaf7d46e33e-inspectBrowserPrediction">
export async function inspectBrowserPrediction(attempt: string, actor: UiActor) {
  if (actor.kind !== "human") throw Error("A human inspects private browser results in a separate window");
  return transition<{ actual: unknown; evidence: unknown }>({ operation: "reveal", attempt, actor });
}

</storycode:copy>
<a id="f-2aaf7d46e33e-recoverBrowserPrediction"></a>
### `recoverBrowserPrediction`

Caller holds no live worker. A cross-tab runtime lease is the independent witness.

<storycode:copy id="f-2aaf7d46e33e-recoverBrowserPrediction">
/** Caller holds no live worker. A cross-tab runtime lease is the independent witness. */
export async function recoverBrowserPrediction(seal: BrowserSeal, actor: UiActor): Promise<{ interrupted: number; execution_replayed: false; current_certification: false }> {
  if (!navigator.locks) throw Error("This browser cannot establish a runtime recovery lease");
  return navigator.locks.request("hickory-browser-prediction-runtime", { ifAvailable: true }, async lease => {
    if (!lease) throw Error("Another browser prediction runtime lease is active; worker absence is unconfirmed, so recovery is refused");
    return transition({ operation: "recover", actor, scenarios: [seal.scenario] });
  });
}
</storycode:copy>

### Assemble `apps/web/src/tour/browserPrediction.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/tour/browserPrediction.ts"><storycode:paste select="#f-2aaf7d46e33e-BROWSER_PREDICTION_KEY"/><storycode:paste select="#f-2aaf7d46e33e-BrowserInputs"/><storycode:paste select="#f-2aaf7d46e33e-BrowserIdentity"/><storycode:paste select="#f-2aaf7d46e33e-BrowserSeal"/><storycode:paste select="#f-2aaf7d46e33e-BrowserSummary"/><storycode:paste select="#f-2aaf7d46e33e-hash"/><storycode:paste select="#f-2aaf7d46e33e-parseBrowserInputs"/><storycode:paste select="#f-2aaf7d46e33e-savedBrowserPredictionInputs"/><storycode:paste select="#f-2aaf7d46e33e-browserPredictionSource"/><storycode:paste select="#f-2aaf7d46e33e-transition"/><storycode:paste select="#f-2aaf7d46e33e-sealBrowserPrediction"/><storycode:paste select="#f-2aaf7d46e33e-browserPredictionHistory"/><storycode:paste select="#f-2aaf7d46e33e-checkRuntimeAssets"/><storycode:paste select="#f-2aaf7d46e33e-runBrowserPrediction"/><storycode:paste select="#f-2aaf7d46e33e-promoteBrowserPrediction"/><storycode:paste select="#f-2aaf7d46e33e-inspectBrowserPrediction"/><storycode:paste select="#f-2aaf7d46e33e-recoverBrowserPrediction"/></storycode:file>

## Implementation review

### Implementation

- [apps/web/src/components/BrowserPredictionPractice.tsx](contract.md#output-49c7b898ffd6)
- [apps/web/src/components/WorkflowTour.tsx](contract.md#output-39bbc44cf019)
- [apps/web/src/debug/browser/runtime.ts](../../../mechanisms/apps/web/src/debug/browser/runtime.ts.md#output-6c24fa24bf81)
- [apps/web/src/debug/browser/transport.ts](../../../mechanisms/apps/web/src/debug/browser/transport.ts.md#output-4357b3feff1c)
- [apps/web/src/editor/generated/hick-lang/hick_lang.d.ts](contract.md#output-4412ea75a6ea)
- [apps/web/src/editor/generated/hick-lang/hick_lang.js](contract.md#output-c53582515263)
- [apps/web/src/editor/generated/hick-lang/hick_lang_bg.wasm](../../../shared/files/apps/web/src/editor/generated/hick-lang/hick_lang_bg.wasm.md)
- [apps/web/src/editor/generated/hick-lang/hick_lang_bg.wasm.d.ts](contract.md#output-3b08e5ec33ba)
- [apps/web/src/editor/hickLang.ts](../../../mechanisms/apps/web/src/editor/hickLang.ts.md#output-1cd2ae06e1c5)
- [apps/web/src/embed/StaticUiSurface.tsx](contract.md#output-88aa201d7149)
- [apps/web/src/embed/uiHostSession.ts](../../../mechanisms/apps/web/src/embed/uiHostSession.ts.md#output-585a885211aa)
- [apps/web/src/lib/uiActions.ts](../../../mechanisms/apps/web/src/lib/uiActions.ts.md#output-0cc430d9eb03)
- [apps/web/src/lib/uiReading.ts](../../../mechanisms/apps/web/src/lib/uiReading.ts.md#output-fb2fe531fbd6)
- [apps/web/src/tour/browserPrediction.ts](contract.md#output-2aaf7d46e33e)
- [apps/web/src/views/LandingView.tsx](../../../mechanisms/apps/web/src/views/LandingView.tsx.md#output-e86ad48a0990)
- [crates/hick-lang-wasm/src/lib.rs](../../../mechanisms/crates/hick-lang-wasm/src/lib.rs.md#output-9a9e430ed3f3)
- [crates/hick-lang-wasm/src/prediction.rs](../../../mechanisms/crates/hick-lang-wasm/src/prediction.rs.md#output-d27c09b74da1)
- [crates/hick-observe/src/lib.rs](../../../mechanisms/crates/hick-observe/src/lib.rs.md#output-022e74f036b0)
- [crates/hick-observe/src/prediction.rs](../../../mechanisms/crates/hick-observe/src/prediction.rs.md#output-f5f25bde8505)

### Tests and fixtures

- [apps/web/e2e/staticUiParity.spec.ts](../../../mechanisms/apps/web/e2e/staticUiParity.spec.ts.md#output-b95b6714a2a7)
- [apps/web/src/components/BrowserPredictionPractice.test.tsx](../../../mechanisms/apps/web/src/components/BrowserPredictionPractice.test.tsx.md#output-32e2b9b729a8)
- [apps/web/src/lib/uiActions.test.ts](../../../mechanisms/apps/web/src/lib/uiActions.ts.md#output-f5b85cef7913)
- [apps/web/src/tour/browserPrediction.test.ts](../../../mechanisms/apps/web/src/tour/browserPrediction.test.ts.md#output-5eb82555b921)
- [crates/hick-observe/src/prediction_tests.rs](../../../mechanisms/crates/hick-observe/src/prediction_tests.rs.md#output-a797b872866a)

### Evidence status

- Implementation review: not verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.

</storycode:doc>
