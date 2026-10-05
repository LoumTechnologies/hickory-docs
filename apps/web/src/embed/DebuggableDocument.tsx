import { useEffect, useMemo, useRef, useState } from "react";
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
}
export function DebuggableDocument(props: DebuggableDocumentProps) {
  const current = useRef(props); current.current = props;
  const [backend, setBackend] = useState<ReturnType<typeof createBrowserDebugger> | null>(null);
  const [output, setOutput] = useState("");
  const [runSource, setRunSource] = useState<string | null>(null);
  const staleRef = useRef(false);
  const [runRevision, setRunRevision] = useState<string | null>(null);
  const [view, setView] = useState<EditorView | null>(null);
  const [watch, setWatch] = useState("");
  const [children, setChildren] = useState<Record<number, Variable[]>>({});
  useEffect(() => {
    const local = createBrowserDebugger({
      readDocument: () => { setRunSource(current.current.source); return { source: current.current.source, revision: current.current.revision }; },
      onOutput: (text) => setOutput((previous) => previous + text),
      onRevision: (revision) => { setRunRevision(revision); setOutput(""); },
    });
    setBackend(local);
    const off = local.client.on((event) => {
      if (event.event === "children") setChildren((previous) => ({ ...previous, [event.reference]: event.variables }));
      if (event.event === "stopped" || event.event === "ended") setChildren({});
    });
    return () => { off(); local.dispose(); };
  }, []);
  const initial = useRef({ source: props.source, breakpoint: props.startPausedAt });
  const startedOn = useRef<typeof backend>(null);
  const debug = useDebuggerOver(backend?.client ?? null, props.path,
    initial.current.breakpoint === undefined ? [] : [initial.current.breakpoint]);
  useEffect(() => {
    if (!backend || !view || initial.current.breakpoint === undefined || startedOn.current === backend) return;
    startedOn.current = backend;
    // Never start unexpectedly over a draft typed while assets were loading.
    if (current.current.source === initial.current.source) debug.start();
  }, [backend, view, debug.start]);
  const debugRef = useRef(debug); debugRef.current = debug;
  const extensions = useMemo(() => debugEditor({
    onToggleBreakpoint: (line) => { if (!staleRef.current) debugRef.current.toggleBreakpoint(line); },
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
  function variables(values: Variable[], depth = 0) {
    return <ul>{values.map((variable) => <li key={variable.name}>
      {variable.variables_reference > 0 && depth < 5 && <button type="button" aria-label={`Expand ${variable.name}`}
        onClick={() => { if (session) backend?.client.children(session, variable.variables_reference); }}>▸</button>}
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
    <DebugStrip status={debug.status} message={debug.message} capabilities={debug.capabilities}
      frames={debug.frames} selectedFrame={debug.selectedFrame} watches={debug.watches} exitCode={debug.exitCode}
      onStart={start} onStop={debug.stop} onStep={debug.step} onSelectFrame={debug.selectFrame}
      onJumpHere={() => {}} onAddWatch={() => document.getElementById(`watch-${props.path}`)?.focus()}
      exceptionFilters={debug.exceptionFilters} onToggleExceptionFilter={debug.toggleExceptionFilter} onRemoveWatch={debug.removeWatch} />
    <DocumentEmbed {...props} extensions={extensions} onViewReady={setView} />
    {debug.status === "paused" && <div className="browser-debug-inspection">
      <p>Paused on line {(debug.pausedLine ?? 0) + 1}{stale ? " of the previous revision" : ""}</p>
      <h3>Variables</h3>
      <div aria-label="Live variables">{variables(debug.variables)}</div>
      <form onSubmit={(event) => { event.preventDefault(); if (watch.trim()) { debug.addWatch(watch.trim()); setWatch(""); } }}>
        <label>Read-only watch <input id={`watch-${props.path}`} aria-label="Watch expression" value={watch} onChange={(event) => setWatch(event.target.value)} /></label>
        <button type="submit">Watch</button>
      </form>
      <ul aria-label="Watch values">{debug.watches.map((value) => <li key={value.expression}><code>{value.expression} = {value.value ?? "…"}</code></li>)}</ul>
    </div>}
    <pre className="browser-debug-output" aria-label="Program output" aria-live="polite">{output || "Output appears here when the program runs."}</pre>
  </div>;
}
