// The agent pane's history: the session document, with line numbers, the
// conversation's elements drawn as cards in place of their source, and the
// provenance each element declared handed to the ribbon overlay.
//
// A lens, not a document (docs/specs/freeform/lenses.md): read-only, no
// save path, the same cards and the same overlay every other view gets.
// See docs/guarantees/agent/an-answer-in-the-agent-pane-has-ribbons.md.
import { useEffect, useMemo, useRef, useState } from "react";
import { createPortal } from "react-dom";
import { EditorState, StateField } from "@codemirror/state";
import { Decoration, EditorView, WidgetType, lineNumbers } from "@codemirror/view";
import type { DecorationSet } from "@codemirror/view";

import { api } from "../api/client";
import type { Block, SessionLink, SessionViewResponse } from "../api/types";
import { editorChrome } from "../editor/chrome";
import { RenderedRegistry, renderableBlocks, renderedBlocks, setRenderedBlocks } from "../editor/rendered";
import type { RenderedSlot } from "../editor/rendered";
import { structureOf } from "../editor/wysiwyg";
import { elementViews } from "../elements";
import type { SlotContext } from "../elements";
import { conversationAnchor } from "../lib/conversationLineage";
import { registerLens, ribbonLinksOf } from "../lib/lensSources";
import { byteToChar } from "../lib/offsets";

/** Nothing: what the file's own chrome is drawn as. */
class Blank extends WidgetType {
  toDOM() {
    const el = document.createElement("span");
    el.className = "session-lens__chrome";
    return el;
  }
}

/**
 * The lines that are the FILE's, not the conversation's — the XML
 * declaration and the root element's open and close tags — folded away.
 * The record is still the record; a reader of the conversation is not
 * reading XML.
 */
const sessionChrome = StateField.define<DecorationSet>({
  create: (state) => chromeDecorations(state),
  update: (value, tr) => (tr.docChanged ? chromeDecorations(tr.state) : value),
  provide: (field) => EditorView.decorations.from(field),
});

function chromeDecorations(state: EditorState): DecorationSet {
  const doc = state.doc;
  const structure = structureOf(state);
  const lines = new Set<number>();
  if (doc.lines > 0 && doc.line(1).text.startsWith("<?xml")) lines.add(1);
  for (const tag of structure.tags) {
    if (tag.name === "session") {
      lines.add(doc.lineAt(tag.from).number);
      lines.add(doc.lineAt(Math.max(tag.to - 1, tag.from)).number);
    }
  }
  const ranges = [...lines]
    .sort((a, b) => a - b)
    .map((n) => doc.line(n))
    .filter((line) => line.to > line.from)
    .map((line) => Decoration.replace({ widget: new Blank(), block: true }).range(line.from, line.to));
  return Decoration.set(ranges, true);
}

/**
 * The provenance under each answer: every link whose source lies between
 * this answer's first line and the next speaker's — what the turn read,
 * wrote and pointed at — keyed by the answer's block start.
 */
export function linksByAnswer(
  state: EditorState,
  links: readonly SessionLink[],
): Map<number, SessionLink[]> {
  const doc = state.doc;
  const speakers = structureOf(state)
    .blocks.filter((b) => b.name === "assistant" || b.name === "user")
    .sort((a, b) => a.from - b.from);
  const out = new Map<number, SessionLink[]>();
  const users = speakers.filter(block => block.name === "user");
  users.forEach((user, i) => {
    const end = users[i + 1]?.from ?? doc.length;
    const answer = speakers.filter(block => block.name === "assistant" && block.from > user.from && block.from < end).at(-1);
    if (!answer) return;
    const startLine = doc.lineAt(user.from).number;
    const endLine = end === doc.length ? doc.lines + 1 : doc.lineAt(end).number;
    const mine = links.filter(link => link.lines[0] >= startLine && link.lines[0] < endLine);
    if (mine.length > 0) out.set(answer.from, mine);
  });
  return out;
}

export interface SessionLensProps {
  /** The session file, folder-relative. */
  path: string;
  /** Anything that means "the file may have changed" — a finished turn. */
  stamp?: string;
  showCollapsedLineage?: boolean;
}

