import { useEffect, useRef } from "react";
import { EditorState } from "@codemirror/state";
import { EditorView, keymap, lineNumbers, highlightActiveLine } from "@codemirror/view";
import { defaultKeymap, history, historyKeymap, indentWithTab } from "@codemirror/commands";
import * as Y from "yjs";
import { Awareness } from "y-protocols/awareness";
import { yCollab } from "y-codemirror.next";
import { hickHighlight } from "./hickHighlight";
import type { Realtime } from "../api/realtime";

export interface SourceEditorProps {
  docId: string;
  /** Initial .hick source, used to seed the Y.Doc when it is empty. */
  initialSource: string;
  realtime: Realtime;
  onChange?: (source: string) => void;
  /** Byte span to select (provenance click-through from the rendered view). */
  selectSpan?: [number, number] | null;
}

/** CodeMirror 6 editor over the raw .hick source, collaborative via Yjs. */
export function SourceEditor({
  docId,
  initialSource,
  realtime,
  onChange,
  selectSpan,
}: SourceEditorProps) {
  const hostRef = useRef<HTMLDivElement>(null);
  const viewRef = useRef<EditorView | null>(null);

  useEffect(() => {
    const host = hostRef.current;
    if (!host) return;

    const ydoc = new Y.Doc();
    const ytext = ydoc.getText("source");
    const awareness = new Awareness(ydoc);
    awareness.setLocalStateField("user", {
      name: localStorage.getItem("hickory.name") ?? "anonymous",
      color: "#8f6f3f",
    });
    realtime.bindDoc(ydoc, awareness);

    // Seed only when the shared doc is empty (fresh doc / mock mode); when
    // collaborating, the server's sync step supplies the content.
    if (ytext.length === 0 && initialSource.length > 0) {
      ytext.insert(0, initialSource);
    }

    const view = new EditorView({
      parent: host,
      state: EditorState.create({
        doc: ytext.toString(),
        extensions: [
          lineNumbers(),
          history(),
          highlightActiveLine(),
          keymap.of([...defaultKeymap, ...historyKeymap, indentWithTab]),
          hickHighlight,
          yCollab(ytext, awareness),
          EditorView.lineWrapping,
          EditorView.updateListener.of((u) => {
            if (u.docChanged) onChange?.(u.state.doc.toString());
          }),
        ],
      }),
    });
    viewRef.current = view;

    return () => {
      view.destroy();
      viewRef.current = null;
      awareness.destroy();
      ydoc.destroy();
    };
    // Recreate the editor per document.
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [docId, realtime]);

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

  return <div ref={hostRef} className="source-editor" />;
}
