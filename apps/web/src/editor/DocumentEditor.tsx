import { useCallback, useEffect, useLayoutEffect, useMemo, useRef, useState } from "react";
import { createPortal } from "react-dom";
import { Compartment, EditorState, Transaction } from "@codemirror/state";
import { comparisonField, setComparison } from "./comparison";
import { EditorView, keymap, placeholder } from "@codemirror/view";
import {
  defaultKeymap,
  history,
  historyKeymap,
  indentWithTab,
} from "@codemirror/commands";
import { search, searchKeymap } from "@codemirror/search";
import { wordMotionBindings } from "./wordMotion";
import { unwrapParagraphs } from "./unwrapParagraphs";
import * as Y from "yjs";
import { Awareness } from "y-protocols/awareness";
import { yCollab } from "y-codemirror.next";
import type { Extension } from "@codemirror/state";
import { EnvRegistry, setVerifiedExpects, structureOf, wysiwyg } from "./wysiwyg";
import { taskCheckboxes } from "./taskList";
import { renderedMath } from "./mathRender";
import { completions } from "../lsp/completion";
import { EditorRuler } from "./EditorRuler";
import { WRAP_DEFAULT, proseWrap } from "./wrapColumn";
import { editorChrome } from "./chrome";
import { blameGutter } from "./blameGutter";
import { useBlame } from "./useBlame";
import { mathSpans } from "../lib/math";
import {
  diagnosticRanges,
  offsetToPosition,
  positionToOffset,
  setLspDiagnostics,
} from "../lsp/cmLsp";
import type { LspClient, LspDiagnostic } from "../lsp/client";
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
  editorExtensions?: Extension[];
  preserveBytes?: boolean;
  comparisonBase?: string | null;
  readOnly?: boolean;
  docId: string;
  /** Initial .md source, used to seed the Y.Doc when it is empty. */
  initialSource: string;
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
  onDebugFile?: (path: string) => void;
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

/**
 * The popover's height cap, matching `max-height` on `.cm-card-popover`.
 *
 * Used only as the first guess for where the popover opens; a layout effect
 * corrects it against the real height before paint. Content past this scrolls
 * inside the popover rather than growing it, so a streaming transcript can
 * never push its own Run button off the screen.
 */
const POPOVER_HEIGHT = 320;

/** Whether two card lists would draw the same rail. */
export function sameCards(a: readonly DocCard[], b: readonly DocCard[]): boolean {
  if (a.length !== b.length) return false;
  return a.every((card, i) => card.key === b[i].key && card.at === b[i].at);
}

