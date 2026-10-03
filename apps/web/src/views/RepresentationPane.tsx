import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import { EditorView } from "@codemirror/view";
import { foldEffect, unfoldAll } from "@codemirror/language";
import { DocumentEditor } from "../editor/DocumentEditor";
import { LocalRealtime, getWorkspaceRealtime } from "../api/realtime";
import { representations, type Comparison, type Representation } from "../api/representations";
import { PromptPanel, usePrompt } from "../components/PromptPanel";
import { ChatDock } from "../components/ChatDock";
import { RepresentationTools } from "./RepresentationTools";
import { lspSupport, setLspDiagnostics, type DiagnosticSpan } from "../lsp/cmLsp";
import { debugEditor, setBreakpointMarks, setPausedLine, type BreakpointMark } from "../debug/cmDebug";
import type { DebugSession } from "../debug/useDebugger";
import { filePosition, representationOffset } from "../lsp/representationMapping";
import { comparisonChanges } from "../editor/comparison";
import { FILES_CHANGED_EVENT } from "../shell/FolderTreePane";
import { openLiterate, showRepresentation } from "../lib/openRepresentation";
import { onFlushSaves } from "../lib/flushSaves";
import { isAction } from "../lib/keymap";

const noCells = new Set<string>();
const buffers = new Map<string, string>();
const message = (error: unknown) => error instanceof Error ? error.message : String(error);

