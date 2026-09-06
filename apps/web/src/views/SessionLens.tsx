// The agent pane's history: the session document, with line numbers, the
// conversation's elements drawn as cards in place of their source, and the
// provenance each element declared handed to the ribbon overlay.
//
// A lens, not a document (docs/specs/freeform/lenses.md): read-only, no
// save path, the same cards and the same overlay every other view gets.
// See docs/guarantees/agent/an-answer-in-the-agent-pane-has-ribbons.md.
import { useEffect, useMemo, useRef, useState } from "react";
import { createPortal } from "react-dom";
import { EditorState } from "@codemirror/state";
import { EditorView, lineNumbers } from "@codemirror/view";

import { api } from "../api/client";
import type { SessionViewResponse } from "../api/types";
import { editorChrome } from "../editor/chrome";
import { RenderedRegistry, renderableBlocks, renderedBlocks, setRenderedBlocks } from "../editor/rendered";
import type { RenderedSlot } from "../editor/rendered";
import { structureOf } from "../editor/wysiwyg";
import { elementViews } from "../elements";
import type { SlotContext } from "../elements";
import { registerLens, ribbonLinksOf } from "../lib/lensSources";

export interface SessionLensProps {
  /** The session file, folder-relative. */
  path: string;
  /** Anything that means "the file may have changed" — a finished turn. */
  stamp?: string;
}

export function SessionLens({ path, stamp }: SessionLensProps) {
  const hostRef = useRef<HTMLDivElement | null>(null);
  const viewRef = useRef<EditorView | null>(null);
  const registry = useMemo(() => new RenderedRegistry(), []);
  const [slots, setSlots] = useState<RenderedSlot[]>([]);
  const [data, setData] = useState<SessionViewResponse | null>(null);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => registry.subscribe(() => setSlots(registry.list())), [registry]);

  useEffect(() => {
    let live = true;
    api.sessionView(path).then(
      (answer) => {
        if (!live) return;
        setError(null);
        setData(answer);
      },
      (e: unknown) => live && setError(e instanceof Error ? e.message : String(e)),
    );
    return () => {
      live = false;
    };
  }, [path, stamp]);

  // One editor for the lens's life; a new version of the file replaces its
  // text rather than remounting, so the cards already on screen stay put.
  useEffect(() => {
    const host = hostRef.current;
    if (!host || !data) return;
    let view = viewRef.current;
    if (!view) {
      view = new EditorView({
        parent: host,
        state: EditorState.create({
          doc: data.source,
          extensions: [
            editorChrome("document"),
            lineNumbers(),
            EditorState.readOnly.of(true),
            EditorView.editable.of(false),
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
    return registerLens({
      path: data.path,
      view,
      source: data.source,
      links: ribbonLinksOf(data.path, data.links),
    });
  }, [data, registry]);

  useEffect(
    () => () => {
      viewRef.current?.destroy();
      viewRef.current = null;
    },
    [],
  );

  const cx: SlotContext = useMemo(
    () => ({
      view: viewRef.current,
      path,
      execBlocks: [],
      diagramBlocks: [],
      sessionBlocks: data?.blocks ?? [],
      runningCells: new Set(),
      replaying: [],
      replaceBlockContent: () => {},
    }),
    [path, data],
  );

  return (
    <div className="session-lens" data-session-lens={path}>
      {error && <p className="error">{error}</p>}
      <div ref={hostRef} className="editor-cm-host session-lens__editor" />
      {slots.map((slot) => {
        const element = elementViews[slot.kind];
        return element ? createPortal(element.render(slot, cx), slot.el, slot.key) : null;
      })}
    </div>
  );
}
