<ui:doc xmlns:ui="http://www.hickorydocs.com/1.0">
# Retain the editor instance while moving a document

For maintainers iterating on the equivalent Apple, Android and browser editor frontends.

CodeMirror owns typing, selection, scroll and undo. Cache the editor by buffer identity and move its existing DOM between hosts. Buffer notifications update the status, not React-controlled document text. A source move reconfigures the cached editor’s address attribute while retaining the same DOM, selection, scroll and text history.

The minimap reserves forty units beside the text rather than overlaying readable lines. Its navigation changes scroll, never saved bytes. Language controls float over the editor and do not reflow it. Browser capabilities use the existing LSP client and its snapshot guards; native keyword and word-list previews deliberately do not claim semantic results. Native styling does not replace text, preserving the platform undo and selection model. Preview coloring skips very large files instead of delaying readable text indefinitely.

The parser is an independent background capability. Until it loads, literate debugger controls and rich parsing remain detached from the existing editor; a rejected load leaves readable source available. Each rich owner also awaits parser readiness, so a fast file open cannot invoke synchronous parsing before WebAssembly initialization.

Text from the initial read remains selectable while ownership is pending.
When the same buffer acquires authoritative edit access, reconfigure the
mounted editor's permission facet without replacing its document or history.

After text is visible, Markdown files load and attach the existing display-only prose decorations independently of LSP. Headings, bold and italic update with typing without rewriting source or rebuilding the editor. Fenced code stays literal; plain files keep their original appearance.

Search uses CodeMirror’s native search transactions and keymap. The panel is an absolute overlay, so showing it cannot resize the cached editor. Literal search treats backslashes as text; regex mode enables capture replacements. Read-only policy stays in editor state and search navigation reveals folded matches.

The editor installs CodeMirror folding with its initial text. A linear indentation scan supplies regions without waiting for a parser or server; the existing LSP compartment may add language-specific regions later. Markdown ATX and Setext headings also fold through their next peer or ancestor; fenced and Hickory literal payloads suppress heading discovery. A dedicated left gutter shows − on expanded headers and + on collapsed headers. CodeMirror anchors these controls to its measured lines. Folding changes editor decorations rather than buffer bytes, and the existing cached view keeps folds across tab moves. Debug launch actions keep their separate right-hand rail.

Tab inserts a detected indentation unit at the next stop, while selections use the editor’s block indentation. Nonblank leading whitespace selects tabs or the greatest common divisor of space indents; empty files default to four spaces. This policy is available immediately, before language services arrive.

Indentation inference samples at most 200 initial lines and 16,384 characters.
Keep that work bounded now that Backspace can request the same policy as Tab;
large-file keystrokes must not repeatedly split the entire document. The sample
is a heuristic, not authoritative formatter configuration.

## Move through identifiers without replacing native editing

Keep Ctrl and Alt/Option independent, persisted editor preferences. Platform default
leaves the key with the host; custom movement touches only the document selection,
including read-only documents. Whole identifiers include underscores; pieces stop
at underscore gaps, camel-case transitions, acronym endings and number runs. Shift
retains the anchor so reversing direction contracts a selection before extending it.
Only the adjacent logical line is scanned; no parser, language service or document
rebuild runs for these arrows. Composition and unrelated modifier combinations stay
with the native editor. This is logical source movement, not visual bidi navigation;
platform default remains available for native linguistic and visual navigation.
Settings live in existing menus, keeping readable text spatially stable.

Smart Enter delegates to the [shared indentation mechanism](../../../../smart-enter.md), then
applies one ordinary edit and its proposed caret. The complete replacement range,
including whitespace before an existing closer, passes the same hidden-source,
composition and permission checks as other edits. Changing the persisted Enter
mode affects open editors; the platform retains its own undo transaction.

Standalone CSV uses the same isolated stylesheet as inline rich widgets. Mount its established grid in a stable shadow root so empty cells, resize handles and menus retain their real geometry without importing the full workspace theme into the simple shell. Loading this optional surface never replaces the cached source editor or its undo history.

CSV opens as editable text. Load the grid and its stylesheet together in the
background, then enable the existing view switch. A completed download only
enables that switch. Changing views requires a person to invoke it, so a slow
grid can never take focus from a person already editing the file.

## Separate semantic questions from their controls

A provider can answer with locations, nested document symbols, flat workspace symbols or call-hierarchy items. Normalize those shapes to named source locations once. Outlines inherit the queried document identity; workspace symbols and calls must carry their own identity. Caller/callee reads prepare provider items before asking for incoming and outgoing calls. The origin lease stays current through preparation so an obsolete reply cannot start the follow-up. A simple identifier from a named result travels with its source position. The host checks the exact identifier and its boundaries on a different file before activating it.

The workspace buffer owns its language session, matching Apple and Android. Hiding a tab must not close the provider's document and erase usages or unsaved semantic knowledge. A small per-buffer subscription lets a mounted editor borrow current capabilities and diagnostics without owning their lifecycle; standalone document surfaces retain their existing local session. Provider attachment still follows readable text, and removing a buffer or unmounting the workspace closes its owner.

Use the engine's document identity for literate controls and source folding.
The same retained main can use a literal non-Markdown filename. The usual
Markdown extension still provides a presentation hint while metadata loads.

A query identifies a language-provider operation; its answer is a list of source locations. Menus and floating results consume these values without owning the LSP session or pane layout. Admit only authored workspace URI paths for opening. Other provider identities remain visible as unsupported results. Each request captures source, selection, provider and a monotonic ticket; late answers never navigate or replace a newer query. No query edits or saves text.

The Launch profiles control requests the catalog only when invoked. Command
profiles use the same source and catalog revision as Debug, then hand the
new session to the workspace terminal. Opening and typing in a source buffer
do not request the catalog.

