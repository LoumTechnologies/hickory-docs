<ui:doc xmlns:ui="http://www.hickorydocs.com/1.0">
# Retain the editor instance while moving a document

For maintainers iterating on the equivalent Apple, Android and browser editor frontends.

CodeMirror owns typing, selection, scroll and undo. Cache the editor by buffer identity and move its existing DOM between hosts. Buffer notifications update the status, not React-controlled document text.

The minimap reserves forty units beside the text rather than overlaying readable lines. Its navigation changes scroll, never saved bytes. Language controls float over the editor and do not reflow it. Browser capabilities use the existing LSP client and its snapshot guards; native keyword and word-list previews deliberately do not claim semantic results. Native styling does not replace text, preserving the platform undo and selection model. Preview coloring skips very large files instead of delaying readable text indefinitely.

After text is visible, Markdown files load and attach the existing display-only prose decorations independently of LSP. Headings, bold and italic update with typing without rewriting source or rebuilding the editor. Fenced code stays literal; plain files keep their original appearance.

Search uses CodeMirror’s native search transactions and keymap. The panel is an absolute overlay, so showing it cannot resize the cached editor. Literal search treats backslashes as text; regex mode enables capture replacements. Read-only policy stays in editor state and search navigation reveals folded matches.

The editor installs CodeMirror folding with its initial text. A linear indentation scan supplies regions without waiting for a parser or server; the existing LSP compartment may add language-specific regions later. A dedicated left gutter shows − on expanded headers and + on collapsed headers. CodeMirror anchors these controls to its measured lines. Folding changes editor decorations rather than buffer bytes, and the existing cached view keeps folds across tab moves. Debug launch actions keep their separate right-hand rail.

