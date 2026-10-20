import { registerDebugSurface, preferDebugSurface, representedFrameOffset } from "../debug/preferredDebugLocation";
import { PaneHeaderContent } from "../shell/PaneHeader";
import { SurfaceInfo } from "../components/SurfaceInfo";
import { RepresentationActions } from "./RepresentationActions";
import type { UiOutcome } from "../lib/uiActions";
import { ChangePreview } from "../components/ChangePreview";
import { DocumentComparison } from "./DocumentReading";
import { viewBuffers } from "../lib/viewBuffers";
import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import { EditorView } from "@codemirror/view";
import { foldEffect, unfoldAll } from "@codemirror/language";
import { DocumentEditor } from "../editor/DocumentEditor";
import { LocalRealtime, getWorkspaceRealtime } from "../api/realtime";
import { representations, type Comparison, type Representation } from "../api/representations";
import { PromptPanel, usePrompt } from "../components/PromptPanel";
import { ChatDock } from "./deferredAgent";
import { RepresentationTools } from "./RepresentationTools";
import { lspSupport, setLspDiagnostics, type DiagnosticSpan } from "../lsp/cmLsp";
import { debugEditor, setBreakpointMarks, setPausedLine, type BreakpointMark } from "../debug/cmDebug";
import type { DebugSession } from "../debug/useDebugger";
import { filePosition, representationOffset } from "../lsp/representationMapping";
import { comparisonChanges } from "../editor/comparison";
import { FILES_CHANGED_EVENT } from "../shell/FolderTreePane";
import { openLiterate, showRepresentation } from "../lib/openRepresentation";
import { isAction } from "../lib/keymap";

const noCells = new Set<string>();
const buffers = new Map<string, string>();
const message = (error: unknown) => error instanceof Error ? error.message : String(error);

