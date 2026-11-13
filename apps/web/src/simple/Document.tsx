import { createPortal } from "react-dom";
import type { TablePanelProps } from "../components/TablePanel";
import { smartEnter, enterMode } from "./SmartEnter";
import { gutterIconLayout, layoutEditorGutters } from "../editor/gutterIconLayout";
import { lazy, Suspense, useEffect, useLayoutEffect, useRef, useState, useSyncExternalStore } from "react";
import { Compartment, EditorState, StateEffect, EditorSelection } from "@codemirror/state";
import { EditorView, keymap, drawSelection, lineNumbers, rectangularSelection, crosshairCursor } from "@codemirror/view";
import { defaultKeymap, history, historyKeymap, indentMore, indentLess } from "@codemirror/commands";
import { codeFolding, foldKeymap, foldService, foldGutter, indentUnit } from "@codemirror/language";
import { search, searchKeymap, openSearchPanel, SearchQuery, setSearchQuery } from "@codemirror/search";
import { completionKeymap, startCompletion } from "@codemirror/autocomplete";
import { useWorkspaceLsp } from "../lsp/useLsp";
import { lspSupport, setLspDiagnostics, diagnosticRanges } from "../lsp/cmLsp";
import { lspFeatures } from "../lsp/cmLspFeatures";
import { completions } from "../lsp/completion";
import { positionToUtf16, utf16ToPosition } from "../lsp/positions";
import { sameDocumentUri } from "../lsp/documentUri";
import { Buffer } from "./buffers";
import { hickLangReady, loadHickLang } from "../editor/hickLang";
import { languageFromPath } from "../editor/languages";
import { isDebuggable } from "../debug/languages";
import { useFollowingDebugSession } from "../debug/plainDebugHosts";
import { IDLE_SESSION } from "../debug/useDebugger";
import { debugEditor, debugStateEffects, revealLine, breakpointLine, setDebuggerOutput } from "../debug/cmDebug";
import { DebugLaunchRail } from "../debug/DebugRail";
import { DebugStrip } from "../debug/DebugStrip";
import { observationSourceChanged } from "../debug/observations";
import { api } from "../api/client";
import { launchWithProfiles } from "../debug/launchProfiles";
import { claimReveal, onRevealLine, openLocation } from "../lib/revealLine";
const CsvGrid = lazy(async () => {
  const [{ TablePanel }, css] = await Promise.all([import("../components/TablePanel"), import("../styles.css?inline")]);
  return { default: function CsvSurface(props: TablePanelProps) {
    const host = useRef<HTMLDivElement>(null);
    const [root, setRoot] = useState<ShadowRoot | null>(null);
    useLayoutEffect(() => { if (host.current) setRoot(host.current.shadowRoot ?? host.current.attachShadow({ mode: "open" })); }, []);
    return <div ref={host} className="simple-csv-grid">{root && createPortal(<><style>{css.default.replaceAll(":root", ":host")}</style><TablePanel {...props} /></>, root)}</div>;
  } };
});
export type WordNavigationMode = "native" | "word" | "pieces";
export function wordNavigationMode(modifier: "control" | "alt"): WordNavigationMode {
  try { const value = localStorage.getItem(`editor.${modifier}Arrow`); return value === "word" || value === "pieces" ? value : "native"; }
  catch { return "native"; }
}
export function simpleWordOffset(source: string, offset: number, forward: boolean, pieces: boolean): number {
  const probe = !forward && offset > 0 ? offset - 1 : offset;
  const start = probe === 0 ? 0 : source.lastIndexOf("\n", probe - 1) + 1;
  const newline = source.indexOf("\n", probe), end = newline < 0 ? source.length : newline + 1;
  const pattern = pieces ? /(?:\p{Lu}\p{M}*)+(?=\p{Lu}\p{M}*\p{Ll})|(?:\p{Lu}\p{M}*)?(?:\p{Ll}\p{M}*)+|(?:\p{Lu}\p{M}*)+|(?:\p{N}\p{M}*)+|\p{L}[\p{L}\p{M}\u200C\u200D]*/gu : /[\p{L}\p{N}_][\p{L}\p{M}\p{N}_\u200C\u200D]*/gu;
  const matches = [...source.slice(start, end).matchAll(pattern)];
  if (forward) return matches.map(match => start + match.index! + match[0].length).find(at => at > offset) ?? end;
  return matches.map(match => start + match.index!).filter(at => at < offset).pop() ?? start;
}
export function simpleWordMove(view: EditorView, forward: boolean, extend: boolean, mode: WordNavigationMode): boolean {
  if (mode === "native" || view.composing) return false;
  // CodeMirror keeps lines indexed, avoiding a whole-document copy on each arrow.
  const ranges = view.state.selection.ranges.map(range => {
    const probe = !forward && range.head > 0 ? range.head - 1 : range.head;
    const line = view.state.doc.lineAt(probe);
    const local = line.text + (line.to < view.state.doc.length ? "\n" : "");
    const head = !extend && !range.empty ? (forward ? range.to : range.from) : line.from + simpleWordOffset(local, range.head - line.from, forward, mode === "pieces");
    return extend ? EditorSelection.range(range.anchor, head) : EditorSelection.cursor(head);
  });
  view.dispatch({ selection: EditorSelection.create(ranges, view.state.selection.mainIndex), scrollIntoView: true, userEvent: "select" });
  return true;
}
const wordNavigationKeys = ["control", "alt"].flatMap(modifier => [false, true].flatMap(forward => [false, true].map(extend => ({
  key: `${modifier === "control" ? "Ctrl" : "Alt"}-${extend ? "Shift-" : ""}Arrow${forward ? "Right" : "Left"}`,
  run: (view: EditorView) => simpleWordMove(view, forward, extend, wordNavigationMode(modifier as "control" | "alt")),
}))));
// Infer from nonblank lines only: blank scratch lines must not change policy.
export function simpleIndent(source: string): string {
  const prefixes = source.slice(0, 16_384).split("\n").slice(0, 200).filter(line => line.trim()).map(line => line.match(/^[ \t]*/)![0]);
  if (prefixes.filter(prefix => prefix.startsWith("\t")).length > prefixes.filter(prefix => prefix.startsWith(" ")).length) return "\t";
  const widths = prefixes.filter(prefix => /^ +$/.test(prefix)).map(prefix => prefix.length);
  const gcd = (a: number, b: number): number => b ? gcd(b, a % b) : a;
  const width = widths.reduce(gcd, 0);
  return " ".repeat(width > 0 && width <= 8 ? width : 4);
}
export function simpleEnter(view: EditorView, path: string): boolean {
  if (view.state.readOnly || view.composing) return false;
  const configured = enterMode(), mode = configured === "smart" && view.state.selection.ranges.length > 1 ? "copy" : configured;
  const source = view.state.doc.toString(), unit = simpleIndent(source);
  view.dispatch(view.state.changeByRange(range => {
    const plan = smartEnter(source, range.from, range.to, path, unit, mode);
    return { changes: { from: plan.from, to: plan.to, insert: plan.text }, range: EditorSelection.cursor(plan.caret) };
  }), { userEvent: "input.type" });
  return true;
}
export function simpleTab(view: EditorView): boolean {
  if (view.state.readOnly) return false;
  if (view.state.selection.ranges.some(range => !range.empty)) return indentMore(view);
  const unit = simpleIndent(view.state.doc.toString());
  view.dispatch(view.state.changeByRange(range => {
    const line = view.state.doc.lineAt(range.head);
    const column = range.head - line.from;
    const text = unit === "\t" ? unit : " ".repeat(unit.length - column % unit.length);
    return { changes: { from: range.from, to: range.to, insert: text }, range: EditorSelection.cursor(range.from + text.length) };
  }), { userEvent: "input.indent" });
  return true;
}
export function simpleFoldRanges(source: string, markdown = false): { line: number; from: number; to: number }[] {
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
  if (markdown) {
    const headings: { line: number; level: number; from: number }[] = [], sections: typeof headings = [];
    let at = 0, fence = "", payload = "", previous: { line: number; from: number } | null = null;
    const starts: number[] = [];
    rows.forEach((row, line) => {
      starts.push(at); const from = at + row.length; at = from + 1;
      const trimmed = row.trim();
      if (payload) { if (trimmed.includes(`:${payload}>`) && trimmed.includes("</")) payload = ""; previous = null; return; }
      if (fence) { if (trimmed.length >= fence.length && [...trimmed].every(char => char === fence[0])) fence = ""; previous = null; return; }
      const marker = row.match(/^ {0,3}(`{3,}|~{3,})/);
      if (marker) { fence = marker[1]; previous = null; return; }
      const literal = row.match(/^ {0,3}<[\w.-]+:(file|copy|cut|exec|patch|entry|diagram|math|table)\b[^>]*>/);
      if (literal) { if (!trimmed.includes("</") && !trimmed.endsWith("/>")) payload = literal[1]; previous = null; return; }
      const heading = row.match(/^ {0,3}(#{1,6})(?:[ \t]+|$)/);
      if (heading) { headings.push({ line, level: heading[1].length, from }); previous = null; }
      else if (previous && /^ {0,3}(?:=+|-+)\s*$/.test(row)) { headings.push({ ...previous, level: trimmed[0] === "=" ? 1 : 2 }); previous = null; }
      else previous = trimmed && !/^(?: {4}|\t)/.test(row) ? { line, from } : null;
    });
    for (const heading of [...headings, { line: rows.length, level: 0, from: source.length }]) {
      while (sections.length && sections[sections.length - 1].level >= heading.level) {
        const top = sections.pop()!, to = heading.line < rows.length ? starts[heading.line] - 1 : source.length;
        if (to > top.from) { const old = ranges.findIndex(range => range.line === top.line); if (old >= 0) ranges.splice(old, 1); ranges.push({ line: top.line, from: top.from, to }); }
      }
      sections.push(heading);
    }
  }
  return ranges.sort((a, b) => a.from - b.from);
}
const foldCache = new WeakMap<EditorState["doc"], Map<number, { from: number; to: number }>>();
function localFold(state: EditorState, from: number, markdown: boolean) {
  let ranges = foldCache.get(state.doc);
  if (!ranges) { ranges = new Map(simpleFoldRanges(state.doc.toString(), markdown).map(range => [range.from, range])); foldCache.set(state.doc, ranges); }
  return ranges.get(state.doc.lineAt(from).to) ?? null;
}
const editors = new WeakMap<Buffer, { view: EditorView; language: Compartment; prose: Compartment; richAttached?: boolean; toggleBreakpoint?: (line: number) => void }>();
export const SIMPLE_EDITOR_ATTACHED = "hickory:simple-editor-attached";
export function editorForBuffer(buffer: Buffer) { return editors.get(buffer); }
export function openFind(buffer: Buffer) { const view = editors.get(buffer)?.view; if (view) openSearchPanel(view); }
export function Document({ buffer }: { buffer: Buffer }) {
  const host = useRef<HTMLDivElement>(null);
  const [csvReady, setCsvReady] = useState(false);
  const [railWidth, setRailWidth] = useState(24);
  const [csvSource, setCsvSource] = useState(false);
  const csv = /\.csv$/i.test(buffer.file.path);
  const tableSource = buffer.text;
  useEffect(() => { if (!csv) return; let cancelled = false;
    void import("../components/TablePanel").then(() => { if (!cancelled) setCsvReady(true); }).catch(() => { /* Keep readable CSV text if the optional grid cannot load. */ });
    return () => { cancelled = true; };
  }, [csv]);
  useSyncExternalStore(buffer.subscribe, buffer.snapshot);
  const [view, setView] = useState<EditorView | null>(null);
  const [parserReady, setParserReady] = useState(hickLangReady);
  useEffect(() => {
    let cancelled = false;
    void loadHickLang().then(() => { if (!cancelled) setParserReady(true); }).catch(() => { /* Source remains readable when the parser is unavailable. */ });
    return () => { cancelled = true; };
  }, []);
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
        history(), layoutEditorGutters.of(false), gutterIconLayout, indentUnit.of(simpleIndent(buffer.text)), EditorState.tabSize.of(4), drawSelection(), EditorState.allowMultipleSelections.of(true), rectangularSelection(), crosshairCursor(), lineNumbers({ domEventHandlers: { mousedown: (editor, block, event) => {
          const line = breakpointLine(editor.state, editor.state.doc.lineAt(block.from).number - 1);
          if (line === null) return false;
          event.preventDefault(); editors.get(buffer)?.toggleBreakpoint?.(line); return true;
        } } }), codeFolding(), foldGutter({ openText: "−", closedText: "+" }), search({ top: true, literal: true }),
        EditorState.transactionExtender.of(tr => {
          const query = tr.effects.find(effect => effect.is(setSearchQuery))?.value as SearchQuery | undefined;
          return query && !query.literal ? { effects: setSearchQuery.of(new SearchQuery({ ...query, literal: true })) } : null;
        }),
        foldService.of((state, from) => localFold(state, from, /\.(md|markdown)$/i.test(buffer.file.path))), EditorView.lineWrapping, language.of([]), prose.of([]),
        keymap.of([...wordNavigationKeys, { key: "Enter", run: view => simpleEnter(view, buffer.file.path) }, { key: "Tab", run: simpleTab, shift: indentLess }, { key: "Ctrl-Space", run: startCompletion }, ...searchKeymap, ...foldKeymap, ...completionKeymap, ...defaultKeymap, ...historyKeymap]),
        EditorState.readOnly.of(buffer.readonly), EditorView.editable.of(!buffer.readonly),
        EditorView.contentAttributes.of({ "aria-label": buffer.file.path }),
        EditorView.updateListener.of(update => { if (update.docChanged) buffer.edit(update.state.doc.toString()); }),
        EditorView.theme({ "&": { height: "100%" }, ".cm-scroller": { overflow: "auto", fontFamily: "ui-monospace, monospace", fontSize: "13px" } }),
      ] }) });
      cached = { view, language, prose }; editors.set(buffer, cached); window.dispatchEvent(new Event(SIMPLE_EDITOR_ATTACHED));
    }
    cached.toggleBreakpoint = line => debugRef.current.toggleBreakpoint(line);
    host.current?.append(cached.view.dom);
    window.dispatchEvent(new Event(SIMPLE_EDITOR_ATTACHED));
    setView(cached.view);
    return () => { cached.view.dom.remove(); };
  }, [buffer]);
  useEffect(() => {
    if (!parserReady || !/\.(md|markdown)$/i.test(buffer.file.path) || buffer.text.length > 200_000) return;
    let cancelled = false;
    void import("../editor/markdownStyling").then(({ markdownStyling }) => {
      const cached = editors.get(buffer);
      if (!cancelled && cached && !cached.richAttached) cached.view.dispatch({ effects: cached.prose.reconfigure(markdownStyling()) });
    }).catch(() => { /* Readable source remains available if styling cannot load. */ });
    return () => { cancelled = true; };
  }, [buffer, parserReady]);
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
    if (!view || !debuggable || literate && !parserReady) return;
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
  }, [view, debuggable, language, literate, parserReady]);
  useEffect(() => {
    if (!view || !debuggable || literate && !parserReady) return;
    view.dispatch({ effects: setDebuggerOutput.of({ events: changed ? [] : debug.output ?? [], program: debug.program, literate, path: buffer.file.path }) });
    view.dispatch({ effects: debugStateEffects(changed ? { ...debug, pausedLine: null, variables: [], frames: [], watches: [] } : debug) });
    if (!changed && debug.pausedLine !== null) {
      const line = view.state.doc.line(Math.max(1, Math.min(view.state.doc.lines, debug.pausedLine + 1)));
      view.dispatch({ effects: EditorView.scrollIntoView(line.from, { y: "nearest" }) });
    }
  }, [view, debuggable, debug, changed, literate, parserReady]);
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
      { preferProgram: true, isCurrent: () => !!view?.dom.isConnected && buffer.text === source && !buffer.dirty && !buffer.saving });
  };
  return <div className="document-body">
    {csv && <button type="button" className="simple-source-toggle" onClick={() => setCsvSource(value => !value)}>{csvSource ? "Edit table as grid" : "Edit source text"}</button>}
    {csv && csvReady && !csvSource && view && <Suspense fallback={<span>Loading table…</span>}><CsvGrid source={buffer.text} chooseFormulaLanguage header delimiter="," sourceCurrent={() => !buffer.readonly && view.state.doc.toString() === tableSource && buffer.text === tableSource}
      onChange={buffer.readonly ? undefined : text => { if (!buffer.readonly && buffer.text === tableSource && view.state.doc.toString() === tableSource) view.dispatch({ changes: { from: 0, to: view.state.doc.length, insert: text }, userEvent: "input.table" }); }} /></Suspense>}
    <div className="editor-row" style={{ ...(csv && csvReady && !csvSource && view ? { display: "none" } : {}), "--simple-rail-width": `${railWidth}px` } as React.CSSProperties}><div ref={host} className="editor-host editor-cm-host" /><Minimap buffer={buffer} />
      <div className="simple-rail-resize" role="separator" aria-label="Resize right gutter" tabIndex={0}
        onKeyDown={event => { if (event.key === "ArrowLeft" || event.key === "ArrowRight") { event.preventDefault(); setRailWidth(width => Math.max(24, Math.min(220, width + (event.key === "ArrowLeft" ? 22 : -22)))); } }}
        onPointerDown={event => { event.currentTarget.setPointerCapture(event.pointerId); }}
        onPointerMove={event => { if (event.currentTarget.hasPointerCapture(event.pointerId)) setRailWidth(width => Math.max(24, Math.min(220, width - event.movementX))); }} />
      <DebugLaunchRail view={view} plainPath={debuggable && !literate ? buffer.file.path : undefined} enabled={["idle", "finished", "failed"].includes(debug.status)} onStart={debuggable && (!literate || parserReady) ? start : undefined} />
    </div>
    {debuggable && view && (!literate || parserReady) && <DebugStrip editorView={view} pausedLine={changed ? null : debug.pausedLine}
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