<ui:copy id="simple-browser-semantic-locations">
import {sourceText,authoredSource,authoredOffset,editorOffset,editorOffsetIn,authoredChanges,authoredInsert} from './SourceText';
import {sourceSnippets,moveSourceSnippet,finishSourceSnippet,openSourceSnippetChoices,sourceSnippetMenuState,readSourceEditorIntent,dispatchSourceSnippetTransactions} from './SourceSnippets';
import {useCoverageLineActions} from "./SourceCoverage";
import {useSourceTestLineActions} from "./SourceTestGutters";
import { useEffect, useLayoutEffect, useRef, useState, useSyncExternalStore } from "react";
import { semanticPath, semanticLocations, querySemanticLocations, type SemanticQuery, type SemanticLocation } from "./Intelligence";
export { semanticQueries, semanticPath, semanticLocations, querySemanticLocations } from "./Intelligence";
export type { SemanticQuery, SemanticLocation } from "./Intelligence";
const terminalReadings = new WeakMap<Buffer, { revision: number; id: string; at: number; program: string; text: string }>();
const semanticCommands = new WeakMap<Buffer, (query: SemanticQuery, text?: string) => void>();
export function openSemanticQuery(buffer: Buffer, query: SemanticQuery, text = "") { semanticCommands.get(buffer)?.(query, text); }

</ui:copy>

The mounted editor keeps normalized display lines beside the exact authored source field. Every authoritative source comparison and language-service coordinate crosses that boundary explicitly; local folding and indentation continue to inspect display text. Capability configuration attaches to the same state and cannot rewrite line endings or native undo. Raw clipboard separators pass through the authored edit builder before CodeMirror can normalize them.

