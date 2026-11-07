import { useLayoutEffect, useRef, useSyncExternalStore } from "react";
import { EditorState } from "@codemirror/state";
import { EditorView, keymap, drawSelection } from "@codemirror/view";
import { defaultKeymap, history, historyKeymap } from "@codemirror/commands";
import { Buffer } from "./buffers";
const editors = new WeakMap<Buffer, EditorView>();
export function Document({ buffer }: { buffer: Buffer }) {
  const host = useRef<HTMLDivElement>(null);
  useSyncExternalStore(buffer.subscribe, buffer.snapshot);
  useLayoutEffect(() => {
    let editor = editors.get(buffer);
    if (!editor) {
      editor = new EditorView({ state: EditorState.create({ doc: buffer.text, extensions: [
        history(), drawSelection(), EditorView.lineWrapping, keymap.of([...defaultKeymap, ...historyKeymap]),
        EditorState.readOnly.of(buffer.readonly), EditorView.editable.of(!buffer.readonly),
        EditorView.contentAttributes.of({ "aria-label": buffer.file.path }),
        EditorView.updateListener.of(update => { if (update.docChanged) buffer.edit(update.state.doc.toString()); }),
        EditorView.theme({ "&": { height: "100%" }, ".cm-scroller": { overflow: "auto", fontFamily: "ui-monospace, monospace", fontSize: "13px" } }),
      ] }) });
      editors.set(buffer, editor);
    }
    host.current?.append(editor.dom);
    return () => { editor.dom.remove(); };
  }, [buffer]);
  return <div className="document-body">
    <div ref={host} className="editor-host" />
    {buffer.error && <div role="alert" className="notice">{buffer.error}</div>}
  </div>;
}
