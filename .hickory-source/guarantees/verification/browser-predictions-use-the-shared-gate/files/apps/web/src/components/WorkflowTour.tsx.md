<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# apps/web/src/components/WorkflowTour.tsx

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.

- [The engineer tour distinguishes replay from execution](../../../../../../../authoring/the-engineer-tour-distinguishes-replay-from-execution/contract.md)
- [Browser-local prediction practice uses the shared gate](../../../../../contract.md)

Captured SHA-256: `bcbfdf0e57d7f60afd0b032f1bb27482f7e17086c88e916a886cb749baa55b4d`.

<book:entry path="apps/web/src/components/WorkflowTour.tsx">
import { BrowserPredictionPractice } from "./BrowserPredictionPractice";
import { UiLink } from "./UiLink";
import { lazy, Suspense, useContext, useRef, useLayoutEffect } from "react";
const NativeWorkflowTour = lazy(() => import("./NativeWorkflowTour").then(m => ({ default: m.NativeWorkflowTour })));
import type { EditorView } from "@codemirror/view";
import { emit } from "../analytics/events";
import { useEffect, useState } from "react";
import { UiActionContext, useUiCommands, useUiRevision } from "../lib/uiActionBridge";
import { useUiReading } from "../lib/uiReading";
import type { UiCommand, UiValue } from "../lib/uiActions";
import { DebuggableDocument } from "../embed/DebuggableDocument";
import { scenarioSource, TOUR_STEPS } from "../tour/scenario";
import "./WorkflowTour.css";
const KEY = "hickory.aiEngineerTour";
export function WorkflowTour(props: { surface: "site" | "app"; predictionOnly?: boolean; onPredictionPractice?: () => void; onOpen?: (path: string, line?: number) => void; focused?: { path?: string; source?: string; view?: EditorView | null } }) {
  return props.surface === "app" ? <Suspense fallback={<p>Loading native workflow…</p>}><NativeWorkflowTour onOpen={props.onOpen} focused={props.focused} /></Suspense> : <BrowserWorkflowTour surface="site" predictionOnly={props.predictionOnly} onPredictionPractice={props.onPredictionPractice} />;
}
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
</book:entry>

</book:doc>
