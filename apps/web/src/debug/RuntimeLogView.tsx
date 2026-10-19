import { useEffect, useMemo, useState } from "react";
import { EditorView } from "@codemirror/view";
import { EditingSurface } from "../embed/EditingSurface";
import { RibbonOverlay } from "../shell/Ribbons";
import { openReview, sha256 } from "../lib/reviews";
import type { RuntimeRun, RuntimeEvent, RuntimeLocation } from "./runtimeObservations";
import { logLinks, outputLines, printable } from "./runtimeLog";
import { RUNTIME_REVEAL, revealRuntimeEvent } from "./runtimeNavigation";
import "./runtimeLog.css";

export function RuntimeLog({ run }: { run: RuntimeRun }) {
  const [open, setOpen] = useState(false);
  const [pair, setPair] = useState("");
  const [sourceId, setSourceId] = useState("");
  const [pendingLine, setPendingLine] = useState<number | null>(null);
  const [style, setStyle] = useState<"bands" | "braces">("braces");
  const [selected, setSelected] = useState("");
  const [container, setContainer] = useState<HTMLDivElement | null>(null);
  const [logView, setLogView] = useState<EditorView | null>(null);
  const [sourceView, setSourceView] = useState<EditorView | null>(null);
  const pairs = [...new Set(run.events.flatMap(event => event.observation.kind === "output" ? [JSON.stringify([event.observation.stream, event.observation.category])] : []))];
  const chosen = pair && pairs.includes(pair) ? pair : pairs[0];
  const [stream, category] = chosen ? JSON.parse(chosen) as string[] : ["stdout", "log"];
  const lines = useMemo(() => outputLines(run, stream, category), [run, stream, category]);
  const logPath = `runtime-log-${run.id}.txt`;
  const source = run.sources.find(source => source.id === sourceId) ?? run.sources[0];
  const sourcePath = `runtime-source-${run.id}-${encodeURIComponent(source.id)}.txt`;
  const sourceText = new TextDecoder().decode(new Uint8Array(source.bytes));
  const logText = lines.map(line => printable(line.bytes).replace(/\n$/, "") + (line.terminated ? "" : " ⟪partial line⟫")).join("\n");
  const links = useMemo(() => logLinks(run, lines, logPath, sourcePath, source.id), [run, lines, logPath, sourcePath, source.id]);
  const sources = useMemo(() => [{ docPath: logPath, docSource: logText, view: logView ?? undefined },
    { docPath: sourcePath, docSource: sourceText, view: sourceView ?? undefined }], [logView, sourceView, logPath, sourcePath, logText, sourceText]);
  function jump(view: EditorView | null, line: number) {
    if (!view) return; const target = view.state.doc.line(Math.max(1, Math.min(line, view.state.doc.lines)));
    view.dispatch({ selection: { anchor: target.from }, effects: EditorView.scrollIntoView(target.from, { y: "center" }) }); view.focus();
  }
  function navigate(event: RuntimeEvent, location: RuntimeLocation | null) {
    setSelected(event.id); revealRuntimeEvent(run.id, event.id);
    const captured = run.sources.find(item => item.id === location?.source);
    if (location?.bytes && captured) {
      setSourceId(captured.id);
      setPendingLine(new TextDecoder().decode(new Uint8Array(captured.bytes.slice(0, location.bytes.start))).split("\n").length);
    }
  }
  useEffect(() => { if (pendingLine !== null && sourceView) { jump(sourceView, pendingLine); setPendingLine(null); } }, [pendingLine, source.id, sourceText, sourceView]);
  useEffect(() => {
    const reveal = (event: Event) => {
      const detail = (event as CustomEvent<{ run: string; event: string }>).detail;
      if (detail.run !== run.id) return;
      setOpen(true); setSelected(detail.event);
      const captured = run.events.find(item => item.id === detail.event);
      if (captured?.observation.kind === "output") setPair(JSON.stringify([captured.observation.stream, captured.observation.category]));
      const index = lines.findIndex(line => line.contributors.some(item => item.event === detail.event));
      if (index >= 0) jump(logView, index + 1);
    };
    window.addEventListener(RUNTIME_REVEAL, reveal); return () => window.removeEventListener(RUNTIME_REVEAL, reveal);
  }, [run.id, lines, logView]);
  useEffect(() => { const index = lines.findIndex(line => line.contributors.some(item => item.event === selected)); if (index >= 0) jump(logView, index + 1); }, [selected, lines, logView]);
  async function challenge(event: RuntimeEvent) {
    const exact = links.find(link => link.key.includes(`:${event.id}:`) && link.review);
    if (exact?.review) { openReview(exact.review); return; }
    const record = JSON.stringify(run);
    openReview({ kind: "runtime-record", path: `runtime-record://${run.id}`, quote: JSON.stringify(event),
      source_hash: await sha256(record), lines: [1, 1], relationship: { run: run.id, event, source_revision: source.revision, precision: event.emission?.precision ?? "unknown" } });
  }
  return <details className="runtime-log" open={open} onToggle={event => setOpen(event.currentTarget.open)}>
    <summary>Execution log and source relationships · run {run.id}</summary>
    {open && <>
      <p>Observed execution provenance · revision {source.revision}. These links are separate from byte lineage, model context and declared support. Emission and calling context are different locations. Captured source is historical.</p>
      <label>Runtime log stream <select value={chosen ?? ""} onChange={event => setPair(event.target.value)}>{pairs.map(value => <option key={value} value={value}>{JSON.parse(value).join(" · ")}</option>)}</select></label>
      <label>Captured source <select value={source.id} onChange={event => setSourceId(event.target.value)}>{run.sources.map(item => <option key={item.id} value={item.id}>{item.uri} · {item.revision}</option>)}</select></label>
      <label>Execution connector style <select value={style} onChange={event => setStyle(event.target.value as "bands" | "braces")}><option value="braces">Braces</option><option value="bands">Ribbons</option></select></label>
      <div className="runtime-log-panes" ref={setContainer}>
        <div><h4>Captured log · {stream}/{category}</h4><EditingSurface readOnly value={logText} onViewReady={setLogView} ariaLabel="Captured execution log" /></div>
        <div><h4>Captured source · {source.uri}</h4><EditingSurface readOnly value={sourceText} onViewReady={setSourceView} ariaLabel="Captured execution source" /></div>
        <RibbonOverlay container={container} sources={sources} files={[]} links={links} ribbonStyle={style} visibility="always" onNavigate={target => {
          if (target.kind !== "path") return;
          const link = links.find(item => item.key === target.linkKey);
          if (link) { const event = run.events.find(event => link.key.includes(`:${event.id}:`)); if (event) revealRuntimeEvent(run.id, event.id); }
          jump(target.path === logPath ? logView : sourceView, target.lines?.[0] ?? 1);
        }} />
      </div>
      <ol aria-label="Execution line contributors">{lines.map((line, index) => <li key={index}>
        <span>Log line {index + 1} · {line.terminated ? "LF terminated" : "partial"} · {line.contributors.length} contributions</span>
        {line.contributors.map((item, occurrence) => {
          const event = run.events.find(event => event.id === item.event)!;
          return <div key={`${item.event}:${occurrence}`} data-runtime-event={event.id}>
            <strong>{selected === event.id ? "Selected · " : ""}{event.id}</strong> · invocation {event.invocation ?? "unknown"} · chunk bytes {item.chunk_bytes.start}–{item.chunk_bytes.end} → line bytes {item.line_bytes.start}–{item.line_bytes.end}
            <button onClick={() => navigate(event, event.emission)}>Emission · {event.emission?.precision ?? "unknown"}</button>
            {event.callers.map((caller, callerIndex) => <button key={callerIndex} onClick={() => navigate(event, caller)}>Calling context {callerIndex + 1} · {caller.precision}</button>)}
            <button onClick={() => void challenge(event)}>Challenge runtime relationship</button>
          </div>;
        })}
      </li>)}</ol>
      {run.events.filter(event => event.observation.kind !== "output").map(event => <p key={event.id}>{selected === event.id ? "Selected · " : ""}{event.id} · typed {event.observation.kind} observation; not inferred from stderr.
        <button onClick={() => navigate(event, event.observation.kind === "exception" ? event.observation.throwing : event.emission)}>Throwing/emitting location</button>
        <button onClick={() => void challenge(event)}>Challenge runtime relationship</button></p>)}
    </>}
  </details>;
}