<ui:copy id="simple-1a4a59fc95-0">
import { createPortal } from "react-dom";
import type { TablePanelProps } from "../components/TablePanel";
import { smartEnter, enterMode } from "./SmartEnter";
import { gutterIconLayout, layoutEditorGutters } from "../editor/gutterIconLayout";
import { Compartment, EditorState, StateEffect, EditorSelection } from "@codemirror/state";
import { EditorView, keymap, drawSelection, lineNumbers, rectangularSelection, crosshairCursor } from "@codemirror/view";
import { addCursorAbove, addCursorBelow, defaultKeymap, history, historyKeymap, undo,redo,undoDepth,redoDepth,indentMore, indentLess } from "@codemirror/commands";
import { codeFolding, foldKeymap, foldService, foldGutter, indentUnit } from "@codemirror/language";
import { search, searchKeymap, openSearchPanel, SearchQuery, setSearchQuery } from "@codemirror/search";
import { completionKeymap, startCompletion } from "@codemirror/autocomplete";
import { useSimpleLanguage, useHasWorkspaceLanguages } from "./WorkspaceLanguages";
import { lspSupport, setLspDiagnostics, diagnosticRanges } from "../lsp/cmLsp";
import { lspFeatures } from "../lsp/cmLspFeatures";
import { completions } from "../lsp/completion";
import {sourceCompletion} from "./SourceCompletion";
import {supportedSnippetTransformPrograms} from "./SnippetTransform";
import {resolveSnippetCompletion} from "./SnippetContext";
import {readCompletionAcceptance} from "../lsp/completionAcceptance";
import { positionToUtf16, utf16ToPosition } from "../lsp/positions";
import { sameDocumentUri,documentUri } from "../lsp/documentUri";
import { diagnosticBoundSource } from "../lsp/diagnosticRevision";
import { Buffer } from "./buffers";
import { hickLangReady, loadHickLang } from "../editor/hickLang";
import { languageFromPath } from "../editor/languages";
import { isDebuggable } from "../debug/languages";
import { useFollowingDebugSession } from "../debug/plainDebugHosts";
import { IDLE_SESSION } from "../debug/useDebugger";
import { debugEditor, debugStateEffects, revealLine, breakpointLine, setDebuggerOutput } from "../debug/cmDebug";
import { useGitEditorComparison } from "./GitHistory";
import { simplePatchHunks } from "../lib/diff";
import { DebugLaunchRail } from "../debug/DebugRail";
import { DebugStrip } from "../debug/DebugStrip";
import { observationSourceChanged } from "../debug/observations";
import { api, request } from "../api/client";
import { launchWithProfiles } from "../debug/launchProfiles";
import { claimRevealPosition, onRevealLine, openProviderLocation, openCapturedLocation } from "../lib/revealLine";
async function loadCsvGrid() {
  const [{ TablePanel }, css] = await Promise.all([import("../components/TablePanel"), import("../styles.css?inline")]);
  return { TablePanel, css: css.default.replaceAll(":root", ":host") };
}
type CsvGridModules = Awaited<ReturnType<typeof loadCsvGrid>>;
function CsvGrid({ modules: { TablePanel, css }, ...props }: TablePanelProps & { modules: CsvGridModules }) {
  const host = useRef<HTMLDivElement>(null);
  const [root, setRoot] = useState<ShadowRoot | null>(null);
  useLayoutEffect(() => { if (host.current) setRoot(host.current.shadowRoot ?? host.current.attachShadow({ mode: "open" })); }, []);
  return <div ref={host} className="simple-csv-grid">{root && createPortal(<><style>{css}</style><TablePanel {...props} /></>, root)}</div>;
}
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
  let ranges: { line: number; from: number; to: number }[] = [];
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
    const starts: number[] = [], patchRows = new Set<number>(), patchFolds: typeof ranges = [];
    let patchStart = -1;
    function finishPatch(end: number) {
      if (patchStart < 0) return;
      for (let row = patchStart; row < end; row++) patchRows.add(row);
      for (const hunk of simplePatchHunks(rows, patchStart + 1, end)) {
        const endOf = (line: number) => starts[line] + rows[line].replace(/\r$/, "").length;
        patchFolds.push({ line: hunk.header, from: endOf(hunk.header), to: endOf(hunk.end) });
      }
      patchStart = -1;
    }
    rows.forEach((row, line) => {
      starts.push(at); const from = at + row.length; at = from + 1;
      const trimmed = row.trim();
      if (payload) { if (trimmed.includes(`:${payload}>`) && trimmed.includes("</")) { finishPatch(line); payload = ""; } previous = null; return; }
      if (fence) { if (trimmed.length >= fence.length && [...trimmed].every(char => char === fence[0])) { finishPatch(line); fence = ""; } previous = null; return; }
      const marker = row.match(/^ {0,3}(`{3,}|~{3,})/);
      if (marker) {
        fence = marker[1];
        if (/^(diff|patch)(?:\s|$)/.test(row.slice(marker[0].length).trim())) patchStart = line;
        previous = null; return;
      }
      const literal = row.match(/^ {0,3}<[\w.-]+:(file|copy|cut|exec|patch|entry|diagram|math|table)\b[^>]*>/);
      if (literal) { if (!trimmed.includes("</") && !trimmed.endsWith("/>")) {
        payload = literal[1]; if (payload === "patch") patchStart = line;
      } previous = null; return; }
      const heading = row.match(/^ {0,3}(#{1,6})(?:[ \t]+|$)/);
      if (heading) { headings.push({ line, level: heading[1].length, from }); previous = null; }
      else if (previous && /^ {0,3}(?:=+|-+)\s*$/.test(row)) { headings.push({ ...previous, level: trimmed[0] === "=" ? 1 : 2 }); previous = null; }
      else previous = trimmed && !/^(?: {4}|\t)/.test(row) ? { line, from } : null;
    });
    finishPatch(rows.length);
    ranges = ranges.filter(range => !patchRows.has(range.line)).concat(patchFolds);
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
import {mountRetainedEditor,type RetainedEditorMount} from "./EditorMounts";
const debugPositions=new WeakMap<Buffer,string>();
const editors = new WeakMap<Buffer, RetainedEditorMount&{ language: Compartment; prose: Compartment; completion:Compartment; address:Compartment; permission:Compartment; richAttached?: boolean; toggleBreakpoint?: (line: number) => void }>();
export const SIMPLE_EDITOR_ATTACHED = "hickory:simple-editor-attached";
export function editorForBuffer(buffer: Buffer) { return editors.get(buffer); }
export function openFind(buffer: Buffer) { const view = editors.get(buffer)?.view; if (view) openSearchPanel(view); }
export function sourceSnippetContext(buffer:Buffer|undefined){const view=buffer?editors.get(buffer)?.view:undefined,menu=view?sourceSnippetMenuState(view):null;return {sourceUndo:!!view&&!view.composing&&undoDepth(view.state)>0,sourceRedo:!!view&&!view.composing&&redoDepth(view.state)>0,snippet:!!menu?.active,snippetChoices:!!menu?.choices}}
export function runEditorCommand(buffer: Buffer,command: "tools"|"rectangle"|"cursorBelow"|"cursorAbove"|"snippetNext"|"snippetPrevious"|"snippetChoices"|"snippetFinish"|"undo"|"redo") {
  const view=editors.get(buffer)?.view;if(!view)return false;view.focus();
  if(command==="undo")return !buffer.readonly&&undo(view);
  if(command==="redo")return !buffer.readonly&&redo(view);
  if(command==="tools")return startCompletion(view);
  if(command==="snippetNext")return moveSourceSnippet(view,1);
  if(command==="snippetPrevious")return moveSourceSnippet(view,-1);
  if(command==="snippetChoices")return openSourceSnippetChoices(view);
  if(command==="snippetFinish")return finishSourceSnippet(view);
  if(command==="cursorBelow")return addCursorBelow(view);
  if(command==="cursorAbove")return addCursorAbove(view);
  const range=view.state.selection.main,doc=view.state.doc,anchor=doc.lineAt(range.anchor),head=doc.lineAt(range.head);
  const left=Math.min(range.anchor-anchor.from,range.head-head.from),right=Math.max(range.anchor-anchor.from,range.head-head.from),ranges=[];
  const column=(text:string,at:number)=>{at=Math.min(at,text.length);return at>0&&at<text.length&&/[\uD800-\uDBFF]/.test(text[at-1])&&/[\uDC00-\uDFFF]/.test(text[at])?at-1:at;};
  for(let number=Math.min(anchor.number,head.number);number<=Math.max(anchor.number,head.number);number++){const line=doc.line(number);ranges.push(EditorSelection.range(line.from+column(line.text,left),line.from+column(line.text,right)));}
  view.dispatch({selection:EditorSelection.create(ranges),scrollIntoView:true});return true;
}

function usesLiteratePresentation(buffer: Buffer): boolean {
  return Boolean(buffer.file.doc_id) || /\.(md|markdown)$/i.test(buffer.file.path);
}

export function Document({ buffer, current, renderTools }: {
  buffer: Buffer; current?: () => boolean; renderTools?: (view: EditorView) => React.ReactNode;
}) {
  const host = useRef<HTMLDivElement>(null);
  const [csvGrid, setCsvGrid] = useState<CsvGridModules | null>(null);
  const [csvFailed, setCsvFailed] = useState(false);
  const [railWidth, setRailWidth] = useState(24);
  const [csvGridSelected, setCsvGridSelected] = useState(false);
  const csv = /\.csv$/i.test(buffer.file.path);
  const tableSource = buffer.text;
  useEffect(() => { if (!csv) return; let cancelled = false;
    void loadCsvGrid().then(modules => { if (!cancelled) setCsvGrid(modules); }).catch(() => { if (!cancelled) setCsvFailed(true); });
    return () => { cancelled = true; };
  }, [csv]);
  useSyncExternalStore(buffer.subscribe, buffer.snapshot);
  const [view, setView] = useState<EditorView | null>(null);
  const showingCsvGrid = csv && csvGridSelected && !!csvGrid && !!view;
  const [parserReady, setParserReady] = useState(hickLangReady);
  useEffect(() => {
    let cancelled = false;
    void loadHickLang().then(() => { if (!cancelled) setParserReady(true); }).catch(() => { /* Source remains readable when the parser is unavailable. */ });
    return () => { cancelled = true; };
  }, []);
  const language = languageFromPath(buffer.file.path);
  const literate = usesLiteratePresentation(buffer);
  const debuggable = !buffer.transient && buffer.startupWorkspace === undefined && (literate || isDebuggable(language));
  const debug = useFollowingDebugSession(buffer.file.path,undefined,buffer,()=>!buffer.resourceDeleted&&(current?.()??true),{
    read:()=>{const file=buffer.file,text=buffer.text,namespace=buffer.namespace,revision=buffer.sourceRevision;return {uri:documentUri(file.path),workspace:namespace,namespace,identity:buffer,text,saved:!buffer.dirty&&!buffer.saving&&!buffer.resourceDeleted,current:()=>buffer.file===file&&buffer.sourceRevision===revision&&buffer.text===text&&buffer.namespace===namespace&&!buffer.resourceDeleted&&(current?.()??true)};},subscribe:buffer.subscribe
  }) ?? IDLE_SESSION;
  const launchPosition = useRef<{ at?: number; revision: typeof buffer.sourceRevision } | null>(null);
  const debugRef = useRef(debug); debugRef.current = debug;
  const changed = buffer.dirty || observationSourceChanged(debug.reading, buffer.text);
  const frame=debug.frames.find(item=>item.id===debug.selectedFrame),frameHash=frame?.in_document&&debug.positionSourceBound!==false&&!(frame.source_reference&&frame.source_reference>0)?frame.source_hash:null;
  const [debugBinding,setDebugBinding]=useState<{source:string;hash:string;frame:number;current:()=>boolean;interaction:number}|null>(null);
  const sourceBound=!!debugBinding&&debugBinding.source===buffer.text&&debugBinding.hash===frameHash&&debugBinding.frame===debug.selectedFrame&&debugBinding.current();
  useEffect(()=>{if(debug.status!=="paused"||!frame||!frame.in_document||!frameHash||!/^[0-9a-f]{64}$/.test(frameHash))return;const captured=debug.captureFrame?.(frame.id);if(!captured)return;
    const source=buffer.text,interaction=editorForBuffer(buffer)?.interaction??0;let retired=false;
    for(const character of source)if(character.length===1&&character.charCodeAt(0)>=0xD800&&character.charCodeAt(0)<=0xDFFF)return;
    void crypto.subtle.digest("SHA-256",new TextEncoder().encode(source)).then(bytes=>{const hash=[...new Uint8Array(bytes)].map(byte=>byte.toString(16).padStart(2,"0")).join("");if(!retired&&buffer.text===source&&captured.current()&&hash===frameHash)setDebugBinding({source,hash,frame:frame.id,current:captured.current,interaction});}).catch(()=>{});return()=>{retired=true;};
  },[buffer,buffer.text,debug.status,debug.threadStop,debug.selectedThread,debug.selectedFrame,frameHash,debug.captureFrame]);
  const positionLine=frame?.in_document?frame.line:null;
  const positionCurrent=!changed&&sourceBound&&typeof positionLine==="number"&&Number.isInteger(positionLine)&&positionLine>=0&&(!view||positionLine<view.state.doc.lines);
  const [debugNotice, setDebugNotice] = useState("");
  const session = useSimpleLanguage(buffer);
  const workspaceLanguages = useHasWorkspaceLanguages();
  const [semanticResult, setSemanticResult] = useState<{ source: string; anchor: number; head: number; ticket: number; message: string; locations: SemanticLocation[] } | null>(null);
  const semanticTicket = useRef(0);
  function semanticOrigin(view: EditorView, ticket: number, source: string, anchor: number, head: number) {
    const {file, namespace, sourceRevision} = buffer;
    const owner = editorForBuffer(buffer), mount = owner?.mount;
    return () => semanticTicket.current === ticket && view.dom.isConnected && !!owner && owner.view === view
      && editorForBuffer(buffer) === owner && owner.mount === mount
      && buffer.file === file && buffer.namespace === namespace && buffer.sourceRevision === sourceRevision
      && !buffer.resourceDeleted && authoredSource(view.state) === source
      && view.state.selection.main.anchor === anchor && view.state.selection.main.head === head;
  }
  const semanticPanel = useRef<HTMLDivElement>(null);
  useEffect(() => { if (semanticResult) semanticPanel.current?.focus(); }, [semanticResult !== null]);
  const [, refreshSemanticSelection] = useState(0);
  useEffect(() => {
    if (!view) return;
    const query = async (kind: SemanticQuery, text = "") => {
      const id = ++semanticTicket.current, client = session.client, source = authoredSource(view.state);
      const { anchor, head } = view.state.selection.main;
      const current = semanticOrigin(view, id, source, anchor, head);
      const result = { source, anchor, head, ticket: id, message: "Searching…", locations: [] as SemanticLocation[] };
      setSemanticResult(result);
      if (!client || !session.ready) { setSemanticResult({ ...result, message: "Language services unavailable. Install a server for this workspace, then retry." }); return; }
      try {
        const locations = await querySemanticLocations(client, session.uri, utf16ToPosition(source, authoredOffset(view.state,head)), kind, text, current);
        if (current()) setSemanticResult({ ...result, locations, message: locations.length ? `${locations.length} locations` : "No results. The provider may still be indexing or may not support this query. Retry or choose another symbol." });
      } catch (error) { if (current()) setSemanticResult({ ...result, message: `${String(error)}. Retry language services.` }); }
    };
    semanticCommands.set(buffer, (kind, text) => { void query(kind, text); });
    return () => { semanticTicket.current++; semanticCommands.delete(buffer); };
  }, [buffer, view, session.client, session.uri, session.ready]);
  useEffect(() => {
    if (!view) return;
    const compartment = new Compartment();
    view.dispatch({ effects: StateEffect.appendConfig.of(compartment.of(EditorView.updateListener.of(update => {
      if (update.docChanged || update.selectionSet) { semanticTicket.current++; refreshSemanticSelection(value => value + 1); }
    }))) });
    return () => { view.dispatch({ effects: compartment.reconfigure([]) }); };
  }, [view]);
  const [capabilities, refreshCapabilities] = useState(0);
  useEffect(() => session.client?.onServerCapabilities(() => refreshCapabilities(value => value + 1)), [session.client]);
  useLayoutEffect(() => {
    let cached = editors.get(buffer);
    if (!cached) {
      const language = new Compartment(), prose = new Compartment(), completion=new Compartment(), address=new Compartment(), permission=new Compartment();
      const view = new EditorView({ dispatchTransactions:dispatchSourceSnippetTransactions,state: EditorState.create({ doc: buffer.text, ...(buffer.recoveryView?{selection:EditorSelection.single(editorOffsetIn(buffer.text,buffer.recoveryView.selection.anchor),editorOffsetIn(buffer.text,buffer.recoveryView.selection.head))}:{}), extensions: [
        sourceText(buffer.text), history(), EditorView.domEventHandlers({paste:(event,view)=>{const text=event.clipboardData?.getData('text/plain');if(!text||view.state.readOnly||view.composing)return false;event.preventDefault();view.dispatch(authoredInsert(view.state,text));return true;}}), sourceSnippets({writable:()=>!buffer.readonly&&!buffer.saving,notice:value=>buffer.setCompletionNotice(value)}), layoutEditorGutters.of(false), gutterIconLayout, indentUnit.of(simpleIndent(buffer.text)), EditorState.tabSize.of(4), drawSelection(), EditorState.allowMultipleSelections.of(true), rectangularSelection(), crosshairCursor(), lineNumbers({ domEventHandlers: { mousedown: (editor, block, event) => {
          const line = breakpointLine(editor.state, editor.state.doc.lineAt(block.from).number - 1);
          if (line === null) return false;
          event.preventDefault(); editors.get(buffer)?.toggleBreakpoint?.(line); return true;
        } } }), codeFolding(), foldGutter({ openText: "−", closedText: "+" }), search({ top: true, literal: true }),
        EditorState.transactionExtender.of(tr => {
          const query = tr.effects.find(effect => effect.is(setSearchQuery))?.value as SearchQuery | undefined;
          return query && !query.literal ? { effects: setSearchQuery.of(new SearchQuery({ ...query, literal: true })) } : null;
        }),
        foldService.of((state, from) => localFold(state, from, usesLiteratePresentation(buffer))), EditorView.lineWrapping, language.of([]), prose.of([]), completion.of(completions({})),
        keymap.of([...wordNavigationKeys, { key: "Enter", run: view => simpleEnter(view, buffer.file.path) }, { key: "Tab", run: simpleTab, shift: indentLess }, { key: "Ctrl-Space", run: startCompletion }, ...searchKeymap, ...foldKeymap, ...completionKeymap, ...defaultKeymap, ...historyKeymap]),
        permission.of([EditorState.readOnly.of(buffer.readonly), EditorView.editable.of(!buffer.readonly)]),
        // Rich widgets also dispatch transactions. A recorded or provisional
        // source can only receive the exact text held by its buffer.
        EditorState.transactionFilter.of(transaction => transaction.docChanged && buffer.readonly
          && (buffer.transient || buffer.startupWorkspace !== undefined)
          && authoredSource(transaction.state) !== buffer.text ? [] : transaction),
        address.of(EditorView.contentAttributes.of({ "aria-label": buffer.file.path })),
        EditorView.updateListener.of(update => {
          if(update.selectionSet){const held=editors.get(buffer);if(held)held.interaction++;}
          if (update.docChanged||authoredSource(update.startState)!==authoredSource(update.state)) buffer.edit(authoredSource(update.state));
        }),
        EditorView.theme({ "&": { height: "100%" }, ".cm-scroller": { overflow: "auto", fontFamily: "ui-monospace, monospace", fontSize: "13px" } }),
      ] }) });
      cached = { view, language, prose, completion, address,permission,interaction:0 }; editors.set(buffer, cached);view.scrollDOM.addEventListener("scroll",()=>{const held=editors.get(buffer);if(held)held.interaction++;}); window.dispatchEvent(new Event(SIMPLE_EDITOR_ATTACHED));
    }
    cached.view.dispatch({effects:cached.address.reconfigure(EditorView.contentAttributes.of({"aria-label":buffer.file.path}))});
    cached.toggleBreakpoint = line => { if (!buffer.transient) debugRef.current.toggleBreakpoint(line); };
    if(!host.current)return;
    const held=cached,release=mountRetainedEditor(buffer,held,host.current,()=>editors.get(buffer)===held);
    window.dispatchEvent(new Event(SIMPLE_EDITOR_ATTACHED));
    if(buffer.recoveryView){const saved=buffer.recoveryView,interaction=held.interaction,revision=buffer.sourceRevision;buffer.recoveryView=undefined;requestAnimationFrame(()=>{if(editors.get(buffer)===held&&held.interaction===interaction&&buffer.sourceRevision===revision){held.view.scrollDOM.scrollLeft=saved.scroll_x;held.view.scrollDOM.scrollTop=saved.scroll_y;}});}
    setView(cached.view);
    return release;
  }, [buffer]);
  useLayoutEffect(() => {
    const cached = editors.get(buffer);
    if (!cached || cached.view.state.readOnly === buffer.readonly) return;
    cached.view.dispatch({effects:cached.permission.reconfigure([EditorState.readOnly.of(buffer.readonly), EditorView.editable.of(!buffer.readonly)])});
  }, [buffer, buffer.readonly]);
  useEffect(() => {
    if (!parserReady || !/\.(md|markdown)$/i.test(buffer.file.path) || buffer.text.length > 200_000) return;
    let cancelled = false;
    void import("../editor/markdownStyling").then(({ markdownStyling }) => {
      const cached = editors.get(buffer);
      if (!cancelled && cached && !cached.richAttached) cached.view.dispatch({ effects: [cached.prose.reconfigure(markdownStyling()), cached.view.scrollSnapshot()] });
    }).catch(() => { /* Readable source remains available if styling cannot load. */ });
    return () => { cancelled = true; };
  }, [buffer, parserReady]);
  useEffect(() => {
    const cached = editors.get(buffer);
    if (!cached || !session.client || !session.ready) return;
    const { view, language,completion } = cached;
    let providerCurrent = true;
    const positionAt = (offset: number, editor: EditorView) => utf16ToPosition(authoredSource(editor.state), authoredOffset(editor.state,offset));
    const offsetAt = (position: { line: number; character: number }, editor: EditorView) => editorOffset(editor.state,positionToUtf16(authoredSource(editor.state), position));
    view.dispatch({ effects: [language.reconfigure([
      ...lspSupport({ client: session.client, uri: session.uri, positionAt,
        onReferences: locations => {
          if (!providerCurrent) return;
          const { anchor, head } = view.state.selection.main;
          setSemanticResult({ source: authoredSource(view.state), anchor, head, ticket: ++semanticTicket.current, message: `${locations.length} locations`, locations: semanticLocations(locations) });
        },
        navigationIntent: editor => {
          const {anchor, head} = editor.state.selection.main;
          const current = semanticOrigin(editor, ++semanticTicket.current, authoredSource(editor.state), anchor, head);
          return () => providerCurrent && current();
        },
        onNavigate: (target, originCurrent) => { if (!providerCurrent || !originCurrent?.()) return; if (sameDocumentUri(target.uri, session.uri)) {
          const anchor = offsetAt(target.range.start, view);
          view.dispatch({ selection: { anchor }, effects: EditorView.scrollIntoView(anchor, { y: "center" }) });
        } else {
          const path = semanticPath(target.uri);
          if (path) openProviderLocation(path, target.range.start.line + 1, target.range.start.character, originCurrent);
          else { const { anchor, head } = view.state.selection.main; setSemanticResult({ source: authoredSource(view.state), anchor, head, ticket: ++semanticTicket.current, message: "Provider target is outside authored workspace files.", locations: [target] }); }
        } } }),
      ...lspFeatures({ client: session.client, uri: session.uri, positionAt, offsetAt, inlayHints: true }),
    ]),completion.reconfigure(!buffer.readonly ? completions({ lsp: { client: session.client, uri: session.uri,transformPrograms:supportedSnippetTransformPrograms,
        positionAt: (offset, state) => utf16ToPosition(authoredSource(state), authoredOffset(state,offset)) },
        lspItem:(item,context)=>sourceCompletion(item,context,{uri:session.uri,current:()=>providerCurrent,writable:()=>!buffer.readonly&&!buffer.saving&&!buffer.transient,
          resolve:async original=>{
            const client=session.client!,uri=session.uri,source=authoredSource(view.state),doc=view.state.doc,selection=view.state.selection,revision=buffer.sourceRevision,file=buffer.file,path=file.path,namespace=buffer.namespace,intent=readSourceEditorIntent(view.state),job=readCompletionAcceptance(view)?.job;
            const current=()=>providerCurrent&&buffer.namespace===namespace&&buffer.file===file&&buffer.file.path===path&&buffer.sourceRevision===revision&&!buffer.readonly&&!buffer.saving&&!buffer.resourceDeleted&&!buffer.transient&&view.dom.isConnected&&view.state.doc===doc&&view.state.selection.eq(selection)&&readSourceEditorIntent(view.state)===intent&&(!job||readCompletionAcceptance(view)?.job===job&&readCompletionAcceptance(view)?.phase==='resolving');
            return await resolveSnippetCompletion(original,{source,from:authoredOffset(view.state,selection.main.from),to:authoredOffset(view.state,selection.main.to),current,request:async item=>await client.request("completionItem/resolve",{...item,textDocument:{uri}}) as import("../lsp/client").CompletionItem});
          },
          group:{
            query:(offset,source)=>session.client!.completion(session.uri,utf16ToPosition(source,offset),{triggerKind:1},supportedSnippetTransformPrograms),
            resolve:item=>session.client!.resolveCompletion(item),
          },
          notice:text=>buffer.setCompletionNotice(text)}) }) : completions({}))] });
    return () => { providerCurrent = false; view.dispatch({ effects: [language.reconfigure([]),completion.reconfigure(completions({}))] }); };
  }, [buffer, session.client, session.uri, session.ready, capabilities]);
  useEffect(() => {
    const cached = editors.get(buffer);
    if (!cached || !session.client || !session.ready) return;
    cached.view.dispatch({ effects: setLspDiagnostics.of(diagnosticRanges(session.diagnostics.filter(diagnostic => (diagnosticBoundSource(diagnostic) ?? session.diagnosticSource) === buffer.text),
      position => editorOffset(cached.view.state,positionToUtf16(authoredSource(cached.view.state), position)))) });
  }, [buffer, session.client, session.ready, session.diagnostics, session.diagnosticSource, buffer.text, capabilities]);
  // Attach debugger state to the existing editor. Callbacks read the current
  // session so tab moves and incoming DAP events cannot retain obsolete verbs.
  useEffect(() => {
    if (!view || !debuggable || literate && !parserReady) return;
    const compartment = new Compartment();
    view.dispatch({ effects: StateEffect.appendConfig.of(compartment.of(debugEditor({ language: literate ? undefined : language,
      lineNumbers: false, numberBreakpoints: true,
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
    const reading = terminalReadings.get(buffer);
    if (reading?.revision === buffer.sourceRevision) {
      view.dispatch({ effects: setDebuggerOutput.of({ events: reading.text ? [{ category: "console", text: reading.text }] : [],
        program: reading.program, literate, path: buffer.file.path, launchAt: reading.at }) });
    } else {
      view.dispatch({ effects: setDebuggerOutput.of({ events: changed ? [] : debug.output ?? [], program: debug.program, literate, path: buffer.file.path,
        launchAt: launchPosition.current?.revision === buffer.sourceRevision ? launchPosition.current?.at : undefined }) });
    }
    view.dispatch({ effects: debugStateEffects(!positionCurrent ? { ...debug, pausedLine: null, variables: [], frames: [], watches: [] } : {...debug,pausedLine:positionLine}) });
    if(debug.status==="starting"||debug.status==="running")debugPositions.delete(buffer);
    if (positionCurrent&&positionLine!==null&&positionLine!==undefined) {
      const key=JSON.stringify([debug.observationScope?.run??null,debug.threadStop??null,debug.selectedThread??null,debug.selectedFrame,positionLine]);
      if(debugPositions.get(buffer)!==key){debugPositions.set(buffer,key);if(debugBinding?.interaction===editorForBuffer(buffer)?.interaction){const line=view.state.doc.line(positionLine+1);view.dispatch({effects:EditorView.scrollIntoView(line.from,{y:"nearest"})});}}
    }
  }, [view, debuggable, debug, changed,positionCurrent,positionLine,debugBinding, literate, parserReady]);
  useEffect(() => {
    if (!view || editors.get(buffer)?.view !== view || !view.dom.isConnected) return;
    const reveal = () => {
      const position = claimRevealPosition(buffer.file.path,authoredSource(view.state)); if (!position) return;
      if (position.character === undefined) { revealLine(view, position.line - 1); return; }
      if (position.line > view.state.doc.lines || position.character > view.state.doc.line(position.line).length) { setDebugNotice("Location no longer fits this source. Query again."); return; }
      const anchor = view.state.doc.line(position.line).from + position.character;
      view.dispatch({ selection: { anchor }, effects: EditorView.scrollIntoView(anchor, { y: "center" }) }); view.focus();
    };
    reveal(); return onRevealLine(path => { if (path === buffer.file.path) reveal(); });
  }, [view, buffer]);
  const start = async (program?: string, choose = false, at?: number) => {
    if(buffer.resourceDeleted){setDebugNotice("This retained source was deleted. Its existing debugger still addresses the captured launch; save a copy before starting a new session.");return;}
    if (buffer.dirty || buffer.saving) { setDebugNotice("Save this file before debugging so the debugger runs the text shown here."); return; }
    setDebugNotice("");
    const source = buffer.text, revision = buffer.sourceRevision, target = literate ? program : undefined;
    await launchWithProfiles({ uri: documentUri(buffer.file.path), program: target, ...(literate ? { expected_document: source } : { expected_source: source }) },
      profile => {
        terminalReadings.delete(buffer);
        launchPosition.current = { at, revision };
        debugRef.current.start(target, {}, undefined, profile);
      },
      { choose, preferProgram: true, isCurrent: () => !!view?.dom.isConnected && buffer.sourceRevision === revision && buffer.text === source && !buffer.resourceDeleted && !buffer.dirty && !buffer.saving,
        onRun: id => {
          if (!literate || !target || at === undefined || !/^term-[0-9a-f]{32}$/.test(id)) return;
          const reading = { id, at, program: target, revision, text: "" };
          terminalReadings.set(buffer, reading);
          async function read() {
            const active = () => terminalReadings.get(buffer) === reading && buffer.sourceRevision === revision && !buffer.resourceDeleted;
            for (let attempt = 0; attempt < 120 && active(); attempt++) {
              try {
                const page = await request<{ session: string; bytes: number[]; output_done?: boolean }>("GET",
                  `/api/terminals/${id}/output?view=screen&offset=0&limit=16384`);
                if (!active() || page.session !== id || page.bytes.length > 16384) return;
                const text = new TextDecoder().decode(Uint8Array.from(page.bytes));
                if (reading.text !== text) {
                  reading.text = text;
                  const editor = editors.get(buffer)?.view;
                  editor?.dispatch({ effects: setDebuggerOutput.of({ events: text ? [{ category: "console", text }] : [],
                    program: target, literate, path: buffer.file.path, launchAt: at }) });
                }
                if (page.output_done === true) return;
              } catch { return; }
              await new Promise(resolve => setTimeout(resolve, 500));
            }
          }
          void read();
        } });
  };
  const sourceTestActions=useSourceTestLineActions(buffer,view);
  const coverageActions=useCoverageLineActions(buffer,view);
  const gitComparison=useGitEditorComparison(buffer,view);
  const semanticCurrent = !!view && !!semanticResult && semanticResult.ticket === semanticTicket.current && semanticResult.source === buffer.text && semanticResult.anchor === view.state.selection.main.anchor && semanticResult.head === view.state.selection.main.head;
  return <div className="document-body">
    {gitComparison.panel}
    {semanticResult && <div ref={semanticPanel} tabIndex={-1} className="simple-semantic-results" role="dialog" aria-label="Semantic locations" onKeyDown={event => { if (event.key === "Escape") { semanticTicket.current++; setSemanticResult(null); view?.focus(); } }}>
      <button type="button" aria-label="Close semantic locations" onClick={() => { semanticTicket.current++; setSemanticResult(null); view?.focus(); }}>×</button>
      <p role="status">{semanticCurrent ? semanticResult.message : "Source or cursor changed. Run the query again."}</p>
      {semanticResult.locations.map(location => { const path = semanticPath(location.uri); return <button type="button" key={JSON.stringify(location)} disabled={!path || !semanticCurrent} onClick={() => {
        if (!path || !view) return;
        const originCurrent = semanticOrigin(view, semanticResult.ticket, semanticResult.source, semanticResult.anchor, semanticResult.head);
        if (!originCurrent()) return;
        if (!workspaceLanguages && sameDocumentUri(location.uri, session.uri)) {
          const anchor = positionToUtf16(buffer.text, location.range.start);
          view.dispatch({ selection: { anchor }, effects: EditorView.scrollIntoView(anchor, { y: "center" }) }); view.focus();
        } else {
          const name = location.name;
          const providerName = name && /^[\p{ID_Start}_$][\p{ID_Continue}$]*$/u.test(name) ? name : undefined;
          openProviderLocation(path, location.range.start.line + 1, location.range.start.character, originCurrent, providerName);
        }
        setSemanticResult(null);
      }}>{location.name ? `${location.name} · ` : ""}{path ?? location.uri}:{location.range.start.line + 1}:{location.range.start.character + 1}{!path ? " (unsupported target)" : ""}</button>; })}
    </div>}
    {csv && <button type="button" className="simple-source-toggle" disabled={!csvGrid || !view} title={csvFailed ? "Table grid unavailable. Keep editing source or reload to retry the grid." : !csvGrid ? "Loading the table grid" : undefined} onClick={() => setCsvGridSelected(value => !value)}>{showingCsvGrid ? "Edit source text" : "Edit table as grid"}</button>}
    {showingCsvGrid && csvGrid && view && <CsvGrid modules={csvGrid} source={buffer.text} chooseFormulaLanguage header delimiter="," sourceCurrent={() => !buffer.readonly && authoredSource(view.state) === tableSource && buffer.text === tableSource}
      onChange={buffer.readonly ? undefined : text => { if (!buffer.readonly && buffer.text === tableSource && authoredSource(view.state) === tableSource) view.dispatch({ ...authoredChanges(view.state,[{from:0,to:authoredSource(view.state).length,insert:text}]), userEvent: "input.table" }); }} />}
    <div className="editor-row" style={{ ...(showingCsvGrid ? { display: "none" } : {}), "--simple-rail-width": `${railWidth}px` } as React.CSSProperties}><div ref={host} className="editor-host editor-cm-host" />{!buffer.transient && <Minimap buffer={buffer} />}
      <div className="simple-rail-resize" role="separator" aria-label="Resize right gutter" tabIndex={0}
        onKeyDown={event => { if (event.key === "ArrowLeft" || event.key === "ArrowRight") { event.preventDefault(); setRailWidth(width => Math.max(24, Math.min(220, width + (event.key === "ArrowLeft" ? 22 : -22)))); } }}
        onPointerDown={event => { event.currentTarget.setPointerCapture(event.pointerId); }}
        onPointerMove={event => { if (event.currentTarget.hasPointerCapture(event.pointerId)) setRailWidth(width => Math.max(24, Math.min(220, width - event.movementX))); }} />
      <DebugLaunchRail actions={[...sourceTestActions,...coverageActions,...gitComparison.actions]} view={view} plainPath={debuggable && !literate ? buffer.file.path : undefined} enabled={["idle", "finished", "failed"].includes(debug.status)} onStart={debuggable && (!literate || parserReady) ? (program, at) => start(program, false, at) : undefined}
        onProfiles={debuggable && (!literate || parserReady) ? (program, at) => start(program, true, at) : undefined} />
    </div>
    {debuggable && view && (!literate || parserReady) && <DebugStrip editorView={view} pausedLine={positionCurrent?positionLine:null}
      status={debug.status} program={debug.program} message={debug.message} offerInstall={debug.offerInstall}
      onInstall={offer => api.installTool(offer.kind, offer.language).then(() => undefined)}
      variableRoots={debug.variableRoots} valueNode={debug.valueNode} valueHistory={debug.valueHistory} valueNotice={debug.valueNotice} expandValue={debug.expandValue} collapseValue={debug.collapseValue} closeValues={debug.closeValues} refreshState={debug.refreshState} positionSourceBound={debug.positionSourceBound} sourceTestIntent={debug.sourceTestIntent} sourceTest={debug.sourceTest} releaseSourceTest={debug.releaseSourceTest} launchNotices={debug.launchNotices} breakpointNotice={debug.breakpointNotice} breakpointIntentions={debug.breakpointIntentions} onRunTo={debug.runTo} onJumpTo={debug.jumpTo} variables={debug.variables} threads={debug.threads} selectedThread={debug.selectedThread} threadStop={debug.threadStop} threadPending={debug.threadPending} threadSelecting={debug.threadSelecting} threadMessage={debug.threadMessage} frameVariablesPending={debug.frameVariablesPending} frameVariablesMessage={debug.frameVariablesMessage} stackMessage={debug.stackMessage} historicalFrames={debug.historicalFrames} observationScope={debug.observationScope} readThreads={debug.readThreads} selectThread={debug.selectThread}
      adapterSource={debug.adapterSource} readAdapterSource={debug.readAdapterSource}
      capabilities={debug.capabilities} frames={debug.frames} selectedFrame={debug.selectedFrame}
      watches={debug.watches} exitCode={debug.exitCode} buildOutput={debug.buildOutput}
      onSelectFrame={id => {debug.selectFrame(id);const captured=debug.captureFrame?.(id);if(!captured){setDebugNotice("Debugger frame changed; select it again from the current stack.");return;}const frame=captured.frame;if((frame.source_reference??0)>0){debug.readAdapterSource?.(id);return;}const path=frame.in_document?buffer.file.path:frame.source,line=frame.in_document?frame.line:frame.source_line;if(path)openCapturedLocation(path,line==null?null:line+1,frame.source_hash??null,captured.current,frame.source_reference);}}
      onStep={debug.step} onJumpHere={() => debug.jumpTo(view.state.doc.lineAt(view.state.selection.main.head).number - 1)}
      onStart={() => start(debug.program ?? undefined)} onStop={debug.stop} onAddWatch={() => { const expression = window.prompt("Expression to watch"); if (expression) debug.addWatch(expression); }}
      exceptionFilters={debug.exceptionFilters} onToggleExceptionFilter={debug.toggleExceptionFilter} onRemoveWatch={debug.removeWatch} />}
    {debugNotice && <div role="alert" className="notice">{debugNotice}</div>}
    {view && renderTools?.(view)}
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

<ui:file path="apps/web/src/simple/Document.tsx"><ui:paste select="#simple-browser-semantic-locations"/><ui:paste select="#simple-1a4a59fc95-0"/><ui:paste select="#simple-document-minimap"/></ui:file>

</ui:doc>
