<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/web/src/editor/DocumentEditor.tsx

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

When an exact Agent proposal changes a managed draft, prepare the edit against the current editor state. Detach the view from Yjs before the source transaction so Apply and Undo stay local. The same room still observes external changes for Save admission. If another owner refuses the group during commit, restore this view's state and viewport before anything is published.

Provider completion supplies a batch with exact source changes, fields and
selection history. Capture the mounted editor and saved-source lease once per
query, so every offered row uses the same proof. Rebuild the batch on the detached state through Main's
filters before installing it. Scaffold protection checks the resulting changes
after linked input has expanded to its mirrors.

<a id="output-aab82870e517"></a>
## `apps/web/src/editor/DocumentEditor.tsx`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [A Derived Diagram Keeps Your Layout](../../../../../guarantees/authoring/a-derived-diagram-keeps-your-layout/contract.md), [A Diagram Names What Proves It](../../../../../guarantees/authoring/a-diagram-names-what-proves-it/contract.md), [A Drawn Diagram Is Document Text](../../../../../guarantees/authoring/a-drawn-diagram-is-document-text/contract.md), [A Fence Becomes A Cell That Runs](../../../../../guarantees/authoring/a-fence-becomes-a-cell-that-runs/contract.md), [A Generated Picture Shows As A Picture](../../../../../guarantees/authoring/a-generated-picture-shows-as-a-picture/contract.md), [A Keystroke Redraws Only The Editor](../../../../../guarantees/authoring/a-keystroke-redraws-only-the-editor/contract.md), [A Literate File Opens Rendered, And Its Source Is One Click Away](../../../../../guarantees/authoring/a-literate-file-opens-rendered/contract.md), [A Table Is A Dataset And A Paragraph](../../../../../guarantees/authoring/a-table-is-a-dataset-and-a-paragraph/contract.md), [An Equation Renders And Shows Its Source](../../../../../guarantees/authoring/an-equation-renders-and-shows-its-source/contract.md), [An Image Dropped In Becomes A File And A Reference](../../../../../guarantees/authoring/an-image-dropped-in-becomes-a-file-and-a-reference/contract.md), [An Ingested Scaffold Opens As A Tree](../../../../../guarantees/authoring/an-ingested-scaffold-opens-as-a-tree/contract.md), [Inserting An Element Writes Hick You Could Have Typed Yourself](../../../../../guarantees/authoring/inserting-an-element-writes-hick-you-could-have-typed/contract.md), [Markdown Quotes And Task Lists Are Read As Prose](../../../../../guarantees/authoring/markdown-quotes-and-task-lists-are-read-as-prose/contract.md), [Markdown Paragraphs Unwrap By Default](../../../../../guarantees/authoring/paragraphs-unwrap-by-default/contract.md), [Pasting A URL Over Words Makes A Link](../../../../../guarantees/authoring/pasting-a-url-over-words-makes-a-link/contract.md), [Several Cursors Edit At Once](../../../../../guarantees/authoring/several-cursors-edit-at-once/contract.md), [The Gutters Never Skip A Number](../../../../../guarantees/authoring/the-gutters-never-skip-a-number/contract.md), [Word Navigation Is The Person's Choice](../../../../../guarantees/authoring/word-navigation-is-the-persons-choice/contract.md), [A Document's Text Is Never Duplicated Or Silently Rewritten By Sync](../../../../../guarantees/collaboration/document-text-never-duplicates/contract.md), [A comparison keeps current code editable](../../../../../guarantees/editing/a-comparison-keeps-current-code-editable/contract.md), [Undo Only Undoes What This Client Typed](../../../../../guarantees/editor-intelligence/undo-never-destroys-unowned-content/contract.md), [A run result is distinct from what was checked](../../../../../guarantees/execution/a-run-result-is-distinct-from-what-was-checked/contract.md), [An Element Is Drawn By Its View](../../../../../guarantees/language/an-element-is-drawn-by-its-view/contract.md).

<a id="f-aab82870e517-DocumentEditorProps"></a>
### `DocumentEditorProps`

<storycode:copy id="f-aab82870e517-DocumentEditorProps">
import { immutableScaffolds } from "./immutableScaffolds";
import { performanceInline } from "../performance/inline";
import { coverageGutter } from "../coverage/gutter";
import { replaceCapturedBlock } from "./replaceCapturedBlock";
import type { UiActor } from "../lib/uiActions";
import { useEditorEditing } from "./useEditorEditing";
import { DebugLaunchRail } from "../debug/DebugRail";
import { useCallback, useEffect, useLayoutEffect, useMemo, useRef, useState } from "react";
import { createPortal } from "react-dom";
import { Compartment, EditorSelection, EditorState, Prec, Transaction } from "@codemirror/state";
import type { CompletionContext } from "@codemirror/autocomplete";
import { comparisonField, setComparison } from "./comparison";
import { EditorView, keymap, placeholder } from "@codemirror/view";
import {
  defaultKeymap,
  history,
  historyKeymap,
  indentWithTab,
  isolateHistory,
} from "@codemirror/commands";
import { computeEdits } from "../lib/diff";
import { persistentSearchHighlights } from "../lib/workspaceSearch";
import { search, searchKeymap } from "@codemirror/search";
import { wordMotionBindings } from "./wordMotion";
import { requestMenuAction } from "../lib/menuBridge";
import { unwrapParagraphs } from "./unwrapParagraphs";
import * as Y from "yjs";
import { Awareness } from "y-protocols/awareness";
import { yCollab, ySyncFacet } from "y-codemirror.next";
import type { Extension } from "@codemirror/state";
import { EnvRegistry, parserBecameReady, setVerifiedExpects, structureOf, wysiwyg } from "./wysiwyg";
import { hickLangReady, useHickLangStatus } from "./hickLang";
import type { ManagedDocumentPreview } from "../views/folderPane";
import { PaneHeaderContent } from "../shell/PaneHeader";
import { taskCheckboxes } from "./taskList";
import { renderedMath } from "./mathRender";
import { completions } from "../lsp/completion";
import { readCompletionAcceptance } from "../lsp/completionAcceptance";
import { utf16ToPosition } from "../lsp/positions";
import { sourceCompletion } from "../simple/SourceCompletion";
import { resolveSnippetCompletion } from "../simple/SnippetContext";
import { dispatchSourceSnippetTransactions, readSourceEditorIntent, sourceSnippets } from "../simple/SourceSnippets";
import { supportedSnippetTransformPrograms } from "../simple/SnippetTransform";
import { authoredOffset, authoredSource } from "../simple/SourceText";
import { useUiCommands, useUiRevision } from "../lib/uiActionBridge";
import { WRAP_DEFAULT, WRAP_MIN, WRAP_MAX, proseWrap } from "./wrapColumn";
import { editorChrome, paneSearch } from "./chrome";
import { blameGutter } from "./blameGutter";
import { useBlame } from "./useBlame";
import { mathSpans } from "../lib/math";
import {
  diagnosticRanges,
  offsetToPosition,
  positionToOffset,
  setLspDiagnostics,
} from "../lsp/cmLsp";
import type { CompletionItem, LspClient, LspDiagnostic } from "../lsp/client";
import { multipleCursors } from "./multiCursor";
import { hickoryFolding, ingestedFolds, sessionWorkFolds } from "./folding";
import {
  forgetEditor,
  forgetFocusedEditor,
  markActiveEditor,
  markFocusedEditor,
} from "./activeEditor";
import type { EnvSlot } from "./wysiwyg";
import {
  containerNamesOf,
  execBlocksOf,
  expectRangeOf,
  proseFences,
  verbatimRanges,
} from "./hickDoc";
import { lineHighlightField } from "./lineHighlight";
import { mdLinks } from "./mdLinks";
import { matchExecBlock } from "../lib/blockMatch";
import { base64Of, mdPaste } from "./mdPaste";
import { RightRail } from "./RightRail";
import { CardRail, type CardState } from "./CardRail";
import { cardsOf, type DocCard } from "./cards";
import {
  RenderedRegistry,
  isRendered,
  renderBlock,
  renderableBlocks,
  renderedBlocks,
  setRenderedBlocks,
  showBlockSource,
  type RenderedSlot,
} from "./rendered";
import { popoverTop } from "../lib/cardRail";
import { actionsFor, hasReplay } from "../lib/railActions";
import { useEditorActions } from "./uiEditorActions";
import type { RailAction } from "../lib/railActions";
import type { TableLayout } from "../components/TablePanel";
import { elementViews, slotKindOf, type SlotContext } from "../elements";
import { FenceTable, tableElementFor } from "../components/FenceTable";
import { isTabularFence } from "../lib/csv";
import { FenceConvert } from "../components/FenceConvert";
import { EnvCard } from "../components/EnvCard";
import { api } from "../api/client";
import type { Realtime } from "../api/realtime";
import type { DiagramBlock, ExecBlock, ExecutorInfo } from "../api/types";