export function RepresentationPane({ id, initialBase = "", initialTarget = "" }: { id: string; initialBase?: string; initialTarget?: string }) {
  const [record, setRecord] = useState<Representation | null>(null);
  const [source, setSource] = useState("");
  const [mappedSource, setMappedSource] = useState("");
  const [files, setFiles] = useState<Representation["files"]>([]);
  const [editor, setEditor] = useState<EditorView | null>(null);
  const [notice, setNotice] = useState("");
  const [busy, setBusy] = useState(false);
  const [base, setBase] = useState(initialBase);
  const [target, setTarget] = useState(initialTarget);
  const [comparison, setComparison] = useState<Comparison | null>(null);
  const [agent, setAgent] = useState(false);
  const prompt=usePrompt();
  const local = useMemo(() => new LocalRealtime(), [id]);
  const realtime = useMemo(() => getWorkspaceRealtime() ?? new LocalRealtime(), []);
  const viewRef = useRef<EditorView | null>(null);
  const current = useRef({ source, files, record, busy, comparison }); current.current = { source, files, record, busy, comparison };
  const debugSessions = useRef(new Map<string, DebugSession>());
  const diagnostics = useRef(new Map<string, DiagnosticSpan[]>());
  const [debugTick, setDebugTick] = useState(0);
  const dirty = !!record && source !== record.source && (!comparison || comparison.editable);
  const accept = useCallback((next: Representation) => {
    setRecord(next); setFiles(next.files); setMappedSource(next.source);
    const text = buffers.get(id) ?? next.source;
    setSource(text);
    const view = viewRef.current;
    if (view && view.state.doc.toString() !== text) view.dispatch({ changes: { from: 0, to: view.state.doc.length, insert: text } });
  }, [id]);
  useEffect(() => { let live = true; representations.get(id).then(v => { if (live) accept(v); }, e => { if (live) setNotice(message(e)); }); return () => { live = false; local.close(); }; }, [id, accept, local]);
  const refresh = useCallback(async () => {
    const now = current.current;
    if (now.comparison?.target || now.busy || (now.record && now.source !== now.record.source && !now.comparison?.target)) return;
    try { accept(await representations.refresh(id)); } catch (e) { setNotice(message(e)); }
  }, [id, accept]);
  useEffect(() => { window.addEventListener(FILES_CHANGED_EVENT, refresh); window.addEventListener("focus", refresh); return () => { window.removeEventListener(FILES_CHANGED_EVENT, refresh); window.removeEventListener("focus", refresh); }; }, [refresh]);
  useEffect(() => {
    if (!record || comparison?.target) return;
    let live = true;
    const timer = setTimeout(() => { representations.preview(id, record.revision, source).then(next => { if (live) { setFiles(next); setMappedSource(source); } }, () => { if (live) setFiles([]); }); }, 250);
    return () => { live = false; clearTimeout(timer); };
  }, [id, record, source, comparison?.target]);
  const save = useCallback(async () => {
    const now = current.current;
    if (!now.record || now.busy || now.comparison?.target) return false;
    setBusy(true); setNotice("");
    try {
      const next = await representations.edit(id, now.record.revision, now.source);
      buffers.delete(id); accept(next);
      window.dispatchEvent(new Event(FILES_CHANGED_EVENT));
      return true;
    } catch (e) { setNotice(message(e)); return false; } finally { setBusy(false); }
  }, [id, accept]);
  useEffect(() => onFlushSaves(async () => { const now = current.current; if (now.record && now.source !== now.record.source && !now.comparison?.target) await save(); }), [save]);
  useEffect(() => { const key = (event: KeyboardEvent) => { if (viewRef.current?.hasFocus && isAction(event, "file.save")) { event.preventDefault(); void save(); } }; window.addEventListener("keydown", key); return () => window.removeEventListener("keydown", key); }, [save]);
  const change = useCallback((text: string) => { setSource(text); if (!current.current.comparison?.target) buffers.set(id, text); }, [id]);
  const extensions = useMemo(() => [
    ...lspSupport({ client: null, uri: "", positionAt: () => null, onNavigate: () => {} }),
    ...debugEditor({ onToggleBreakpoint: line => {
      const now = current.current, view = viewRef.current;
      if (!view || now.comparison?.target || now.source !== now.record?.source) return;
      for (const file of now.files) { const position = filePosition(now.source, file, view.state.doc.line(line + 1).from); if (position) { debugSessions.current.get(file.path)?.toggleBreakpoint(position.line); break; } }
    } }),
  ], []);
  const onDiagnostics = useCallback((path: string, spans: DiagnosticSpan[]) => { diagnostics.current.set(path, spans); viewRef.current?.dispatch({ effects: setLspDiagnostics.of([...diagnostics.current.values()].flat()) }); }, []);
  const onDebug = useCallback((path: string, session: DebugSession) => { debugSessions.current.set(path, session); setDebugTick(n => n + 1); }, []);
  useEffect(() => {
    if (!editor || comparison?.target) return;
    const marks: BreakpointMark[] = []; let paused: number | null = null;
    for (const file of files) {
      const debug = debugSessions.current.get(file.path); if (!debug) continue;
      const line = (n: number) => { const offset = representationOffset(source, file, { line: n, character: 0 }); return offset === null ? null : editor.state.doc.lineAt(offset).number - 1; };
      for (const bp of debug.breakpoints) { const mapped = line(bp.line); if (mapped !== null) marks.push({ ...bp, line: mapped, conditional: !!(bp.condition || bp.hit_condition || bp.log_message) }); }
      if (debug.pausedLine !== null) paused = line(debug.pausedLine);
    }
    editor.dispatch({ effects: [setBreakpointMarks.of(marks), setPausedLine.of(paused)] });
  }, [editor, files, source, debugTick, comparison?.target]);
  const compare = async () => {
    if (dirty) { setNotice("Save your edits before changing the comparison."); return; }
    try {
      const next = await representations.compare(id, base || "HEAD", target || undefined);
      setComparison(next);
      setSource(next.target_source);
      if (editor && editor.state.doc.toString() !== next.target_source) editor.dispatch({ changes: { from: 0, to: editor.state.doc.length, insert: next.target_source } });
    } catch (e) { setNotice(message(e)); }
  };
  useEffect(() => { if (record && initialBase) void compare(); /* opening comparison */ }, [record?.id]); // eslint-disable-line react-hooks/exhaustive-deps
  if (!record) return <p role={notice ? "alert" : "status"}>{notice || "Opening literate view…"}</p>;
  const historical = !!comparison?.target;
  return <section className="representation-pane" aria-label="Literate editor">
    <div className="doc-tab-toolbar" role="toolbar" aria-label="Literate view actions">
      <span>{record.backing.kind === "files" ? "Source-backed view" : "Document-backed view"}</span>
      <button className="btn btn-primary" disabled={!dirty || busy || historical} onClick={() => void save()}>Save code and view</button>
      <button className="btn" disabled={dirty || busy} onClick={() => void refresh()}>Refresh from source</button>
      <button className="btn" disabled={busy || !dirty || historical} onClick={() => { buffers.delete(id); accept(record); setNotice(""); }}>Discard unsaved edits</button>
      {record.backing.kind === "files" && <button className="btn" disabled={dirty || busy} onClick={() => void openLiterate(record.backing).catch(e => setNotice(message(e)))}>Rebuild reading from current files</button>}
      <button className="btn" disabled={dirty} onClick={() => void representations.keep(id).then(v => setNotice(`View kept locally at ${v.path}`), e => setNotice(message(e)))}>Keep view locally</button>
      <button className="btn" onClick={() => setAgent(v => !v)}>Explain or edit with ACP</button>
    </div>
    <form className="representation-comparison" onSubmit={e => { e.preventDefault(); void compare(); }}>
      <label>Compare from <input aria-label="Comparison base" value={base} placeholder="HEAD, branch, or commit" onChange={e => setBase(e.target.value)} /></label>
      <label>to <input aria-label="Comparison target" value={target} placeholder="working tree" onChange={e => setTarget(e.target.value)} /></label>
      <button className="btn" disabled={dirty}>Show diff</button>
      {comparison && <button type="button" className="btn" disabled={dirty} onClick={() => { setComparison(null); setTarget(""); buffers.delete(id); accept(record); }}>Show current state</button>}
      {comparison && <button type="button" className="btn" onClick={() => {
        if (!editor) return; unfoldAll(editor);
        const changes = comparisonChanges(comparison.source, source);
        let previous = 0;
        for (const change of [...changes, { from: source.length, to: source.length, removed: "" }]) {
          const start = editor.state.doc.lineAt(previous).number + 3, end = editor.state.doc.lineAt(change.from).number - 3;
          if (end > start) editor.dispatch({ effects: foldEffect.of({ from: editor.state.doc.line(start).to, to: editor.state.doc.line(end).to }) });
          previous = change.to;
        }
      }}>Fold unchanged regions</button>}
    </form>
    {comparison && record.backing.kind === "files" && <p className="muted">Explanations belong to this reading arrangement; they are not evidence from the compared commit.</p>}
    {comparison && <p className="muted">{comparison.base.slice(0, 12)} → {comparison.target?.slice(0, 12) ?? "working tree"}{historical ? " · historical, read-only; open a worktree to run this revision" : " · current side is editable"}</p>}
    {record.explanation_stale && <p role="status">Source changed. AI explanations need review or regeneration.</p>}
    {record.local_warning && <p role="alert">{record.local_warning}</p>}
    {notice && <p role="alert">{notice}</p>}
    {!historical && files.map(file => <RepresentationTools key={file.path} file={file} source={mappedSource} view={editor} saved={!dirty} onDiagnostics={onDiagnostics} onDebug={onDebug} onMessage={setNotice} askText={prompt.askText} askChoice={prompt.askChoice} />)}
    <DocumentEditor docId={`lens:${id}`} initialSource={source} realtime={local} onChange={change} onViewReady={v => { viewRef.current = v; setEditor(v); }}
      execBlocks={[]} runningCells={noCells} onRunCell={() => setNotice("Run the repository's own tests or open its persistent document to execute cells.")}
      readOnly={historical} comparisonBase={comparison?.source ?? null} lspExtensions={extensions} />
    <PromptPanel prompt={prompt.prompt} onSettle={prompt.settle} />
    {agent && <ChatDock docId={`lens:${id}`} realtime={realtime} initialPrompt={`Read literate view ${id} using read_literate_view, then organize_literate_view to explain its code and arrange exact source fragments. Preserve every source byte.`}
      contextLabel={`Literate view ${id}; choose an installed ACP agent`}
      getContext={async () => ({ buffers: [{ name: `Literate view ${id}`, path: null, content: source, focused: true }] })}
      onAgentFinished={() => { if (!dirty) void representations.get(id).then(accept); else setNotice("The agent finished. Save or discard your local edits before refreshing its changes."); }} />}
  </section>;
}

