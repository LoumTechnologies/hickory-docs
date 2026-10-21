import { UiLink } from "./UiLink";
import { lazy, Suspense } from "react";
const NativeWorkflowTour = lazy(() => import("./NativeWorkflowTour").then(m => ({ default: m.NativeWorkflowTour })));
import type { EditorView } from "@codemirror/view";
import { emit } from "../analytics/events";
import { useEffect, useState } from "react";
import { useUiCommands, useUiRevision } from "../lib/uiActionBridge";
import { useUiReading } from "../lib/uiReading";
import type { UiCommand, UiValue } from "../lib/uiActions";
import { DebuggableDocument } from "../embed/DebuggableDocument";
import { scenarioSource, TOUR_STEPS } from "../tour/scenario";
import "./WorkflowTour.css";
const KEY = "hickory.aiEngineerTour";
export function WorkflowTour(props: { surface: "site" | "app"; onOpen?: (path: string, line?: number) => void; focused?: { path?: string; source?: string; view?: EditorView | null } }) {
  return props.surface === "app" ? <Suspense fallback={<p>Loading native workflow…</p>}><NativeWorkflowTour onOpen={props.onOpen} focused={props.focused} /></Suspense> : <BrowserWorkflowTour surface="site" />;
}
function BrowserWorkflowTour({ surface }: { surface: "site" | "app" }) {
  const [open, setOpen] = useState(false);
  const [step, setStep] = useState(() => { const saved = Number(localStorage.getItem(KEY)); return Number.isInteger(saved) && saved >= 0 && saved < TOUR_STEPS.length ? saved : 0; });
  const [view, setView] = useState<EditorView | null>(null);
  const [objection, setObjection] = useState("");
  const [failed, setFailed] = useState(false);
  const [passed, setPassed] = useState(false);
  useEffect(() => { localStorage.setItem(KEY, String(step)); }, [step]);
  useEffect(() => { if (open && surface === "site") emit({ name: "demo_engaged", demo_id: "ai-engineer-tour", step: String(step) }); }, [open, step, surface]);
  const corrected = step >= 4;
  const source = scenarioSource(corrected);
  useEffect(() => {
    if (!view) return;
    const needle = step >= 3 ? "function elapsed" : step === 1 ? "The deadline bounds" : "Meeting evidence";
    const anchor = view.state.doc.toString().indexOf(needle);
    if (anchor >= 0) view.dispatch({ selection: { anchor }, scrollIntoView: true });
  }, [view, step]);
  const canContinue = step === 2 ? !!objection.trim() : step === 3 ? failed : step === 4 ? passed : true;
  const restart = () => { setStep(0); setFailed(false); setPassed(false); setObjection(""); };
  const download = () => {
    const url = URL.createObjectURL(new Blob([scenarioSource(true)], { type: "text/markdown" }));
    const link = document.createElement("a"); link.href = url; link.download = "deadline-spec.md"; link.click(); setTimeout(() => URL.revokeObjectURL(url), 1000);
  };
  const revision = useUiRevision(() => [open, step, objection, failed, passed]);
  useUiReading("browser workflow tour", open ? { open, step, title: TOUR_STEPS[step].title, text: TOUR_STEPS[step].text, objection, can_continue: canContinue,
    counterexample_observed: failed, correction_observed: passed, proposal: "recorded example", execution: "live browser", human_attestation: false } : { open });
  const command = (key: string, label: string, enabled: boolean, effect: (args: Record<string, UiValue>) => void, parameters: UiCommand["parameters"] = {}) => ({ key: `browser-tour:${key}`, command: {
    label, parent: "browser-tour", subject: "browser-tour", parameters, prerequisites: [], read: () => ({ revision: revision(), enabled }),
    execute: async (args: Record<string, UiValue>) => { effect(args); return { status: "completed" as const, value: { controller_update_requested: true, human_attestation: false, execution_certified: false } }; },
  } });
  useUiCommands([
    command("visibility", "Set browser workflow tour visibility", true, args => setOpen(args.open as boolean), { open: { type: "boolean" } }),
    ...(open ? [command("back", "Previous browser workflow step", step > 0, () => setStep(step - 1)),
      command("next", "Continue browser workflow step", canContinue && step < TOUR_STEPS.length - 1, () => setStep(step + 1)),
      command("restart", "Restart browser workflow tour", true, restart), command("close", "Skip and close browser workflow tour", true, () => setOpen(false)),
      command("download", "Download browser workflow executable spec", true, download),
      ...(step === 2 ? [command("objection", "Set browser workflow practice objection", true, args => setObjection(args.text as string), { text: { type: "string", maxLength: 65536 } })] : [])] : []),
  ]);
  return <div className="workflow-tour-launcher">
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
        <button onClick={download}>Download executable spec</button>
      </div>
      <DebuggableDocument key={corrected ? "corrected" : "original"} path="deadline-spec.md" source={source} revision={corrected ? "corrected-example" : "original-example"} readOnly onViewReady={setView}
        onExecutionResult={result => { if (result.revision === "original-example" && result.exitCode !== 0) setFailed(true); if (result.revision === "corrected-example" && result.exitCode === 0) setPassed(true); }} />
      {step === 3 && !failed && <p role="status">Run the original code to observe the counterexample.</p>}
      {step === 4 && !passed && <p role="status">Run the corrected code to verify the checks.</p>}
      {step === 5 && <p>Complete native workflow: import a transcript, challenge the real claim, explicitly update stale agent context, send a real investigation, inspect source-linked runtime/debug evidence, profile Python and rerun correctness, then resolve the thread yourself. The desktop tour creates a separate practice Git repository. These native actions and actual model calls are not simulated by this homepage. <UiLink href="./snapshot.html?index=publications/index.json">Inspect published verification</UiLink>.</p>}
      {step === 5 && <p>Local verification: <code>hick weave deadline-spec.md</code>, then <code>hick test deadline-spec.md</code> (Node.js required). In the workspace, enable author colors in Lineage settings; use Review threads to retain an objection and its resolution.</p>}
    </section>}
  </div>;
}