<ui:copy id="simple-1a4a59fc95-0">
import { useEffect, useLayoutEffect, useRef, useState, useSyncExternalStore } from "react";
import { Compartment, EditorState, StateEffect } from "@codemirror/state";
import { EditorView, keymap, drawSelection, lineNumbers, rectangularSelection, crosshairCursor } from "@codemirror/view";
import { defaultKeymap, history, historyKeymap } from "@codemirror/commands";
import { codeFolding, foldKeymap, foldService, foldGutter } from "@codemirror/language";
import { search, searchKeymap, openSearchPanel, SearchQuery, setSearchQuery } from "@codemirror/search";
import { completionKeymap, startCompletion } from "@codemirror/autocomplete";
import { useWorkspaceLsp } from "../lsp/useLsp";
import { lspSupport, setLspDiagnostics, diagnosticRanges } from "../lsp/cmLsp";
import { lspFeatures } from "../lsp/cmLspFeatures";
import { completions } from "../lsp/completion";
import { positionToUtf16, utf16ToPosition } from "../lsp/positions";
import { sameDocumentUri } from "../lsp/documentUri";
import { Buffer } from "./buffers";
import { languageFromPath } from "../editor/languages";
import { isDebuggable } from "../debug/languages";
import { useFollowingDebugSession } from "../debug/plainDebugHosts";
import { IDLE_SESSION } from "../debug/useDebugger";
import { debugEditor, debugStateEffects, revealLine } from "../debug/cmDebug";
import { DebugLaunchRail } from "../debug/DebugRail";
import { DebugStrip } from "../debug/DebugStrip";
import { observationSourceChanged } from "../debug/observations";
import { api } from "../api/client";
import { launchWithProfiles } from "../debug/launchProfiles";
import { claimReveal, onRevealLine, openLocation } from "../lib/revealLine";
export function simpleFoldRanges(source: string): { line: number; from: number; to: number }[] {
  const rows = source.split("\n"), stack: { line: number; indent: number; from: number }[] = [];
  const ranges: { line: number; from: number; to: number }[] = [];
  let offset = 0, last = -1;
  for (let line = 0; line <= rows.length; line++) {
    const row = rows[line], indent = row === undefined ? -1 : row.match(/^[ \t]*/)![0].replace(/\t/g, "    ").length;
    if (row !== undefined && !row.trim()) { offset += row.length + 1; continue; }
    while (stack.length && stack[stack.length - 1].indent >= indent) {
      const header = stack.pop()!;
      if (last > header.from) ranges.push({ line: header.line, from: header.from, to: last });
    }
    if (row !== undefined) { last = offset + row.length; stack.push({ line, indent, from: last }); offset = last + 1; }
  }
  return ranges.sort((a, b) => a.from - b.from);
}
const foldCache = new WeakMap<EditorState["doc"], Map<number, { from: number; to: number }>>();
function localFold(state: EditorState, from: number) {
  let ranges = foldCache.get(state.doc);
  if (!ranges) { ranges = new Map(simpleFoldRanges(state.doc.toString()).map(range => [range.from, range])); foldCache.set(state.doc, ranges); }
  return ranges.get(state.doc.lineAt(from).to) ?? null;
}
const editors = new WeakMap<Buffer, { view: EditorView; language: Compartment; prose: Compartment; richAttached?: boolean }>();
export const SIMPLE_EDITOR_ATTACHED = "hickory:simple-editor-attached";
export function editorForBuffer(buffer: Buffer) { return editors.get(buffer); }
export function openFind(buffer: Buffer) { const view = editors.get(buffer)?.view; if (view) openSearchPanel(view); }
export function Document({ buffer }: { buffer: Buffer }) {
  const host = useRef<HTMLDivElement>(null);
  useSyncExternalStore(buffer.subscribe, buffer.snapshot);
  const [view, setView] = useState<EditorView | null>(null);
  const language = languageFromPath(buffer.file.path);
  const literate = /\.(md|markdown)$/i.test(buffer.file.path);
  const debuggable = literate || isDebuggable(language);
  const debug = useFollowingDebugSession(buffer.file.path) ?? IDLE_SESSION;
  const debugRef = useRef(debug); debugRef.current = debug;
  const changed = buffer.dirty || observationSourceChanged(debug.reading, buffer.text);
  const [debugNotice, setDebugNotice] = useState("");
  const session = useWorkspaceLsp(buffer.file.path, buffer.text);
  const [capabilities, refreshCapabilities] = useState(0);
  useEffect(() => session.client?.onServerCapabilities(() => refreshCapabilities(value => value + 1)), [session.client]);
  useLayoutEffect(() => {
    let cached = editors.get(buffer);
    if (!cached) {
      const language = new Compartment(), prose = new Compartment();
      const view = new EditorView({ state: EditorState.create({ doc: buffer.text, extensions: [
        history(), drawSelection(), EditorState.allowMultipleSelections.of(true), rectangularSelection(), crosshairCursor(), lineNumbers(), codeFolding(), foldGutter({ openText: "−", closedText: "+" }), search({ top: true, literal: true }),
        EditorState.transactionExtender.of(tr => {
          const query = tr.effects.find(effect => effect.is(setSearchQuery))?.value as SearchQuery | undefined;
          return query && !query.literal ? { effects: setSearchQuery.of(new SearchQuery({ ...query, literal: true })) } : null;
        }),
        foldService.of(localFold), EditorView.lineWrapping, language.of([]), prose.of([]),
        keymap.of([{ key: "Ctrl-Space", run: startCompletion }, ...searchKeymap, ...foldKeymap, ...completionKeymap, ...defaultKeymap, ...historyKeymap]),
        EditorState.readOnly.of(buffer.readonly), EditorView.editable.of(!buffer.readonly),
        EditorView.contentAttributes.of({ "aria-label": buffer.file.path }),
        EditorView.updateListener.of(update => { if (update.docChanged) buffer.edit(update.state.doc.toString()); }),
        EditorView.theme({ "&": { height: "100%" }, ".cm-scroller": { overflow: "auto", fontFamily: "ui-monospace, monospace", fontSize: "13px" } }),
      ] }) });
      cached = { view, language, prose }; editors.set(buffer, cached); window.dispatchEvent(new Event(SIMPLE_EDITOR_ATTACHED));
    }
    host.current?.append(cached.view.dom);
    window.dispatchEvent(new Event(SIMPLE_EDITOR_ATTACHED));
    setView(cached.view);
    return () => { cached.view.dom.remove(); };
  }, [buffer]);
  useEffect(() => {
    if (!/\.(md|markdown)$/i.test(buffer.file.path) || buffer.text.length > 200_000) return;
    let cancelled = false;
    void import("../editor/markdownStyling").then(({ markdownStyling }) => {
      const cached = editors.get(buffer);
      if (!cancelled && cached && !cached.richAttached) cached.view.dispatch({ effects: cached.prose.reconfigure(markdownStyling()) });
    }).catch(() => { /* Readable source remains available if styling cannot load. */ });
    return () => { cancelled = true; };
  }, [buffer]);
  useEffect(() => {
    const cached = editors.get(buffer);
    if (!cached || !session.client || !session.ready) return;
    const { view, language } = cached;
    const positionAt = (offset: number, editor: EditorView) => utf16ToPosition(editor.state.doc.toString(), offset);
    const offsetAt = (position: { line: number; character: number }, editor: EditorView) => positionToUtf16(editor.state.doc.toString(), position);
    view.dispatch({ effects: language.reconfigure([
      ...lspSupport({ client: session.client, uri: session.uri, positionAt,
        onNavigate: target => { if (sameDocumentUri(target.uri, session.uri)) {
          const anchor = offsetAt(target.range.start, view);
          view.dispatch({ selection: { anchor }, effects: EditorView.scrollIntoView(anchor, { y: "center" }) });
        } } }),
      ...lspFeatures({ client: session.client, uri: session.uri, positionAt, offsetAt, inlayHints: true }),
      ...(!buffer.readonly ? [completions({ lsp: { client: session.client, uri: session.uri,
        positionAt: (offset, state) => utf16ToPosition(state.doc.toString(), offset) } })] : []),
    ]) });
    return () => { view.dispatch({ effects: language.reconfigure([]) }); };
  }, [buffer, session.client, session.uri, session.ready, capabilities]);
  useEffect(() => {
    const cached = editors.get(buffer);
    if (!cached || !session.client || !session.ready) return;
    cached.view.dispatch({ effects: setLspDiagnostics.of(diagnosticRanges(session.diagnostics,
      position => positionToUtf16(cached.view.state.doc.toString(), position))) });
  }, [buffer, session.client, session.ready, session.diagnostics, capabilities]);
  // Attach debugger state to the existing editor. Callbacks read the current
  // session so tab moves and incoming DAP events cannot retain obsolete verbs.
  useEffect(() => {
    if (!view || !debuggable) return;
    const compartment = new Compartment();
    view.dispatch({ effects: StateEffect.appendConfig.of(compartment.of(debugEditor({ language: literate ? undefined : language,
      onToggleBreakpoint: line => debugRef.current.toggleBreakpoint(line),
      onSelectFrame: id => debugRef.current.selectFrame(id),
      onEvaluate: expression => debugRef.current.query(expression),
      evaluationOwner: () => debugRef.current.reading,
      onAddWatch: expression => debugRef.current.addWatch(expression),
      onSetBreakpointCondition: (line, patch) => debugRef.current.setBreakpointCondition(line, patch),
    }))) });
    return () => { view.dispatch({ effects: compartment.reconfigure([]) }); };
  }, [view, debuggable, language, literate]);
  useEffect(() => {
    if (!view || !debuggable) return;
    view.dispatch({ effects: debugStateEffects(changed ? { ...debug, pausedLine: null, variables: [], frames: [], watches: [] } : debug) });
    if (!changed && debug.pausedLine !== null) {
      const line = view.state.doc.line(Math.max(1, Math.min(view.state.doc.lines, debug.pausedLine + 1)));
      view.dispatch({ effects: EditorView.scrollIntoView(line.from, { y: "nearest" }) });
    }
  }, [view, debuggable, debug, changed]);
  useEffect(() => {
    if (!view) return;
    const reveal = () => { const line = claimReveal(buffer.file.path); if (line !== null) revealLine(view, line - 1); };
    reveal(); return onRevealLine(path => { if (path === buffer.file.path) reveal(); });
  }, [view, buffer]);
  const start = async (program?: string) => {
    if (buffer.dirty || buffer.saving) { setDebugNotice("Save this file before debugging so the debugger runs the text shown here."); return; }
    setDebugNotice("");
    const source = buffer.text, target = literate ? program : undefined;
    await launchWithProfiles({ uri: `hick:///${buffer.file.path}`, program: target, ...(literate ? { expected_document: source } : { expected_source: source }) },
      profile => debugRef.current.start(target, {}, undefined, profile),
      { isCurrent: () => !!view?.dom.isConnected && buffer.text === source && !buffer.dirty && !buffer.saving });
  };
  return <div className="document-body">
    <div className="editor-row"><div ref={host} className="editor-host editor-cm-host" /><Minimap buffer={buffer} />
      <DebugLaunchRail view={view} plainPath={debuggable && !literate ? buffer.file.path : undefined} enabled={["idle", "finished", "failed"].includes(debug.status)} onStart={debuggable ? start : undefined} />
    </div>
    {debuggable && view && <DebugStrip editorView={view} pausedLine={changed ? null : debug.pausedLine}
      status={debug.status} program={debug.program} message={debug.message} offerInstall={debug.offerInstall}
      onInstall={offer => api.installTool(offer.kind, offer.language).then(() => undefined)}
      capabilities={debug.capabilities} frames={debug.frames} selectedFrame={debug.selectedFrame}
      watches={debug.watches} exitCode={debug.exitCode} buildOutput={debug.buildOutput}
      onSelectFrame={id => { debug.selectFrame(id); const frame = debug.frames.find(item => item.id === id);
        if (frame?.line != null) revealLine(view, frame.line);
        else if (frame?.source && frame.source_line != null) openLocation(frame.source, frame.source_line + 1); }}
      onStep={debug.step} onJumpHere={() => debug.jumpTo(view.state.doc.lineAt(view.state.selection.main.head).number - 1)}
      onStart={() => start(debug.program ?? undefined)} onStop={debug.stop} onAddWatch={() => { const expression = window.prompt("Expression to watch"); if (expression) debug.addWatch(expression); }}
      exceptionFilters={debug.exceptionFilters} onToggleExceptionFilter={debug.toggleExceptionFilter} onRemoveWatch={debug.removeWatch} />}
    {debugNotice && <div role="alert" className="notice">{debugNotice}</div>}
    {buffer.error && <div role="alert" className="notice">{buffer.error}</div>}
  </div>;
}
</ui:copy>