export interface DocumentEditorProps {
  capturedProject?: SlotContext["capturedProject"];
  editorExtensions?: Extension[];
  preserveBytes?: boolean;
  /** Saving an existing tab is not a fresh open: preserve its acknowledged text. */
  reflowOnOpen?: boolean;
  comparisonBase?: string | null;
  readOnly?: boolean;
  docId: string;
  editorIdentity?: string;
  /** Initial .md source, used to seed the Y.Doc when it is empty. */
  initialSource: string;
  startup?: ManagedDocumentPreview;
  startupAuthorized?: boolean;
  onStartupRoomReady?: () => void;
  realtime: Realtime;
  onChange?: (source: string) => void;
  /** Byte span to select (provenance click-through from the Output view). */
  selectSpan?: [number, number] | null;
  /** Rendered exec blocks (statuses/transcripts) to attach to cells. */
  execBlocks: ExecBlock[];
  /** Rendered diagram blocks: bodies with pastes resolved server-side, so a
   * DERIVED diagram draws. Optional — without them the raw source draws. */
  diagramBlocks?: DiagramBlock[];
  runningCells: Set<string>;
  onRunCell: (execId: string) => void;
  /** Debug one of the files this document generates, named by its path. */
  onDebugFile?: (path: string, at?: number, choose?: boolean) => void;
  /** Fired with the live EditorView on mount and null on teardown (the Split
   * view uses it to measure ribbon anchors against real geometry). */
  onViewReady?: (view: EditorView | null) => void;
  /** LSP bindings (hover / definition / references) for this buffer. */
  lspExtensions?: Extension[];
  /** Diagnostics for this document, in document coordinates. */
  lspDiagnostics?: LspDiagnostic[];
  /** The language session to ask completions of, alongside the project's
   * own text. Read once, when the editor is built: a session's client is
   * stable for the session's life. */
  lspCompletion?: { client: LspClient; uri: string } | null;
  captureProviderCompletion?: (view: EditorView, uri: string) => DocumentCompletionLease | null;
  /** Dim hint shown while the buffer is empty (the untitled document). */
  placeholderText?: string;
  /** Where PROSE wraps, in columns. Owned by the tab (so it is per-document
   * and survives a restart); code never wraps whatever this says. */
  wrapColumn?: number;
  /** Report a measure the reader dragged on the ruler. */
  onWrapColumn?: (column: number) => void;
  /** This document's path, for the blame column. Absent means no column —
   * an untitled buffer has no history to show. */
  path?: string | null;
  /** How big each table in this document was left, keyed by `tableKey`.
   * Presentation, so it lives in the workspace's state and never in the
   * document — see lib/uiState.ts. */
  tableLayouts?: Record<string, TableLayout>;
  /** Report a table the reader resized. */
  onTableLayout?: (key: string, size: TableLayout) => void;
}

</storycode:copy>
<a id="f-aab82870e517-POPOVER_HEIGHT"></a>
### `POPOVER_HEIGHT`

The popover's height cap, matching `max-height` on `.cm-card-popover`.
Used only as the first guess for where the popover opens; a layout effect
corrects it against the real height before paint. Content past this scrolls
inside the popover rather than growing it, so a streaming transcript can
never push its own Run button off the screen.

<storycode:copy id="f-aab82870e517-POPOVER_HEIGHT">
/**
 * The popover's height cap, matching `max-height` on `.cm-card-popover`.
 *
 * Used only as the first guess for where the popover opens; a layout effect
 * corrects it against the real height before paint. Content past this scrolls
 * inside the popover rather than growing it, so a streaming transcript can
 * never push its own Run button off the screen.
 */
const POPOVER_HEIGHT = 320;

</storycode:copy>
<a id="f-aab82870e517-export_statement"></a>
### `export_statement`

Overlap length of two [from, to) spans.
Kept as exports here for the callers that always found them here; the
code lives beside the other block matching in lib/blockMatch.ts.

Enter continues a bullet or increments an ordered marker at the caret, preserving
indentation and resetting a new task checkbox. An empty item exits the list.
Check every cursor before dispatching one transaction, so mixed code/prose
selections fall back together. Parsed cell payloads and prose fences are excluded:
a shell argument beginning with `- ` must never become a Markdown edit.

<storycode:copy id="f-aab82870e517-export_statement">
/** Overlap length of two [from, to) spans. */
// Kept as exports here for the callers that always found them here; the
// code lives beside the other block matching in lib/blockMatch.ts.
export { assertionStates, matchExecBlock } from "../lib/blockMatch";

/** Continue prose lists only; code and cell payloads keep ordinary Enter. */
export function continueList(view: EditorView): boolean {
  const { state } = view;
  if (state.readOnly) return false;
  const structure = structureOf(state);
  const excluded = [
    ...verbatimRanges(structure.blocks),
    ...proseFences(structure, state.doc.toString()).map(f => [f.from, f.to]),
  ];
  const items = state.selection.ranges.map(range => {
    const line = state.doc.lineAt(range.from);
    const match = /^(\s*)([-+*]|(\d{1,9})([.)]))([ \t]+)(?:\[[ xX]\]([ \t]+))?/.exec(line.text);
    if (!range.empty || !match || range.from < line.from + match[0].length ||
        excluded.some(([from, to]) => range.from >= from && range.from <= to)) return null;
    return { line, match };
  });
  if (items.some(item => !item)) return false;
  view.dispatch(state.changeByRange(range => {
    const { line, match } = items[state.selection.ranges.indexOf(range)]!;
    if (!line.text.slice(match[0].length).trim()) {
      return { changes: { from: line.from, to: line.to, insert: match[1] },
        range: EditorSelection.cursor(line.from + match[1].length) };
    }
    const marker = match[3] ? `${Number(match[3]) + 1}${match[4]}` : match[2];
    const insert = `\n${match[1]}${marker}${match[5]}${match[6] ? `[ ]${match[6]}` : ""}`;
    return { changes: { from: range.from, insert },
      range: EditorSelection.cursor(range.from + insert.length) };
  }), { scrollIntoView: true, userEvent: "input" });
  return true;
}

</storycode:copy>
<a id="f-aab82870e517-DocumentEditor"></a>
### `DocumentEditor`

Initial paragraph reflow is separate from ongoing editing. A tab that has just
been saved can disable reflow on opening its named editor, preserving the bytes
that were acknowledged on disk instead of immediately making them dirty again.