export function SessionLens({ path, stamp, showCollapsedLineage = false }: SessionLensProps) {
  const hostRef = useRef<HTMLDivElement | null>(null);
  const viewRef = useRef<EditorView | null>(null);
  const registry = useMemo(() => new RenderedRegistry(), []);
  const [slots, setSlots] = useState<RenderedSlot[]>([]);
  const [data, setData] = useState<SessionViewResponse | null>(null);
  const [error, setError] = useState<string | null>(null);
  // What the cards read, settled once the editor exists: the server's blocks
  // with their spans in the editor's own units (characters, not bytes —
  // the difference is every em dash the model wrote), and the links under
  // each answer.
  const [facts, setFacts] = useState<{
    blocks: Block[];
    linksAt: Map<number, SessionLink[]>;
    view: EditorView;
  } | null>(null);

  useEffect(() => registry.subscribe(() => setSlots(registry.list())), [registry]);

  useEffect(() => {
    let live = true;
    let pending = false;
    const refresh = () => {
      if (pending || document.hidden) return;
      pending = true;
      api.sessionView(path).then(answer => {
        if (!live) return;
        setError(null);
        setData(previous => JSON.stringify(previous) === JSON.stringify(answer) ? previous : answer);
      }, (e: unknown) => live && setError(e instanceof Error ? e.message : String(e)))
        .finally(() => { pending = false; });
    };
    refresh();
    const timer = window.setInterval(refresh, 5000);
    return () => { live = false; window.clearInterval(timer); };
  }, [path, stamp]);

  // One editor for the lens's life; a new version of the file replaces its
  // text rather than remounting, so the cards already on screen stay put.
  useEffect(() => {
    const host = hostRef.current;
    if (!host || !data) return;
    let view = viewRef.current;
    const changed = !view || view.state.doc.toString() !== data.source;
    if (!view) {
      view = new EditorView({
        parent: host,
        state: EditorState.create({
          doc: data.source,
          extensions: [
            editorChrome("document"),
            lineNumbers(),
            // A conversation reads as prose: it wraps at the pane's edge.
            EditorView.lineWrapping,
            EditorState.readOnly.of(true),
            EditorView.editable.of(false),
            sessionChrome,
            renderedBlocks(registry),
          ],
        }),
      });
      viewRef.current = view;
    } else if (view.state.doc.toString() !== data.source) {
      view.dispatch({
        changes: { from: 0, to: view.state.doc.length, insert: data.source },
      });
    }
    // Every element renders as its card: the lens is the conversation, and
    // the source is one click away on any card, as in a document.
    const blocks = renderableBlocks(structureOf(view.state));
    view.dispatch({ effects: setRenderedBlocks.of(blocks.map((b) => b.from)) });
    // A conversation opens at its newest turn, the way the cards always
    // did; the log is the pane's, so it is the log that scrolls.
    const log = host.closest<HTMLElement>(".chat-log");
    if (log && changed) {
      // Once now, and again after the cards have measured: the editor
      // virtualises against the log, so its height settles over a few
      // frames as widgets mount.
      const toEnd = () => log.scrollTo({ top: log.scrollHeight });
      requestAnimationFrame(toEnd);
      for (const ms of [120, 400, 900]) window.setTimeout(toEnd, ms);
    }
    setFacts({
      blocks: data.blocks.map((b) => ({
        ...b,
        span: [byteToChar(data.source, b.span[0]), byteToChar(data.source, b.span[1])] as [number, number],
      })),
      linksAt: linksByAnswer(view.state, data.links),
      view,
    });

  }, [data, registry]);

  // Recompute links as folds change. Hidden work cannot leave a ribbon
  // implying that its evidence is presently on screen.
  useEffect(() => {
    const host = hostRef.current;
    if (!host || !data || !facts) return;
    let unregister = () => {};
    const sync = () => {
      const visible = data.links.flatMap(link => {
        const anchor = conversationAnchor(host, byteToChar(data.source, link.span[0]), byteToChar(data.source, link.span[1]), showCollapsedLineage);
        return anchor ? ribbonLinksOf(data.path, [link]).map(ribbon => ({ ...ribbon,
          key: `lens:${data.path}:${link.span.join(":")}:${link.family}:${link.to.path}:${link.to.lines?.join(":")}:${link.evidence?.path}:${link.evidence?.lines.join(":")}`, anchor, alwaysVisible: true })) : [];
      });
      unregister();
      unregister = registerLens({ path: data.path, view: facts.view, source: data.source, links: visible });
    };
    sync();
    host.addEventListener("toggle", sync, true);
    return () => { host.removeEventListener("toggle", sync, true); unregister(); };
  }, [data, facts, slots, showCollapsedLineage]);

  useEffect(
    () => () => {
      viewRef.current?.destroy();
      viewRef.current = null;
    },
    [],
  );

  const cx: SlotContext = useMemo(
    () => ({
      view: facts?.view ?? null,
      path,
      execBlocks: [],
      diagramBlocks: [],
      sessionBlocks: facts?.blocks ?? [],
      sessionLinksAt: facts?.linksAt,
      runningCells: new Set(),
      replaying: [],
      replaceBlockContent: () => {},
    }),
    [path, facts],
  );

  return (
    <div className="session-lens" data-session-lens={path}>
      {error && <p className="error">{error}</p>}
      <div ref={hostRef} className="editor-cm-host session-lens__editor" />
      {slots.map((slot) => {
        const element = elementViews[slot.kind];
        return element ? createPortal(<div data-session-from={slot.span[0]} data-session-to={slot.span[1]}>{element.render(slot, cx)}</div>, slot.el, slot.key) : null;
      })}
    </div>
  );
}