export function RepresentationLibrary() {
  const [paths, setPaths] = useState("");
  const [views, setViews] = useState<Representation[]>([]);
  const [error, setError] = useState("");
  useEffect(() => { void representations.list().then(v => setViews(v.views), e => setError(message(e))); }, []);
  return <section className="representation-library"><h2>Literate views</h2>
    <p>Keep ordinary source files in git. Use a personal literate arrangement to read and edit them.</p>
    <form onSubmit={e => { e.preventDefault(); void openLiterate({ kind: "files", paths: paths.split("\n").map(p => p.trim()).filter(Boolean) }).catch(e => setError(message(e))); }}>
      <label>Source files, one relative path per line<textarea aria-label="Literate source files" rows={4} value={paths} onChange={e => setPaths(e.target.value)} /></label>
      <button className="btn btn-primary" disabled={!paths.trim()}>Open literate view</button>
    </form>{error && <p role="alert">{error}</p>}
    <ul>{views.map(view => <li key={view.id}><button className="btn-link" onClick={() => showRepresentation(view.id)}>{view.files.map(f => f.path).join(", ")}</button> <button className="btn" onClick={() => void representations.discard(view.id).then(() => setViews(v => v.filter(item => item.id !== view.id)), e => setError(message(e)))}>Discard view</button></li>)}</ul>
  </section>;
}