export function RepresentationPane({ id, initialBase = "", initialTarget = "", bufferId = id }: { id: string; initialBase?: string; initialTarget?: string; bufferId?: string }) {
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
  const owner = useMemo(() => ({}), [id]);
  const ownerRef = useRef(owner); ownerRef.current = owner;
  const mounted = useRef(true);
  useEffect(() => { mounted.current = true; return () => { mounted.current = false; }; }, []);
  const current = useRef({ source, mappedSource, files, record, busy, comparison }); current.current = { source, mappedSource, files, record, busy, comparison };
  const saveResult = useRef<UiOutcome>({ status: "unavailable", reason: "No save has completed" });
  const refreshResult = useRef<UiOutcome>({ status: "unavailable", reason: "No refresh has completed" });
  const debugSessions = useRef(new Map<string, DebugSession>());
  const diagnostics = useRef(new Map<string, DiagnosticSpan[]>());
  const [debugTick, setDebugTick] = useState(0);
  const dirty = !!record && source !== record.source && (!comparison || comparison.editable);
  const accept = useCallback((next: Representation) => {
    if (ownerRef.current !== owner || next.id !== id) return;
    setRecord(next); setFiles(next.files); setMappedSource(next.source);
    const text = buffers.get(id) ?? next.source;
    current.current = { ...current.current, record: next, source: text, mappedSource: next.source, files: next.files };
    setSource(text);
    const view = viewRef.current;
    if (view && view.state.doc.toString() !== text) view.dispatch({ changes: { from: 0, to: view.state.doc.length, insert: text } });
  }, [id, owner]);
  useEffect(() => { let live = true; representations.get(id).then(v => { if (live) accept(v); }, e => { if (live) setNotice(message(e)); }); return () => { live = false; local.close(); }; }, [id, accept, local]);
  const refresh = useCallback(async () => {
    refreshResult.current = { status: "unavailable", reason: "Refresh refused: local edits, historical view or pending request" };
    const now = { ...current.current };
    if (ownerRef.current !== owner || now.comparison?.target || now.busy || (now.record && now.source !== now.record.source)) return false;
    current.current.busy = true; setBusy(true);
    try {
      const next = await representations.refresh(id);
      if (!mounted.current || ownerRef.current !== owner) return false;
      if (current.current.source !== now.source || current.current.comparison !== now.comparison) {
        setNotice("Source changed while refreshing. Your newer edits were kept; refresh again after saving or discarding them.");
        return false;
      }
      if (next.id !== id) throw new Error("Refresh returned a different literate view.");
      accept(next);
      refreshResult.current = { status: "completed", value: { id: next.id, revision: next.revision, refreshed: true, scope: "Actual source refresh reply; execution not verified" } };
      return true;
    } catch (e) { refreshResult.current = { status: "unavailable", reason: message(e) }; if (ownerRef.current === owner && mounted.current) setNotice(message(e)); return false; }
    finally { if (ownerRef.current === owner) { current.current.busy = false; if (mounted.current) setBusy(false); } }
  }, [id, accept, owner]);
  useEffect(() => { const run = () => { void refresh(); }; window.addEventListener(FILES_CHANGED_EVENT, run); window.addEventListener("focus", run); return () => { window.removeEventListener(FILES_CHANGED_EVENT, run); window.removeEventListener("focus", run); }; }, [refresh]);
  useEffect(() => {
    if (!record || comparison?.target) return;
    let live = true;
    const timer = setTimeout(() => { representations.preview(id, record.revision, source).then(next => { if (live) { setFiles(next); setMappedSource(source); } }, () => { if (live) setFiles([]); }); }, 250);
    return () => { live = false; clearTimeout(timer); };
  }, [id, record, source, comparison?.target]);
  const save = useCallback(async () => {
    saveResult.current = { status: "unavailable", reason: "Save refused: view unavailable, historical or request pending" };
    const now = { ...current.current };
    if (ownerRef.current !== owner || !now.record || now.busy || (now.comparison && !now.comparison.editable)) return false;
    current.current.busy = true;
    setBusy(true); setNotice("");
    try {
      const next = await representations.edit(id, now.record.revision, now.source);
      if (next.id !== id) throw new Error("Save returned a different literate view.");
      if (ownerRef.current !== owner) return false;
      // A user can keep typing while the request is in flight. Preserve
      // those later edits; only the acknowledged text becomes clean.
      if (current.current.source === now.source) buffers.delete(id);
      else buffers.set(id, current.current.source);
      accept(next);
      viewBuffers.set(bufferId, { dirty: buffers.has(id), save, discard });
      setNotice(`Saved to ${next.backing.kind === "files" ? next.files.map(file => file.path).join(", ") : "the owning document"}.`);
      window.dispatchEvent(new Event(FILES_CHANGED_EVENT));
      saveResult.current = { status: "completed", value: { id: next.id, revision: next.revision, clean: !buffers.has(id), files: next.files.map(({ path, hash }) => ({ path, hash })), source_save_acknowledged: true, verified: false } };
      return !buffers.has(id);
    } catch (e) { saveResult.current = { status: "unavailable", reason: message(e) }; if (ownerRef.current === owner && mounted.current) setNotice(message(e)); return false; }
    finally { if (ownerRef.current === owner) { current.current.busy = false; if (mounted.current) setBusy(false); } }
  }, [id, bufferId, accept, owner]);
  const discard = useCallback(() => {
    buffers.delete(id);
    const now = current.current;
    if (now.record) accept(now.record);
    setNotice("");
    viewBuffers.set(bufferId, { dirty: false, save, discard });
  }, [id, bufferId, accept, save]);
  useEffect(() => { viewBuffers.set(bufferId, { dirty, save, discard }); }, [bufferId, dirty, save, discard]);
  useEffect(() => { const key = (event: KeyboardEvent) => { if (viewRef.current?.hasFocus && isAction(event, "file.save")) { event.preventDefault(); void save(); } }; window.addEventListener("keydown", key); return () => window.removeEventListener("keydown", key); }, [save]);
  const change = useCallback((text: string) => { current.current.source = text; setSource(text); if (!current.current.comparison?.target) buffers.set(id, text); }, [id]);
  const extensions = useMemo(() => [
    ...lspSupport({ client: null, uri: "", positionAt: () => null, onNavigate: () => {} }),
    ...debugEditor({ onToggleBreakpoint: line => {
      const now = current.current, view = viewRef.current;
      if (!view || now.comparison?.target || now.source !== now.record?.source) return;
      for (const file of now.files) { const position = filePosition(now.source, file, view.state.doc.line(line + 1).from); if (position) { debugSessions.current.get(file.path)?.toggleBreakpoint(position.line); break; } }
    } }),
  ], []);
  const onDiagnostics = useCallback((path: string, spans: DiagnosticSpan[]) => { diagnostics.current.set(path, spans); viewRef.current?.dispatch({ effects: setLspDiagnostics.of([...diagnostics.current.values()].flat()) }); }, []);
  useEffect(() => registerDebugSurface(id, frame => {
    const view = viewRef.current;
    const now = current.current;
    if (!view || now.source !== now.mappedSource || view.state.doc.toString() !== now.source) return false;
    const offset = representedFrameOffset(now.source, now.files, frame);
    if (offset === null) return false;
    view.dispatch({ selection: { anchor: offset }, scrollIntoView: true });
    return true;
  }), [id]);
  const startDebug = (path: string) => {
    if (dirty) { setNotice("Save your edits before debugging."); return; }
    const debug = debugSessions.current.get(path);
    if (debug) { preferDebugSurface(path, id); debug.start(); }
    else setNotice(`Code tools for ${path} are still opening. Try Debug again.`);
  };
  const onDebug = useCallback((path: string, session: DebugSession) => { debugSessions.current.set(path, session); setDebugTick(n => n + 1); }, []);
  useEffect(() => {
    if (!editor || comparison?.target) return;
    const marks: BreakpointMark[] = []; let paused: number | null = null;
    for (const file of files) {
      const debug = debugSessions.current.get(file.path); if (!debug) continue;
      const line = (n: number) => { const offset = representationOffset(source, file, { line: n, character: 0 }); return offset === null ? null : editor.state.doc.lineAt(offset).number - 1; };
      for (const bp of debug.breakpoints) { const mapped = line(bp.line); if (mapped !== null) marks.push({ ...bp, line: mapped, conditional: !!(bp.condition || bp.hit_condition || bp.log_message) }); }
      if (debug.status === "paused") {
        const frame = debug.frames.find(frame => frame.id === debug.selectedFrame) ?? debug.frames[0];
        const offset = frame && source === mappedSource ? representedFrameOffset(source, files, frame) : null;
        if (offset !== null) paused = editor.state.doc.lineAt(offset).number - 1;
      }
    }
    editor.dispatch({ effects: [setBreakpointMarks.of(marks), setPausedLine.of(paused)] });
  }, [editor, files, source, mappedSource, debugTick, comparison?.target]);
  const compare = async () => {
    const now = { ...current.current };
    if (now.busy || ownerRef.current !== owner) return;
    if (now.record && now.source !== now.record.source && (!now.comparison || now.comparison.editable)) { setNotice("Save your edits before changing the comparison."); return; }
    current.current.busy = true; setBusy(true);
    try {
      const next = await representations.compare(id, base || "HEAD", target || undefined);
      if (!mounted.current || ownerRef.current !== owner) return;
      if (current.current.source !== now.source || current.current.comparison !== now.comparison) {
        setNotice("Source changed while comparing. Your newer edits were kept; request the comparison again."); return;
      }
      current.current.comparison = next; current.current.source = next.target_source;
      setComparison(next); setSource(next.target_source);
      const view = viewRef.current;
      if (view && view.state.doc.toString() !== next.target_source) view.dispatch({ changes: { from: 0, to: view.state.doc.length, insert: next.target_source } });
    } catch (e) { if (ownerRef.current === owner && mounted.current) setNotice(message(e)); }
    finally { if (ownerRef.current === owner) { current.current.busy = false; if (mounted.current) setBusy(false); } }
  };
  useEffect(() => { if (record && initialBase) void compare(); /* opening comparison */ }, [record?.id]); // eslint-disable-line react-hooks/exhaustive-deps
  if (!record) return <p role={notice ? "alert" : "status"}>{notice || "Opening literate view…"}</p>;
  const historical = !!comparison && !comparison.editable;
  return <section className="representation-pane" aria-label="Literate editor">
    <SurfaceInfo name="Literate view" editable={!historical} destination={historical ? "Historical comparison; this view writes nothing." : record.backing.kind === "files" ? `Save writes code to ${record.files.map(file => file.path).join(", ")} and keeps this reading arrangement locally.` : "Save writes changes into the owning document and keeps this reading arrangement locally."} />
    <PaneHeaderContent part="actions"><div className="doc-tab-toolbar" role="toolbar" aria-label="Literate view actions">
      <RepresentationActions record={record} dirty={dirty} busy={busy} historical={historical} agent={agent}
        live={() => [ownerRef.current === owner, current.current.source, current.current.record?.revision, current.current.comparison, current.current.busy]}
        save={async () => { await save(); return saveResult.current; }} refresh={async () => { await refresh(); return refreshResult.current; }}
        discard={discard} toggleAgent={() => setAgent(v => !v)} onNotice={setNotice} />
    </div></PaneHeaderContent>
    {dirty && !historical && <ChangePreview heading="Review changes before saving" collapsible effects="Save writes these edits to the targets named above. This view has no recovery draft; closing without saving discards its edits.">
      <DocumentComparison id={`${id}:preview`} path="arrangement.md" before={record.source} after={source} />
      {files.filter(file => file.content !== record.files.find(base => base.path === file.path)?.content).map(file => <section key={file.path}><strong>{file.path}</strong>
        <DocumentComparison id={`${id}:preview:${file.path}`} path={file.path} before={record.files.find(base => base.path === file.path)?.content ?? ""} after={file.content} />
      </section>)}
    </ChangePreview>}
    <PaneHeaderContent part="actions"><details className="representation-versions"><summary data-tip="Compare revisions; current code stays editable">{comparison ? `${comparison.base.slice(0, 8)} → ${comparison.target?.slice(0, 8) ?? "current"}` : "Compare revisions…"}</summary>
    <div className="representation-versions__menu"><form className="representation-comparison" onSubmit={e => { e.preventDefault(); void compare(); }}>
      <label>Earlier revision <input aria-label="Comparison base" value={base} placeholder="HEAD, branch, or commit" onChange={e => setBase(e.target.value)} /></label>
      <label>Compare with <input aria-label="Comparison target" value={target} placeholder="Current code (editable)" onChange={e => setTarget(e.target.value)} /></label>
      <button className="btn" disabled={dirty}>Show diff</button>
      {comparison && <button type="button" className="btn" disabled={dirty} onClick={() => { setComparison(null); setTarget(""); buffers.delete(id); accept(record); }}>Return to current code</button>}
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
    <p className="muted">Leave the second revision empty to compare with editable current code. A historical target is read-only; use a worktree to run it.</p>
    </form>
    {comparison && record.backing.kind === "files" && <p className="muted">Explanations belong to this reading arrangement; they are not evidence from the compared commit.</p>}
    {comparison && <p className="muted">{comparison.base.slice(0, 12)} → {comparison.target?.slice(0, 12) ?? "working tree"}{historical ? " · historical, read-only; open a worktree to run this revision" : " · current side is editable"}</p>}
    </div></details></PaneHeaderContent>
    {record.explanation_stale && <p role="status">Stale explanation — source changed. Review or regenerate the explanation; this is not an execution verdict.</p>}
    {record.local_warning && <p role="alert">{record.local_warning}</p>}
    {notice && <p role="alert">{notice}</p>}
    {!historical && files.map(file => <RepresentationTools key={file.path} file={file} source={mappedSource} view={editor} saved={!dirty} onStart={() => startDebug(file.path)} onDiagnostics={onDiagnostics} onDebug={onDebug} onMessage={setNotice} askText={prompt.askText} askChoice={prompt.askChoice} />)}
    <DocumentEditor docId={`lens:${id}`} initialSource={source} realtime={local} onChange={change} onViewReady={v => { viewRef.current = v; setEditor(v); }}
      execBlocks={[]} runningCells={noCells} onRunCell={() => setNotice("Run the repository's own tests or open its persistent document to execute cells.")}
      onDebugFile={historical ? undefined : startDebug}
      readOnly={historical} comparisonBase={comparison?.source ?? null} lspExtensions={extensions} />
    <PromptPanel prompt={prompt.prompt} onSettle={prompt.settle} />
    {agent && <ChatDock docId={`lens:${id}`} realtime={realtime} initialPrompt={`Read literate view ${id} through the available reading tools and explain its code. Discover available actions before changing it. Preserve source bytes when rearranging explanations; do not invent unavailable tools or claim a buffer edit has been saved.`}
      contextLabel={`Literate view ${id}; Hickory Python or an installed ACP agent`}
      getContext={async () => ({ live: true, buffers: [{ id: `lens:${id}`, name: `Literate view ${id}`, path: null, content: current.current.source, focused: true }] })}
      onAgentFinished={() => { if (current.current.record && current.current.source === current.current.record.source) void refresh(); else setNotice("The agent finished. Save or discard your local edits before refreshing its changes."); }} />}
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