/** Overlap length of two [from, to) spans. */
// Kept as exports here for the callers that always found them here; the
// code lives beside the other block matching in lib/blockMatch.ts.
export { assertionStates, matchExecBlock } from "../lib/blockMatch";

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
export function DocumentEditor({
  editorExtensions = [],
  preserveBytes = false,
  comparisonBase = null,
  readOnly = false,
  docId,
  initialSource,
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
  // The live view AS STATE, for the right rail — a ref never re-renders.
  const [railView, setRailView] = useState<EditorView | null>(null);
  // Read through a ref: the editor is built once per document, and a callback
  // baked in at construction would go stale the moment the debugger's state
  // changed.
  const onDebugFileRef = useRef(onDebugFile);
  onDebugFileRef.current = onDebugFile;
  // The editor is built once per document, but a buffer that was untitled
  // when it was built has a path as soon as it is first saved — and where an
  // image lands, and what a relative link resolves to, both depend on it. So
  // it is read through a ref at the moment it is needed.
  const pathRef = useRef(path);
  pathRef.current = path;
  // The ruler is memoized — a hundred-odd tick elements that never change
  // while you type — so what it is handed must not change identity per render.
  const onWrapColumnRef = useRef(onWrapColumn);
  onWrapColumnRef.current = onWrapColumn;
  const onWrapColumnStable = useCallback((next: number) => onWrapColumnRef.current?.(next), []);
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

  // Focus is tracked on the document rather than per widget: the widgets are
  // portalled into CodeMirror's DOM and come and go as the fold does, so a
  // handler per widget would be a subscription per render.
  useEffect(() => {
    const onFocusIn = (event: FocusEvent) => {
      const target = event.target;
      setFocusedEl(target instanceof HTMLElement ? target : null);
    };
    document.addEventListener("focusin", onFocusIn);
    return () => document.removeEventListener("focusin", onFocusIn);
  }, []);

  // The action rail's contents, recomputed whenever an edit changes them.
  // Held as state rather than derived per render because the editor
  // deliberately does not re-render on every keystroke.
  const [cards, setCards] = useState<DocCard[]>([]);
  // Which card's popover is open, and the rail-relative top of the icon that
  // opened it.
  const [open, setOpen] = useState<{ card: DocCard; iconTop: number } | null>(null);
  // Cells whose transcript is revealed, by block start. This used to be local
  // state inside CellPanel, back when Replay was a button in the panel; the
  // rail owns the verb now, so the rail's owner owns the state.
  const [replaying, setReplaying] = useState<readonly number[]>([]);
  const popoverRef = useRef<HTMLDivElement | null>(null);
  // Blocks currently showing their result instead of their source.
  const renderedRegistry = useMemo(() => new RenderedRegistry(), []);
  const [renderedSlots, setRenderedSlots] = useState<RenderedSlot[]>([]);
  // The rendered table the caret is inside (by block start), or null. State
  // rather than a ref because the ruler above the editor changes what it
  // draws when the caret is inside a table — but ONLY that: the caret's
  // position itself is not kept, because it changes on every keystroke and
  // nothing rendered here depends on it. A component re-rendering every rail,
  // ruler and portal per keystroke is what a laggy editor is made of.
  const [caretTableAt, setCaretTableAt] = useState<number | null>(null);
  // Which rendered widget has the focus. A rendered table is a FOLD — the
  // caret cannot be inside it, and the widget deliberately swallows its own
  // events so a click in a cell is not read as a click in the text. So "the
  // reader is in this table" is a focus question, not a caret question.
  const [focusedEl, setFocusedEl] = useState<HTMLElement | null>(null);

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
  // The cell panels and diagrams that used to be observed here are gone from
  // the document entirely; they open on the rail, outside the text, where
  // their height cannot move a line at all.
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
    // The presence color travels to peers as a literal value, so resolve the
    // theme's accent once at bind time rather than shipping a var() string.
    const presenceColor =
      getComputedStyle(document.documentElement).getPropertyValue("--accent").trim() || "#5eb0ef";
    awareness.setLocalStateField("user", {
      name: localStorage.getItem("hickory.name") ?? "anonymous",
      color: presenceColor,
    });
    realtime.bindDoc(ydoc, awareness);

    // Seeding is ONLY for realtimes with no server behind them (mock mode).
    // With a server, the room's Y.Doc is built from `docs.source`; seeding
    // here mints an independent copy of the same text under a different
    // client id, and the CRDT merges the two by concatenation — the document
    // doubles on every single connect. (Measured: a 15KB doc reached 49MB.)
    let cancelled = false;
    if (!realtime.serverAuthoritative) {
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
    const seedRendered = (target: EditorView) => {
      if (seeded || target.state.doc.length === 0) return;
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
      parent: host,
      state: EditorState.create({
        doc: ytext.toString(),
        extensions: [
          // Gutters, caret, selection, the find panel — one definition, loaded
          // by every editor in the app. See editor/chrome.ts.
          editorChrome("document"),
          // Before the debug layer, which is where lineNumbers() comes from:
          // gutters lay out in declaration order, and this one belongs
          // OUTSIDE the numbers. Off by default — see editor/blameGutter.ts.
          blameGutter(),
          history(),
          ...(preserveBytes ? [] : [unwrapParagraphs()]),
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
          search({ top: true }),
          // Word motion first: bindings for one key run in registration
          // order and the first to return true wins, so these must arrive
          // before `defaultKeymap`'s own Ctrl+arrow. See editor/wordMotion.ts.
          keymap.of([
            ...wordMotionBindings,
            ...searchKeymap,
            ...defaultKeymap,
            ...historyKeymap,
            indentWithTab,
          ]),
          // The hovered-ribbon line tint, shared with the right rail.
          lineHighlightField,
          wysiwyg(envRegistry, (path) => onDebugFileRef.current?.(path)),
          // Task boxes come from the DOCUMENT's structure parse, not a plain
          // markdown scan: a `- [ ]` inside an exec cell's payload is a
          // command's argument, and turning it into a checkbox would offer to
          // edit a line the reader is not looking at.
          taskCheckboxes((state) => structureOf(state).tasks),
          // Two sources, one list, each marked. The language server knows
          // what is in scope; the project index knows what this codebase
          // calls things. Neither subsumes the other — see lsp/completion.ts.
          completions({
            lsp: lspCompletion
              ? {
                  client: lspCompletion.client,
                  uri: lspCompletion.uri,
                  positionAt: (offset, state) => offsetToPosition(state.doc, offset),
                }
              : undefined,
            project: (prefix, around) =>
              api.complete(prefix, around).then((answer) => answer.suggestions),
          }),
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
          yCollab(ytext, awareness),
          ...(placeholderText ? [placeholder(placeholderText)] : []),
          ...(lspExtensions ?? []),
          ...editorExtensions,
          comparisonField,
          access.of([EditorState.readOnly.of(readOnly), EditorView.editable.of(!readOnly), EditorState.transactionFilter.of(tr => readOnly && tr.docChanged && tr.annotation(Transaction.userEvent) ? [] : tr)]),
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
            if (u.selectionSet || u.docChanged) {
              const head = u.state.selection.main.head;
              const inTable = renderedRegistry
                .list()
                .find((slot) => slot.kind === "table" && head >= slot.span[0] && head <= slot.span[1]);
              const at = inTable ? inTable.at : null;
              setCaretTableAt((current) => (current === at ? current : at));
            }
            if (!u.docChanged) return;
            onChange?.(u.state.doc.toString());
            // The rail follows the text. Replaced only when the list
            // actually differs, so typing inside a cell does not re-render
            // the rail (and close nothing) on every keystroke.
            setCards((previous) => {
              const next = cardsOf(structureOf(u.state), {
                text: u.state.doc.toString(),
              });
              return sameCards(previous, next) ? previous : next;
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
      onViewReady?.(null);
      forgetEditor(view);
      forgetFocusedEditor(view);
      view.destroy();
      viewRef.current = null;
      setRailView(null);
      setOpen(null);
      awareness.destroy();
      ydoc.destroy();
    };
    // Recreate the editor per document.
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [docId, realtime, envRegistry, renderedRegistry]);

  useEffect(() => {
    viewRef.current?.dispatch({ effects: [setComparison.of({ base: comparisonBase, editable: !readOnly }), access.reconfigure([EditorState.readOnly.of(readOnly), EditorView.editable.of(!readOnly), EditorState.transactionFilter.of(tr => readOnly && tr.docChanged && tr.annotation(Transaction.userEvent) ? [] : tr)])] });
  }, [comparisonBase, readOnly, access, docId, realtime]);

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
  // The table the reader is in: the one holding the focus, or failing that
  // the one the caret sits inside — which is how it reads when a document is
  // opened straight onto a table, before anything has been clicked.
  const activeTableEl =
    renderedSlots.find((slot) => slot.kind === "table" && focusedEl && slot.el.contains(focusedEl))
      ?.el ??
    renderedSlots.find((slot) => slot.kind === "table" && slot.at === caretTableAt)?.el ??
    null;

  const renderedAt = renderedSlots.map((slot) => slot.at);

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

  /** Which icons a card puts on the rail. */
  const cardActionsOf = (card: DocCard): RailAction[] => {
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
  const convertFenceCard = (card: DocCard, text: string) => {
    const view = viewRef.current;
    if (!view) return;
    const to = Math.min(card.to, view.state.doc.length);
    const from = Math.min(card.from, to);
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
  };

  /** Replace a fence's BODY, leaving its markers and its info string. */
  const replaceFenceBody = (card: DocCard, fence: { body: string }, csv: string) => {
    const view = viewRef.current;
    if (!view) return;
    const text = view.state.doc.toString();
    const bodyFrom = text.indexOf("\n", card.from) + 1;
    const bodyTo = bodyFrom + fence.body.length;
    if (bodyFrom <= 0 || bodyTo > text.length) return;
    const body = csv.replace(/\n+$/, "");
    if (text.slice(bodyFrom, bodyTo) === body) return;
    view.dispatch({
      changes: { from: bodyFrom, to: bodyTo, insert: body },
      userEvent: "input.table",
    });
  };

  // The popover is the FENCE converter's, and only that. A cell and a diagram
  // render in the document itself now, with their controls on the rendered
  // block — a second copy of the Run button floating beside it would be two
  // answers to the same question.
  const popoverBody = (card: DocCard) => {
    const view = viewRef.current;
    if (!view || card.kind !== "fence") return null;
    const structure = structureOf(view.state);
    const fence = proseFences(structure, view.state.doc.toString())[card.index];
    if (!fence) return null;
    // A ```csv fence is a table somebody pasted into their notes. It gets a
    // grid rather than the "make it a cell" converter — running a CSV file as
    // a shell command is not a thing anybody means — and its own promotion,
    // to a `<hick:table>` that owns a dataset.
    if (isTabularFence(fence.info)) {
      return (
        <FenceTable
          body={fence.body}
          onChange={(csv) => replaceFenceBody(card, fence, csv)}
          onPromote={(path) => convertFenceCard(card, tableElementFor(fence.body, path))}
        />
      );
    }
    return (
      <FenceConvert
        info={fence.info}
        body={fence.body}
        containers={containerNamesOf(structure)}
        onConvert={(text) => convertFenceCard(card, text)}
        onCancel={() => setOpen(null)}
      />
    );
  };

  /**
   * Replace one block's CONTENT — the bytes between its tags — leaving the
   * tags themselves alone.
   *
   * Used by the table grid, whose edits are edits to the document. Content
   * offsets rather than the block span, because rewriting the span would
   * rewrite the opening tag and lose its attributes.
   */
  const replaceBlockContent = useCallback(
    (slot: RenderedSlot, text: string, userEvent = "input.table") => {
      const view = viewRef.current;
      if (!view) return;
      const structure = structureOf(view.state);
      const block = renderableBlocks(structure, view.state.doc.toString()).find((b) => b.from === slot.at);
      if (!block) return;
      const body = block.name === "markdown-table" ? text : text.endsWith("\n") ? text : `${text}\n`;
      if (view.state.doc.sliceString(block.contentFrom, block.contentTo) === body) return;
      view.dispatch({
        changes: { from: block.contentFrom, to: block.contentTo, insert: body },
        effects: block.name === "markdown-table" ? renderBlock.of(block.from) : [],
        // A user event, so the CRDT and the undo history both treat it as
        // typing — which is what it is.
        userEvent,
      });
    },
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
      {assetError && (
        <div className="editor-asset-error" role="status">
          <span>{assetError}</span>
          <button type="button" onClick={() => setAssetError(null)} aria-label="Dismiss">
            ×
          </button>
        </div>
      )}
      <EditorRuler
        view={railView}
        column={wrapColumn}
        onColumn={onWrapColumnStable}
        tableEl={activeTableEl}
      />
      <div className="document-editor with-right-rail with-card-rail">
        <div ref={hostRef} className="editor-cm-host" />
        <RightRail view={railView} />
        <CardRail
          view={railView}
          cards={cards}
          stateOf={cardStateOf}
          openKey={open ? `${open.card.key}:convert` : null}
          renderedAt={renderedAt}
          actionsOf={cardActionsOf}
          replayingAt={replaying}
          onAction={(card, action, iconTop) => {
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
            // Run. A cell the server has not rendered yet has no id to run,
            // and the icon says so rather than doing nothing silently.
            const block = blockOf(card);
            if (block) onRunCell(block.id);
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