The line-number extension mounts with readable text, before language or debugger capabilities. CodeMirror aligns numbers with logical lines through wrapping, scrolling and edits, including an empty final line. It shares the cached editor so attaching capabilities cannot reset numbering or undo.

## Navigate without editing

Draw at most one density mark per two screen pixels and repaint on resize, text changes and scroll. The viewport rectangle comes from the existing editor layout. Pointer navigation and Home/End only dispatch a scroll effect, leaving selection and undo untouched.

<ui:copy id="simple-document-minimap">
function Minimap({ buffer }: { buffer: Buffer }) {
  const canvas = useRef<HTMLCanvasElement>(null);
  useEffect(() => {
    const view = editors.get(buffer)!.view, element = canvas.current!;
    const draw = () => {
      const height = element.clientHeight, width = 40;
      element.width = width * devicePixelRatio; element.height = height * devicePixelRatio;
      const context = element.getContext("2d"); if (!context || !height) return;
      context.scale(devicePixelRatio, devicePixelRatio);
      const doc = view.state.doc, rows = Math.min(doc.lines, Math.ceil(height / 2));
      context.fillStyle = getComputedStyle(element).color;
      for (let row = 0; row < rows; row++) {
        const line = doc.line(1 + Math.floor(row * doc.lines / rows));
        context.globalAlpha = .5;
        context.fillRect(2, row * height / rows, Math.min(36, line.length * .65), 1);
      }
      const first = view.lineBlockAtHeight(view.scrollDOM.scrollTop).from;
      const last = view.lineBlockAtHeight(Math.min(view.contentHeight, view.scrollDOM.scrollTop + view.scrollDOM.clientHeight)).to;
      context.globalAlpha = .18;
      context.fillRect(0, (doc.lineAt(first).number - 1) / doc.lines * height, width,
        Math.max(4, (doc.lineAt(last).number - doc.lineAt(first).number + 1) / doc.lines * height));
    };
    const observer = new ResizeObserver(draw); observer.observe(element);
    view.scrollDOM.addEventListener("scroll", draw); draw();
    return () => { observer.disconnect(); view.scrollDOM.removeEventListener("scroll", draw); };
  }, [buffer, buffer.text]);
  const jump = (fraction: number) => {
    const view = editors.get(buffer)!.view;
    const line = view.state.doc.line(Math.max(1, Math.min(view.state.doc.lines, 1 + Math.floor(fraction * view.state.doc.lines))));
    view.dispatch({ effects: EditorView.scrollIntoView(line.from, { y: "center" }) });
  };
  return <canvas ref={canvas} className="minimap" role="navigation" aria-label="Document minimap" tabIndex={0}
    onPointerDown={event => { event.currentTarget.setPointerCapture(event.pointerId); jump((event.clientY - event.currentTarget.getBoundingClientRect().top) / event.currentTarget.clientHeight); }}
    onPointerMove={event => { if (event.buttons) jump((event.clientY - event.currentTarget.getBoundingClientRect().top) / event.currentTarget.clientHeight); }}
    onKeyDown={event => { if (event.key === "Home" || event.key === "End") { event.preventDefault(); jump(event.key === "Home" ? 0 : 1); } }} />;
}
</ui:copy>

The same server session publishes diagnostics independently of semantic tokens. Map them into the existing editor after its language compartment attaches, so errors become hoverable marks without replacing the buffer. Bind Ctrl-Space explicitly because the completion navigation keymap only handles an already open menu.

## Attach the existing DAP session without replacing text

The browser simple editor now borrows the workspace debugger for supported plain-file languages and Hickory file blocks. Its host lives above the tab, while a compartment attaches breakpoints, paused lines, values and expression evaluation after the readable editor mounts. Callbacks dereference the latest session; moving the DOM preserves undo and selection. Dirty text suppresses positional observations and refuses launch until saved, because the workspace adapter executes saved files. The right rail is a fixed column and its menus float over content. Hickory blocks use the existing source-line mapping and choose their generated path as the launch target. The native frontends use the same engine debug protocol through workspace-owned controllers and fixed native rails.

## Assemble the maintained output

Keep construction order explicit; the explanations above describe the decisions at each boundary.

<ui:file path="apps/web/src/simple/Document.tsx"><ui:paste select="#simple-1a4a59fc95-0"/><ui:paste select="#simple-document-minimap"/></ui:file>

</ui:doc>