The Document view: ONE CodeMirror instance over the raw .md source with
Typora-style decorations (see editor/wysiwyg.ts) and collaborative editing
via Yjs — the user always edits real source; styling never replaces text.
Nothing this renders adds a row to the document. Annotations are inline
widgets on the line they describe, and the UI too tall for a line — a
cell's run strip, a rendered diagram, the fence converter — opens from the
action rail beside the editor, so the left gutter's numbers and the right
rail's never skip. See editor/CardRail.tsx.

<storycode:copy id="f-aab82870e517-DocumentEditor">
/**
 * The Document view: ONE CodeMirror instance over the raw .md source with
 * Typora-style decorations (see editor/wysiwyg.ts) and collaborative editing
 * via Yjs — the user always edits real source; styling never replaces text.
 *
 * Nothing this renders adds a row to the document. Annotations are inline
 * widgets on the line they describe, and the UI too tall for a line — a
 * cell's run strip, a rendered diagram, the fence converter — opens from the
 * action rail beside the editor, so the left gutter's numbers and the right
 * rail's never skip. See editor/CardRail.tsx.
 */
type DocumentDraftStage = {
  current: () => boolean;
  commit: () => void;
  rollback: () => void;
};
export type DocumentCompletionLease = {
  current: () => boolean;
  commit: (transactions: readonly Transaction[], privateDraft: boolean) => void;
};
type DocumentDraftBinding = {
  roomSource: () => string;
  setRetained: (retained: boolean) => void;
  prepareRoom: (realtime: Realtime) => {source: () => string; install: (attach: boolean) => void; discard: () => void};
  suspendRoom: () => {attached: boolean; resume: (expected: string) => boolean};
  prepare: (before: string, after: string, options?: {published: boolean; restoring?: boolean}) => DocumentDraftStage | null;
  prepareCompletion: (transactions: readonly Transaction[], privateDraft: boolean) => DocumentDraftStage | null;
};
const documentDraftBindings = new WeakMap<EditorView, DocumentDraftBinding>();
export function documentDraftBinding(view: EditorView) {
  return documentDraftBindings.get(view);
}

