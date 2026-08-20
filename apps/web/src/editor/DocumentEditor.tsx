import { useEffect, useLayoutEffect, useMemo, useRef, useState } from "react";
import { createPortal } from "react-dom";
import { EditorState, Transaction } from "@codemirror/state";
import { EditorView, keymap, placeholder } from "@codemirror/view";
import {
  defaultKeymap,
  history,
  historyKeymap,
  indentWithTab,
} from "@codemirror/commands";
import { search, searchKeymap } from "@codemirror/search";
import * as Y from "yjs";
import { Awareness } from "y-protocols/awareness";
import { yCollab } from "y-codemirror.next";
import type { Extension } from "@codemirror/state";
import { EnvRegistry, setVerifiedExpects, structureOf, wysiwyg } from "./wysiwyg";
import { taskCheckboxes } from "./taskList";
import { renderedMath } from "./mathRender";
import { EditorRuler } from "./EditorRuler";
import { WRAP_DEFAULT, proseWrap } from "./wrapColumn";
import { mathSpans } from "../lib/math";
import {
  diagnosticRanges,
  positionToOffset,
  setLspDiagnostics,
} from "../lsp/cmLsp";
import type { LspDiagnostic } from "../lsp/client";
import { hickoryFolding } from "./folding";
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
import { DiagramPanel } from "../components/DiagramPanel";
import { MathPanel } from "../components/MathPanel";
import { CellPanel } from "../components/CellPanel";
import { FenceConvert } from "../components/FenceConvert";
import { EnvCard } from "../components/EnvCard";
import { api } from "../api/client";
import type { Realtime } from "../api/realtime";
import type { ExecBlock, ExecutorInfo } from "../api/types";

export interface DocumentEditorProps {
  docId: string;
  /** Initial .hick source, used to seed the Y.Doc when it is empty. */
  initialSource: string;
  realtime: Realtime;
  onChange?: (source: string) => void;
  /** Byte span to select (provenance click-through from the Output view). */
  selectSpan?: [number, number] | null;
  /** Rendered exec blocks (statuses/transcripts) to attach to cells. */
  execBlocks: ExecBlock[];
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
  /** Dim hint shown while the buffer is empty (the untitled document). */
  placeholderText?: string;
  /** Where PROSE wraps, in columns. Owned by the tab (so it is per-document
   * and survives a restart); code never wraps whatever this says. */
  wrapColumn?: number;
  /** Report a measure the reader dragged on the ruler. */
  onWrapColumn?: (column: number) => void;
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
function overlap(a: [number, number], b: [number, number]): number {
  return Math.max(0, Math.min(a[1], b[1]) - Math.max(a[0], b[0]));
}

/**
 * Match a widget slot (source span of the exec block as currently typed) to
 * the server-rendered exec block: best span overlap first, ordinal fallback
 * (spans drift while the user edits above a cell).
 */
export function matchExecBlock(
  slot: { span: [number, number]; index: number },
  blocks: ExecBlock[],
): ExecBlock | undefined {
  let best: ExecBlock | undefined;
  let bestOverlap = 0;
  for (const b of blocks) {
    const o = overlap(slot.span, b.span);
    if (o > bestOverlap) {
      bestOverlap = o;
      best = b;
    }
  }
  return best ?? blocks[slot.index];
}

/**
 * The Document view: ONE CodeMirror instance over the raw .hick source with
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
  docId,
  initialSource,
  realtime,
  onChange,
  selectSpan,
  execBlocks,
  runningCells,
  onRunCell,
  onViewReady,
  lspExtensions,
  lspDiagnostics,
  onDebugFile,
  placeholderText,
  wrapColumn = WRAP_DEFAULT,
  onWrapColumn,
}: DocumentEditorProps) {
  const hostRef = useRef<HTMLDivElement>(null);
  const viewRef = useRef<EditorView | null>(null);
  // The live view AS STATE, for the right rail — a ref never re-renders.
  const [railView, setRailView] = useState<EditorView | null>(null);
  // Read through a ref: the editor is built once per document, and a callback
  // baked in at construction would go stale the moment the debugger's state
  // changed.
  const onDebugFileRef = useRef(onDebugFile);
  onDebugFileRef.current = onDebugFile;
  const envRegistry = useMemo(() => new EnvRegistry(), []);
  const [envSlots, setEnvSlots] = useState<EnvSlot[]>([]);
  const [executorInfo, setExecutorInfo] = useState<ExecutorInfo | null>(null);
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
      const blocks = renderableBlocks(structureOf(target.state));
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
          history(),
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
          keymap.of([...searchKeymap, ...defaultKeymap, ...historyKeymap, indentWithTab]),
          // The hovered-ribbon line tint, shared with the right rail.
          lineHighlightField,
          wysiwyg(envRegistry, (path) => onDebugFileRef.current?.(path)),
          // Task boxes come from the DOCUMENT's structure parse, not a plain
          // markdown scan: a `- [ ]` inside an exec cell's payload is a
          // command's argument, and turning it into a checkbox would offer to
          // edit a line the reader is not looking at.
          taskCheckboxes((state) => structureOf(state).tasks),
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
          yCollab(ytext, awareness),
          ...(placeholderText ? [placeholder(placeholderText)] : []),
          ...(lspExtensions ?? []),
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
          EditorView.updateListener.of((u) => {
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

  // A card that stopped existing must not leave its popover floating over a
  // document that no longer has it.
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

  return (
    <>
      {/* The bordered box holds the editor, its right line-number rail, and
          the action rail outside that; the ribbon overlay anchors on the
          number rail's outer edge through `.with-right-rail`. */}
      <EditorRuler
        view={railView}
        column={wrapColumn}
        onColumn={(next) => onWrapColumn?.(next)}
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
      {renderedSlots.map((slot) => {
        if (slot.kind === "diagram") {
          return createPortal(
            <div className="rendered-diagram">
              <DiagramPanel
                renderer={slot.renderer}
                source={slot.text}
                domId={`hick-diagram-${slot.index}`}
                assertions={slot.asserts.map((id) => ({
                  id,
                  // Wiring a live pass/fail state to the cell that carries
                  // this id is the next step; until then the panel says
                  // "checked by", never "passing", because it does not know.
                  state: "unknown" as const,
                }))}
              />
            </div>,
            slot.el,
            slot.key,
          );
        }
        if (slot.kind === "math") {
          return createPortal(
            <div className="rendered-math">
              <MathPanel source={slot.text} />
            </div>,
            slot.el,
            slot.key,
          );
        }
        const block = matchExecBlock({ span: slot.span, index: slot.index }, execBlocks);
        return createPortal(
          <CellPanel
            block={block}
            running={block ? runningCells.has(block.id) : false}
            command={slot.text}
            replay={replaying.includes(slot.at)}
          />,
          slot.el,
          slot.key,
        );
      })}
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
