import { inlineEvaluationOwner } from "../debug/inlineEval";
import { useUiCommands, useUiRevision } from "../lib/uiActionBridge";
import { useUiReading } from "../lib/uiReading";
import { RuntimeHistory, useBrowserRuns } from "../debug/RuntimeRuns";
import type { RuntimeRun } from "../debug/runtimeObservations";
import { useCallback, useEffect, useId, useMemo, useRef, useState } from "react";
import type { EditorView } from "@codemirror/view";
import { DocumentEmbed } from "./DocumentEmbed";
import type { DocumentEmbedProps } from "./DocumentEmbed";
import { createBrowserDebugger } from "../debug/browser/transport";
import { useDebuggerOver } from "../debug/useDebugger";
import { DebugStrip } from "../debug/DebugStrip";
import { debugEditor, debugStateEffects } from "../debug/cmDebug";
import type { Variable } from "../debug/client";
import "./debug.css";

/** The product's local browser backend, reusable outside the homepage. */
export interface DebuggableDocumentProps extends DocumentEmbedProps {
  /** Explicit opt-in: start once on mount at this 0-based document breakpoint. */
  startPausedAt?: number;
  onRuntime?: (run: RuntimeRun) => void;
  onObservation?: (reading: import("../debug/observations").ObservationReading) => void;
  onExecutionResult?: (result: { revision: string; exitCode: number }) => void;
}
export function DebuggableDocument(props: DebuggableDocumentProps) {
  const runtimeHistory = useBrowserRuns(props.path);
  const runtimeHistoryRef = useRef(runtimeHistory); runtimeHistoryRef.current = runtimeHistory;
  const capturedRevision = useRef(props.revision);
  const current = useRef(props); current.current = props;
  const [backend, setBackend] = useState<ReturnType<typeof createBrowserDebugger> | null>(null);
  const [output, setOutput] = useState("");
  const [runSource, setRunSource] = useState<string | null>(null);
  const staleRef = useRef(false);
  const [runRevision, setRunRevision] = useState<string | null>(null);
  const [view, setView] = useState<EditorView | null>(null);
  const ready = useCallback((value: EditorView | null) => { setView(value); current.current.onViewReady?.(value); }, []);
  const [watch, setWatch] = useState("");
  const [children, setChildren] = useState<Record<number, Variable[]>>({});
  useEffect(() => {
    const local = createBrowserDebugger({
      readDocument: () => { capturedRevision.current = current.current.revision; setRunSource(current.current.source); return { source: current.current.source, revision: current.current.revision }; },
      onRuntime: (run) => { runtimeHistoryRef.current.observe(run); current.current.onRuntime?.(run); },
      onOutput: (text) => setOutput((previous) => previous + text),
      onRevision: (revision) => { setRunRevision(revision); setOutput(""); },
    });
    setBackend(local);
    const off = local.client.on((event) => {
      if (event.event === "finished" && typeof event.exit_code === "number") current.current.onExecutionResult?.({ revision: capturedRevision.current, exitCode: event.exit_code });
      if (event.event === "children") setChildren((previous) => ({ ...previous, [event.reference]: event.variables }));
      if (event.event === "stopped" || event.event === "ended") setChildren({});
    });
    return () => { off(); local.dispose(); };
  }, []);
  const initial = useRef({ source: props.source, breakpoint: props.startPausedAt });
  const startedOn = useRef<typeof backend>(null);
  const debug = useDebuggerOver(backend?.client ?? null, props.path,
    initial.current.breakpoint === undefined ? [] : [initial.current.breakpoint], "new", [props.revision, props.source]);
  useEffect(() => {
    if (!backend || !view || initial.current.breakpoint === undefined || startedOn.current === backend) return;
    startedOn.current = backend;
    // Never start unexpectedly over a draft typed while assets were loading.
    if (current.current.source === initial.current.source) debug.start();
  }, [backend, view, debug.start]);
  useEffect(() => { if (debug.reading) current.current.onObservation?.(debug.reading); }, [debug.reading]);
  const debugRef = useRef(debug); debugRef.current = debug;
  const extensions = useMemo(() => debugEditor({
    onToggleBreakpoint: (line) => { if (!staleRef.current) debugRef.current.toggleBreakpoint(line); },
    evaluationOwner: () => staleRef.current ? null : inlineEvaluationOwner(debugRef.current),
    onEvaluate: (expression) => staleRef.current ? Promise.resolve(null) : debugRef.current.query(expression),
    onAddWatch: (expression) => debugRef.current.addWatch(expression),
    onSelectFrame: (frame) => debugRef.current.selectFrame(frame),
  }), []);
  const stale = runRevision !== null && (runRevision !== props.revision || runSource !== props.source) && ["running", "paused", "starting"].includes(debug.status);
  staleRef.current = stale;
  useEffect(() => {
    view?.dispatch({ effects: debugStateEffects(stale ? { ...debug, breakpoints: [], pausedLine: null, frames: [], variables: [], watches: [] } : debug) });
  }, [view, debug, stale]);
  const start = () => { debug.stop(); setOutput(""); debug.start(); };
  const [session, setSession] = useState<string | null>(null);
  useEffect(() => backend?.client.on((event) => {
    if (event.event === "started") setSession(event.session);
    if (event.event === "finished" || event.event === "ended") setSession(null);
  }), [backend]);
  const parent = `browser-document:${props.path}:${useId()}`;
  const actionRevision = useUiRevision(() => [props.revision, props.source, runRevision, runSource, debug.status, debug.reading?.scope, session, watch, children]);
  const expandable = new Set<number>();
  const collect = (values: Variable[], depth = 0) => { for (const variable of values) if (variable.variables_reference > 0 && depth < 5) { expandable.add(variable.variables_reference); if (children[variable.variables_reference]) collect(children[variable.variables_reference], depth + 1); } };
  collect(debug.variables);
  const expandVariable = (reference: number) => { if (!session || stale || !expandable.has(reference)) throw Error("Variable is unavailable at the current source/stop"); backend?.client.children(session, reference); };
  const submitWatch = () => { if (!stale && watch.trim()) { debug.addWatch(watch.trim()); setWatch(""); } };
  useUiReading("browser document runtime", { path: props.path, revision: props.revision, run_revision: runRevision, stale, status: debug.status, output, watch_draft: debug.status === "paused" ? watch : null, expanded_variables: debug.status === "paused" ? children : {} });
  useUiCommands([
    { key: `${parent}:restart`, command: { label: `Restart browser document (${props.path})`, parent, subject: parent, parameters: {}, prerequisites: [], read: () => ({ revision: actionRevision(), enabled: !!backend && debug.status !== "starting" }), execute: async () => { start(); return { status: "completed", value: { restart_requested: true, execution_result: "Observe the actual runtime output and receipt" } }; } } },
    { key: `${parent}:pause`, command: { label: `Pause browser document (${props.path})`, parent, subject: parent, parameters: {}, prerequisites: [], read: () => ({ revision: actionRevision(), enabled: !!backend && debug.status === "running" }), execute: async () => { backend?.pause(); return { status: "completed", value: { pause_requested: true } }; } } },
    ...(debug.status === "paused" ? [
      { key: `${parent}:watch`, command: { label: `Set browser watch draft (${props.path})`, parent, subject: parent, parameters: { expression: { type: "string" as const, maxLength: 8192 } }, prerequisites: [], read: () => ({ revision: actionRevision(), enabled: true }), execute: async (args: Record<string, import("../lib/uiActions").UiValue>) => { setWatch(args.expression as string); return { status: "completed" as const, value: { draft_changed: true } }; } } },
      { key: `${parent}:submit-watch`, command: { label: `Submit browser watch draft (${props.path})`, parent, subject: parent, parameters: {}, prerequisites: [], read: () => ({ revision: actionRevision(), enabled: !stale && !!watch.trim() }), execute: async () => { submitWatch(); return { status: "completed" as const, value: { watch_requested: true } }; } } },
      ...[...expandable].map(reference => ({ key: `${parent}:variable:${reference}`, command: { label: `Expand browser variable ${reference} (${props.path})`, parent, subject: parent, parameters: {}, prerequisites: [], read: () => ({ revision: actionRevision(), enabled: !stale && !!session }), execute: async () => { expandVariable(reference); return { status: "completed" as const, value: { expansion_requested: true, reference } }; } } })),
    ] : []),
  ]);
  function variables(values: Variable[], depth = 0) {
    return <ul>{values.map((variable) => <li key={variable.name}>
      {variable.variables_reference > 0 && depth < 5 && <button type="button" disabled={!session || stale} aria-label={`Expand ${variable.name}`}
        onClick={() => { expandVariable(variable.variables_reference); }}>▸</button>}
      <code>{variable.name} = {variable.value}</code>
      {depth < 5 && children[variable.variables_reference] && variables(children[variable.variables_reference], depth + 1)}
    </li>)}</ul>;
  }
  return <div className="hickory-browser-debug" data-status={debug.status}>
    <div className="browser-debug-actions">
      <button type="button" disabled={!backend || debug.status === "starting"} onClick={start} aria-label="Debug document">{debug.status === "idle" ? "Debug" : "Restart"}</button>
      {debug.status === "running" && <button type="button" onClick={() => backend?.pause()}>Pause</button>}
      {stale && <span role="status">Running previous revision {runRevision}. <button type="button" onClick={start}>Restart with edits</button></span>}
    </div>
    <DebugStrip reading={debug.reading} readingChanged={stale} status={debug.status} message={debug.message} capabilities={debug.capabilities}
      frames={debug.frames} selectedFrame={debug.selectedFrame} watches={debug.watches} exitCode={debug.exitCode}
      onStart={start} onStop={debug.stop} onStep={debug.step} onSelectFrame={debug.selectFrame}
      onJumpHere={() => {}} onAddWatch={() => document.getElementById(`watch-${props.path}`)?.focus()}
      exceptionFilters={debug.exceptionFilters} onToggleExceptionFilter={debug.toggleExceptionFilter} onRemoveWatch={debug.removeWatch} />
    <DocumentEmbed {...props} extensions={extensions} onViewReady={ready} />
    <RuntimeHistory runs={runtimeHistory.runs} source={props.source} view={view} onPin={runtimeHistory.pin} onDelete={runtimeHistory.remove} />
    {runtimeHistory.error && <p role="alert">Runtime retention unavailable: {runtimeHistory.error}</p>}
    {debug.status === "paused" && <div className="browser-debug-inspection">
      <p>Paused on line {(debug.pausedLine ?? 0) + 1}{stale ? " of the previous revision" : ""}</p>
      <h3>Variables</h3>
      <div aria-label="Live variables">{variables(debug.variables)}</div>
      <form onSubmit={(event) => { event.preventDefault(); submitWatch(); }}>
        <label>Read-only watch <input id={`watch-${props.path}`} aria-label="Watch expression" value={watch} onChange={(event) => setWatch(event.target.value)} /></label>
        <button type="submit" disabled={stale || !watch.trim()}>Watch</button>
      </form>
      <ul aria-label="Watch values">{debug.watches.map((value) => <li key={value.expression}><code>{value.expression} = {value.value ?? "…"}</code></li>)}</ul>
    </div>}
    <pre className="browser-debug-output" aria-label="Program output" aria-live="polite">{output || "Output appears here when the program runs."}</pre>
  </div>;
}