export function DocumentEditor({
  capturedProject,
  editorExtensions = [],
  preserveBytes = false,
  comparisonBase = null,
  readOnly: explicitReadOnly = false,
  docId,
  editorIdentity = docId,
  reflowOnOpen = docId !== "untitled",
  initialSource,
  startup,
  startupAuthorized = false,
  onStartupRoomReady,
  realtime,
  onChange,
  selectSpan,
  execBlocks,
  diagramBlocks,
  runningCells,
  onRunCell,
  onViewReady,
  lspExtensions,
  lspDiagnostics,
  lspCompletion,
  captureProviderCompletion,
  onDebugFile,
  placeholderText,
  wrapColumn = WRAP_DEFAULT,
  onWrapColumn,
  path = null,
  tableLayouts,
  onTableLayout,
}: DocumentEditorProps) {
  const hostRef = useRef<HTMLDivElement>(null);
  const viewRef = useRef<EditorView | null>(null);
  const access = useMemo(() => new Compartment(), []);
  const parserProtection = useMemo(() => new Compartment(), []);
  const collaboration = useMemo(() => new Compartment(), []);
  const retainedAccess = useMemo(() => new Compartment(), []);
  const languageServices = useMemo(() => new Compartment(), []);
  const completionServices = useMemo(() => new Compartment(), []);
  const parserStatus = useHickLangStatus();
  const parserInstalled = useRef<EditorView | null>(null);
  const startStartupRoom = useRef<(() => void) | null>(null);
  const [startupRoomReady, setStartupRoomReady] = useState(false);
  const [startupRoomError, setStartupRoomError] = useState<string | null>(null);
  const restoringDraft = useRef(0);
  const readOnly = explicitReadOnly || parserStatus !== "ready" || !!startup && !startupRoomReady;
  const sourceAccess = useMemo(() => [
    EditorState.readOnly.of(readOnly), EditorView.editable.of(!readOnly),
    EditorState.transactionFilter.of(tr => {
      if (!tr.docChanged) return tr;
      if (restoringDraft.current && tr.isUserEvent("input.unwrap")) return [];
      if ((startup && !startupRoomReady) || !hickLangReady()) {
        // The room may deliver another update just after its compartment
        // attaches, while the view still displays read-only startup state.
        // Admit only text already installed in the room by that sync.
        const room = tr.startState.facet(ySyncFacet);
        return tr.annotation(Transaction.userEvent) === undefined
          && room?.ytext.toString() === tr.newDoc.toString() ? tr : [];
      }
      return explicitReadOnly && tr.annotation(Transaction.userEvent) ? [] : tr;
    }),
  ], [explicitReadOnly, readOnly, startup, startupRoomReady]);
  const completionClient = lspCompletion?.client, completionUri = lspCompletion?.uri;
  const completionSession = useRef(lspCompletion);
  completionSession.current = lspCompletion;
  const [completionNotice, setCompletionNotice] = useState("");
  const completionExtension = useMemo(() => {
    const captures = new WeakMap<CompletionContext, {view: EditorView | null; lease: DocumentCompletionLease | null}>();
    return completions({
      lsp: completionClient && completionUri ? {
        client: completionClient, uri: completionUri,
        transformPrograms: supportedSnippetTransformPrograms,
        positionAt: (offset, state) => offsetToPosition(state.doc, offset),
      } : undefined,
      lspItem: completionClient && completionUri ? (item, context) => {
        let captured = captures.get(context);
        if (!captured) {
          const view = viewRef.current;
          captured = {view, lease: view ? captureProviderCompletion?.(view, completionUri) ?? null : null};
          captures.set(context, captured);
        }
        const {view, lease} = captured;
        const current = () => !!view?.dom.isConnected && viewRef.current === view &&
          completionSession.current?.client === completionClient && completionSession.current?.uri === completionUri &&
          lease?.current() === true;
        const writable = () => !!view && current() && !view.state.readOnly && view.state.facet(EditorView.editable) && !view.composing;
        const request = async (original: CompletionItem) =>
          await completionClient.request("completionItem/resolve", {...original, textDocument: {uri: completionUri}}) as CompletionItem;
        return sourceCompletion(item, context, {
          uri: completionUri, current, writable, notice: setCompletionNotice,
          resolve: original => {
            if (!view || !writable() || !original.data || typeof original.data !== "object" ||
                !("hickory_context" in original.data) || !("hickory_completion" in original.data) ||
                typeof original.data.hickory_completion !== "string" || !original.data.hickory_completion) {
              throw Error("Request completion again from the current managed source.");
            }
            const state = view.state, selection = state.selection;
            const intent = readSourceEditorIntent(state), job = readCompletionAcceptance(view)?.job;
            return resolveSnippetCompletion(original, {
              source: authoredSource(state), from: authoredOffset(state, selection.main.from),
              to: authoredOffset(state, selection.main.to), request,
              current: () => writable() && view.state.doc === state.doc && view.state.selection.eq(selection) &&
                readSourceEditorIntent(view.state) === intent && readCompletionAcceptance(view)?.job === job &&
                readCompletionAcceptance(view)?.phase === "resolving",
            });
          },
          group: {
            query: (offset, source) => completionClient.completion(completionUri, utf16ToPosition(source, offset),
              {triggerKind: 1}, supportedSnippetTransformPrograms),
            resolve: request,
          },
          commit: (editor, transactions, privateDraft) => {
            if (!lease || editor !== view || !writable()) throw Error("The managed source changed. Request completions again.");
            lease.commit(transactions, privateDraft);
          },
        });
      } : undefined,
      project: (prefix, around) => api.complete(prefix, around).then(answer => answer.suggestions),
    });
  }, [completionClient, completionUri, captureProviderCompletion]);
  // The live view AS STATE, for the right rail — a ref never re-renders.
  const [railView, setRailView] = useState<EditorView | null>(null);
  useEditorActions(railView, `document:${docId}`);
  useEditorEditing(railView, { source: `document:${docId}`, label: "document editor", current: () => !readOnly, identity: () => [docId, readOnly] });
  // Read through a ref: the editor is built once per document, and a callback
  // baked in at construction would go stale the moment the debugger's state
  // changed.
  // The editor is built once per document, but a buffer that was untitled
  // when it was built has a path as soon as it is first saved — and where an
  // image lands, and what a relative link resolves to, both depend on it. So
  // it is read through a ref at the moment it is needed.
  const pathRef = useRef(path);
  pathRef.current = path;
  // Wrapping is presentation state. Keep the action available on the existing
  // command surface so the editor has no persistent measuring strip.
  const wrapOwner = `document:${docId}:wrap`;
  const wrapRevision = useUiRevision(() => [docId, path, wrapColumn, !!railView?.dom.isConnected]);
  useUiCommands(onWrapColumn && railView ? [{ key: wrapOwner, command: {
    label: "Set prose wrapping measure", parent: wrapOwner, subject: wrapOwner,
    parameters: { column: { type: "number", minimum: WRAP_MIN, maximum: WRAP_MAX, integer: true } },
    prerequisites: [],
    read: () => ({ revision: wrapRevision(), enabled: railView.dom.isConnected }),
    execute: async args => {
      onWrapColumn(args.column as number);
      return { status: "completed", value: { measure_requested: args.column, source_changed: false } };
    },
  } }] : []);
  // What went wrong with the last drop or paste, shown until the next one.
  // A write that fails silently leaves a note referencing a file that was
  // never created.
  const [assetError, setAssetError] = useState<string | null>(null);
  const envRegistry = useMemo(() => new EnvRegistry(), []);
  const [envSlots, setEnvSlots] = useState<EnvSlot[]>([]);
  const [executorInfo, setExecutorInfo] = useState<ExecutorInfo | null>(null);
  // Who last touched each line, when the column is on. Lazy: nothing is
  // asked of git until somebody turns it on.
  useBlame(railView, path);

  // The action rail's contents, recomputed whenever an edit changes them.
  // Held as state rather than derived per render because the editor
  // deliberately does not re-render on every keystroke.
  const [cards, setCards] = useState<DocCard[]>([]);
  // Which card's popover is open, and the rail-relative top of the icon that
  // opened it.
  const [open, setOpen] = useState<{ card: DocCard; iconTop: number } | null>(null);
  const openRef = useRef(open); openRef.current = open;
  // Cells whose transcript is revealed, by block start. This used to be local
  // state inside CellPanel, back when Replay was a button in the panel; the
  // rail owns the verb now, so the rail's owner owns the state.
  const [replaying, setReplaying] = useState<readonly number[]>([]);
  const popoverRef = useRef<HTMLDivElement | null>(null);
  // Blocks currently showing their result instead of their source.
  const renderedRegistry = useMemo(() => new RenderedRegistry(), []);
  const [renderedSlots, setRenderedSlots] = useState<RenderedSlot[]>([]);
  useEffect(
    () => envRegistry.subscribe(() => setEnvSlots(envRegistry.list())),
    [envRegistry],
  );
  useEffect(
    () => renderedRegistry.subscribe(() => setRenderedSlots(renderedRegistry.list())),
    [renderedRegistry],
  );

  // The environment chip is filled by a React portal AFTER CodeMirror
  // measured its (initially empty) element, and its text can change width.
  // CM caches per-line geometry, so ask it to re-measure whenever one
  // resizes — the classic "ArrowUp jumps half a screen" bug otherwise.
  //
  // Inline cell output also changes height as execution streams; observe its
  // slot alongside rendered blocks so cursor geometry follows the output.
  useEffect(() => {
    if (typeof ResizeObserver === "undefined") return;
    let last = new Map<Element, number>();
    let scheduled = false;
    const observer = new ResizeObserver((entries) => {
      // Only re-measure when a slot's height actually changed — Chrome fires
      // an initial callback per observe(), and an unconditional
      // requestMeasure can feed back into layout and loop.
      let changed = false;
      for (const e of entries) {
        const h = Math.round(e.contentRect.height);
        if (last.get(e.target) !== h) {
          last.set(e.target, h);
          changed = true;
        }
      }
      if (!changed || scheduled) return;
      scheduled = true;
      requestAnimationFrame(() => {
        scheduled = false;
        viewRef.current?.requestMeasure();
      });
    });
    for (const slot of envSlots) observer.observe(slot.el);
    // A rendered block is a replacement for real lines, so its height IS the
    // document's height there. A diagram settles after the engine draws and a
    // transcript grows while it streams; both must re-measure or every
    // vertical cursor motion below works from stale geometry.
    for (const slot of renderedSlots) observer.observe(slot.el);
    return () => {
      observer.disconnect();
      last = new Map();
    };
  }, [envSlots, renderedSlots]);
  useEffect(() => {
    api.executor().then(setExecutorInfo, () => setExecutorInfo(null));
  }, []);

  useEffect(() => {
    const host = hostRef.current;
    if (!host) return;

    const ydoc = new Y.Doc();
    const ytext = ydoc.getText("source");
    const awareness = new Awareness(ydoc);
    let activeRoom = {doc: ydoc, text: ytext, awareness};
    // The presence color travels to peers as a literal value, so resolve the
    // theme's accent once at bind time rather than shipping a var() string.
    const presenceColor =
      getComputedStyle(document.documentElement).getPropertyValue("--accent").trim() || "#5eb0ef";
    awareness.setLocalStateField("user", {
      name: localStorage.getItem("hickory.name") ?? "anonymous",
      color: presenceColor,
    });
    if (!startup) realtime.bindDoc(ydoc, awareness);

    // Seeding is ONLY for realtimes with no server behind them (mock mode).
    // With a server, the room's Y.Doc is built from `docs.source`; seeding
    // here mints an independent copy of the same text under a different
    // client id, and the CRDT merges the two by concatenation — the document
    // doubles on every single connect. (Measured: a 15KB doc reached 49MB.)
    let cancelled = false;
    if (!startup && !realtime.serverAuthoritative) {
      void realtime.whenSynced().then(() => {
        if (!cancelled && ytext.length === 0 && initialSource.length > 0) {
          ytext.insert(0, initialSource);
        }
      });
    }

    // Render-by-default happens ONCE per open document: a file is opened to
    // be read, so its cells and diagrams show their results. It is not
    // re-applied afterwards, because a block you just typed turning into a
    // picture under the caret is the opposite of helpful.
    let seeded = false;
    const parserAtOpen = hickLangReady();
    const seedRendered = (target: EditorView) => {
      if (startup || !parserAtOpen || seeded || target.state.doc.length === 0) return;
      seeded = true;
      // A session opened as a DOCUMENT keeps its framed text (see
      // editor/session.test.tsx); the cards are the lens's way of drawing
      // it (views/SessionLens.tsx). Everything else renders by default.
      const blocks = renderableBlocks(structureOf(target.state), target.state.doc.toString()).filter(
        (b) => !slotKindOf(b)?.startsWith("session-"),
      );
      if (blocks.length === 0) return;
      // Out of the update that triggered it: dispatching from inside an
      // updateListener re-enters CodeMirror mid-update.
      queueMicrotask(() => {
        if (!target.dom.isConnected) return;
        target.dispatch({
          effects: setRenderedBlocks.of(blocks.map((b) => b.from)),
        });
      });
    };

    const view = new EditorView({
      dispatchTransactions: dispatchSourceSnippetTransactions,
      parent: host,
      state: EditorState.create({
        doc: startup?.source.content ?? ytext.toString(),
        extensions: [
          // Gutters, caret, selection, the find panel — one definition, loaded
          // by every editor in the app. See editor/chrome.ts.
          editorChrome("document"),
          // Before the debug layer, which is where lineNumbers() comes from:
          // gutters lay out in declaration order, and this one belongs
          // OUTSIDE the numbers. Off by default — see editor/blameGutter.ts.
          blameGutter(),
          coverageGutter(() => pathRef.current ?? undefined),
          performanceInline(() => pathRef.current ?? undefined),
          history(),
          ...(preserveBytes ? [] : [unwrapParagraphs({ onOpen: reflowOnOpen })]),
          // Undo must never reach content this client did not type.
          //
          // The room's first sync arrives as an ordinary document change, and
          // CodeMirror's history treated "the document appeared" as an
          // undoable local edit: a few Ctrl+Z presses erased the WHOLE
          // document, and the deletion synced to every other client. The same
          // applies to every edit a collaborator makes.
          //
          // Everything the CRDT applies — remote updates, the initial sync,
          // the mock-mode seed — is dispatched without a user event, which is
          // exactly what distinguishes it from typing. Keep those out of the
          // history and undo means "undo what I did".
          EditorState.transactionExtender.of((tr) =>
            tr.docChanged && tr.annotation(Transaction.userEvent) === undefined
              ? { annotations: Transaction.addToHistory.of(false) }
              : null,
          ),
          // In-buffer find (Mod-F). First in the keymap so nothing shadows
          // it; the shifted chord stays free for the shell's project search.
          // The panel sits on top — a find that covers the line it found is
          // a find that answers a question by hiding the answer.
          // Project matches stay visible after the search overlay closes.
          persistentSearchHighlights,
          paneSearch("document"),
          search({ top: true }),
          // Word motion first: bindings for one key run in registration
          // order and the first to return true wins, so these must arrive
          // before `defaultKeymap`'s own Ctrl+arrow. See editor/wordMotion.ts.
          // Tab indents the current line; Shift-Tab removes an indentation level.
          keymap.of([
            ...wordMotionBindings,
            ...searchKeymap,
            { key: "Enter", run: continueList },
            ...defaultKeymap,
            ...historyKeymap,
            indentWithTab,
          ]),
          // The hovered-ribbon line tint, shared with the right rail.
          lineHighlightField,
          wysiwyg(envRegistry),
          // Task boxes come from the DOCUMENT's structure parse, not a plain
          // markdown scan: a `- [ ]` inside an exec cell's payload is a
          // command's argument, and turning it into a checkbox would offer to
          // edit a line the reader is not looking at.
          taskCheckboxes((state) => structureOf(state).tasks),
          // Two sources, one list, each marked. The language server knows
          // what is in scope; the project index knows what this codebase
          // calls things. Neither subsumes the other — see lsp/completion.ts.
          completionServices.of(completionExtension),
          sourceSnippets({writable: () => !!viewRef.current && !viewRef.current.state.facet(ySyncFacet), notice: setCompletionNotice}),
          ...multipleCursors(),
          // Inline and display maths written in PROSE. The verbatim ranges of
          // the document are excluded, so `$PATH` in a shell cell stays a
          // shell variable and `$` in a generated file stays a byte of that
          // file. A `<hick:math>` block is not handled here at all: it is a
          // rendered block with a rail icon (see editor/rendered.ts).
          renderedMath((state) => {
            const structure = structureOf(state);
            return mathSpans(
              state.doc.toString(),
              verbatimRanges(structure.blocks),
            );
          }),
          renderedBlocks(renderedRegistry),
          hickoryFolding(),
          // A session opens with the agent's work folded — the dock's "show
          // work", in the editor. See editor/folding.ts.
          sessionWorkFolds(),
          // A scaffold opens as a tree: one line per ingested file.
          ingestedFolds(),
          // CodeMirror runs transaction filters from last to first. Check the
          // fully expanded linked snippet edit after its filters add mirrors.
          parserProtection.of(parserAtOpen ? Prec.high(immutableScaffolds()) : []),
          collaboration.of(startup ? [] : yCollab(ytext, awareness)),
          ...(placeholderText ? [placeholder(placeholderText)] : []),
          languageServices.of(lspExtensions ?? []),
          ...editorExtensions,
          comparisonField,
          retainedAccess.of([]),
          access.of(sourceAccess),
          // Which buffer the Insert menu writes into. Recorded on focus
          // rather than read at insert time: opening the panel takes the
          // focus away from every editor on the page.
          EditorView.focusChangeEffect.of((_state, focusing) => {
            const live = viewRef.current;
            if (focusing && live) {
              markActiveEditor(live);
              markFocusedEditor(live);
            }
            return null;
          }),
          // Prose wraps at the ruler's measure; code keeps its lines and takes
          // the whole pane. `proseWrap` turns lineWrapping on for both and
          // then lets code opt out, line by line — see editor/wrapColumn.ts.
          proseWrap((state) => {
            const structure = structureOf(state);
            const text = state.doc.toString();
            return [
              ...verbatimRanges(structure.blocks),
              ...proseFences(structure, text).map(
                (fence) => [fence.from, fence.to] as [number, number],
              ),
            ];
          }),
          // Links and images in prose: styled, Mod-clickable, and — for an
          // image — drawn under the line that references it. The verbatim
          // ranges are excluded for the same reason maths excludes them: a
          // `[…](…)` inside a shell cell is shell.
          mdLinks({
            docPath: () => pathRef.current,
            images: true,
            includes: state => structureOf(state).tags
              .filter(tag => tag.name === "include" && !tag.closing && !!tag.attrs.file)
              .map(tag => ({ from: tag.from, to: tag.to, target: tag.attrs.file })),
            skip: (state) => verbatimRanges(structureOf(state).blocks),
            // Click a generated picture and land on the block that writes
            // it — the way back from the output to its source.
            onOpenImageSource: (rootRelative) => {
              const view = viewRef.current;
              if (!view) return false;
              const docDir = (pathRef.current ?? "").split("/").slice(0, -1).join("/");
              const block = structureOf(view.state).blocks.find((b) => {
                if (b.name !== "file" || !b.attrs.path) return false;
                const abs = docDir ? `${docDir}/${b.attrs.path}` : b.attrs.path;
                return abs === rootRelative || b.attrs.path === rootRelative;
              });
              if (!block) return false;
              view.dispatch({
                selection: { anchor: block.from },
                scrollIntoView: true,
              });
              view.focus();
              return true;
            },
          }),
          // A URL pasted over a selection becomes a link; an image pasted or
          // dropped is written into the folder and referenced. Both write
          // ordinary markdown — see editor/mdPaste.ts.
          mdPaste({
            docPath: () => pathRef.current,
            upload: async (file, docPath) => {
              const saved = await api.saveAsset(file.name, await base64Of(file), docPath);
              return { relative: saved.relative };
            },
            isProse: (state, from) =>
              !verbatimRanges(structureOf(state).blocks).some(
                ([vFrom, vTo]) => from >= vFrom && from < vTo,
              ),
            onError: setAssetError,
          }),
          EditorView.updateListener.of((u) => {
            if (!u.docChanged && !u.transactions.some(tr => tr.effects.some(effect => effect.is(parserBecameReady)))) return;
            if (u.docChanged) onChange?.(u.state.doc.toString());
            // Actions need current ranges and metadata even when no icon
            // moved. CardRail separately compares positions for layout.
            const next = cardsOf(structureOf(u.state), { text: u.state.doc.toString() });
            setCards(next);
            setOpen(current => {
              if (!current) return null;
              let replaced = false;
              u.changes.iterChangedRanges((from, to) => { if (from <= current.card.from && to >= current.card.to) replaced = true; });
              const from = u.changes.mapPos(current.card.from, -1), to = u.changes.mapPos(current.card.to, 1);
              const card = next.find(card => card.kind === current.card.kind && card.from === from && card.to === to);
              return replaced || !card ? null : { ...current, card };
            });
            // The room's first sync arrives as an ordinary document change,
            // so this — not construction — is usually where a collaborative
            // document first has anything to render.
            seedRendered(u.view);
          }),
        ],
      }),
    });
    viewRef.current = view;
    function stageDraft(state: EditorState, detach: Transaction | null, edits: readonly Transaction[]): DocumentDraftStage {
      const current = () => viewRef.current === view && view.dom.isConnected && view.state === state;
      let undo: (() => void) | undefined;
      return {
        current,
        commit() {
          if (!current()) throw Error("The document editor changed before the draft was applied.");
          const top = view.scrollDOM.scrollTop, left = view.scrollDOM.scrollLeft, opened = openRef.current;
          undo = () => {
            // Keep the same editor, history and selection. Its open-time reflow
            // must not edit the restored view or reach a reattached room.
            restoringDraft.current++;
            try { view.setState(state); }
            finally { queueMicrotask(() => { restoringDraft.current--; }); }
            view.scrollDOM.scrollTop = top; view.scrollDOM.scrollLeft = left;
            setCards(cardsOf(structureOf(state), {text: state.doc.toString()}));
            setOpen(opened);
          };
          // Destroy the Yjs observer before presenting any source transaction.
          if (detach) view.dispatch(detach);
          view.update(edits);
          if (view.state !== edits.at(-1)!.state) throw Error("The document editor refused the prepared draft.");
        },
        rollback() { undo?.(); undo = undefined; },
      };
    }
    documentDraftBindings.set(view, {
      roomSource: () => activeRoom.text.toString(),
      setRetained(retained) {
        view.dispatch({effects: retainedAccess.reconfigure(retained
          ? [EditorState.readOnly.of(true), EditorView.editable.of(false)] : [])});
      },
      prepareRoom(realtime) {
        const doc = new Y.Doc(), text = doc.getText("source"), presence = new Awareness(doc);
        presence.setLocalStateField("user", activeRoom.awareness.getLocalState()?.user ?? {
          name: localStorage.getItem("hickory.name") ?? "anonymous", color: presenceColor,
        });
        const fresh = {doc, text, awareness: presence};
        // A restored room is server-owned. The fresh client starts empty.
        realtime.bindDoc(doc, presence);
        let installed = false;
        return {
          source: () => text.toString(),
          install(attach) {
            if (viewRef.current !== view || !view.dom.isConnected || installed) throw Error("The retained editor changed.");
            if (attach && view.state.doc.toString() !== text.toString()) throw Error("The restored room differs from the editor.");
            if (attach) view.dispatch({effects: collaboration.reconfigure(yCollab(text, presence))});
            const previous = activeRoom;
            activeRoom = fresh;
            installed = true;
            previous.awareness.destroy();
            previous.doc.destroy();
          },
          discard() {
            if (installed) return;
            presence.destroy();
            doc.destroy();
          },
        };
      },
      suspendRoom() {
        const extension = view.state.facet(ySyncFacet) ? collaboration.get(view.state) : undefined;
        if (extension) view.dispatch({effects: collaboration.reconfigure([])});
        return {
          attached: !!extension,
          resume(expected) {
            if (!extension) return true;
            if (viewRef.current !== view || !view.dom.isConnected || activeRoom.text.toString() !== expected ||
                view.state.doc.toString() !== expected) return false;
            // Reuse the original sync extension and its undo manager.
            view.dispatch({effects: collaboration.reconfigure(extension)});
            return true;
          },
        };
      },
      prepare(before, after, options) {
        const state = view.state;
        if ((state.readOnly && !options?.restoring) || state.doc.toString() !== before || !view.dom.isConnected) return null;
        const detach = state.facet(ySyncFacet) ? state.update({effects: collaboration.reconfigure([])}) : null;
        const changes = computeEdits(before, after).map(edit => ({from: edit.start, to: edit.end, insert: edit.text}));
        const edit = (detach?.state ?? state).update({
          changes, userEvent: options?.published ? undefined : "input.apply",
          annotations: isolateHistory.of("full"),
        });
        if (edit.newDoc.toString() !== after) return null;
        return stageDraft(state, detach, [edit]);
      },
      prepareCompletion(transactions, privateDraft) {
        const state = view.state;
        if (!transactions.length || !view.dom.isConnected || view.composing || state.readOnly ||
            !state.facet(EditorView.editable) || parserInstalled.current !== view ||
            (!privateDraft && transactions.some(tr => tr.docChanged || tr.effects.some(effect => effect.value != null)))) return null;
        const detach = privateDraft && state.facet(ySyncFacet) ? state.update({effects: collaboration.reconfigure([])}) : null;
        // Simple prepares an exact batch with filters disabled. Re-run the
        // same transactions through Main's filters without the room facet.
        let original = state, filtered = detach?.state ?? state;
        const edits: Transaction[] = [];
        for (const proposed of transactions) {
          if (proposed.startState !== original) return null;
          const checked = filtered.update(proposed);
          if (authoredSource(checked.state) !== authoredSource(proposed.state) ||
              !checked.newSelection.eq(proposed.newSelection, true) ||
              JSON.stringify(checked.changes.toJSON()) !== JSON.stringify(proposed.changes.toJSON()) ||
              checked.effects.length !== proposed.effects.length ||
              checked.effects.some((effect, index) => effect !== proposed.effects[index]) ||
              (privateDraft && !!checked.state.facet(ySyncFacet))) return null;
          original = proposed.state;
          filtered = checked.state;
          edits.push(checked);
        }
        return stageDraft(state, detach, edits);
      },
    });
    parserInstalled.current = parserAtOpen ? view : null;
    if (startup) {
      let joining = false;
      startStartupRoom.current = () => {
        if (joining || !startup.current() || !realtime.serverAuthoritative) return;
        joining = true;
        realtime.bindDoc(ydoc, awareness);
        void realtime.whenSynced().then(async () => {
          if (cancelled || viewRef.current !== view || !startup.current()) return;
          await startup.confirmRoom();
          if (cancelled || viewRef.current !== view || !startup.current()) return;
          const confirmed = ytext.toString();
          if (confirmed !== startup.source.content) {
            setStartupRoomError("The document room differs from this read-only source preview. Reopen the document to read its current source.");
            return;
          }
          if (confirmed !== view.state.doc.toString()) {
            // Treating CR as a visible character makes Yjs positions line up
            // but lets ordinary Delete or Backspace split a CRLF separator.
            setStartupRoomError("This preview is read-only because the editor cannot preserve the document's line endings during collaboration.");
            return;
          }
          // The room already contains these bytes. Attaching a plugin without
          // a text transaction preserves history and sends no source update.
          view.dispatch({effects: collaboration.reconfigure(yCollab(ytext, awareness))});
          setStartupRoomReady(true);
          onStartupRoomReady?.();
        }).catch(error => { if (!cancelled && startup.current()) setStartupRoomError(String(error)); });
      };
    }
    setRailView(view);
    // Debug handle for driving the editor from automation (kept out of the
    // normal path; enable with localStorage "hickory.debug" = "1").
    if (
      localStorage.getItem("hickory.debug") === "1" ||
      import.meta.env.MODE === "test"
    ) {
      (window as unknown as { __hickoryView?: EditorView }).__hickoryView =
        view;
    }
    setEnvSlots(envRegistry.list());
    // The rail's first fill. The room's initial sync arrives as an ordinary
    // document change, so the updateListener keeps it current from here.
    setCards(cardsOf(structureOf(view.state), { text: view.state.doc.toString() }));
    seedRendered(view);
    onViewReady?.(view);

    return () => {
      cancelled = true;
      startStartupRoom.current = null;
      onViewReady?.(null);
      documentDraftBindings.delete(view);
      forgetEditor(view);
      forgetFocusedEditor(view);
      view.destroy();
      viewRef.current = null;
      setRailView(null);
      setOpen(null);
      activeRoom.awareness.destroy();
      activeRoom.doc.destroy();
    };
    // Recreate the editor per document.
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [editorIdentity, realtime, envRegistry, renderedRegistry, startup]);

  useEffect(() => {
    viewRef.current?.dispatch({effects: [setComparison.of({base: comparisonBase, editable: !readOnly}), access.reconfigure(sourceAccess)]});
  }, [comparisonBase, readOnly, access, sourceAccess, docId, realtime]);
  useEffect(() => {
    if (startupAuthorized) startStartupRoom.current?.();
  }, [startupAuthorized, startup, realtime]);
  useEffect(() => {
    railView?.dispatch({effects: [languageServices.reconfigure(lspExtensions ?? []), completionServices.reconfigure(completionExtension)]});
  }, [railView, languageServices, lspExtensions, completionServices, completionExtension]);
  useEffect(() => {
    if (!railView || parserStatus !== "ready" || parserInstalled.current === railView) return;
    parserInstalled.current = railView;
    railView.dispatch({effects: [parserBecameReady.of(null), parserProtection.reconfigure(Prec.high(immutableScaffolds())), railView.scrollSnapshot()],
      annotations: Transaction.addToHistory.of(false)});
  }, [railView, parserStatus, parserProtection]);



  // Close a card whose source disappeared.
  useEffect(() => {
    setOpen((current) =>
      current && cards.some((c) => c.key === current.card.key) ? current : null,
    );
  }, [cards]);

  // Escape closes the popover from anywhere inside it, including the editor.
  useEffect(() => {
    if (!open) return;
    const onKey = (event: KeyboardEvent) => {
      if (event.key === "Escape") setOpen(null);
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, [open]);

  // Push verified-expect ranges into the editor: for each exec cell whose
  // last run is ok AND that has an expect block, style the expect body in the
  // source as the verified output (the single visible copy — see CellPanel).
  useEffect(() => {
    const view = viewRef.current;
    if (!view) return;
    const structure = structureOf(view.state);
    const spans: [number, number][] = [];
    execBlocksOf(structure).forEach((exec, index) => {
      const block = matchExecBlock(
        { span: [exec.from, exec.to], index },
        execBlocks,
      );
      if (block?.status === "ok" && block.expect) {
        const range = expectRangeOf(structure, exec);
        if (range) spans.push(range);
      }
    });
    view.dispatch({ effects: setVerifiedExpects.of(spans) });
  }, [execBlocks]);

  // Diagnostics arrive in LSP line/character coordinates; translate against
  // the live buffer so they stay put while the user types.
  useEffect(() => {
    const view = viewRef.current;
    if (!view) return;
    view.dispatch({
      effects: setLspDiagnostics.of(
        diagnosticRanges(lspDiagnostics ?? [], (p) =>
          positionToOffset(view.state.doc, p),
        ),
      ),
    });
  }, [lspDiagnostics]);

  useEffect(() => {
    const view = viewRef.current;
    if (!view || !selectSpan) return;
    const max = view.state.doc.length;
    const from = Math.min(selectSpan[0], max);
    const to = Math.min(selectSpan[1], max);
    view.dispatch({
      selection: { anchor: from, head: to },
      scrollIntoView: true,
    });
    view.focus();
  }, [selectSpan]);

  // What an exec card's icon says before it is opened. The rail is the only
  // place a cell's state is visible now, so this is not decoration: it is the
  // replacement for the status chip that used to sit under every cell.
  const cardStateOf = (card: DocCard): CardState => {
    if (card.kind !== "exec") return "idle";
    const block = matchExecBlock({ span: [card.from, card.to], index: card.index }, execBlocks);
    if (!block) return "unknown";
    if (runningCells.has(block.id)) return "running";
    if (block.status === "ok") return "ok";
    if (block.status === "failed") return "failed";
    if (block.status === "stale") return "stale";
    return "idle";
  };

  /**
   * Flip one block between its result and its source.
   *
   * This is what the rail icon does for a cell or a diagram. It is also the
   * only way back from a rendered block, so it must never silently no-op:
   * the position stored in the field is the block's start, which is exactly
   * what the card carries.
   */
  // Which blocks are rendered, for the rail. Derived from the slots rather
  // than read out of the editor state: a slot exists for exactly the blocks
  // that are rendered, and it is already React state that updates when one
  // appears or goes away.
  const renderedAt = renderedSlots.filter((slot) => !slot.outputOnly).map((slot) => slot.at);

  const toggleRenderedAt = (at: number) => {
    const view = viewRef.current;
    if (!view) return;
    const showing = isRendered(view.state, at);
    view.dispatch({ effects: showing ? showBlockSource.of(at) : renderBlock.of(at) });
    if (showing) {
      // Going to source is a request to read or edit the text: put the caret
      // in it, so the next keystroke lands where the eye already is.
      const pos = Math.min(at, view.state.doc.length);
      view.dispatch({ selection: { anchor: pos }, scrollIntoView: true });
      view.focus();
    }
  };
  const toggleRendered = (card: DocCard) => toggleRenderedAt(card.at);

  /** The server's exec block for a card, when it knows about one. */
  const blockOf = (card: DocCard) =>
    card.kind === "exec"
      ? matchExecBlock({ span: [card.from, card.to], index: card.index }, execBlocks)
      : undefined;

  /** Recorded code remains a fence. Its own review and debugger controls
   * live in the recorded-message gutter. */
  const cardActionsOf = (card: DocCard): RailAction[] => {
    if (readOnly && preserveBytes && card.kind === "fence") return [];
    if (card.kind !== "exec") return actionsFor(card.kind);
    const block = blockOf(card);
    return actionsFor("exec", {
      source: !card.insidePicture,
      replay: hasReplay({
        status: block?.status,
        hasExpect: !!block?.expect,
        transcriptLength: block?.transcript?.length ?? 0,
        running: block ? runningCells.has(block.id) : false,
      }),
    });
  };

  const toggleReplayAt = (at: number) =>
    setReplaying((current) =>
      current.includes(at) ? current.filter((p) => p !== at) : [...current, at],
    );

  /** Replace a fence with the exec cell built from it. */
  const convertFenceCard = (card: DocCard, text: string, expected: EditorState["doc"]) => {
    const view = viewRef.current;
    if (!view || view.state.readOnly || view.state.doc !== expected) return false;
    const { from, to } = card;
    view.dispatch({
      changes: { from, to, insert: text },
      // The caret lands in the new cell's body, which is where the next edit
      // goes. A userEvent so one Ctrl+Z takes the whole conversion back.
      selection: { anchor: Math.min(from + text.indexOf("\n") + 1, from + text.length) },
      scrollIntoView: true,
      userEvent: "input.convertFence",
    });
    setOpen(null);
    view.focus();
    return true;
  };

  /** Commit fence data and its number-format attribute together. */
  const replaceFenceBody = (card: DocCard, fence: { body: string }, csv: string, formats: string, expected: EditorState["doc"], actor?: UiActor) => {
    const view = viewRef.current;
    if (!view || view.state.readOnly || view.state.doc !== expected) return;
    const text = view.state.doc.toString();
    const bodyFrom = text.indexOf("\n", card.from) + 1;
    const bodyTo = bodyFrom + fence.body.length;
    if (bodyFrom <= 0 || bodyTo > text.length) return;
    const body = csv.replace(/\n+$/, "");
    const opening = text.slice(card.from, bodyFrom);
    const stripped = opening.replace(/\s+formats="[^"]*"/, "");
    const nextOpening = formats ? stripped.replace(/\r?\n$/, ` formats="${formats}"$&`) : stripped;
    if (text.slice(bodyFrom, bodyTo) === body && opening === nextOpening) return;
    view.dispatch({
      changes: { from: card.from, to: bodyTo, insert: nextOpening + body },
      userEvent: actor?.kind === "agent" ? "input.uiAgentTable" : "input.table",
    });
  };

  // The popover converts fences; elements render their controls in place.
  const popoverBody = (card: DocCard) => {
    const view = viewRef.current;
    const unavailable = <p role="status">This code fence changed or disappeared. Close this panel and select the current fence.</p>;
    if (!view || card.kind !== "fence") return unavailable;
    const structure = structureOf(view.state);
    const expected = view.state.doc;
    const fence = proseFences(structure, expected.toString()).find(f => f.from === card.from && f.to === card.to);
    if (!fence) return unavailable;
    // A ```csv fence is a table somebody pasted into their notes. It gets a
    // grid rather than the "make it a cell" converter — running a CSV file as
    // a shell command is not a thing anybody means — and its own promotion,
    // to a `<hick:table>` that owns a dataset.
    if (isTabularFence(fence.info)) {
      return (
        <FenceTable
          body={fence.body}
          formats={fence.info.match(/\bformats="([^"]*)"/)?.[1] ?? ""}
          sourceCurrent={() => viewRef.current?.state.doc === expected && !viewRef.current?.state.readOnly}
          onChange={(csv, formats, actor) => replaceFenceBody(card, fence, csv, formats ?? "", expected, actor)}
          onPromote={(path) => convertFenceCard(card, tableElementFor(fence.body, path, fence.info.match(/\bformats="([^"]*)"/)?.[1]), expected)}
        />
      );
    }
    return (
      <FenceConvert
        info={fence.info}
        body={fence.body}
        containers={containerNamesOf(structure)}
        onConvert={(text) => convertFenceCard(card, text, expected)}
        onCancel={() => setOpen(null)}
      />
    );
  };

  /** Commit table data in its existing source representation. */
  const replaceBlockContent = useCallback(
    (slot: RenderedSlot, text: string, userEvent = "input.table") => replaceCapturedBlock(viewRef.current, slot, text, userEvent),
    [],
  );

  // Place the popover against its REAL height, before paint.
  //
  // The popover opens level with its icon and only slides up as far as it
  // must to fit — which needs the height it actually rendered at, not the
  // cap it is allowed to reach. Measured in a layout effect so the corrected
  // position is the first one painted, and re-measured while it grows (a
  // transcript streaming in, a diagram the engine has not drawn yet).
  useLayoutEffect(() => {
    const el = popoverRef.current;
    const box = railView?.dom;
    if (!el || !open || !box) return;
    const place = () => {
      el.style.top = `${popoverTop(open.iconTop, el.offsetHeight, {
        top: 0,
        height: box.clientHeight,
      })}px`;
    };
    place();
    if (typeof ResizeObserver === "undefined") return;
    const observer = new ResizeObserver(place);
    observer.observe(el);
    return () => observer.disconnect();
  }, [open, railView]);

  // What every rendered element is handed, beside its own slot. One object
  // so a view's signature never grows a parameter per element.
  const slotContext: SlotContext = {
    capturedProject,
    docId,
    view: railView,
    path: path ?? null,
    execBlocks,
    diagramBlocks: diagramBlocks ?? [],
    runningCells,
    replaying,
    executor: executorInfo ?? undefined,
    tableLayouts,
    onTableLayout,
    replaceBlockContent,
  };

  return (
    <>
      {/* The bordered box holds the editor, its right line-number rail, and
          the action rail outside that; the ribbon overlay anchors on the
          number rail's outer edge through `.with-right-rail`. */}
      {/* Inside a table the ruler stops measuring prose and names the
          table's columns instead, so A1 is readable off the same stick that
          is already there rather than out of a second row of furniture. */}
      {startupRoomError && <PaneHeaderContent part="info">
        <span role="status" className="save-state save-state-failed" title={startupRoomError}>{startupRoomError}</span>
      </PaneHeaderContent>}
      {completionNotice && <PaneHeaderContent part="info">
        <span role="status" className="save-state" title={completionNotice}>{completionNotice}</span>
      </PaneHeaderContent>}
      {assetError && (
        <div className="editor-asset-error" role="status">
          <span>{assetError}</span>
          <button type="button" onClick={() => setAssetError(null)} aria-label="Dismiss">
            ×
          </button>
        </div>
      )}
      <div className="document-editor with-right-rail with-card-rail">
        <div ref={hostRef} className="editor-cm-host" />
        <RightRail view={railView} />
        <DebugLaunchRail view={railView} onStart={onDebugFile} onProfiles={onDebugFile ? (path,at)=>onDebugFile(path,at,true) : undefined} enabled={!readOnly} />
        <CardRail
          documentId={docId}
          view={railView}
          cards={cards}
          stateOf={cardStateOf}
          openKey={open ? `${open.card.key}:convert` : null}
          renderedAt={renderedAt}
          actionsOf={cardActionsOf}
          replayingAt={replaying}
          onAction={(card, action, iconTop) => {
            // Resolve the clicked card against the live parse rather than
            // trusting data captured by a previous React render.
            const view = viewRef.current;
            if (!view) return;
            const current = cardsOf(structureOf(view.state), { text: view.state.doc.toString() })
              .find(candidate => candidate.kind === card.kind && candidate.from === card.from && candidate.to === card.to);
            if (!current) { setOpen(null); return; }
            card = current;
            if (action === "convert") {
              setOpen((current) => (current?.card.key === card.key ? null : { card, iconTop }));
              return;
            }
            setOpen(null);
            if (action === "source") {
              toggleRendered(card);
              return;
            }
            if (action === "replay") {
              toggleReplayAt(card.at);
              return;
            }
            // A new cell needs a saved document before it has a runnable id.
            const block = blockOf(card);
            if (block) onRunCell(block.id);
            else requestMenuAction("save");
          }}
        />
        {open && (
          <div
            className="cm-card-popover"
            role="dialog"
            aria-label={open.card.label}
            ref={popoverRef}
            // A first guess, corrected below before paint. Most popovers are
            // far shorter than the cap, and placing them as if they were the
            // cap would slide every one of them away from its icon.
            style={{ top: popoverTop(open.iconTop, POPOVER_HEIGHT, { top: 0, height: 0 }) }}
            // The editor takes the selection on mousedown; a click on a
            // button in here must not also move the caret behind it.
            onMouseDown={(event) => event.stopPropagation()}
          >
            <button
              type="button"
              className="cm-card-popover__close"
              aria-label="Close"
              onClick={() => setOpen(null)}
            >
              ×
            </button>
            {popoverBody(open.card)}
          </div>
        )}
      </div>
      {renderedSlots.map((slot) =>
        createPortal(elementViews[slot.kind].render(slot, slotContext), slot.el, slot.key),
      )}
      {envSlots.map((slot) =>
        createPortal(
          <EnvCard
            name={slot.name}
            image={slot.image}
            rules={slot.rules}
            executor={executorInfo}
          />,
          slot.el,
          slot.key,
        ),
      )}
    </>
  );
}
</storycode:copy>

### Assemble `apps/web/src/editor/DocumentEditor.tsx`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/editor/DocumentEditor.tsx"><storycode:paste select="#f-aab82870e517-DocumentEditorProps"/><storycode:paste select="#f-aab82870e517-POPOVER_HEIGHT"/><storycode:paste select="#f-aab82870e517-export_statement"/><storycode:paste select="#f-aab82870e517-DocumentEditor"/></storycode:file>

</storycode:doc>
